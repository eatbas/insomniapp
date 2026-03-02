use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow,
};

use crate::state::AppState;

const WINDOW_LEFT_MARGIN: i32 = 24;
const WINDOW_BOTTOM_MARGIN: i32 = 72;

fn position_main_window(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };

    let window_size = window.outer_size().or_else(|_| window.inner_size());
    let Ok(window_size) = window_size else {
        return;
    };

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let x = monitor_pos.x + WINDOW_LEFT_MARGIN;
    let y =
        monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - WINDOW_BOTTOM_MARGIN;

    let clamped_x = x.max(monitor_pos.x);
    let clamped_y = y.max(monitor_pos.y);

    let _ = window.set_position(PhysicalPosition::new(clamped_x, clamped_y));
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

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                status.enabled = !status.enabled;
                let _ = app.emit("status-update", status.clone());
            }
            "show" => {
                show_main_window(app);
            }
            "quit" => {
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
                let app = tray.app_handle();
                show_main_window(&app);
            }
        })
        .build(app)?;

    Ok(())
}
