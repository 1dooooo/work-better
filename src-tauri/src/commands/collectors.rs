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

    // 分组启用状态（默认启用）
    let feishu_group_enabled = config
        .as_ref()
        .and_then(|c| c.collectors.group_enabled.get("feishu").copied())
        .unwrap_or(true);

    let system_group_enabled = config
        .as_ref()
        .and_then(|c| c.collectors.group_enabled.get("system").copied())
        .unwrap_or(true);

    // ===== 飞书采集器 =====
    // 已实现 Collector trait 的模块直接使用，未实现的使用 wrapper
    let feishu_collectors: Vec<(String, std::sync::Arc<dyn wb_collector::traits::Collector>)> = vec![
        // 消息采集器（wrapper）
        ("feishu".to_string(), std::sync::Arc::new(wb_collector::feishu::collector::FeishuCollector::new(chat_id.clone(), 50))),
        // 以下模块已实现 Collector trait，直接使用
        ("feishu.bitable".to_string(), std::sync::Arc::new(wb_collector::feishu::bitable::FeishuBitableCollector)),
        ("feishu.meetings".to_string(), std::sync::Arc::new(wb_collector::feishu::meetings::FeishuMeetingCollector)),
        ("feishu.emails".to_string(), std::sync::Arc::new(wb_collector::feishu::emails::FeishuEmailCollector)),
        ("feishu.minutes".to_string(), std::sync::Arc::new(wb_collector::feishu::minutes::FeishuMinutesCollector)),
        ("feishu.okr".to_string(), std::sync::Arc::new(wb_collector::feishu::okr::FeishuOkrCollector)),
        ("feishu.wiki".to_string(), std::sync::Arc::new(wb_collector::feishu::wiki::FeishuWikiCollector)),
        ("feishu.spreadsheets".to_string(), std::sync::Arc::new(wb_collector::feishu::spreadsheets::FeishuSpreadsheetCollector)),
        // 以下模块需要 wrapper
        ("feishu.docs".to_string(), std::sync::Arc::new(wb_collector::feishu::wrappers::FeishuDocsCollectorWrapper::new(50))),
        ("feishu.projects".to_string(), std::sync::Arc::new(wb_collector::feishu::wrappers::FeishuProjectsCollectorWrapper::new(50))),
        ("feishu.calendar".to_string(), std::sync::Arc::new(wb_collector::feishu::wrappers::FeishuCalendarCollectorWrapper::new(50))),
        ("feishu.approvals".to_string(), std::sync::Arc::new(wb_collector::feishu::wrappers::FeishuApprovalsCollectorWrapper::new(50))),
    ];

    for (id, collector) in feishu_collectors {
        let enabled = config
            .as_ref()
            .and_then(|c| c.collectors.enabled.get(&id).copied())
            .unwrap_or(true); // 默认启用
        manager.register(collector).await;
        if !enabled {
            manager.disable(&id).await;
        }
    }

    // ===== 系统采集器 =====
    let system_collectors: Vec<(String, std::sync::Arc<dyn wb_collector::traits::Collector>)> = vec![
        ("system.app_switch".to_string(), std::sync::Arc::new(wb_collector::system::app_switch::AppSwitchCollector::new())),
        ("system.browser_history".to_string(), std::sync::Arc::new(wb_collector::system::browser::BrowserHistoryCollector::new())),
    ];

    for (id, collector) in system_collectors {
        let enabled = config
            .as_ref()
            .and_then(|c| c.collectors.enabled.get(&id).copied())
            .unwrap_or(true); // 默认启用
        manager.register(collector).await;
        if !enabled {
            manager.disable(&id).await;
        }
    }

    // 根据配置文件设置分组启用状态
    if !feishu_group_enabled {
        manager.disable_group("feishu").await;
    }
    if !system_group_enabled {
        manager.disable_group("system").await;
    }

    eprintln!("[collectors] Registered 14 collectors (12 feishu + 2 system)");
    eprintln!("[collectors] Groups: feishu={}, system={}", feishu_group_enabled, system_group_enabled);
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

/// 验证采集器 ID 是否有效
///
/// 检查 ID 是否为已注册的采集器，防止无效或恶意输入。
fn validate_collector_id(id: &str) -> Result<(), String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err("采集器 ID 不能为空".to_string());
    }
    // 允许的字符：字母、数字、下划线、点
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return Err(format!("采集器 ID 包含非法字符: '{}'", id));
    }
    Ok(())
}

/// 验证采集器分组 ID 是否有效
///
/// 只允许已知的分组：feishu、system。
fn validate_group_id(group_id: &str) -> Result<(), String> {
    let trimmed = group_id.trim();
    if trimmed.is_empty() {
        return Err("分组 ID 不能为空".to_string());
    }
    // 只允许已知分组
    if !["feishu", "system"].contains(&trimmed) {
        return Err(format!("未知的分组 ID: '{}'", group_id));
    }
    Ok(())
}

