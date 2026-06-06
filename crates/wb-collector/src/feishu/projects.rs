//! 飞书项目/任务采集器

use serde::{Deserialize, Serialize};
use wb_core::error::Result;
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;

/// lark-cli 任务列表响应
#[derive(Debug, Deserialize)]
struct LarkTasksResponse {
    data: Option<LarkTasksData>,
}

#[derive(Debug, Deserialize)]
struct LarkTasksData {
    items: Option<Vec<LarkTask>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LarkTask {
    task_id: Option<String>,
    title: Option<String>,
    status: Option<String>,
    assignee: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
}

/// 飞书项目/任务采集器
pub struct FeishuProjectsCollector;

impl FeishuProjectsCollector {
    /// 采集任务列表
    ///
    /// # Arguments
    /// * `limit` - 最大采集数量
    pub fn collect(limit: u32) -> Result<Vec<Event>> {
        let limit_str = limit.to_string();
        let args = vec!["task", "list", "--page-size", &limit_str];

        let response: LarkTasksResponse = runner::execute_json("lark-cli", &args)?;

        let items = response.data.and_then(|d| d.items).unwrap_or_default();

        let events: Vec<Event> = items
            .into_iter()
            .filter_map(Self::convert_task)
            .collect();

        Ok(events)
    }

    /// 将 lark-cli 任务转换为 Event
    fn convert_task(task: LarkTask) -> Option<Event> {
        let task_id = task.task_id.clone()?;

        let raw_payload = serde_json::to_string(&task).ok()?;

        let content = serde_json::json!({
            "task_id": task.task_id,
            "title": task.title,
            "status": task.status,
            "assignee": task.assignee,
        });

        let mut event = Event::new(
            Source::FeishuProject,
            Confidence::Medium,
            EventType::TaskUpdate,
            content,
            raw_payload,
        );

        // 使用 task_id 作为事件 id（保证幂等）
        event.id = task_id;

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_task() {
        let task = LarkTask {
            task_id: Some("task-001".to_string()),
            title: Some("实现登录功能".to_string()),
            status: Some("in_progress".to_string()),
            assignee: Some("user-001".to_string()),
            create_time: Some("1717689600".to_string()),
            update_time: Some("1717689700".to_string()),
        };

        let event = FeishuProjectsCollector::convert_task(task);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.id, "task-001");
        assert_eq!(event.source, Source::FeishuProject);
        assert_eq!(event.source_confidence, Confidence::Medium);
        assert_eq!(event.event_type, EventType::TaskUpdate);
        assert_eq!(event.content["title"], "实现登录功能");
        assert_eq!(event.content["status"], "in_progress");
        assert_eq!(event.content["assignee"], "user-001");
    }

    #[test]
    fn test_convert_task_no_id_returns_none() {
        let task = LarkTask {
            task_id: None,
            title: Some("无 ID 任务".to_string()),
            status: None,
            assignee: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuProjectsCollector::convert_task(task);
        assert!(event.is_none());
    }

    #[test]
    fn test_convert_task_missing_optional_fields() {
        let task = LarkTask {
            task_id: Some("task-002".to_string()),
            title: None,
            status: None,
            assignee: None,
            create_time: None,
            update_time: None,
        };

        let event = FeishuProjectsCollector::convert_task(task).unwrap();
        assert_eq!(event.id, "task-002");
        assert_eq!(event.content["status"], serde_json::Value::Null);
    }
}
