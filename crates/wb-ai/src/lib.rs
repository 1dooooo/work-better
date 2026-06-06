//! wb-ai: AI 模型适配层

pub mod adapter;
pub mod anthropic;
pub mod config;

pub use adapter::{Classification, Extraction, ModelAdapter};
pub use anthropic::AnthropicAdapter;
pub use config::ModelConfig;
