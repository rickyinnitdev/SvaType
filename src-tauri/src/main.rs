#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
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
