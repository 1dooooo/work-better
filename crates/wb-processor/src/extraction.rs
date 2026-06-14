//! EntityExtractor —— 从模型输出中提取结构化数据

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, Utc};
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
    /// AI 提取的截止日期原文（如 "明天下午5点"、"2026-06-15"）
    pub due_date: Option<String>,
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
            due_date: extraction.due_date.clone(),
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

        let due_date = parsed
            .get("due_date")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "null")
            .map(String::from);

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
            due_date,
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
            task_due: data.due_date.as_deref().and_then(parse_due_date_from_text),
            task_priority: None,
            task_progress: None,
            model_used: model_used.to_string(),
            confidence: data.confidence,
            needs_review: data.confidence < 0.8,
            obsidian_path: String::new(),
        }
    }
}

/// 解析 AI 提取的截止日期字符串为 `DateTime<Utc>`
///
/// 支持：
/// - ISO 格式: "2026-06-15"、"2026-06-15 17:00"、"2026/06/15"
/// - 相对表达: "明天"、"后天"、"下周一"~"下周日"、"本周五"、"月底"
/// - 时间附着: "明天下午5点"、"下周一10:00"
pub fn parse_due_date_from_text(raw: &str) -> Option<DateTime<Utc>> {
    let now = Local::now();
    let today = now.date_naive();

    // 1. 尝试 ISO 格式
    if let Some(date) = try_parse_iso_date(raw) {
        let time = extract_time_from_text(raw).unwrap_or(NaiveTime::from_hms_opt(18, 0, 0)?);
        let naive_dt = date.and_time(time);
        return Some(naive_to_local_utc(naive_dt));
    }

    // 2. 中文相对日期
    let base_date = if raw.contains("后天") {
        today + chrono::Duration::days(2)
    } else if raw.contains("明天") {
        today + chrono::Duration::days(1)
    } else if raw.contains("今天") {
        today
    } else if let Some(weekday) = extract_next_weekday(raw, today) {
        weekday
    } else if raw.contains("月底") {
        let next_month = if today.month() == 12 {
            NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)?
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)?
        };
        next_month - chrono::Duration::days(1)
    } else if raw.contains("年底") || raw.contains("年末") {
        NaiveDate::from_ymd_opt(today.year(), 12, 31)?
    } else {
        return None;
    };

    let time = extract_time_from_text(raw).unwrap_or(NaiveTime::from_hms_opt(18, 0, 0)?);
    let naive_dt = base_date.and_time(time);
    Some(naive_to_local_utc(naive_dt))
}

/// 将 NaiveDateTime（视为本地时间）转换为 DateTime<Utc>
fn naive_to_local_utc(naive: chrono::NaiveDateTime) -> DateTime<Utc> {
    // 使用本地时区偏移将 NaiveDateTime 解释为本地时间，再转为 UTC
    let local_now = Local::now();
    let offset = *local_now.offset();
    match naive.and_local_timezone(offset) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
        _ => naive.and_utc(),
    }
}

/// 尝试解析 ISO 日期格式
fn try_parse_iso_date(text: &str) -> Option<NaiveDate> {
    for fmt in &["%Y-%m-%d", "%Y/%m/%d"] {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() >= 10 {
            let candidate: String = chars[..10].iter().collect();
            if let Ok(date) = NaiveDate::parse_from_str(&candidate, fmt) {
                return Some(date);
            }
        }
    }
    None
}

/// 从文本中提取时间（如 "下午5点"、"17:00"、"10点30分"）
fn extract_time_from_text(text: &str) -> Option<NaiveTime> {
    let chars: Vec<char> = text.chars().collect();

    // 模式 1: "HH:MM"
    for i in 0..chars.len().saturating_sub(4) {
        if chars[i].is_ascii_digit()
            && chars.get(i + 1).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 2) == Some(&':')
            && chars.get(i + 3).map_or(false, |c| c.is_ascii_digit())
            && chars.get(i + 4).map_or(false, |c| c.is_ascii_digit())
        {
            let h: u32 = chars[i..i + 2].iter().collect::<String>().parse().ok()?;
            let m: u32 = chars[i + 3..i + 5].iter().collect::<String>().parse().ok()?;
            if h < 24 && m < 60 {
                return NaiveTime::from_hms_opt(h, m, 0);
            }
        }
    }

    // 模式 2: "下午/上午X点" 或纯 "X点"
    let is_afternoon = text.contains("下午");
    let is_morning = text.contains("上午");

    for i in 0..chars.len() {
        // 两位数小时 "10点"、"15点"
        if i + 2 < chars.len()
            && chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars.get(i + 2) == Some(&'点')
        {
            let h_str: String = chars[i..i + 2].iter().collect();
            if let Ok(h) = h_str.parse::<u32>() {
                let hour = if is_afternoon && h < 12 {
                    h + 12
                } else if is_morning && h == 12 {
                    0
                } else {
                    h
                };
                if h < 24 {
                    return NaiveTime::from_hms_opt(hour, 0, 0);
                }
            }
        }
        // 单位数小时 "5点"
        if chars[i].is_ascii_digit() && chars.get(i + 1) == Some(&'点') {
            let h: u32 = chars[i].to_digit(10)?;
            let hour = if is_afternoon && h < 12 {
                h + 12
            } else if is_morning && h == 12 {
                0
            } else {
                h
            };
            // 检查 "点" 后面是否有分钟
            let minute = if i + 2 < chars.len()
                && chars[i + 2].is_ascii_digit()
                && chars.get(i + 3).map_or(true, |c| *c == '分' || !c.is_ascii_digit())
            {
                chars[i + 2].to_digit(10).unwrap_or(0)
            } else {
                0
            };
            return NaiveTime::from_hms_opt(hour, minute, 0);
        }
    }

    None
}

