//! Global hotkey registration for VR recenter (dedicated thread owns the manager).

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{AppHandle, Emitter};

use crate::commands::AppState;

enum HotkeyCmd {
    Sync { app: AppHandle, hotkey: String },
}

static HOTKEY_TX: OnceLock<Sender<HotkeyCmd>> = OnceLock::new();

fn hotkey_tx() -> Sender<HotkeyCmd> {
    HOTKEY_TX
        .get_or_init(|| {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || hotkey_thread(rx));
            tx
        })
        .clone()
}

fn hotkey_thread(rx: mpsc::Receiver<HotkeyCmd>) {
    let mut manager: Option<GlobalHotKeyManager> = None;
    let mut registered: Option<HotKey> = None;
    let mut app: Option<AppHandle> = None;

    loop {
        while let Ok(cmd) = rx.try_recv() {
            let HotkeyCmd::Sync { app: handle, hotkey } = cmd;
            app = Some(handle);
            if let Some(m) = manager.take() {
                if let Some(key) = registered.take() {
                    let _ = m.unregister(key);
                }
            }
            let trimmed = hotkey.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some(parsed) = parse_hotkey(trimmed) else {
                tracing::warn!("Invalid VR recenter hotkey: {trimmed}");
                continue;
            };
            match GlobalHotKeyManager::new() {
                Ok(m) => {
                    if m.register(parsed).is_ok() {
                        registered = Some(parsed);
                        manager = Some(m);
                    } else {
                        tracing::warn!("Failed to register hotkey: {trimmed}");
                    }
                }
                Err(e) => tracing::warn!("Global hotkey manager unavailable: {e}"),
            }
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                if let Some(handle) = app.as_ref() {
                    let _ = handle.emit("vr-recenter", ());
                }
            }
        }

        thread::sleep(Duration::from_millis(30));
    }
}

pub fn sync_hotkey(app: &AppHandle, state: &Arc<AppState>) {
    let hotkey = state.settings.lock().vr_recenter_hotkey.clone();
    let _ = hotkey_tx().send(HotkeyCmd::Sync {
        app: app.clone(),
        hotkey,
    });
}

fn parse_hotkey(s: &str) -> Option<HotKey> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return None;
    }
    let key_str = parts.last()?;
    let mut mods = Modifiers::empty();
    for part in &parts[..parts.len() - 1] {
        mods |= match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Modifiers::CONTROL,
            "alt" => Modifiers::ALT,
            "shift" => Modifiers::SHIFT,
            "super" | "win" | "meta" => Modifiers::SUPER,
            _ => return None,
        };
    }
    let code = parse_key_code(key_str)?;
    Some(HotKey::new(Some(mods), code))
}

fn parse_key_code(s: &str) -> Option<Code> {
    let upper = s.to_ascii_uppercase();
    if upper.len() == 1 {
        let c = upper.chars().next()?;
        if ('A'..='Z').contains(&c) {
            return key_letter(c);
        }
        if ('0'..='9').contains(&c) {
            return digit(c);
        }
        return None;
    }
    if let Some(rest) = upper.strip_prefix('F') {
        let n: u8 = rest.parse().ok()?;
        return function_key(n);
    }
    None
}

fn key_letter(c: char) -> Option<Code> {
    Some(match c {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'C' => Code::KeyC,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        _ => return None,
    })
}

fn digit(c: char) -> Option<Code> {
    Some(match c {
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        _ => return None,
    })
}

fn function_key(n: u8) -> Option<Code> {
    Some(match n {
        1 => Code::F1,
        2 => Code::F2,
        3 => Code::F3,
        4 => Code::F4,
        5 => Code::F5,
        6 => Code::F6,
        7 => Code::F7,
        8 => Code::F8,
        9 => Code::F9,
        10 => Code::F10,
        11 => Code::F11,
        12 => Code::F12,
        _ => return None,
    })
}
