//! 采集器管理 Tauri 命令

use std::sync::{Arc, OnceLock};
use wb_collector::manager::CollectorManager;

/// 全局 CollectorManager 实例
static COLLECTOR_MANAGER: OnceLock<Arc<CollectorManager>> = OnceLock::new();

/// 获取全局 CollectorManager 实例。
///
/// 首次调用时自动初始化。
pub fn get_collector_manager() -> &'static Arc<CollectorManager> {
    COLLECTOR_MANAGER.get_or_init(|| Arc::new(CollectorManager::new()))
}

/// 注册内置采集器到全局 CollectorManager。
///
/// 应在 Tauri setup 阶段调用，确保采集器在任何命令调用之前就绪。
/// 从配置文件读取启用状态，确保与前端显示一致。
pub async fn register_builtin_collectors() {
    let manager = get_collector_manager();

    // 从配置读取 chat_id 和启用状态
    let config = super::settings::load_config_for_collect().ok();
    let chat_id = config
        .as_ref()
        .and_then(|c| c.collectors.feishu_chat_id.as_deref())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("oc_default")
        .to_string();

    let feishu_enabled = config
        .as_ref()
        .and_then(|c| c.collectors.enabled.get("feishu").copied())
        .unwrap_or(false);

    let app_switch_enabled = config
        .as_ref()
        .and_then(|c| c.collectors.enabled.get("system.app_switch").copied())
        .unwrap_or(true); // 默认启用

    let browser_enabled = config
        .as_ref()
        .and_then(|c| c.collectors.enabled.get("system.browser_history").copied())
        .unwrap_or(true); // 默认启用

    // 注册飞书消息采集器
    let feishu_collector = std::sync::Arc::new(
        wb_collector::feishu::collector::FeishuCollector::new(chat_id, 50),
    );
    manager.register(feishu_collector).await;

    // 注册系统采集器
    let app_switch_collector = std::sync::Arc::new(
        wb_collector::system::app_switch::AppSwitchCollector::new(),
    );
    manager.register(app_switch_collector).await;

    let browser_collector = std::sync::Arc::new(
        wb_collector::system::browser::BrowserHistoryCollector::new(),
    );
    manager.register(browser_collector).await;

    // 根据配置文件设置启用状态
    if !feishu_enabled {
        manager.disable("feishu").await;
    }
    if !app_switch_enabled {
        manager.disable("system.app_switch").await;
    }
    if !browser_enabled {
        manager.disable("system.browser_history").await;
    }

    eprintln!("[collectors] Registered 3 collectors: feishu, system.app_switch, system.browser_history");
    eprintln!("[collectors] Enabled: feishu={}, app_switch={}, browser={}",
        feishu_enabled, app_switch_enabled, browser_enabled);
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
///
/// 同时更新内存中的 CollectorManager 和配置文件，确保状态一致。
#[tauri::command]
pub async fn enable_collector(id: String) -> Result<(), String> {
    // 1. 更新内存状态
    let manager = get_collector_manager();
    manager.enable(&id).await;

    // 2. 持久化到配置文件
    let mut config = super::settings::load_config()?;
    config.collectors.enabled.insert(id, true);
    super::settings::save_config_pub(&config)?;

    Ok(())
}

/// 禁用指定采集器
///
/// 同时更新内存中的 CollectorManager 和配置文件，确保状态一致。
#[tauri::command]
pub async fn disable_collector(id: String) -> Result<(), String> {
    // 1. 更新内存状态
    let manager = get_collector_manager();
    manager.disable(&id).await;

    // 2. 持久化到配置文件
    let mut config = super::settings::load_config()?;
    config.collectors.enabled.insert(id, false);
    super::settings::save_config_pub(&config)?;

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

/// 采集器详细状态信息（包含调度器状态）
#[derive(serde::Serialize)]
pub struct CollectorDetailedStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub health_level: String,
    pub health_message: Option<String>,
    pub last_run: Option<String>,
    pub last_status: Option<String>,
    pub last_summary: Option<String>,
}

/// 获取所有采集器的详细状态
///
/// 包含启用状态、健康状态、最近执行结果等信息。
/// 用于前端展示采集器的实时工作状态。
#[tauri::command]
pub async fn get_collector_statuses() -> Result<Vec<CollectorDetailedStatus>, String> {
    let manager = get_collector_manager();
    let scheduler = super::scheduler::get_scheduler();

    let collector_ids = manager.list().await;
    let mut statuses = Vec::with_capacity(collector_ids.len());

    for id in &collector_ids {
        let enabled = manager.is_enabled(id).await;
        let health = manager.health_check(id).await;

        let (health_level, health_message) = match health {
            Some(status) => {
                let level = match status.level {
                    wb_collector::traits::HealthLevel::Healthy => "healthy",
                    wb_collector::traits::HealthLevel::Degraded => "degraded",
                    wb_collector::traits::HealthLevel::Unhealthy => "unhealthy",
                };
                (level.to_string(), status.message)
            }
            None => ("unknown".to_string(), None),
        };

        // 从调度器获取最近执行结果
        let task_info = scheduler.get_task_info(id).await;
        let last_result = scheduler.get_last_result(id).await;

        let (last_run, last_status, last_summary) = if let Some(result) = last_result {
            (
                Some(result.finished_at.to_rfc3339()),
                Some(format!("{:?}", result.status)),
                Some(result.summary),
            )
        } else if let Some(info) = task_info {
            (
                info.last_run.map(|t| t.to_rfc3339()),
                info.last_status.map(|s| format!("{:?}", s)),
                None,
            )
        } else {
            (None, None, None)
        };

        // 获取采集器名称
        let name = match id.as_str() {
            "feishu" => "飞书消息",
            "system.app_switch" => "前台应用",
            "system.browser_history" => "浏览历史",
            _ => id.as_str(),
        };

        statuses.push(CollectorDetailedStatus {
            id: id.clone(),
            name: name.to_string(),
            enabled,
            health_level,
            health_message,
            last_run,
            last_status,
            last_summary,
        });
    }

    Ok(statuses)
}
