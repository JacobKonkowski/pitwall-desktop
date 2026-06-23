//! Register / unregister the PitWall OpenXR API layer with the loader.
//!
//! OpenXR discovers implicit API layers from per-user registry values under
//! `HKCU\Software\Khronos\OpenXR\1\ApiLayers\Implicit`: the value name is the
//! full path to the layer's JSON manifest and the `REG_DWORD` data is `0` when
//! enabled (non-zero disables it). This is exactly how RaceLab VR and
//! OpenKneeboard register their layers.

#[cfg(windows)]
const IMPLICIT_LAYERS_KEY: &str = r"Software\Khronos\OpenXR\1\ApiLayers\Implicit";

/// File name of the manifest, used to recognize our value among others.
pub const MANIFEST_FILE: &str = "pitwall_openxr_layer.json";

#[cfg(windows)]
pub use windows_impl::{install_layer, is_layer_installed, uninstall_layer};

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegEnumValueW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn open_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<HKEY> {
        let key = wide(IMPLICIT_LAYERS_KEY);
        let mut hkey = HKEY::default();
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key.as_ptr()),
                0,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                access,
                None,
                &mut hkey,
                None,
            )
        };
        (status == ERROR_SUCCESS).then_some(hkey)
    }

    /// True if the PitWall layer manifest is registered and enabled (data == 0).
    pub fn is_layer_installed() -> bool {
        let Some(hkey) = open_key(KEY_READ) else {
            return false;
        };
        let mut found = false;
        let mut index = 0u32;
        loop {
            let mut name = [0u16; 1024];
            let mut name_len = name.len() as u32;
            let mut data = [0u8; 8];
            let mut data_len = data.len() as u32;
            let status = unsafe {
                RegEnumValueW(
                    hkey,
                    index,
                    windows::core::PWSTR(name.as_mut_ptr()),
                    &mut name_len,
                    None,
                    None,
                    Some(data.as_mut_ptr()),
                    Some(&mut data_len as *mut u32),
                )
            };
            if status != ERROR_SUCCESS {
                break;
            }
            let value_name = String::from_utf16_lossy(&name[..name_len as usize]);
            if value_name.to_lowercase().ends_with(MANIFEST_FILE) {
                let enabled = data_len >= 4
                    && u32::from_ne_bytes([data[0], data[1], data[2], data[3]]) == 0;
                if enabled {
                    found = true;
                    break;
                }
            }
            index += 1;
        }
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        found
    }

    /// Register `manifest_path` as an enabled implicit API layer.
    pub fn install_layer(manifest_path: &str) -> anyhow::Result<()> {
        let Some(hkey) = open_key(KEY_WRITE) else {
            anyhow::bail!("Could not open OpenXR implicit layers registry key");
        };
        let value = wide(manifest_path);
        let data = 0u32.to_ne_bytes();
        let status = unsafe {
            RegSetValueExW(hkey, PCWSTR(value.as_ptr()), 0, REG_DWORD, Some(data.as_slice()))
        };
        unsafe {
            let _ = RegCloseKey(hkey);
        }
        if status != ERROR_SUCCESS {
            anyhow::bail!("Failed to write OpenXR layer registry value");
        }
        Ok(())
    }

    /// Remove the PitWall layer registration (best effort; succeeds if absent).
    pub fn uninstall_layer(manifest_path: &str) -> anyhow::Result<()> {
        let Some(hkey) = open_key(KEY_WRITE) else {
            return Ok(());
        };
        let value = wide(manifest_path);
        unsafe {
            let _ = RegDeleteValueW(hkey, PCWSTR(value.as_ptr()));
            let _ = RegCloseKey(hkey);
        }
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn is_layer_installed() -> bool {
    false
}

#[cfg(not(windows))]
pub fn install_layer(_manifest_path: &str) -> anyhow::Result<()> {
    anyhow::bail!("OpenXR layer install is only supported on Windows")
}

#[cfg(not(windows))]
pub fn uninstall_layer(_manifest_path: &str) -> anyhow::Result<()> {
    Ok(())
}
