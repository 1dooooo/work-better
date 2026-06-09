//! 设置命令

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use wb_collector::runner;
use wb_storage::config::{AppConfig, CollectorConfig};

/// lark-cli 工具路径
const LARK_CLI: &str = "/opt/homebrew/bin/lark-cli";

/// 模型配置（前端 DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub token_budget: u32,
}

/// 采集器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub healthy: bool,
}

/// 存储配置（前端 DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub vault_path: String,
    pub db_path: String,
}

/// 配置文件路径：~/.work-better/config.json
fn config_path() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|e| format!("无法获取 HOME 环境变量: {e}"))?;
    Ok(PathBuf::from(home).join(".work-better").join("config.json"))
}

/// 从配置文件加载 AppConfig，文件不存在时返回默认值
///
/// 此函数为 `pub(crate)` 可见性，供 `collect` 和 `collectors` 模块使用。
pub(crate) fn load_config() -> Result<AppConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {e}"))?;
    AppConfig::from_json(&content)
        .map_err(|e| format!("解析配置文件失败: {e}"))
}

/// 对外暴露的配置加载函数（用于采集模块）
pub fn load_config_for_collect() -> Result<AppConfig, String> {
    load_config()
}

/// 将 AppConfig 保存到配置文件
fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let json = config
        .to_json()
        .map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("写入配置文件失败: {e}"))
}

/// 获取模型配置
#[tauri::command]
pub async fn get_model_config() -> Result<ModelConfig, String> {
    let app_config = load_config()?;
    Ok(ModelConfig {
        api_endpoint: app_config.model.api_endpoint,
        api_key: String::new(), // 出于安全考虑，不从文件回传 api_key
        token_budget: app_config.model.token_budget,
    })
}

/// 保存模型配置
#[tauri::command]
pub async fn save_model_config(config: ModelConfig) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.model.api_endpoint = config.api_endpoint;
    app_config.model.token_budget = config.token_budget;
    // api_key 单独处理（暂不持久化到 config.json，避免明文存储风险）
    save_config(&app_config)
}

/// 获取采集器状态列表
#[tauri::command]
pub async fn get_collector_statuses() -> Result<Vec<CollectorStatus>, String> {
    let app_config = load_config()?;
    let statuses = build_collector_statuses(&app_config.collectors);
    Ok(statuses)
}

/// 检查 lark-cli 工具是否真实可用
fn check_lark_cli_available() -> bool {
    runner::check_tool_available(LARK_CLI)
}

/// 根据 CollectorConfig 构建采集器状态列表
///
/// 对飞书采集器执行真实的健康检查（检查 lark-cli 是否可用），
/// 而非仅依赖 enabled 状态。
fn build_collector_statuses(collectors: &CollectorConfig) -> Vec<CollectorStatus> {
    let lark_available = check_lark_cli_available();

    let known_collectors: Vec<(&str, &str)> = vec![
        ("feishu", "飞书"),
        ("manual", "手动输入"),
    ];

    known_collectors
        .into_iter()
        .map(|(id, name)| {
            let enabled = collectors.enabled.get(id).copied().unwrap_or(false);
            let healthy = match id {
                "feishu" => enabled && lark_available,
                "manual" => enabled,
                _ => false,
            };
            CollectorStatus {
                id: id.to_string(),
                name: name.to_string(),
                enabled,
                healthy,
            }
        })
        .collect()
}

/// 获取飞书采集模式
#[tauri::command]
pub async fn get_feishu_mode() -> Result<String, String> {
    let app_config = load_config()?;
    Ok(app_config.collectors.feishu_mode)
}

/// 保存飞书采集模式
#[tauri::command]
pub async fn save_feishu_mode(mode: String) -> Result<(), String> {
    if mode != "cli" && mode != "api" {
        return Err(format!("无效的飞书模式 '{}'，仅支持 'cli' 或 'api'", mode));
    }
    let mut app_config = load_config()?;
    app_config.collectors.feishu_mode = mode;
    save_config(&app_config)
}

/// 获取飞书会话 ID 配置
#[tauri::command]
pub async fn get_feishu_chat_id() -> Result<String, String> {
    let app_config = load_config()?;
    Ok(app_config.collectors.feishu_chat_id.clone().unwrap_or_default())
}

/// 保存飞书会话 ID 配置
#[tauri::command]
pub async fn save_feishu_chat_id(chat_id: String) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.collectors.feishu_chat_id = Some(chat_id.trim().to_string());
    save_config(&app_config)
}

/// 获取存储配置
#[tauri::command]
pub async fn get_storage_config() -> Result<StorageConfig, String> {
    let app_config = load_config()?;
    Ok(StorageConfig {
        vault_path: app_config.storage.vault_path,
        db_path: app_config.storage.db_path,
    })
}

/// 保存存储配置
#[tauri::command]
pub async fn save_storage_config(config: StorageConfig) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.storage.vault_path = config.vault_path;
    app_config.storage.db_path = config.db_path;
    save_config(&app_config)
}

/// 获取开发者模式状态
#[tauri::command]
pub async fn get_developer_mode() -> Result<bool, String> {
    let app_config = load_config()?;
    Ok(app_config.developer_mode)
}

/// 保存开发者模式状态
#[tauri::command]
pub async fn save_developer_mode(enabled: bool) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.developer_mode = enabled;
    save_config(&app_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_build_collector_statuses_from_config() {
        let mut enabled = HashMap::new();
        enabled.insert("feishu".into(), true);

        let collectors = CollectorConfig {
            enabled,
            feishu_mode: "cli".into(),
            feishu_chat_id: None,
        };

        let statuses = build_collector_statuses(&collectors);
        assert_eq!(statuses.len(), 2);

        let feishu = statuses.iter().find(|s| s.id == "feishu").unwrap();
        assert!(feishu.enabled);
        // healthy depends on lark-cli availability -- we just check it doesn't panic
        assert_eq!(feishu.name, "飞书");

        let manual = statuses.iter().find(|s| s.id == "manual").unwrap();
        assert!(!manual.enabled);
        assert_eq!(manual.name, "手动输入");
    }

    #[test]
    fn test_build_collector_statuses_all_disabled() {
        let collectors = CollectorConfig::default();
        let statuses = build_collector_statuses(&collectors);
        assert!(statuses.iter().all(|s| !s.enabled));
        // disabled collectors should always be unhealthy
        assert!(statuses.iter().all(|s| !s.healthy));
    }

    #[test]
    fn test_config_path_is_under_home() {
        let path = config_path().unwrap();
        assert!(path.to_string_lossy().contains(".work-better"));
        assert!(path.to_string_lossy().ends_with("config.json"));
    }

    #[test]
    fn test_build_collector_statuses_manual_enabled_is_healthy() {
        let mut enabled = HashMap::new();
        enabled.insert("manual".into(), true);

        let collectors = CollectorConfig {
            enabled,
            feishu_mode: "cli".into(),
            feishu_chat_id: None,
        };

        let statuses = build_collector_statuses(&collectors);
        let manual = statuses.iter().find(|s| s.id == "manual").unwrap();
        assert!(manual.enabled);
        assert!(manual.healthy);
    }
}
