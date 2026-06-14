use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// 工作记录分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum Category {
    Task,
    Meeting,
    Communication,
    Research,
    Review,
    Planning,
    Document,
    Decision,
}

/// 工作记录 —— 处理层的输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct WorkRecord {
    pub id: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    pub source_event_ids: Vec<String>,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub category: Category,
    pub project: Option<String>,
    pub people: Vec<String>,
    pub tags: Vec<String>,
    pub task_status: Option<String>,
    #[ts(type = "string | null")]
    pub task_due: Option<DateTime<Utc>>,
    pub task_priority: Option<String>,
    pub task_progress: Option<String>,
    pub model_used: String,
    pub confidence: f64,
    pub needs_review: bool,
    pub obsidian_path: String,
}

impl WorkRecord {
    /// 创建新工作记录
    pub fn new(
        title: String,
        summary: String,
        detail: String,
        category: Category,
        source_event_ids: Vec<String>,
        model_used: String,
        confidence: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            source_event_ids,
            title,
            summary,
            detail,
            category,
            project: None,
            people: Vec::new(),
            tags: Vec::new(),
            task_status: None,
            task_due: None,
            task_priority: None,
            task_progress: None,
            model_used,
            confidence,
            needs_review: confidence < 0.8,
            obsidian_path: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_record_new() {
        let record = WorkRecord::new(
            "Test Title".to_string(),
            "Test Summary".to_string(),
            "Test Detail".to_string(),
            Category::Task,
            vec!["evt-1".to_string()],
            "gpt-4".to_string(),
            0.9,
        );

        assert!(!record.id.is_empty(), "id should not be empty");
        assert_eq!(
            record.needs_review, false,
            "confidence 0.9 should not need review"
        );

        let low_confidence_record = WorkRecord::new(
            "Low Confidence".to_string(),
            "Summary".to_string(),
            "Detail".to_string(),
            Category::Meeting,
            vec![],
            "gpt-3.5".to_string(),
            0.5,
        );
        assert_eq!(
            low_confidence_record.needs_review, true,
            "confidence 0.5 should need review"
        );
    }
}
