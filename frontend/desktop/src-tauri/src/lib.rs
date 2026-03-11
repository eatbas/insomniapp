mod commands;
mod disguise;
mod idle;
mod keepawake;
mod meeting;
mod platform;
mod state;
mod tray;

use state::AppState;
use tauri::WindowEvent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::show_main_window(app);
        }))
        .manage(AppState::default())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide both main and disguise windows instead of destroying them
                if window.label() == "main" || window.label() == "disguise" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::toggle_enabled,
            commands::update_settings,
            commands::open_disguise_window,
            commands::get_disguise_state,
            commands::list_running_apps,
            commands::apply_disguise,
            commands::reset_disguise,
            commands::debug_log,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            let handle = app.handle().clone();
            disguise::initialize(&handle);
            platform::init_display_state_monitor();
            keepawake::start_engine(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
