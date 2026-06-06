//! 从邮件中识别请求和承诺

use super::discovery::PendingTask;
use super::model::{TaskPriority, TaskSource};

/// 邮件任务关键词
const EMAIL_KEYWORDS: &[&str] = &[
    "请确认",
    "请回复",
    "请审批",
    "请审核",
    "请查阅",
    "请查阅并回复",
    "截止",
    "deadline",
    "Deadline",
    "DDL",
    "ddl",
    "需要你",
    "期望",
    "务必",
    "拜托",
];

/// 截止日期关键词 —— 匹配时标记紧急
const DEADLINE_KEYWORDS: &[&str] = &[
    "截止",
    "deadline",
    "Deadline",
    "DDL",
    "ddl",
    "务必",
];

/// 从邮件文本中发现候选任务
pub fn discover_from_email(text: &str) -> Vec<PendingTask> {
    let mut results = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        for keyword in EMAIL_KEYWORDS {
            if let Some(pos) = trimmed.find(keyword) {
                let after = trimmed[pos + keyword.len()..].trim();
                let task_text = after
                    .trim_start_matches(|c: char| [':', '：', ',', '，', '。', ' '].contains(&c))
                    .trim();

                if !task_text.is_empty() && task_text.chars().count() >= 1 {
                    let priority = if DEADLINE_KEYWORDS.iter().any(|k| trimmed.contains(k)) {
                        TaskPriority::P1
                    } else {
                        TaskPriority::P2
                    };

                    // 尝试提取截止日期信息
                    let due_date = extract_due_date(trimmed);

                    results.push(PendingTask::new(
                        task_text,
                        None,
                        TaskSource::Email,
                        priority,
                        due_date,
                        trimmed,
                    ));
                    break; // 一行只匹配一次
                }
            }
        }
    }

    results
}

/// 从文本中提取简单的截止日期模式
///
/// 支持格式: "截止 2026-06-15"、"deadline 2026/06/15"、"DDL: 06-15"
fn extract_due_date(text: &str) -> Option<String> {
    // 尝试匹配 YYYY-MM-DD 或 YYYY/MM/DD
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len().saturating_sub(9) {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
            && (chars[i + 4] == '-' || chars[i + 4] == '/')
            && chars[i + 5].is_ascii_digit()
            && chars[i + 6].is_ascii_digit()
            && (chars[i + 7] == '-' || chars[i + 7] == '/')
            && chars[i + 8].is_ascii_digit()
            && chars[i + 9].is_ascii_digit()
        {
            let date: String = chars[i..i + 10].iter().collect();
            return Some(date);
        }
    }
    // 尝试匹配 MM-DD 或 MM/DD（无年份）
    for i in 0..chars.len().saturating_sub(4) {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && (chars[i + 2] == '-' || chars[i + 2] == '/')
            && chars[i + 3].is_ascii_digit()
            && chars[i + 4].is_ascii_digit()
        {
            // 确保前面不是数字（排除 YYYY-MM-DD 的一部分）
            if i == 0 || !chars[i - 1].is_ascii_digit() {
                let date: String = chars[i..i + 5].iter().collect();
                return Some(date);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_please_confirm() {
        let text = "请确认：需求文档中的功能范围是否正确";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "需求文档中的功能范围是否正确");
        assert_eq!(tasks[0].source, TaskSource::Email);
    }

    #[test]
    fn test_discover_deadline_high_priority() {
        let text = "截止 2026-06-15：请回复项目进度";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P1);
        assert_eq!(tasks[0].due_date, Some("2026-06-15".to_string()));
    }

    #[test]
    fn test_discover_deadline_slash_format() {
        let text = "deadline 2026/06/30 请审批采购申请";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].due_date, Some("2026/06/30".to_string()));
    }

    #[test]
    fn test_discover_ddl_keyword() {
        let text = "DDL: 06-20 提交代码审查";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P1);
    }

    #[test]
    fn test_discover_normal_priority() {
        let text = "请查阅并回复：本月的周报汇总";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::P2);
    }

    #[test]
    fn test_no_match_on_greeting() {
        let text = "Hi team,\nBest regards";
        let tasks = discover_from_email(text);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_extract_due_date_yyyy_mm_dd() {
        assert_eq!(extract_due_date("截止 2026-06-15"), Some("2026-06-15".to_string()));
        assert_eq!(extract_due_date("deadline 2026/12/31"), Some("2026/12/31".to_string()));
    }

    #[test]
    fn test_extract_due_date_mm_dd() {
        assert_eq!(extract_due_date("DDL: 06-20"), Some("06-20".to_string()));
    }

    #[test]
    fn test_no_due_date() {
        assert_eq!(extract_due_date("请确认需求文档"), None);
    }

    #[test]
    fn test_origin_text_preserved() {
        let text = "请确认：API 接口文档";
        let tasks = discover_from_email(text);
        assert_eq!(tasks[0].origin_text, "请确认：API 接口文档");
    }

    #[test]
    fn test_multiple_email_tasks() {
        let text = "请审批：采购申请\n普通段落\n请回复：本周工作计划";
        let tasks = discover_from_email(text);
        assert_eq!(tasks.len(), 2);
    }
}
