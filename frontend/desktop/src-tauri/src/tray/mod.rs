//! System tray icon and its menu.
//!
//! Window placement lives in [`layout`] as a pure function. This module is a
//! thin adapter over the Tauri tray, menu, and window APIs, none of which can
//! be constructed without a real windowing system, so it is excluded from
//! coverage: see the coverage policy in `README.md`.

pub mod layout;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow,
};

use crate::{disguise, state::AppState};
use layout::Point;

fn position_main_window(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };

    let Ok(window_size) = window.outer_size().or_else(|_| window.inner_size()) else {
        return;
    };

    let monitor_origin = monitor.position();
    let position = layout::main_window_position(
        Point {
            x: monitor_origin.x,
            y: monitor_origin.y,
        },
        monitor.size().height,
        window_size.height,
    );

    let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
}

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        position_main_window(&window);
        let _ = window.set_focus();
    }
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Disable", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle_item, &show_item, &quit_item])?;

    let tooltip = disguise::DEFAULT_APP_NAME.to_string();

    let _tray = TrayIconBuilder::with_id(disguise::TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                status.toggle_enabled();
                let _ = app.emit("status-update", status.clone());
            }
            "show" => {
                show_main_window(app);
            }
            "quit" => {
                disguise::clear_disguise_on_quit(app);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
