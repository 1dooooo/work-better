//! Work Better Tauri 应用

mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::events::get_events,
            commands::events::get_unprocessed_count,
            commands::events::mark_event_processed,
            commands::collect::trigger_feishu_collect,
            commands::collect::trigger_manual_capture,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
