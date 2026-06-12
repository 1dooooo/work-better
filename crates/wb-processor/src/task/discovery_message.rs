//! 从聊天消息中识别任务

use super::discovery::PendingTask;
use super::model::{TaskPriority, TaskSource};

/// 请求性表达关键词（按长度降序排列，优先匹配更长的模式）
const MESSAGE_KEYWORDS: &[&str] = &[
    "尽快处理",
    "尽快完成",
    "尽快确认",
    "尽快回复",
    "尽快反馈",
    "可以帮忙",
    "请帮忙",
    "请你帮忙",
    "麻烦你",
    "请你",
    "帮忙",
    "能不能",
    "麻烦",
    "尽快",
    // 个人意图表达
    "我需要",
    "我要",
    "我打算",
    "我计划",
    "我准备",
    "我得",
    "我必须",
];

/// 紧急关键词 —— 匹配时提升优先级
const URGENT_KEYWORDS: &[&str] = &[
    "尽快",
    "紧急",
    "马上",
    "立即",
    "ASAP",
    "urgent",
    "今天内",
    "今天之内",
];

/// 从聊天消息文本中发现候选任务
pub fn discover_from_message(text: &str) -> Vec<PendingTask> {
    let mut results = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        for keyword in MESSAGE_KEYWORDS {
            if let Some(pos) = trimmed.find(keyword) {
                // 取关键词之后的文本作为任务内容
                let after = trimmed[pos + keyword.len()..].trim();
                let task_text = after
                    .trim_start_matches(|c: char| [':', '：', ',', '，', '。', ' '].contains(&c))
                    .trim();

                if !task_text.is_empty() && task_text.chars().count() >= 1 {
                    let priority = if URGENT_KEYWORDS.iter().any(|k| trimmed.contains(k)) {
                        TaskPriority::P1
                    } else {
                        TaskPriority::P2
                    };

                    results.push(PendingTask::new(
                        task_text,
                        None,
                        TaskSource::Message,
                        priority,
                        None,
                        trimmed,
                    ));
                    break; // 一行只匹配一次
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
    fn test_discover_please_help() {
        let text = "请你帮忙检查一下登录接口的问题";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        // 匹配 "请你帮忙" 后取剩余文本
        assert_eq!(tasks[0].title, "检查一下登录接口的问题");
        assert_eq!(tasks[0].source, TaskSource::Message);
    }

    #[test]
    fn test_discover_can_you() {
        let text = "能不能把这个 bug 修一下？";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].title.contains("bug"));
    }

    #[test]
    fn test_discover_urgent_priority() {
        let text = "尽快修复生产环境的崩溃问题";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P1);
    }

    #[test]
    fn test_discover_normal_priority() {
        let text = "请你帮忙更新一下文档";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P2);
    }

    #[test]
    fn test_no_match_on_greeting() {
        let text = "你好\n今天辛苦了";
        let tasks = discover_from_message(text);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_multiple_messages() {
        let text = "请你帮忙写个测试\n其他聊天内容\n麻烦提交一下 PR";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_origin_text_preserved() {
        let text = "请你帮忙检查 API 返回值";
        let tasks = discover_from_message(text);
        assert_eq!(tasks[0].origin_text, "请你帮忙检查 API 返回值");
    }

    #[test]
    fn test_personal_intention_wo_yao() {
        let text = "我要10点要发邮件给 bob";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "10点要发邮件给 bob");
    }

    #[test]
    fn test_personal_intention_wo_xuyao() {
        let text = "我需要准备明天的会议材料";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "准备明天的会议材料");
    }

    #[test]
    fn test_personal_intention_wo_dasuan() {
        let text = "我打算下周完成这份报告";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "下周完成这份报告");
    }
}
