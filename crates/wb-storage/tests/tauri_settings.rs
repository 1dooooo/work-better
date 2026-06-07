//! B3: Tauri Settings Persistence Integration Tests
//!
//! Tests the AppConfig serialization/deserialization layer that the
//! Tauri settings commands use for persistence.

use std::collections::HashMap;
use wb_storage::config::{
    AppConfig, CollectorConfig, ModelConfig, ScheduledTaskConfig, SchedulerConfig, StorageConfig,
};

// ---------------------------------------------------------------------------
// B3-01: Read/write settings roundtrip
// ---------------------------------------------------------------------------

#[test]
fn b3_01_settings_roundtrip() {
    let mut enabled = HashMap::new();
    enabled.insert("feishu".to_string(), true);
    enabled.insert("manual".to_string(), false);

    let config = AppConfig {
        collectors: CollectorConfig {
            enabled,
            feishu_mode: "api".to_string(),
            feishu_chat_id: Some("oc_test123".to_string()),
        },
        model: ModelConfig {
            small_model: "claude-3-haiku".to_string(),
            large_model: "claude-3-opus".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            token_budget: 8192,
        },
        storage: StorageConfig {
            vault_path: "/tmp/test-vault".to_string(),
            db_path: "/tmp/test.db".to_string(),
        },
        scheduler: SchedulerConfig {
            enabled: true,
            tasks: vec![ScheduledTaskConfig {
                name: "daily-collect".to_string(),
                cron: "0 8 * * *".to_string(),
                enabled: true,
            }],
        },
    };

    let json = config.to_json().unwrap();
    let deserialized = AppConfig::from_json(&json).unwrap();
    assert_eq!(config, deserialized);
}

// ---------------------------------------------------------------------------
// B3-02: Default values
// ---------------------------------------------------------------------------

#[test]
fn b3_02_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.collectors.feishu_mode, "cli");
    assert_eq!(config.collectors.feishu_chat_id, None);
    assert!(config.collectors.enabled.is_empty());
    assert_eq!(config.model.small_model, "gpt-4o-mini");
    assert_eq!(config.model.large_model, "gpt-4o");
    assert_eq!(config.model.api_endpoint, "https://api.openai.com/v1");
    assert_eq!(config.model.token_budget, 4096);
    assert_eq!(config.storage.vault_path, "~/Documents/Obsidian");
    assert_eq!(config.storage.db_path, "~/.work-better/data.db");
    assert!(config.scheduler.enabled);
    assert!(config.scheduler.tasks.is_empty());
}

#[test]
fn b3_02_default_collector_config() {
    let config = CollectorConfig::default();
    assert!(config.enabled.is_empty());
    assert_eq!(config.feishu_mode, "cli");
    assert_eq!(config.feishu_chat_id, None);
}

#[test]
fn b3_02_default_model_config() {
    let config = ModelConfig::default();
    assert_eq!(config.small_model, "gpt-4o-mini");
    assert_eq!(config.large_model, "gpt-4o");
    assert_eq!(config.api_endpoint, "https://api.openai.com/v1");
    assert_eq!(config.token_budget, 4096);
}

#[test]
fn b3_02_default_scheduler_config() {
    let config = SchedulerConfig::default();
    assert!(config.enabled);
    assert!(config.tasks.is_empty());
}

#[test]
fn b3_02_default_storage_config() {
    let config = StorageConfig::default();
    assert_eq!(config.vault_path, "~/Documents/Obsidian");
    assert_eq!(config.db_path, "~/.work-better/data.db");
}

// ---------------------------------------------------------------------------
// B3-03: Validation - invalid JSON
// ---------------------------------------------------------------------------

#[test]
fn b3_03_invalid_json_returns_error() {
    let result = AppConfig::from_json("not valid json");
    assert!(result.is_err());
}

#[test]
fn b3_03_empty_json_returns_error() {
    let result = AppConfig::from_json("");
    assert!(result.is_err());
}