/// 提取 "下周X" 或 "本周X" 对应的日期
fn extract_next_weekday(text: &str, today: NaiveDate) -> Option<NaiveDate> {
    let weekdays = [
        ("周一", chrono::Weekday::Mon),
        ("周二", chrono::Weekday::Tue),
        ("周三", chrono::Weekday::Wed),
        ("周四", chrono::Weekday::Thu),
        ("周五", chrono::Weekday::Fri),
        ("周六", chrono::Weekday::Sat),
        ("周日", chrono::Weekday::Sun),
        ("周天", chrono::Weekday::Sun),
        ("星期一", chrono::Weekday::Mon),
        ("星期二", chrono::Weekday::Tue),
        ("星期三", chrono::Weekday::Wed),
        ("星期四", chrono::Weekday::Thu),
        ("星期五", chrono::Weekday::Fri),
        ("星期六", chrono::Weekday::Sat),
        ("星期日", chrono::Weekday::Sun),
        ("星期天", chrono::Weekday::Sun),
    ];

    let is_next_week = text.contains("下周") || text.contains("下个星期");
    let is_this_week = text.contains("本周") || text.contains("这周") || text.contains("这个星期");

    for (name, weekday) in &weekdays {
        if text.contains(name) {
            let current_weekday = today.weekday();
            let diff = weekday.num_days_from_monday() as i32
                - current_weekday.num_days_from_monday() as i32;
            let days_until = if is_next_week {
                let base = if diff <= 0 { diff + 7 } else { diff };
                base + 7
            } else if is_this_week {
                if diff < 0 { diff + 7 } else { diff }
            } else {
                if diff <= 0 { diff + 7 } else { diff }
            };
            return Some(today + chrono::Duration::days(days_until as i64));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
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
            due_date: None,
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
            due_date: None,
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

    #[test]
    fn test_parse_due_date_from_text_iso() {
        let dt = parse_due_date_from_text("2026-06-15").unwrap();
        // 默认时间 18:00 本地 → 转为 UTC 后日期可能偏移，用本地时间验证
        let local_dt = dt.with_timezone(&Local);
        assert_eq!(local_dt.format("%Y-%m-%d").to_string(), "2026-06-15");
    }

    #[test]
    fn test_parse_due_date_from_text_iso_with_time() {
        let dt = parse_due_date_from_text("2026-06-15 14:30").unwrap();
        // 返回 UTC，转回本地时间应为 14:30
        let local_dt = dt.with_timezone(&Local);
        assert_eq!(local_dt.format("%Y-%m-%d %H:%M").to_string(), "2026-06-15 14:30");
    }

    #[test]
    fn test_parse_due_date_from_text_tomorrow() {
        let dt = parse_due_date_from_text("明天").unwrap();
        let expected = (Local::now().date_naive() + chrono::Duration::days(1)).and_hms_opt(18, 0, 0).unwrap().and_utc();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), expected.format("%Y-%m-%d").to_string());
    }

    #[test]
    fn test_parse_due_date_from_text_tomorrow_with_time() {
        let dt = parse_due_date_from_text("明天下午5点").unwrap();
        let expected_date = Local::now().date_naive() + chrono::Duration::days(1);
        assert_eq!(dt.format("%Y-%m-%d").to_string(), expected_date.format("%Y-%m-%d").to_string());
        // 本地时间应为 17 点（下午5点）
        let local_dt = dt.with_timezone(&Local);
        assert_eq!(local_dt.hour(), 17, "明天下午5点应为本地 17 点，实际: {}", local_dt);
    }

    #[test]
    fn test_parse_due_date_from_text_next_week() {
        let dt = parse_due_date_from_text("下周一").unwrap();
        assert_eq!(dt.weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn test_parse_due_date_from_text_none() {
        assert!(parse_due_date_from_text("普通文本").is_none());
        assert!(parse_due_date_from_text("").is_none());
    }

    #[test]
    fn test_to_work_record_propagates_due_date() {
        let data = ExtractedData {
            title: "Test".to_string(),
            summary: "Sum".to_string(),
            detail: "Det".to_string(),
            category: Category::Task,
            project: None,
            people: vec![],
            tags: vec![],
            task_status: Some("in_progress".to_string()),
            confidence: 0.9,
            due_date: Some("2026-06-15".to_string()),
        };
        let event = make_event();
        let record = EntityExtractor::to_work_record(&data, &event, "mock-model");
        assert!(record.task_due.is_some(), "task_due 应从 due_date 传播");
    }
}
