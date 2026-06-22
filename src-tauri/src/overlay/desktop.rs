//! Desktop pop-out overlay window (monitor / companion screen).

use std::sync::Arc;

use tauri::{AppHandle, Manager, WebviewWindow, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::commands::AppState;
use crate::settings::{load_settings, save_settings};

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
    let win = WebviewWindowBuilder::new(
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

    // Remember where the user left the overlay so it reopens in the same spot.
    let app_handle = app.clone();
    let event_win = win.clone();
    win.on_window_event(move |event| {
        if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
            save_overlay_geometry(&app_handle, &event_win);
        }
    });

    Ok(())
}

fn save_overlay_geometry(app: &AppHandle, win: &WebviewWindow) {
    let (Ok(pos), Ok(size)) = (win.outer_position(), win.inner_size()) else {
        return;
    };
    let state = app.state::<Arc<AppState>>();
    let mut settings = state.settings.lock();
    settings.overlay_x = pos.x;
    settings.overlay_y = pos.y;
    settings.overlay_width = size.width;
    settings.overlay_height = size.height;
    let _ = save_settings(&settings);
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
