#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
mod commands;
mod content;
mod error;
mod paths;
mod storage;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::profiles_list,
            commands::profiles_create,
            commands::profiles_update,
            commands::profiles_archive,
            commands::profiles_delete,
            commands::profile_touch,
            commands::session_save,
            commands::session_checkpoint,
            commands::checkpoints_list,
            commands::checkpoint_discard,
            commands::history_list,
            commands::history_detail,
            commands::history_delete,
            commands::history_trend,
            commands::settings_get,
            commands::settings_set,
            commands::progress_get,
            commands::backup_create,
            commands::backup_restore,
            commands::export_csv,
            commands::open_data_folder,
            commands::custom_text_list,
            commands::custom_text_save,
            commands::custom_text_delete,
            commands::analysis_get,
            commands::weak_keys,
            commands::practice_text,
            commands::drill_generate
        ])
        .setup(|app| {
            let database = storage::db::Database::open(paths::data_dir()?)
                .map_err(|error| std::io::Error::other(error.message))?;
            app.manage(std::sync::Arc::new(database));
            #[cfg(debug_assertions)]
            let content_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("content");
            #[cfg(not(debug_assertions))]
            let content_root = app.path().resource_dir()?.join("content");
            app.manage(std::sync::Arc::new(content::Content::new(content_root)));
            let window = app.get_webview_window("main").expect("main window exists");
            if window.primary_monitor()?.is_some_and(|monitor| {
                f64::from(monitor.size().width) / monitor.scale_factor() <= 1366.0
            }) {
                window.maximize()?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run SvaType");
}
