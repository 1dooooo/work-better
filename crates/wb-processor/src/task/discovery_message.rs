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

/// 截止日期 / 承诺性关键词 —— 匹配时识别为任务
const DEADLINE_KEYWORDS: &[&str] = &[
    "完成",
    "发布",
    "上线",
    "交付",
    "截止",
    "deadline",
    "Deadline",
    "DDL",
    "ddl",
    "务必",
    "承诺",
    "答应",
    "提交",
    "搞定",
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

/// 中文时间表达式 —— 出现时识别为带截止日期的任务
///
/// 匹配 "明天下午5点"、"下周一"、"本周内" 等模式，
/// 不依赖特定关键词，而是检测时间指涉词 + 数字组合。
const TIME_EXPRESSIONS: &[&str] = &[
    "今天",
    "明天",
    "后天",
    "下周",
    "本周",
    "这周",
    "月底",
    "年底",
    "周日",
    "周一",
    "周二",
    "周三",
    "周四",
    "周五",
    "周六",
];

/// 数字 + 时间单位模式（如 "5点"、"10号"、"15日"）
///
/// 配合 TIME_EXPRESSIONS 使用：一行中同时出现时间词和数字时间，
/// 或出现 "X点/X号/X日" 这类绝对时间指涉，即视为有截止日期。
fn has_time_expression(text: &str) -> bool {
    // 检查是否包含中文时间指涉词
    let has_time_word = TIME_EXPRESSIONS.iter().any(|t| text.contains(t));

    // 检查是否包含 "X点"、"X号"、"X日"、"X月" 模式
    let has_time_number = {
        let chars: Vec<char> = text.chars().collect();
        let mut found = false;
        for i in 0..chars.len() {
            if chars[i].is_ascii_digit() && i + 1 < chars.len() {
                let next = chars[i + 1];
                if ['点', '号', '日', '月', '时'].contains(&next) {
                    found = true;
                    break;
                }
            }
        }
        found
    };

    has_time_word || has_time_number
}

/// 从文本中提取中文时间描述作为 due_date
///
/// 尽可能捕获 "明天下午5点"、"下周一"、"2026-06-15" 等表达。
fn extract_due_date(text: &str) -> Option<String> {
    // 1. 尝试匹配 YYYY-MM-DD 或 YYYY/MM/DD
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len().saturating_sub(9) {
        if chars[i].is_ascii_digit()
            && chars.get(i + 1).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 2).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 3).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 4).map_or(false, |c| *c == '-' || *c == '/')
            && chars.get(i + 5).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 6).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 7).map_or(false, |c| *c == '-' || *c == '/')
            && chars.get(i + 8).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 9).map_or(false, |c| c.is_ascii_digit())
        {
            let date: String = chars[i..i + 10].iter().collect();
            // 提取日期后面可能的时间部分（如 " 17:00"）
            let rest: String = chars[i + 10..].iter().collect();
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                // 可能有时间，取到下一个非时间字符
                let time_part: String = rest_trimmed
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == ':' || *c == ' ')
                    .collect();
                let time_trimmed = time_part.trim();
                if !time_trimmed.is_empty() {
                    return Some(format!("{} {}", date, time_trimmed));
                }
            }
            return Some(date);
        }
    }

    // 2. 尝试匹配中文时间表达式（取包含时间词的最短有意义片段）
    for time_word in TIME_EXPRESSIONS {
        if let Some(pos) = text.find(time_word) {
            // 从时间词位置向后扫描，捕获 "明天下午5点" 这样的完整表达
            let after: String = text[pos..].chars().take(20).collect();
            // 去掉尾部的非时间内容
            let trimmed = after.trim_end_matches(|c: char| {
                ['，', '。', ',', '.', ' ', '！', '!', '？', '?'].contains(&c)
            });
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// 从聊天消息文本中发现候选任务
pub fn discover_from_message(text: &str) -> Vec<PendingTask> {
    let mut results = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 路径 1：关键词匹配（原有逻辑）
        let mut matched_by_keyword = false;
        for keyword in MESSAGE_KEYWORDS {
            if let Some(pos) = trimmed.find(keyword) {
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

                    let due_date = extract_due_date(trimmed);

                    results.push(PendingTask::new(
                        task_text,
                        None,
                        TaskSource::Message,
                        priority,
                        due_date,
                        trimmed,
                    ));
                    matched_by_keyword = true;
                    break;
                }
            }
        }

        if matched_by_keyword {
            continue;
        }

        // 路径 2：截止日期关键词匹配（新增）
        let mut matched_by_deadline = false;
        for keyword in DEADLINE_KEYWORDS {
            if let Some(pos) = trimmed.find(keyword) {
                let after = trimmed[pos + keyword.len()..].trim();
                let task_text = after
                    .trim_start_matches(|c: char| [':', '：', ',', '，', '。', ' '].contains(&c))
                    .trim();

                if !task_text.is_empty() && task_text.chars().count() >= 1 {
                    let priority = if URGENT_KEYWORDS.iter().any(|k| trimmed.contains(k))
                        || has_time_expression(trimmed)
                    {
                        TaskPriority::P1
                    } else {
                        TaskPriority::P2
                    };

                    let due_date = extract_due_date(trimmed);

                    results.push(PendingTask::new(
                        task_text,
                        None,
                        TaskSource::Message,
                        priority,
                        due_date,
                        trimmed,
                    ));
                    matched_by_deadline = true;
                    break;
                }
            }
        }

        if matched_by_deadline {
            continue;
        }

        // 路径 3：纯时间表达式匹配（新增 —— 捕获 "明天下午5点完成代码发布" 这类句子）
        // 条件：包含时间表达式 + 包含动词性词汇（完成/发布/上线/提交等动作词）
        if has_time_expression(trimmed) {
            let action_verbs = [
                "完成", "发布", "上线", "提交", "交付", "搞定", "处理", "修复",
                "部署", "发版", "review", "Review", "合并", "merge",
            ];
            if action_verbs.iter().any(|v| trimmed.contains(*v)) {
                // 整行作为任务内容
                let priority = if URGENT_KEYWORDS.iter().any(|k| trimmed.contains(k)) {
                    TaskPriority::P1
                } else {
                    TaskPriority::P1 // 有明确时间的默认 P1
                };

                let due_date = extract_due_date(trimmed);

                results.push(PendingTask::new(
                    trimmed,
                    None,
                    TaskSource::Message,
                    priority,
                    due_date,
                    trimmed,
                ));
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

    // ── 新增：截止日期关键词测试 ────────────────────────────────

    #[test]
    fn test_deadline_keyword_wancheng() {
        let text = "明天下午5点完成代码发布";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1, "应识别 '完成' 为截止日期关键词");
        assert_eq!(tasks[0].source, TaskSource::Message);
        assert!(tasks[0].due_date.is_some(), "应提取到截止日期");
    }

    #[test]
    fn test_deadline_keyword_fabu() {
        let text = "本周五发布新版本";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1, "应识别 '发布' 为截止日期关键词");
        assert_eq!(tasks[0].priority, TaskPriority::P1, "有时间表达应为 P1");
    }

    #[test]
    fn test_deadline_keyword_shangxian() {
        let text = "下周一上线支付功能";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1, "应识别 '上线' 为截止日期关键词");
    }

    #[test]
    fn test_deadline_keyword_jiaofu() {
        let text = "月底交付 v2.0 版本";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1, "应识别 '交付' 为截止日期关键词");
    }

    #[test]
    fn test_time_expression_with_verb() {
        let text = "明天下午5点完成代码发布";
        let tasks = discover_from_message(text);
        assert_eq!(tasks.len(), 1);
        // 应该通过路径 2（deadline keyword）或路径 3（time + verb）匹配
        assert!(tasks[0].due_date.is_some());
        let due = tasks[0].due_date.as_ref().unwrap();
        assert!(due.contains("明天"), "due_date 应包含 '明天'，实际: {}", due);
    }

    #[test]
    fn test_extract_due_date_relative() {
        assert!(extract_due_date("明天下午5点").is_some());
        assert!(extract_due_date("下周一").is_some());
        assert!(extract_due_date("2026-06-15").is_some());
        assert!(extract_due_date("普通文本没有时间").is_none());
    }

    #[test]
    fn test_has_time_expression() {
        assert!(has_time_expression("明天下午5点完成"));
        assert!(has_time_expression("下周一开会"));
        assert!(has_time_expression("15点提交"));
        assert!(!has_time_expression("普通聊天内容"));
    }

    #[test]
    fn test_no_false_positive_on_wancheng_alone() {
        // "完成" 单独出现但没有时间上下文时，仍应匹配（它是 DEADLINE_KEYWORD）
        let text = "已经完成代码审查";
        let tasks = discover_from_message(text);
        // "完成" 后面是 "代码审查"，应作为任务
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_greeting_with_today_no_false_positive() {
        // "今天辛苦了" 不应匹配（没有动词性词汇）
        let text = "今天辛苦了";
        let tasks = discover_from_message(text);
        assert!(tasks.is_empty());
    }
}
