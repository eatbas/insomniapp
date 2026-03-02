mod commands;
mod idle;
mod keepawake;
mod meeting;
mod platform;
mod state;
mod tray;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::toggle_enabled,
            commands::update_settings,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            let handle = app.handle().clone();
            keepawake::start_engine(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
