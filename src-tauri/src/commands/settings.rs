//! 设置命令

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use wb_collector::runner;
use wb_storage::config::{AppConfig, CollectorConfig};

/// lark-cli 工具路径
const LARK_CLI: &str = "/opt/homebrew/bin/lark-cli";



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
///
/// 支持 `WORK_BETTER_HOME` 环境变量覆盖（dev.sh 使用）。
fn config_path() -> Result<PathBuf, String> {
    let home = std::env::var("WORK_BETTER_HOME")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("无法获取 HOME 环境变量: {e}"))?;
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
///
/// 此函数为 `pub(crate)` 可见性，供 `collectors` 模块在启用/禁用采集器时持久化状态。
pub(crate) fn save_config_pub(config: &AppConfig) -> Result<(), String> {
    save_config(config)
}

/// 将 AppConfig 保存到配置文件（内部实现）
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
pub async fn get_model_config() -> Result<wb_storage::config::ModelConfig, String> {
    let app_config = load_config()?;
    Ok(app_config.model)
}

/// 保存模型配置
#[tauri::command]
pub async fn save_model_config(config: wb_storage::config::ModelConfig) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.model = config;
    save_config(&app_config)
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

// ─── 模型管理 ─────────────────────────────────────────────────────

// ─── 快捷键配置 ─────────────────────────────────────────────────

/// 快捷键配置项（前端 DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub id: String,
    pub label: String,
    pub key: String,
    pub modifiers: Vec<String>,
}

/// 默认快捷键配置
fn default_shortcuts() -> Vec<ShortcutConfig> {
    vec![
        ShortcutConfig {
            id: "capture".into(),
            label: "快速捕获".into(),
            key: "Space".into(),
            modifiers: vec!["cmd".into(), "shift".into()],
        },
        ShortcutConfig {
            id: "search".into(),
            label: "全局搜索".into(),
            key: "K".into(),
            modifiers: vec!["cmd".into()],
        },
        ShortcutConfig {
            id: "task".into(),
            label: "新建任务".into(),
            key: "N".into(),
            modifiers: vec!["cmd".into(), "shift".into()],
        },
    ]
}

use wb_storage::config::ShortcutEntry;

/// 获取快捷键配置
#[tauri::command]
pub async fn get_shortcut_config() -> Result<Vec<ShortcutConfig>, String> {
    let app_config = load_config()?;
    if app_config.shortcuts.is_empty() {
        Ok(default_shortcuts())
    } else {
        Ok(app_config
            .shortcuts
            .into_iter()
            .map(|e| ShortcutConfig {
                id: e.id,
                label: e.label,
                key: e.key,
                modifiers: e.modifiers,
            })
            .collect())
    }
}

/// 保存快捷键配置（保存后立即热重载全局快捷键）
#[tauri::command]
pub async fn save_shortcut_config(app: AppHandle, config: Vec<ShortcutConfig>) -> Result<(), String> {
    let mut app_config = load_config()?;
    app_config.shortcuts = config
        .into_iter()
        .map(|c| ShortcutEntry {
            id: c.id,
            label: c.label,
            key: c.key,
            modifiers: c.modifiers,
        })
        .collect();
    save_config(&app_config)?;

    // 热重载：立即重新注册全局快捷键
    crate::register_shortcuts(&app, &app_config)
}

// ─── 系统状态 ─────────────────────────────────────────────────

/// 系统状态（供菜单栏展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub collectors_total: usize,
    pub collectors_healthy: usize,
    pub scheduler_running: bool,
    pub unprocessed_count: usize,
    /// 今日已处理事件数
    pub today_processed_count: usize,
}

/// 获取系统状态（菜单栏用）
#[tauri::command]
pub async fn get_system_status() -> Result<SystemStatus, String> {
    let app_config = load_config()?;
    let statuses = build_collector_statuses(&app_config.collectors);
    let total = statuses.len();
    let healthy = statuses.iter().filter(|s| s.healthy).count();

    // 查询今日已处理事件数
    let today_processed_count = query_today_processed_count();

    Ok(SystemStatus {
        collectors_total: total,
        collectors_healthy: healthy,
        scheduler_running: app_config.scheduler.enabled,
        unprocessed_count: 0, // 由前端单独查询
        today_processed_count,
    })
}

/// 查询今日已处理事件数
///
/// 从 SQLite events 表查询 processed_at 在今日 00:00 之后的记录数。
/// 如果查询失败（如表不存在），返回 0。
fn query_today_processed_count() -> usize {
    // TODO: 当 SQLite events 查询接口就绪后，替换为真实查询
    // 示例 SQL: SELECT COUNT(*) FROM events WHERE processed = 1 AND processed_at >= <today_midnight>
    // 目前先返回 0，不影响前端展示
    0
}

/// 模型信息（从 API 获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

/// 构建 API URL，避免重复 /v1
///
/// 用户可能填 `https://api.example.com/v1` 或 `https://api.example.com/v1/`，
/// 需要智能拼接，避免变成 `/v1/v1/models`。
fn build_api_url(endpoint: &str, path: &str) -> String {
    let base = endpoint.trim_end_matches('/').trim_end_matches("/v1");
    format!("{}/v1/{}", base, path.trim_start_matches('/'))
}

