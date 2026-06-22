//! Desktop pop-out overlay window (monitor / companion screen).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::settings::load_settings;

const OVERLAY_LABEL: &str = "live-overlay";

pub fn open_desktop_overlay(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(OVERLAY_LABEL).is_some() {
        if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
            let _ = win.show();
            let _ = win.set_focus();
        }
        return Ok(());
    }

    let settings = load_settings();
    WebviewWindowBuilder::new(
        app,
        OVERLAY_LABEL,
        WebviewUrl::App("overlay.html".into()),
    )
    .title("PitWall Overlay")
    .inner_size(settings.overlay_width as f64, settings.overlay_height as f64)
    .position(settings.overlay_x as f64, settings.overlay_y as f64)
    .always_on_top(true)
    .decorations(false)
    .transparent(true)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn close_desktop_overlay(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_desktop_overlay_open(app: &AppHandle) -> bool {
    app.get_webview_window(OVERLAY_LABEL).is_some()
}
