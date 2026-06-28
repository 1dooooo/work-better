//! 测试模式管理命令
//!
//! 提供测试模式的启用/禁用和测试数据清理功能。
//! 测试模式下，手动捕获会跳过 AI 处理，直接创建事件。

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 测试模式状态
///
/// 通过 Tauri `manage()` 注入，所有命令函数通过 `State<'_, TestModeState>` 提取。
#[derive(Clone)]
pub struct TestModeState {
    /// 测试模式是否启用
    enabled: Arc<AtomicBool>,
    /// 测试数据目录（用于清理）
    data_dir: Arc<Mutex<Option<PathBuf>>>,
}

impl TestModeState {
    /// 创建新的测试模式状态（默认禁用）
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            data_dir: Arc::new(Mutex::new(None)),
        }
    }

    /// 检查测试模式是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// 获取测试数据目录
    pub async fn get_data_dir(&self) -> Option<PathBuf> {
        self.data_dir.lock().await.clone()
    }
}

impl Default for TestModeState {
    fn default() -> Self {
        Self::new()
    }
}

/// 启用/禁用测试模式
///
/// # Arguments
/// - `enabled`: 是否启用测试模式
/// - `data_dir`: 测试数据目录路径（可选）
#[tauri::command]
pub async fn set_test_mode(
    test_mode: tauri::State<'_, TestModeState>,
    enabled: bool,
    data_dir: Option<String>,
) -> Result<(), String> {
    test_mode.enabled.store(enabled, Ordering::Relaxed);

    let dir = data_dir.map(PathBuf::from);
    *test_mode.data_dir.lock().await = dir;

    eprintln!(
        "[test_mode] {} with data dir: {:?}",
        if enabled { "enabled" } else { "disabled" },
        test_mode.get_data_dir().await
    );

    Ok(())
}

/// 清理测试数据并重置测试模式
///
/// 删除测试数据目录（如果存在），并将测试模式重置为禁用状态。
#[tauri::command]
pub async fn cleanup_test_data(
    test_mode: tauri::State<'_, TestModeState>,
) -> Result<(), String> {
    if let Some(data_dir) = test_mode.get_data_dir().await {
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)
                .map_err(|e| format!("Failed to cleanup test data: {}", e))?;
            eprintln!("[test_mode] Cleaned up test data directory: {:?}", data_dir);
        }
    }

    // 重置测试模式
    test_mode.enabled.store(false, Ordering::Relaxed);
    *test_mode.data_dir.lock().await = None;

    eprintln!("[test_mode] Reset to disabled");

    Ok(())
}
