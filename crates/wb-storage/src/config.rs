//! 配置管理 —— 应用配置的加载与序列化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 采集器配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectorConfig {
    pub enabled: HashMap<String, bool>,
    pub feishu_mode: String,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: HashMap::new(),
            feishu_mode: "cli".into(),
        }
    }
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub small_model: String,
    pub large_model: String,
    pub api_endpoint: String,
    pub token_budget: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            small_model: "gpt-4o-mini".into(),
            large_model: "gpt-4o".into(),
            api_endpoint: "https://api.openai.com/v1".into(),
            token_budget: 4096,
        }
    }
}

/// 定时任务配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledTaskConfig {
    pub name: String,
    pub cron: String,
    pub enabled: bool,
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub tasks: Vec<ScheduledTaskConfig>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tasks: Vec::new(),
        }
    }
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    pub vault_path: String,
    pub db_path: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            vault_path: "~/Documents/Obsidian".into(),
            db_path: "~/.work-better/data.db".into(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub collectors: CollectorConfig,
    pub model: ModelConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    pub scheduler: SchedulerConfig,
}

impl AppConfig {
    /// 从 JSON 字符串加载配置
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.collectors.feishu_mode, "cli");
        assert_eq!(config.model.small_model, "gpt-4o-mini");
        assert_eq!(config.model.large_model, "gpt-4o");
        assert_eq!(config.model.token_budget, 4096);
        assert!(config.scheduler.enabled);
        assert!(config.scheduler.tasks.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut enabled = HashMap::new();
        enabled.insert("feishu".into(), true);
        enabled.insert("git".into(), false);

        let config = AppConfig {
            collectors: CollectorConfig {
                enabled,
                feishu_mode: "api".into(),
            },
            model: ModelConfig {
                small_model: "claude-3-haiku".into(),
                large_model: "claude-3-opus".into(),
                api_endpoint: "https://api.anthropic.com".into(),
                token_budget: 8192,
            },
            storage: StorageConfig {
                vault_path: "/tmp/test-vault".into(),
                db_path: "/tmp/test.db".into(),
            },
            scheduler: SchedulerConfig {
                enabled: true,
                tasks: vec![ScheduledTaskConfig {
                    name: "daily-collect".into(),
                    cron: "0 8 * * *".into(),
                    enabled: true,
                }],
            },
        };

        let json = config.to_json().unwrap();
        let deserialized = AppConfig::from_json(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "collectors": {
                "enabled": {"feishu": true},
                "feishu_mode": "cli"
            },
            "model": {
                "small_model": "gpt-4o-mini",
                "large_model": "gpt-4o",
                "api_endpoint": "https://api.openai.com/v1",
                "token_budget": 4096
            },
            "scheduler": {
                "enabled": false,
                "tasks": []
            }
        }"#;

        let config = AppConfig::from_json(json).unwrap();
        assert_eq!(config.collectors.feishu_mode, "cli");
        assert!(config.collectors.enabled["feishu"]);
        assert!(!config.scheduler.enabled);
    }

    #[test]
    fn test_from_json_with_tasks() {
        let json = r#"{
            "collectors": {
                "enabled": {},
                "feishu_mode": "api"
            },
            "model": {
                "small_model": "model-a",
                "large_model": "model-b",
                "api_endpoint": "http://localhost:8080",
                "token_budget": 2048
            },
            "scheduler": {
                "enabled": true,
                "tasks": [
                    {"name": "hourly", "cron": "0 * * * *", "enabled": true},
                    {"name": "daily", "cron": "0 0 * * *", "enabled": false}
                ]
            }
        }"#;

        let config = AppConfig::from_json(json).unwrap();
        assert_eq!(config.scheduler.tasks.len(), 2);
        assert_eq!(config.scheduler.tasks[0].name, "hourly");
        assert!(config.scheduler.tasks[0].enabled);
        assert!(!config.scheduler.tasks[1].enabled);
    }

    #[test]
    fn test_from_invalid_json() {
        let result = AppConfig::from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_collector_config_default() {
        let config = CollectorConfig::default();
        assert!(config.enabled.is_empty());
        assert_eq!(config.feishu_mode, "cli");
    }

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.small_model, "gpt-4o-mini");
        assert_eq!(config.api_endpoint, "https://api.openai.com/v1");
    }

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert!(config.enabled);
        assert!(config.tasks.is_empty());
    }

    #[test]
    fn test_json_output_is_pretty() {
        let config = AppConfig::default();
        let json = config.to_json().unwrap();
        assert!(json.contains('\n'));
        assert!(json.contains("  "));
    }
}
