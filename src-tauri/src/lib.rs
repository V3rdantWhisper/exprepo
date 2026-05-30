mod commands;
mod config;
mod env_runner;
mod error;
mod model;
mod repo;
mod secrets;

use std::sync::Mutex;

use tauri::Manager;

use commands::AppState;
use config::AppConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            secrets::init_store();
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&config_dir)?;
            std::fs::create_dir_all(&data_dir)?;

            let config_path = config_dir.join("config.json");
            let envs_path = config_dir.join("environments.json");
            let cfg = AppConfig::load_or_default(&config_path, &data_dir)?;

            app.manage(AppState {
                config: Mutex::new(cfg),
                config_path,
                envs_path,
                runner: Default::default(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_repo_path,
            commands::repo_init,
            commands::repo_clone,
            commands::repo_status,
            commands::repo_commit,
            commands::repo_push,
            commands::repo_pull,
            commands::repo_get_remote,
            commands::repo_set_remote,
            commands::cve_list,
            commands::cve_get,
            commands::cve_create,
            commands::cve_update_meta,
            commands::exp_create,
            commands::exp_update_meta,
            commands::wp_create,
            commands::file_read,
            commands::file_write,
            commands::env_list,
            commands::env_running,
            commands::env_save,
            commands::env_delete,
            commands::env_launch,
            commands::env_stop,
            commands::env_export_guide,
            commands::secret_set_token,
            commands::secret_has_token,
            commands::secret_delete_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
