//! Work Better Tauri 应用

mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            commands::events::init_event_log(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::events::get_events,
            commands::events::get_unprocessed_count,
            commands::events::mark_event_processed,
            commands::collect::trigger_feishu_collect,
            commands::collect::trigger_manual_capture,
            commands::collectors::list_collectors,
            commands::collectors::enable_collector,
            commands::collectors::disable_collector,
            commands::collectors::check_collector_health,
            commands::scheduler::list_scheduled_tasks,
            commands::scheduler::pause_scheduler,
            commands::scheduler::resume_scheduler,
            commands::scheduler::is_scheduler_paused,
            commands::capture::show_capture_window,
            commands::capture::hide_capture_window,
            commands::settings::get_model_config,
            commands::settings::save_model_config,
            commands::settings::get_collector_statuses,
            commands::settings::get_storage_config,
            commands::settings::save_storage_config,
            commands::notify::send_notification,
            commands::notify::get_pending_notifications,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
