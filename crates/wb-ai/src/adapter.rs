//! 模型适配器 trait 定义

use serde::{Deserialize, Serialize};
use wb_core::event::Event;

/// 分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    /// 分类标签（如 task, meeting, communication 等）
    pub category: String,
    /// 置信度 0-1
    pub confidence: f64,
    /// 理由
    pub reasoning: String,
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    /// 标题
    pub title: String,
    /// 摘要
    pub summary: String,
    /// 详细内容（markdown）
    pub detail: String,
    /// 涉及的人
    pub people: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 项目
    pub project: Option<String>,
    /// 置信度
    pub confidence: f64,
}

/// 模型适配器异步 trait
#[async_trait::async_trait]
pub trait ModelAdapter: Send + Sync {
    /// 对事件进行分类
    async fn classify(&self, event: &Event) -> wb_core::error::Result<Classification>;

    /// 从事件中提取结构化信息
    async fn extract(&self, event: &Event) -> wb_core::error::Result<Extraction>;

    /// 生成摘要
    async fn summarize(&self, text: &str) -> wb_core::error::Result<String>;
}
