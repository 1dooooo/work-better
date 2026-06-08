//! Work Better Tauri 应用

mod commands;

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

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

            // 异步注册内置采集器
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                commands::collectors::register_builtin_collectors().await;
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
            commands::settings::get_feishu_mode,
            commands::settings::save_feishu_mode,
            commands::settings::get_feishu_chat_id,
            commands::settings::save_feishu_chat_id,
            commands::settings::get_storage_config,
            commands::settings::save_storage_config,
            commands::notify::send_notification,
            commands::notify::get_pending_notifications,
            commands::capture::take_screenshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
