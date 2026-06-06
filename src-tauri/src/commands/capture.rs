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

/// 截图捕获（macOS screencapture）
///
/// 调用系统截图工具，截图后显示捕获窗口。
/// `-i` 交互模式，`-c` 写入剪贴板。
#[tauri::command]
pub async fn take_screenshot(app: AppHandle) -> Result<(), String> {
    use std::time::Duration;
    use tokio::process::Command;

    let status = tokio::time::timeout(
        Duration::from_secs(60),
        Command::new("screencapture")
            .args(["-i", "-c"])
            .status(),
    )
    .await
    .map_err(|_| "截图超时，请重试".to_string())?
    .map_err(|e| format!("screencapture 执行失败: {e}"))?;

    if status.success() {
        if let Some(window) = app.get_webview_window("capture") {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("截图已取消".to_string())
    }
}
