mod error;
mod models;
mod services;
mod commands;
mod state;
mod utils;

use tauri::Manager;
use state::AppState;
use services::config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let launcher_config = config::load_config().unwrap_or_default();

    tauri::Builder::default()
        .manage(AppState::new(launcher_config))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_java_path,
            commands::set_game_directory,
            commands::detect_java,
            commands::set_version_isolation,
            commands::list_local_versions,
            commands::list_accounts,
            commands::add_offline_account,
            commands::select_account,
            commands::remove_account,
            commands::launch_game,
            commands::kill_game,
            commands::get_launch_command,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_maximizable(false);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}