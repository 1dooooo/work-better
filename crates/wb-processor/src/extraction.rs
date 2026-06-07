//! EntityExtractor —— 从模型输出中提取结构化数据

use chrono::Utc;
use uuid::Uuid;

use wb_core::event::Event;
use wb_core::record::{Category, WorkRecord};

/// 从模型提取输出中解析出的结构化数据
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedData {
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub category: Category,
    pub project: Option<String>,
    pub people: Vec<String>,
    pub tags: Vec<String>,
    pub task_status: Option<String>,
    pub confidence: f64,
}

/// 实体提取器：将模型输出（Extraction JSON）转换为 WorkRecord 字段
pub struct EntityExtractor;

impl EntityExtractor {
    /// 从模型输出中提取结构化数据
    ///
    /// `model_output` 是 TaskRunner.run_extract 返回的 JSON 字符串，
    /// 格式与 wb_ai::adapter::Extraction 对应。
    pub fn extract(model_output: &str, category: &Category) -> ExtractedData {
        // 尝试解析为 Extraction 格式
        if let Ok(extraction) = serde_json::from_str::<wb_ai::Extraction>(model_output) {
            return Self::from_extraction(&extraction, category);
        }

        // 回退：从原始 JSON 字符串中尽量提取
        Self::from_raw_json(model_output, category)
    }

    /// 从 Extraction 结构体构建 ExtractedData
    fn from_extraction(extraction: &wb_ai::Extraction, category: &Category) -> ExtractedData {
        ExtractedData {
            title: extraction.title.clone(),
            summary: extraction.summary.clone(),
            detail: extraction.detail.clone(),
            category: category.clone(),
            project: extraction.project.clone(),
            people: extraction.people.clone(),
            tags: extraction.tags.clone(),
            task_status: Self::infer_task_status(category),
            confidence: extraction.confidence,
        }
    }

    /// 从原始 JSON 中提取数据（回退路径）
    fn from_raw_json(json_str: &str, category: &Category) -> ExtractedData {
        let parsed: serde_json::Value =
            serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);

        let title = parsed
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let summary = parsed
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let detail = parsed
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or(json_str)
            .to_string();

        let people: Vec<String> = parsed
            .get("people")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let tags: Vec<String> = parsed
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let project = parsed
            .get("project")
            .and_then(|v| v.as_str())
            .map(String::from);

        let confidence = parsed
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        ExtractedData {
            title,
            summary,
            detail,
            category: category.clone(),
            project,
            people,
            tags,
            task_status: Self::infer_task_status(category),
            confidence,
        }
    }

    /// 根据分类推断默认的 task_status
    fn infer_task_status(category: &Category) -> Option<String> {
        match category {
            Category::Task => Some("in_progress".to_string()),
            _ => None,
        }
    }

    /// 将 ExtractedData 与 Event 信息合并为 WorkRecord
    pub fn to_work_record(data: &ExtractedData, event: &Event, model_used: &str) -> WorkRecord {
        WorkRecord {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            source_event_ids: vec![event.id.clone()],
            title: data.title.clone(),
            summary: data.summary.clone(),
            detail: data.detail.clone(),
            category: data.category.clone(),
            project: data.project.clone(),
            people: data.people.clone(),
            tags: data.tags.clone(),
            task_status: data.task_status.clone(),
            task_due: None,
            task_progress: None,
            model_used: model_used.to_string(),
            confidence: data.confidence,
            needs_review: data.confidence < 0.8,
            obsidian_path: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wb_core::event::{Confidence, EventType, Source};

    fn make_event() -> Event {
        Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            json!({"text": "test content"}),
            "raw".to_string(),
        )
    }

    #[test]
    fn test_extract_from_valid_extraction_json() {
        let json = json!({
            "title": "完成 PR Review",
            "summary": "审查了前端代码变更",
            "detail": "## 变更内容\n- 修复了登录bug",
            "people": ["张三", "李四"],
            "tags": ["review", "frontend"],
            "project": Some("work-better"),
            "confidence": 0.92
        })
        .to_string();

        let data = EntityExtractor::extract(&json, &Category::Task);
        assert_eq!(data.title, "完成 PR Review");
        assert_eq!(data.people, vec!["张三", "李四"]);
        assert_eq!(data.project, Some("work-better".to_string()));
        assert_eq!(data.task_status, Some("in_progress".to_string()));
    }

    #[test]
    fn test_extract_meeting_has_no_task_status() {
        let json = json!({
            "title": "周会",
            "summary": "讨论进度",
            "detail": "详细会议内容...",
            "people": ["Alice"],
            "tags": [],
            "project": null,
            "confidence": 0.85
        })
        .to_string();

        let data = EntityExtractor::extract(&json, &Category::Meeting);
        assert_eq!(data.task_status, None);
        assert_eq!(data.category, Category::Meeting);
    }

    #[test]
    fn test_extract_from_raw_json_fallback() {
        let json = r#"{"title": "Fallback Title", "summary": "Fallback summary"}"#;
        let data = EntityExtractor::extract(json, &Category::Task);
        assert_eq!(data.title, "Fallback Title");
        assert_eq!(data.summary, "Fallback summary");
        assert_eq!(data.confidence, 0.5); // default
    }

    #[test]
    fn test_extract_malformed_json_uses_defaults() {
        let data = EntityExtractor::extract("not json at all", &Category::Task);
        assert_eq!(data.title, "Untitled");
        assert_eq!(data.confidence, 0.5);
    }

    #[test]
    fn test_to_work_record() {
        let data = ExtractedData {
            title: "Test".to_string(),
            summary: "Sum".to_string(),
            detail: "Det".to_string(),
            category: Category::Task,
            project: Some("proj".to_string()),
            people: vec!["A".to_string()],
            tags: vec!["t1".to_string()],
            task_status: Some("done".to_string()),
            confidence: 0.88,
        };
        let event = make_event();
        let record = EntityExtractor::to_work_record(&data, &event, "mock-model");

        assert_eq!(record.title, "Test");
        assert_eq!(record.source_event_ids, vec![event.id.clone()]);
        assert_eq!(record.model_used, "mock-model");
        assert_eq!(record.confidence, 0.88);
        assert!(!record.needs_review); // 0.88 >= 0.8
    }

    #[test]
    fn test_to_work_record_low_confidence_needs_review() {
        let data = ExtractedData {
            title: "Low".to_string(),
            summary: "S".to_string(),
            detail: "D".to_string(),
            category: Category::Task,
            project: None,
            people: vec![],
            tags: vec![],
            task_status: Some("in_progress".to_string()),
            confidence: 0.5,
        };
        let event = make_event();
        let record = EntityExtractor::to_work_record(&data, &event, "mock-model");
        assert!(record.needs_review);
    }

    #[test]
    fn test_infer_task_status_for_various_categories() {
        assert_eq!(
            EntityExtractor::infer_task_status(&Category::Task),
            Some("in_progress".to_string())
        );
        assert_eq!(EntityExtractor::infer_task_status(&Category::Meeting), None);
        assert_eq!(
            EntityExtractor::infer_task_status(&Category::Research),
            None
        );
        assert_eq!(
            EntityExtractor::infer_task_status(&Category::Document),
            None
        );
    }
}
