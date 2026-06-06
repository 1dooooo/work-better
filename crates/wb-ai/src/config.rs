//! 模型配置

/// 模型提供商
#[derive(Debug, Clone)]
pub enum ModelProvider {
    Anthropic,
    OpenAi,
}

/// 模型配置
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: ModelProvider,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
}

impl ModelConfig {
    /// 创建 Anthropic 配置
    pub fn anthropic(api_key: String) -> Self {
        Self {
            provider: ModelProvider::Anthropic,
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 4096,
        }
    }

    /// 创建 OpenAI 兼容配置
    pub fn openai(api_key: String, base_url: Option<String>) -> Self {
        Self {
            provider: ModelProvider::OpenAi,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
            model: "gpt-4o".to_string(),
            max_tokens: 4096,
        }
    }

    /// 设置模型名称
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// 设置 base_url
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}
