//! 快速捕获窗口命令

use tauri::{AppHandle, Manager};

/// 显示快速捕获窗口
#[tauri::command]
pub async fn show_capture_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 隐藏快速捕获窗口
#[tauri::command]
pub async fn hide_capture_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