#[test]
fn b3_03_partial_json_uses_defaults_for_storage() {
    // Missing storage section should use defaults (storage has #[serde(default)])
    let json = r#"{
        "collectors": {
            "enabled": {},
            "feishu_mode": "cli"
        },
        "model": {
            "small_model": "gpt-4o-mini",
            "large_model": "gpt-4o",
            "api_endpoint": "https://api.openai.com/v1",
            "token_budget": 4096
        },
        "scheduler": {
            "enabled": true,
            "tasks": []
        }
    }"#;

    let config = AppConfig::from_json(json).unwrap();
    // storage should default when missing
    assert_eq!(config.storage.vault_path, "~/Documents/Obsidian");
    assert_eq!(config.storage.db_path, "~/.work-better/data.db");
}

#[test]
fn b3_03_missing_scheduler_fails() {
    // scheduler is required (no #[serde(default)])
    let json = r#"{
        "collectors": {
            "enabled": {},
            "feishu_mode": "cli"
        },
        "model": {
            "small_model": "gpt-4o-mini",
            "large_model": "gpt-4o",
            "api_endpoint": "https://api.openai.com/v1",
            "token_budget": 4096
        }
    }"#;

    let result = AppConfig::from_json(json);
    assert!(result.is_err(), "Missing scheduler should fail");
}

// ---------------------------------------------------------------------------
// B3-04: Feishu mode validation
// ---------------------------------------------------------------------------

#[test]
fn b3_04_feishu_mode_cli() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;
    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.collectors.feishu_mode, "cli");
}

#[test]
fn b3_04_feishu_mode_api() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "api"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;
    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.collectors.feishu_mode, "api");
}

// ---------------------------------------------------------------------------
// B3-05: Chat ID persistence
// ---------------------------------------------------------------------------

#[test]
fn b3_05_chat_id_present() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli", "feishu_chat_id": "oc_xyz789"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;
    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.collectors.feishu_chat_id, Some("oc_xyz789".to_string()));
}

#[test]
fn b3_05_chat_id_absent_defaults_none() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;
    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.collectors.feishu_chat_id, None);
}

// ---------------------------------------------------------------------------
// B3-06: Scheduled tasks persistence
// ---------------------------------------------------------------------------

#[test]
fn b3_06_tasks_persisted() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "api"},
        "model": {"small_model": "a", "large_model": "b", "api_endpoint": "c", "token_budget": 2048},
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
    assert_eq!(config.scheduler.tasks[0].cron, "0 * * * *");
    assert!(config.scheduler.tasks[0].enabled);
    assert_eq!(config.scheduler.tasks[1].name, "daily");
    assert!(!config.scheduler.tasks[1].enabled);
}

// ---------------------------------------------------------------------------
// B3-07: JSON output is pretty-printed
// ---------------------------------------------------------------------------

#[test]
fn b3_07_json_pretty_printed() {
    let config = AppConfig::default();
    let json = config.to_json().unwrap();
    assert!(json.contains('\n'), "JSON should contain newlines");
    assert!(json.contains("  "), "JSON should contain indentation");
}

// ---------------------------------------------------------------------------
// B3-08: Storage config persistence
// ---------------------------------------------------------------------------

#[test]
fn b3_08_storage_config_custom() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "storage": {"vault_path": "/custom/vault", "db_path": "/custom/db.sqlite"},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;

    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.storage.vault_path, "/custom/vault");
    assert_eq!(config.storage.db_path, "/custom/db.sqlite");
}

// ---------------------------------------------------------------------------
// B3-09: Degradation - missing fields use defaults
// ---------------------------------------------------------------------------

#[test]
fn b3_09_missing_storage_uses_default() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli"},
        "model": {"small_model": "m", "large_model": "M", "api_endpoint": "e", "token_budget": 100},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;

    let config = AppConfig::from_json(json).unwrap();
    assert_eq!(config.storage.vault_path, "~/Documents/Obsidian");
    assert_eq!(config.storage.db_path, "~/.work-better/data.db");
}

#[test]
fn b3_09_missing_model_uses_default() {
    let json = r#"{
        "collectors": {"enabled": {}, "feishu_mode": "cli"},
        "scheduler": {"enabled": true, "tasks": []}
    }"#;

    // This should fail because model is required (no #[serde(default)])
    let result = AppConfig::from_json(json);
    assert!(result.is_err(), "Missing required model section should fail");
}
