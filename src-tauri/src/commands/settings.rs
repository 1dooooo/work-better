//! 设置命令

use serde::{Deserialize, Serialize};

/// 模型配置
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

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub vault_path: String,
    pub db_path: String,
}

/// 获取模型配置
#[tauri::command]
pub async fn get_model_config() -> Result<ModelConfig, String> {
    // TODO: 从配置文件读取
    Ok(ModelConfig {
        api_endpoint: "https://api.openai.com/v1".to_string(),
        api_key: String::new(),
        token_budget: 4096,
    })
}

/// 保存模型配置
#[tauri::command]
pub async fn save_model_config(config: ModelConfig) -> Result<(), String> {
    // TODO: 保存到配置文件
    let _ = config;
    Ok(())
}

/// 获取采集器状态列表
#[tauri::command]
pub async fn get_collector_statuses() -> Result<Vec<CollectorStatus>, String> {
    // TODO: 从采集器模块读取真实状态
    Ok(vec![
        CollectorStatus {
            id: "feishu".to_string(),
            name: "飞书".to_string(),
            enabled: true,
            healthy: true,
        },
        CollectorStatus {
            id: "manual".to_string(),
            name: "手动输入".to_string(),
            enabled: true,
            healthy: true,
        },
    ])
}

/// 获取存储配置
#[tauri::command]
pub async fn get_storage_config() -> Result<StorageConfig, String> {
    // TODO: 从配置文件读取
    Ok(StorageConfig {
        vault_path: "~/Documents/Obsidian".to_string(),
        db_path: "~/.work-better/data.db".to_string(),
    })
}

/// 保存存储配置
#[tauri::command]
pub async fn save_storage_config(config: StorageConfig) -> Result<(), String> {
    // TODO: 保存到配置文件
    let _ = config;
    Ok(())
}