/// 启用指定采集器
///
/// 同时更新内存中的 CollectorManager 和配置文件，确保状态一致。
#[tauri::command]
pub async fn enable_collector(id: String) -> Result<(), String> {
    validate_collector_id(&id)?;

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
    validate_collector_id(&id)?;

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

/// 采集器分组信息（前端 DTO）
#[derive(serde::Serialize)]
pub struct CollectorGroupDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub collectors: Vec<CollectorDetailedStatus>,
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

/// 启用采集器分组
#[tauri::command]
pub async fn enable_collector_group(group_id: String) -> Result<(), String> {
    validate_group_id(&group_id)?;

    let manager = get_collector_manager();
    manager.enable_group(&group_id).await;

    // 持久化到配置文件
    let mut config = super::settings::load_config()?;
    config
        .collectors
        .group_enabled
        .insert(group_id, true);
    super::settings::save_config_pub(&config)?;

    Ok(())
}

/// 禁用采集器分组
#[tauri::command]
pub async fn disable_collector_group(group_id: String) -> Result<(), String> {
    validate_group_id(&group_id)?;

    let manager = get_collector_manager();
    manager.disable_group(&group_id).await;

    // 持久化到配置文件
    let mut config = super::settings::load_config()?;
    config
        .collectors
        .group_enabled
        .insert(group_id, false);
    super::settings::save_config_pub(&config)?;

    Ok(())
}

/// 获取采集器分组信息
#[tauri::command]
pub async fn get_collector_groups() -> Result<Vec<CollectorGroupDto>, String> {
    let manager = get_collector_manager();
    let scheduler = super::scheduler::get_scheduler();

    let groups = manager.get_groups().await;
    let mut result = Vec::with_capacity(groups.len());

    for group in groups {
        let mut collectors = Vec::with_capacity(group.collector_ids.len());

        for id in &group.collector_ids {
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
                "feishu" => "消息",
                "feishu.docs" => "文档",
                "feishu.projects" => "项目",
                "feishu.calendar" => "日历",
                "feishu.meetings" => "会议",
                "feishu.emails" => "邮箱",
                "feishu.approvals" => "审批",
                "feishu.okr" => "OKR",
                "feishu.bitable" => "多维表格",
                "feishu.spreadsheets" => "电子表格",
                "feishu.wiki" => "知识库",
                "feishu.minutes" => "妙记",
                "system.app_switch" => "前台应用",
                "system.browser_history" => "浏览历史",
                _ => id.as_str(),
            };

            collectors.push(CollectorDetailedStatus {
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

        result.push(CollectorGroupDto {
            id: group.id,
            name: group.name,
            enabled: group.enabled,
            collectors,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== validate_collector_id 测试 =====

    #[test]
    fn test_validate_collector_id_empty_string() {
        assert!(validate_collector_id("").is_err());
    }

    #[test]
    fn test_validate_collector_id_whitespace_only() {
        assert!(validate_collector_id("   ").is_err());
    }

    #[test]
    fn test_validate_collector_id_valid_simple() {
        assert!(validate_collector_id("feishu").is_ok());
    }

    #[test]
    fn test_validate_collector_id_valid_with_dot() {
        assert!(validate_collector_id("feishu.docs").is_ok());
    }

    #[test]
    fn test_validate_collector_id_valid_with_underscore() {
        assert!(validate_collector_id("system_app").is_ok());
    }

    #[test]
    fn test_validate_collector_id_valid_mixed() {
        assert!(validate_collector_id("feishu.bitable").is_ok());
        assert!(validate_collector_id("system.browser_history").is_ok());
    }

    #[test]
    fn test_validate_collector_id_invalid_space() {
        assert!(validate_collector_id("feishu docs").is_err());
    }

    #[test]
    fn test_validate_collector_id_invalid_slash() {
        assert!(validate_collector_id("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_collector_id_invalid_semicolon() {
        assert!(validate_collector_id("feishu;rm -rf /").is_err());
    }

    #[test]
    fn test_validate_collector_id_invalid_quote() {
        assert!(validate_collector_id("feishu\"").is_err());
    }

    // ===== validate_group_id 测试 =====

    #[test]
    fn test_validate_group_id_empty_string() {
        assert!(validate_group_id("").is_err());
    }

    #[test]
    fn test_validate_group_id_whitespace_only() {
        assert!(validate_group_id("   ").is_err());
    }

    #[test]
    fn test_validate_group_id_valid_feishu() {
        assert!(validate_group_id("feishu").is_ok());
    }

    #[test]
    fn test_validate_group_id_valid_system() {
        assert!(validate_group_id("system").is_ok());
    }

    #[test]
    fn test_validate_group_id_invalid_unknown() {
        assert!(validate_group_id("unknown").is_err());
    }

    #[test]
    fn test_validate_group_id_invalid_path_traversal() {
        assert!(validate_group_id("../feishu").is_err());
    }

    #[test]
    fn test_validate_group_id_invalid_injection() {
        assert!(validate_group_id("feishu OR 1=1").is_err());
    }
}
