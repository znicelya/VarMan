pub mod commands;
pub mod core;
pub mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_env_vars,
            commands::get_env_var,
            commands::preview_changes,
            commands::apply_changes,
            commands::remove_env_var,
            commands::check_permission,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::preview_backup_restore,
            commands::parse_path_var,
            commands::join_path_var,
            commands::restart_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
