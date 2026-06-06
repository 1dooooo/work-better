//! 采集器管理 Tauri 命令

use std::sync::OnceLock;
use wb_collector::manager::CollectorManager;

/// 全局 CollectorManager 实例
static COLLECTOR_MANAGER: OnceLock<CollectorManager> = OnceLock::new();

/// 获取全局 CollectorManager 实例。
///
/// 首次调用时自动初始化。
pub fn get_collector_manager() -> &'static CollectorManager {
    COLLECTOR_MANAGER.get_or_init(CollectorManager::new)
}

/// 注册内置采集器到全局 CollectorManager。
///
/// 应在 Tauri setup 阶段调用，确保采集器在任何命令调用之前就绪。
pub async fn register_builtin_collectors() {
    let manager = get_collector_manager();

    // 从配置读取 chat_id，注册飞书采集器
    let chat_id = super::settings::load_config_for_collect()
        .ok()
        .and_then(|c| c.collectors.feishu_chat_id)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "oc_default".to_string());

    let feishu_collector = std::sync::Arc::new(
        wb_collector::feishu::collector::FeishuCollector::new(chat_id, 50),
    );
    manager.register(feishu_collector).await;
}

/// 采集器健康信息（序列化返回给前端）
#[derive(serde::Serialize)]
pub struct CollectorHealthInfo {
    pub level: String,
    pub message: Option<String>,
    pub error_count: u32,
}

/// 列出所有已注册的采集器 ID
#[tauri::command]
pub async fn list_collectors() -> Result<Vec<String>, String> {
    let manager = get_collector_manager();
    Ok(manager.list().await)
}

/// 启用指定采集器
#[tauri::command]
pub async fn enable_collector(id: String) -> Result<(), String> {
    let manager = get_collector_manager();
    manager.enable(&id).await;
    Ok(())
}

/// 禁用指定采集器
#[tauri::command]
pub async fn disable_collector(id: String) -> Result<(), String> {
    let manager = get_collector_manager();
    manager.disable(&id).await;
    Ok(())
}

/// 查询指定采集器的健康状态
#[tauri::command]
pub async fn check_collector_health(id: String) -> Result<CollectorHealthInfo, String> {
    let manager = get_collector_manager();
    let status = manager
        .health_check(&id)
        .await
        .ok_or_else(|| format!("Collector '{}' not found", id))?;

    let level = match status.level {
        wb_collector::traits::HealthLevel::Healthy => "healthy",
        wb_collector::traits::HealthLevel::Degraded => "degraded",
        wb_collector::traits::HealthLevel::Unhealthy => "unhealthy",
    }
    .to_string();

    Ok(CollectorHealthInfo {
        level,
        message: status.message,
        error_count: status.error_count,
    })
}
