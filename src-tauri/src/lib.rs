//! Work Better Tauri 应用

mod commands;

use std::sync::Arc;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use wb_collector::collector_task::CollectorTask;
use wb_storage::config::AppConfig;
use wb_storage::sqlite::audit_log::ExecutionLogInsert;

/// 将快捷键字符串 key 转为 Code 枚举
fn parse_key_code(key: &str) -> Option<Code> {
    match key {
        "Space" => Some(Code::Space),
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        _ => None,
    }
}

/// 将 modifiers 字符串列表转为 Modifiers 位掩码
fn parse_modifiers(modifiers: &[String]) -> Modifiers {
    let mut mods = Modifiers::empty();
    for m in modifiers {
        match m.as_str() {
            "cmd" => mods |= Modifiers::SUPER,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" => mods |= Modifiers::ALT,
            "ctrl" => mods |= Modifiers::CONTROL,
            _ => {}
        }
    }
    mods
}

/// 注册全局快捷键（从 AppConfig 读取，供 setup 和 save_shortcut_config 复用）
///
/// 先注销所有已注册快捷键，再按最新配置重新注册。
pub fn register_shortcuts(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    // 注销全部已有快捷键
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("注销快捷键失败: {e}"))?;

    let shortcuts: Vec<(String, Modifiers, Code)> = if config.shortcuts.is_empty() {
        vec![
            ("capture".to_string(), Modifiers::SUPER | Modifiers::SHIFT, Code::Space),
            ("screenshot".to_string(), Modifiers::SUPER | Modifiers::SHIFT, Code::KeyS),
        ]
    } else {
        config
            .shortcuts
            .iter()
            .filter_map(|s| {
                // 仅注册 capture 和 screenshot 类型的全局快捷键
                if s.id != "capture" && s.id != "screenshot" {
                    return None;
                }
                let mods = parse_modifiers(&s.modifiers);
                let code = parse_key_code(&s.key)?;
                Some((s.id.clone(), mods, code))
            })
            .collect()
    };

    for (_id, mods, code) in shortcuts {
        let shortcut = Shortcut::new(Some(mods), code);
        app.global_shortcut()
            .register(shortcut)
            .map_err(|e| format!("注册快捷键失败: {e}"))?;
    }

    Ok(())
}

/// 设置 macOS 菜单栏托盘
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // 创建托盘图标（无右键菜单，左键点击显示 MenuBar 面板）
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
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
        .build(handle)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // 根据快捷键的 key code 区分不同操作
                        match shortcut.key {
                            // Cmd+Shift+S → 截图并进入速记窗口
                            tauri_plugin_global_shortcut::Code::KeyS => {
                                let handle = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    let _ = commands::capture::take_screenshot(handle).await;
                                });
                            }
                            // 其他快捷键（默认 Cmd+Shift+Space）→ 切换速记窗口
                            _ => {
                                if let Some(window) = app.get_webview_window("capture") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 初始化 SQLite EventLog
            let db_path = commands::db::resolve_db_path(app.handle())
                .map_err(|e| e.to_string())?;
            eprintln!("[events] DB path: {}", db_path);
            let event_log = wb_storage::SqliteEventLog::new(&db_path)
                .map_err(|e| format!("Failed to initialize EventLog: {}", e))?;

            // 初始化 AuditLogStore
            let audit_conn = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("Failed to open database for audit: {}", e))?;
            wb_storage::sqlite::schema::initialize_schema(&audit_conn)
                .map_err(|e| format!("Failed to initialize audit schema: {}", e))?;
            let audit_log = wb_storage::AuditLogStore::new(audit_conn);

            // 构建 AppState 并注入
            let state = commands::AppState::new(event_log, Some(audit_log));
            app.manage(state);

            // 注入 TestModeState
            app.manage(commands::test_mode::TestModeState::new());

            // 初始化 Obsidian vault 目录结构
            let vault_config = commands::settings::load_config_for_collect()
                .unwrap_or_default();
            let vault_path = &vault_config.storage.vault_path;
            if !vault_path.is_empty() {
                // 展开 ~ 符号为用户主目录
                let expanded_path = if vault_path.starts_with("~") {
                    let home = std::env::var("WORK_BETTER_HOME")
                        .or_else(|_| std::env::var("HOME"))
                        .unwrap_or_default();
                    vault_path.replacen("~", &home, 1)
                } else {
                    vault_path.clone()
                };
                match wb_storage::obsidian::VaultManager::new(&expanded_path) {
                    Ok(_) => eprintln!("[vault] Initialized at: {}", expanded_path),
                    Err(e) => eprintln!("[vault] Failed to initialize: {}", e),
                }
            }

            // 异步注册内置采集器并启动调度器
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<commands::AppState>();

                // 注册采集器
                commands::collectors::register_builtin_collectors(&state).await;

                // 获取采集器管理器和调度器
                let manager = &state.collector_manager;
                let scheduler = &state.scheduler;

                // 注册执行日志回调：任务完成后写入 execution_logs 表
                let cb_handle = handle.clone();
                scheduler.set_on_complete(Arc::new(move |result| {
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
                    // 在新 tokio task 中执行异步写入，避免阻塞调度器
                    let audit_log = cb_handle.state::<commands::AppState>().audit_log.clone();
                    tokio::spawn(async move {
                        if let Some(store) = audit_log {
                            let guard = store.lock().await;
                            if let Err(e) = guard.insert_execution_log(&record) {
                                eprintln!("[audit] Failed to write execution log: {}", e);
                            }
                        }
                    });
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

            // 注册全局快捷键（从用户配置读取，默认 Cmd+Shift+Space + Cmd+Shift+S）
            let config = commands::settings::load_config_for_collect().unwrap_or_default();
            register_shortcuts(app.handle(), &config).map_err(|e| e.to_string())?;

            eprintln!("[app] AppState initialized");

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
            commands::settings::get_shortcut_config,
            commands::settings::save_shortcut_config,
            commands::settings::get_system_status,
            commands::notify::send_notification,
            commands::notify::get_pending_notifications,
            commands::notify::mark_notification_read,
            commands::notify::clear_read_notifications,
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
            commands::window::show_main_window,
            commands::window::get_main_window,
            commands::log::log_message,
            commands::test_mode::set_test_mode,
            commands::test_mode::cleanup_test_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
