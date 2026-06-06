//! 即时处理路径

use wb_core::error::Result;
use wb_core::event::Event;
use wb_core::record::{Category, WorkRecord};
use wb_ai::ModelAdapter;

/// 即时处理器：调用 AI 提取结构化信息，生成 WorkRecord
pub struct InstantProcessor;

impl InstantProcessor {
    /// 处理单个事件，生成 WorkRecord
    pub async fn process(event: &Event, ai: &dyn ModelAdapter) -> Result<WorkRecord> {
        // 1. 分类
        let classification = ai.classify(event).await?;

        // 2. 提取结构化信息
        let extraction = ai.extract(event).await?;

        // 3. 映射分类到 Category 枚举
        let category = match classification.category.as_str() {
            "task" => Category::Task,
            "meeting" => Category::Meeting,
            "communication" => Category::Communication,
            "research" => Category::Research,
            "review" => Category::Review,
            "planning" => Category::Planning,
            "document" => Category::Document,
            "decision" => Category::Decision,
            _ => Category::Communication,
        };

        // 4. 构造 WorkRecord
        let mut record = WorkRecord::new(
            extraction.title,
            extraction.summary,
            extraction.detail,
            category,
            vec![event.id.clone()],
            "anthropic/claude-sonnet-4-6".to_string(),
            extraction.confidence,
        );

        record.people = extraction.people;
        record.tags = extraction.tags;
        record.project = extraction.project;

        Ok(record)
    }
}