/// 获取可用模型列表
///
/// - OpenAI 兼容端点：调用 GET /v1/models
/// - Anthropic 端点：返回内置 Claude 模型列表
#[tauri::command]
pub async fn list_models(api_endpoint: String, api_key: String) -> Result<Vec<ModelInfo>, String> {
    if api_key.is_empty() {
        return Err("请先配置 API Key".to_string());
    }

    let is_anthropic = api_endpoint.contains("anthropic");

    if is_anthropic {
        // Anthropic 没有标准的模型列表 API，返回内置模型
        Ok(vec![
            ModelInfo { id: "claude-sonnet-4-6".into(), name: "Claude Sonnet 4.6（推荐）".into() },
            ModelInfo { id: "claude-haiku-4-5-20251001".into(), name: "Claude Haiku 4.5（快速）".into() },
            ModelInfo { id: "claude-opus-4-8".into(), name: "Claude Opus 4.8（最强）".into() },
        ])
    } else {
        // OpenAI 兼容端点：调用 /v1/models
        let client = reqwest::Client::new();
        let url = build_api_url(&api_endpoint, "models");

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("请求模型列表失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误 {}: {}", status, body));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelEntry>,
        }
        #[derive(Deserialize)]
        struct ModelEntry {
            id: String,
        }

        let result: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("解析模型列表失败: {}", e))?;

        let mut models: Vec<ModelInfo> = result
            .data
            .into_iter()
            .map(|m| {
                let name = format_model_name(&m.id);
                ModelInfo { id: m.id, name }
            })
            .collect();

        // 按名称排序，常用的排前面
        models.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(models)
    }
}

/// 为 OpenAI 模型 ID 生成友好名称
fn format_model_name(id: &str) -> String {
    match id {
        s if s.contains("gpt-4o-mini") => format!("{}（快速·便宜）", id),
        s if s.contains("gpt-4o") => format!("{}（推荐）", id),
        s if s.contains("gpt-4-turbo") => format!("{}（均衡）", id),
        s if s.contains("gpt-4") => format!("{}（强推理）", id),
        s if s.contains("o1") => format!("{}（推理）", id),
        s if s.contains("o3") => format!("{}（推理）", id),
        s if s.contains("claude") => format!("{}（Claude）", id),
        s if s.contains("deepseek") => format!("{}（DeepSeek）", id),
        _ => id.to_string(),
    }
}

/// 测试结果
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: u64,
}

/// 测试模型连接
///
/// 发送一条简单消息，验证 API Key 和模型是否可用。
#[tauri::command]
pub async fn test_model(
    api_endpoint: String,
    api_key: String,
    model: String,
) -> Result<TestResult, String> {
    if api_key.is_empty() {
        return Err("请先配置 API Key".to_string());
    }
    if model.is_empty() {
        return Err("请先选择模型".to_string());
    }

    let is_anthropic = api_endpoint.contains("anthropic");
    let start = std::time::Instant::now();

    let result = if is_anthropic {
        test_anthropic_model(&api_endpoint, &api_key, &model).await
    } else {
        test_openai_model(&api_endpoint, &api_key, &model).await
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(reply) => Ok(TestResult {
            success: true,
            message: format!("连接成功！模型回复：「{}」", truncate(&reply, 100)),
            latency_ms,
        }),
        Err(e) => Ok(TestResult {
            success: false,
            message: format!("测试失败：{}", e),
            latency_ms,
        }),
    }
}

/// 测试 OpenAI 兼容模型
async fn test_openai_model(endpoint: &str, api_key: &str, model: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = build_api_url(endpoint, "chat/completions");

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 32,
        "messages": [{"role": "user", "content": "Say hello in one word."}]
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("API 错误 {}: {}", status, err_body));
    }

    #[derive(Deserialize)]
    struct ChatResponse {
        choices: Vec<ChatChoice>,
    }
    #[derive(Deserialize)]
    struct ChatChoice {
        message: ChatMessage,
    }
    #[derive(Deserialize)]
    struct ChatMessage {
        content: String,
    }

    let result: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    result
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "API 返回空响应".to_string())
}

/// 测试 Anthropic 模型
async fn test_anthropic_model(endpoint: &str, api_key: &str, model: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = build_api_url(endpoint, "messages");

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 32,
        "messages": [{"role": "user", "content": "Say hello in one word."}]
    });

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("API 错误 {}: {}", status, err_body));
    }

    #[derive(Deserialize)]
    struct MsgResponse {
        content: Vec<ContentBlock>,
    }
    #[derive(Deserialize)]
    struct ContentBlock {
        text: String,
    }

    let result: MsgResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    result
        .content
        .first()
        .map(|c| c.text.clone())
        .ok_or_else(|| "API 返回空响应".to_string())
}

/// 截断字符串
fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", chars[..max_chars].iter().collect::<String>())
    }
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
            group_enabled: HashMap::new(),
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
            group_enabled: HashMap::new(),
            feishu_mode: "cli".into(),
            feishu_chat_id: None,
        };

        let statuses = build_collector_statuses(&collectors);
        let manual = statuses.iter().find(|s| s.id == "manual").unwrap();
        assert!(manual.enabled);
        assert!(manual.healthy);
    }
}
