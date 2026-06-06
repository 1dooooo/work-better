//! wb-ai: AI 模型适配层

pub mod adapter;
pub mod anthropic;
pub mod budget;
pub mod config;
pub mod router;

pub use adapter::{Classification, Extraction, ModelAdapter};
pub use anthropic::AnthropicAdapter;
pub use budget::{OverloadStrategy, TokenBudget};
pub use config::ModelConfig;
pub use router::{ModelRouter, TaskType, UpgradeThreshold};
