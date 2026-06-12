//! Work Better Tauri 应用

mod commands;

use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use wb_collector::collector_task::CollectorTask;
use wb_storage::sqlite::audit_log::ExecutionLogInsert;

/// 设置 macOS 菜单栏托盘
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // 构建托盘菜单
    let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(handle)?;
    let capture_item = MenuItemBuilder::with_id("capture", "快速捕获").build(handle)?;
    let screenshot_item = MenuItemBuilder::with_id("screenshot", "截图").build(handle)?;
    let collect_item = MenuItemBuilder::with_id("collect", "采集飞书").build(handle)?;
    let quit_item = PredefinedMenuItem::quit(handle, Some("退出"))?;

    let menu = MenuBuilder::new(handle)
        .item(&show_item)
        .item(&capture_item)
        .item(&screenshot_item)
        .separator()
        .item(&collect_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Work Better")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("tray") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        // 定位到菜单栏下方
                        if let Some(tray_rect) = tray.rect().ok().flatten() {
                            if let (tauri::Position::Physical(pos), tauri::Size::Physical(size)) =
                                (tray_rect.position, tray_rect.size)
                            {
                                let _ = window.set_position(tauri::Position::Physical(
                                    tauri::PhysicalPosition {
                                        x: pos.x + (size.width as i32 / 2) - 180,
                                        y: pos.y + size.height as i32 + 4,
                                    },
                                ));
                            }
                        }
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "capture" => {
                    if let Some(window) = app.get_webview_window("capture") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "screenshot" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = commands::capture::take_screenshot(handle).await;
                    });
                }
                "collect" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match commands::collect::trigger_feishu_collect(handle, None, None).await {
                            Ok(count) => {
                                eprintln!("[tray] 飞书采集完成: {} 条事件", count);
                            }
                            Err(e) => {
                                eprintln!("[tray] 飞书采集失败: {}", e);
                            }
                        }
                    });
                }
                _ => {}
            }
        })
        .build(handle)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            commands::events::init_event_log(app.handle());
            commands::audit::init_audit_log(app.handle())
                .map_err(|e| e.to_string())?;

            // 初始化 Obsidian vault 目录结构
            let vault_config = commands::settings::load_config_for_collect()
                .unwrap_or_default();
            let vault_path = &vault_config.storage.vault_path;
            if !vault_path.is_empty() {
                // 展开 ~ 符号为用户主目录
                let expanded_path = if vault_path.starts_with("~") {
                    let home = std::env::var("HOME").unwrap_or_default();
                    vault_path.replacen("~", &home, 1)
                } else {
                    vault_path.clone()
                };
                match wb_storage::obsidian::VaultManager::new(&expanded_path) {
                    Ok(_) => eprintln!("[vault] Initialized at: {}", expanded_path),
                    Err(e) => eprintln!("[vault] Failed to initialize: {}", e),
                }
            }

            // 初始化任务管理器
            commands::tasks::init_task_manager();

            // 异步注册内置采集器并启动调度器
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 注册采集器
                commands::collectors::register_builtin_collectors().await;

                // 获取采集器管理器
                let manager = commands::collectors::get_collector_manager();

                // 创建调度器并注册采集任务
                let scheduler = commands::scheduler::get_scheduler();

                // 注册执行日志回调：任务完成后写入 execution_logs 表
                scheduler.set_on_complete(Arc::new(|result| {
                    let status = match result.status {
                        wb_scheduler::task::TaskStatus::Success => "Success",
                        wb_scheduler::task::TaskStatus::Failed => "Failed",
                        wb_scheduler::task::TaskStatus::Timeout => "Timeout",
                        wb_scheduler::task::TaskStatus::Aborted => "Failed",
                    };
                    let record = ExecutionLogInsert {
                        task_id: result.task_id.clone(),
                        task_name: result.task_name.clone(),
                        status: status.to_string(),
                        started_at: result.started_at.to_rfc3339(),
                        finished_at: result.finished_at.to_rfc3339(),
                        duration_ms: result.duration_ms,
                        output: Some(result.summary.clone()),
                        error: result.error.clone(),
                    };
                    if let Some(store) = commands::audit::get_audit_log() {
                        // 在新 tokio task 中执行异步写入，避免阻塞调度器
                        tokio::spawn(async move {
                            let guard = store.lock().await;
                            if let Err(e) = guard.insert_execution_log(&record) {
                                eprintln!("[audit] Failed to write execution log: {}", e);
                            }
                        });
                    }
                })).await;

                // 按照产品设计注册采集任务
                // 参考: docs/architecture/modules/scheduler.md

                // C-02: 飞书任务同步 - 每 30 分钟
                let feishu_task = Arc::new(CollectorTask::new(
                    Arc::clone(manager),
                    "feishu".to_string(),
                    "C-02 飞书任务同步".to_string(),
                    1800, // 30 分钟
                ));
                scheduler.register_with_interval(feishu_task, 1800).await;

                // C-04: 浏览器历史采样 - 每 15 分钟
                let browser_task = Arc::new(CollectorTask::new(
                    Arc::clone(manager),
                    "system.browser_history".to_string(),
                    "C-04 浏览器历史采样".to_string(),
                    900, // 15 分钟
                ));
                scheduler.register_with_interval(browser_task, 900).await;

                // 应用切换采集 - 每 5 分钟（采样，非实时）
                // 设计要求"停留 > 30 秒才记录"，这里采用 5 分钟采样间隔
                let app_switch_task = Arc::new(CollectorTask::new(
                    Arc::clone(manager),
                    "system.app_switch".to_string(),
                    "应用切换采样".to_string(),
                    300, // 5 分钟
                ));
                scheduler.register_with_interval(app_switch_task, 300).await;

                // 启动调度器
                scheduler.start().await;

                eprintln!("[scheduler] Started with collector tasks:");
                eprintln!("[scheduler] - C-02 feishu: every 30 minutes");
                eprintln!("[scheduler] - C-04 browser: every 15 minutes");
                eprintln!("[scheduler] - app_switch: every 5 minutes (sampling)");

                let _ = handle;
            });

            // macOS 菜单栏托盘
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::events::get_events,
            commands::events::get_unprocessed_count,
            commands::events::mark_event_processed,
            commands::events::process_event,
            commands::events::trigger_batch_process,
            commands::collect::trigger_feishu_collect,
            commands::collect::trigger_manual_capture,
            commands::collectors::list_collectors,
            commands::collectors::enable_collector,
            commands::collectors::disable_collector,
            commands::collectors::enable_collector_group,
            commands::collectors::disable_collector_group,
            commands::collectors::check_collector_health,
            commands::collectors::get_collector_statuses,
            commands::collectors::get_collector_groups,
            commands::scheduler::list_scheduled_tasks,
            commands::scheduler::pause_scheduler,
            commands::scheduler::resume_scheduler,
            commands::scheduler::is_scheduler_paused,
            commands::capture::show_capture_window,
            commands::capture::hide_capture_window,
            commands::settings::get_model_config,
            commands::settings::save_model_config,
            commands::settings::get_feishu_mode,
            commands::settings::save_feishu_mode,
            commands::settings::get_feishu_chat_id,
            commands::settings::save_feishu_chat_id,
            commands::settings::get_storage_config,
            commands::settings::save_storage_config,
            commands::notify::send_notification,
            commands::notify::get_pending_notifications,
            commands::capture::take_screenshot,
            commands::audit::get_processing_audits,
            commands::audit::get_execution_logs,
            commands::audit::get_audit_summary,
            commands::settings::get_developer_mode,
            commands::settings::save_developer_mode,
            commands::settings::list_models,
            commands::settings::test_model,
            commands::tasks::discover_tasks_from_text,
            commands::tasks::get_pending_tasks,
            commands::tasks::confirm_pending_task,
            commands::tasks::reject_pending_task,
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
