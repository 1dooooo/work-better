//! 窗口管理命令

use tauri::Manager;

/// 显示主窗口并聚焦
///
/// 用于从 MenuBar 跳转到主窗口（如点击通知时）。
#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 检查主窗口是否存在且可用
///
/// MenuBar.tsx 中用于确认主窗口存在后再 emit 导航事件。
/// 返回 true 表示主窗口可用，false 表示不可用。
#[tauri::command]
pub fn get_main_window(app: tauri::AppHandle) -> bool {
    app.get_webview_window("main").is_some()
}
