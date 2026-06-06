//! wb-ai: AI 模型适配层

pub mod adapter;
pub mod anthropic;
pub mod budget;
pub mod config;
pub mod openai;
pub mod prompt;
pub mod router;
pub mod task_runner;

pub use adapter::{Classification, Extraction, MockAdapter, ModelAdapter};
pub use anthropic::AnthropicAdapter;
pub use budget::{OverloadStrategy, TokenBudget};
pub use config::ModelConfig;
pub use openai::OpenAIAdapter;
pub use router::{ModelRouter, TaskType, UpgradeThreshold};
pub use task_runner::{ModelSize, TaskOutput, TaskRunner};
