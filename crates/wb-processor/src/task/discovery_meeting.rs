//! 从会议纪要 / 妙记中提取待办任务

use super::discovery::PendingTask;
use super::model::{TaskPriority, TaskSource};

/// 会议待办关键词，后面紧跟的句子被识别为任务
const MEETING_KEYWORDS: &[&str] = &[
    "待办",
    "TODO",
    "行动项",
    "Action Item",
    "需要",
    "负责",
    "跟进",
    "落实",
    "安排",
];

/// 从会议文本中发现候选任务
pub fn discover_from_meeting(text: &str) -> Vec<PendingTask> {
    let mut results = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        for keyword in MEETING_KEYWORDS {
            if let Some(pos) = trimmed.find(keyword) {
                // 取关键词之后的文本作为任务内容
                let after = trimmed[pos + keyword.len()..].trim();
                // 去掉常见的分隔符前缀
                let task_text = after
                    .trim_start_matches(|c: char| [':', '：', ',', '，', '。', ' '].contains(&c))
                    .trim();

                if !task_text.is_empty() && task_text.chars().count() >= 1 {
                    results.push(PendingTask::new(
                        task_text,
                        None,
                        TaskSource::Meeting,
                        TaskPriority::P2,
                        None,
                        trimmed,
                    ));
                    break; // 一行只匹配第一个关键词
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_todo_keyword() {
        let text = "TODO: 完成需求文档\n其他内容";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "完成需求文档");
        assert_eq!(tasks[0].source, TaskSource::Meeting);
    }

    #[test]
    fn test_discover_chinese_keywords() {
        let text = "待办：张三负责接口联调\n李四需要准备测试环境";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title, "张三负责接口联调");
        assert_eq!(tasks[1].title, "准备测试环境");
    }

    #[test]
    fn test_discover_action_item() {
        let text = "Action Item: Review the API design by Friday";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].title.contains("Review the API design"));
    }

    #[test]
    fn test_discover_with_colon_separator() {
        let text = "跟进：修复登录页面的样式问题";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "修复登录页面的样式问题");
    }

    #[test]
    fn test_no_match_on_plain_text() {
        let text = "今天天气不错\n大家辛苦了";
        let tasks = discover_from_meeting(text);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_skip_empty_content_after_keyword() {
        // 关键词后没有实际内容
        let text = "TODO:   ";
        let tasks = discover_from_meeting(text);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_origin_text_preserved() {
        let text = "待办：张三负责完成报告";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks[0].origin_text, "待办：张三负责完成报告");
    }

    #[test]
    fn test_multiple_lines() {
        let text = "会议纪要\nTODO: 更新文档\n需要：安排下周会议\n普通讨论内容\n负责：部署新版本";
        let tasks = discover_from_meeting(text);
        assert_eq!(tasks.len(), 3);
    }
}
