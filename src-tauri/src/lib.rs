mod commands;
mod deleter;
mod models;
mod rules;
mod scanner;

use std::sync::Arc;

use commands::ScanState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(ScanState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_platform,
            commands::get_scan_rules,
            commands::start_scan,
            commands::get_last_scan_summary,
            commands::delete_selected,
            commands::get_audit_log,
            commands::is_scan_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
