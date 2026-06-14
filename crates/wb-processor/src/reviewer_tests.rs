use super::*;
use wb_core::record::Category;

fn make_record(confidence: f64) -> WorkRecord {
    WorkRecord::new(
        "Test Title".to_string(),
        "Test summary".to_string(),
        "Detail content that is long enough to pass the length check".to_string(),
        Category::Task,
        vec!["evt-1".to_string()],
        "gpt-4".to_string(),
        confidence,
    )
}

#[test]
fn test_review_well_formed_record_approved() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("in_progress".to_string());

    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result.issues.is_empty());
    assert_eq!(result.reviewer, "rule");
    assert_eq!(result.confidence, 1.0);
}

#[test]
fn test_review_missing_title_needs_fix() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.title = String::new();
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
    assert!(!result.issues.is_empty());
    assert!(result
        .issues
        .iter()
        .any(|i| i.issue_type == "missing_field"));
}

#[test]
fn test_review_low_confidence_needs_fix() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.3);
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    // low_confidence is "high" severity → NeedsFix
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
    assert!(result
        .issues
        .iter()
        .any(|i| i.issue_type == "low_confidence"));
}

#[test]
fn test_review_short_detail_needs_review() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.detail = "short".to_string();
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    // Only medium severity issue → NeedsReview
    assert!(matches!(result.verdict, ReviewVerdict::NeedsReview(_)));
    assert!(result.issues.iter().any(|i| i.issue_type == "format_error"));
}

#[test]
fn test_review_skip_for_archive_record() {
    let agent = ReviewAgent::new();
    // 无来源事件 + 高置信度 → 跳过审核
    let mut record = make_record(0.95);
    record.source_event_ids = Vec::new();
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result.issues.is_empty());
}

#[test]
fn test_review_low_confidence_record_always_reviewed() {
    let agent = ReviewAgent::new();
    // 低置信度但有来源事件 → 应该审核
    let mut record = make_record(0.4);
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
}

#[test]
fn test_custom_rule_added() {
    use wb_core::audit::Issue;

    struct DuplicateTitleRule;
    impl ReviewRule for DuplicateTitleRule {
        fn name(&self) -> &str {
            "duplicate_title"
        }
        fn check(&self, record: &WorkRecord) -> Option<Issue> {
            if record.title == record.summary {
                Some(Issue {
                    issue_type: "duplicate_content".to_string(),
                    severity: "low".to_string(),
                    description: "Title and summary are identical".to_string(),
                    suggestion: "Differentiate title from summary".to_string(),
                })
            } else {
                None
            }
        }
    }

    let agent = ReviewAgent::new().with_rule(Box::new(DuplicateTitleRule));

    let mut record = make_record(0.9);
    record.title = "Same".to_string();
    record.summary = "Same".to_string();
    record.task_status = Some("done".to_string());

    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result
        .issues
        .iter()
        .any(|i| i.issue_type == "duplicate_content"));
}

#[test]
fn test_review_confidence_score_reflects_pass_rate() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.3); // triggers confidence + category (no task_status)
    record.detail = "short".to_string(); // triggers content_length

    let result = agent.review(&record);
    // 3 out of 4 rules fail → confidence = 1/4 = 0.25
    assert!(result.confidence < 0.5);
}

// ─── A6-15~22: ReviewAgent verdict combinations ────────────────

// A6-15: All rules pass → Approved
#[test]
fn test_verdict_all_rules_pass() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("in_progress".to_string());
    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result.issues.is_empty());
}

// A6-16: Only critical (missing title) → NeedsFix
#[test]
fn test_verdict_only_critical_issue() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.title = String::new();
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
}

// A6-17: Only high (low confidence) → NeedsFix
#[test]
fn test_verdict_only_high_issue() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.3);
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
}

// A6-18: Only medium (short detail) → NeedsReview
#[test]
fn test_verdict_only_medium_issue() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.detail = "short".to_string();
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsReview(_)));
}

// A6-19: Only low (non-task with task_status) → Approved
#[test]
fn test_verdict_only_low_issue() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.category = Category::Meeting;
    record.task_status = Some("done".to_string());
    record.detail = "A sufficiently long detail for length check".to_string();
    let result = agent.review(&record);
    // "low" severity is not critical/high/medium → Approved
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result.issues.iter().any(|i| i.severity == "low"));
}

// A6-20: Critical + medium → NeedsFix (critical dominates)
#[test]
fn test_verdict_critical_plus_medium() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.title = String::new(); // critical
    record.detail = "short".to_string(); // medium (format_error)
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
}

// A6-21: High + medium → NeedsFix (high dominates)
#[test]
fn test_verdict_high_plus_medium() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.3); // high (low_confidence)
    record.detail = "short".to_string(); // medium (format_error)
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
}

// A6-22: Skip review for archive record (no sources + high confidence)
#[test]
fn test_verdict_skip_archive_record() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.95);
    record.source_event_ids = Vec::new();
    record.task_status = Some("done".to_string());
    // Even if title/detail would fail, should_review returns false
    record.title = String::new();
    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert!(result.issues.is_empty());
}

// ─── Additional edge cases ─────────────────────────────────────

#[test]
fn test_reviewer_name_is_rule() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    let result = agent.review(&record);
    assert_eq!(result.reviewer, "rule");
}

#[test]
fn test_confidence_score_reflects_pass_rate() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    // All 4 rules pass → confidence = 4/4 = 1.0
    let result = agent.review(&record);
    assert_eq!(result.confidence, 1.0);
}

#[test]
fn test_confidence_score_partial_pass() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.3); // fails confidence rule
    record.task_status = Some("done".to_string());
    record.detail = "short".to_string(); // fails content_length
                                         // 2 of 4 fail → confidence = 2/4 = 0.5
    let result = agent.review(&record);
    assert!((result.confidence - 0.5).abs() < f64::EPSILON);
}

// ─── TieredReview 集成测试 ─────────────────────────────────────

/// 创建带 TieredReview 的 ReviewAgent
fn agent_with_tiered() -> ReviewAgent {
    ReviewAgent::new().with_tiered_review(TieredReview::default_config())
}

/// 创建指定类别的记录
fn make_record_with_category(category: Category) -> WorkRecord {
    WorkRecord::new(
        "Test Title".to_string(),
        "Test summary".to_string(),
        "Detail content that is long enough to pass the length check".to_string(),
        category,
        vec!["evt-1".to_string()],
        "gpt-4".to_string(),
        0.9,
    )
}

#[test]
fn test_review_report_uses_small_model() {
    let agent = agent_with_tiered();
    let record = make_record_with_category(Category::Review);
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "small_model",
        "Category::Review 应使用 small_model 审核"
    );
}

#[test]
fn test_review_task_uses_rule_only() {
    let agent = agent_with_tiered();
    let record = make_record_with_category(Category::Task);
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "rule",
        "Category::Task 应仅使用规则层审核"
    );
}

#[test]
fn test_review_document_uses_small_model() {
    let agent = agent_with_tiered();
    let record = make_record_with_category(Category::Document);
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "small_model",
        "Category::Document 应使用 small_model 审核"
    );
}

#[test]
fn test_review_meeting_uses_rule_only() {
    let agent = agent_with_tiered();
    let record = make_record_with_category(Category::Meeting);
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "rule",
        "Category::Meeting 应仅使用规则层审核"
    );
}

#[test]
fn test_without_tiered_always_uses_rule() {
    let agent = ReviewAgent::new(); // 没有 tiered_review
    let record = make_record_with_category(Category::Review);
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "rule",
        "无 tiered_review 时所有类别都应使用规则层"
    );
}

// ─── Task 1: LargeModelReview 集成测试 ────────────────────────

/// 创建带 TieredReview + LargeModelReview 的 ReviewAgent
fn agent_with_large_model() -> ReviewAgent {
    ReviewAgent::new()
        .with_tiered_review(TieredReview::default_config())
        .with_large_model_review(LargeModelReview::default())
}

// 测试1: detail > 500 字 → reviewer=="large_model"
#[test]
fn test_review_large_content_uses_large_model() {
    let agent = agent_with_large_model();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    // 生成 > 500 字的 detail
    record.detail = "这是一段很长的分析内容。".repeat(50); // > 500 chars
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "large_model",
        "detail > 500 字应使用 large_model 审核"
    );
}

// 测试2: people >= 3 → reviewer=="large_model"
#[test]
fn test_review_many_people_uses_large_model() {
    let agent = agent_with_large_model();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()];
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "large_model",
        "people >= 3 应使用 large_model 审核"
    );
}

// 测试3: 短内容 + 无 people → reviewer=="rule"
#[test]
fn test_review_simple_task_uses_rule_only() {
    let agent = agent_with_large_model();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec![];
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "rule",
        "短内容 + 无 people 应仅使用规则层审核"
    );
}

// ─── Task 2: UserConfirmPush 集成测试 ────────────────────────

// 测试4: people > 0 + Approved → 创建 ConfirmRequest
#[test]
fn test_review_involving_others_creates_confirm_request() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string(), "Bob".to_string()];

    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert_eq!(
        agent.pending_confirm_count(),
        1,
        "people > 0 + Approved 应创建 ConfirmRequest"
    );
}

// 测试5: people == 0 → 不创建 ConfirmRequest
#[test]
fn test_review_no_people_skips_push() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec![];

    let result = agent.review(&record);
    assert_eq!(result.verdict, ReviewVerdict::Approved);
    assert_eq!(
        agent.pending_confirm_count(),
        0,
        "people == 0 不应创建 ConfirmRequest"
    );
}

// ─── Task 2 扩展: 涉及他人审核逻辑 ────────────────────────────

// 涉及他人（people > 0）应使用小模型审核 + 推送
#[test]
fn test_review_involving_others_uses_small_model() {
    let agent = agent_with_tiered();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string(), "Bob".to_string()];
    // 人名需出现在内容中，否则小模型覆盖度检查会报 issue
    record.detail = "Alice 和 Bob 一起完成了详细的任务分析报告，内容足够长以通过长度检查".to_string();
    // Category::Task 且 people > 0 → 应使用 small_model
    let result = agent.review(&record);
    assert_eq!(
        result.reviewer, "small_model",
        "people > 0（涉及他人）应使用 small_model 审核"
    );
    // 同时应创建确认推送（Approved 或 NeedsReview）
    assert!(
        agent.pending_confirm_count() > 0,
        "涉及他人 + 非 NeedsFix 应创建确认推送"
    );
}

// 涉及他人 + NeedsReview 时仍应创建推送（verdict 非 NeedsFix 即可）
#[test]
fn test_review_involving_others_needs_review_still_pushes() {
    let agent = agent_with_tiered();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string()];
    // detail 包含人名但长度短 → 触发 format_error (medium) → NeedsReview
    record.detail = "Alice 完成".to_string();
    let result = agent.review(&record);
    // NeedsReview 不是 NeedsFix，应仍然创建推送
    assert!(
        matches!(
            result.verdict,
            ReviewVerdict::Approved | ReviewVerdict::NeedsReview(_)
        ),
        "NeedsReview 时仍应创建推送"
    );
    assert_eq!(agent.pending_confirm_count(), 1);
}

// 涉及他人 + NeedsFix 时不创建推送
#[test]
fn test_review_involving_others_needs_fix_no_push() {
    let agent = agent_with_tiered();
    let mut record = make_record(0.3); // 低置信度 → NeedsFix
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string()];
    let result = agent.review(&record);
    assert!(
        matches!(result.verdict, ReviewVerdict::NeedsFix(_)),
        "低置信度应触发 NeedsFix"
    );
    assert_eq!(
        agent.pending_confirm_count(),
        0,
        "NeedsFix 时不应创建推送"
    );
}

// ─── work_record_to_processor_output 测试 ──────────────────────

#[test]
fn test_work_record_to_output_review_category() {
    let record = make_record_with_category(Category::Review);
    let output = work_record_to_processor_output(&record);
    assert_eq!(output.output_type, OutputType::Summary);
    assert_eq!(output.content, record.detail);
    assert!((output.confidence - record.confidence).abs() < f64::EPSILON);
}

#[test]
fn test_work_record_to_output_task_category() {
    let record = make_record_with_category(Category::Task);
    let output = work_record_to_processor_output(&record);
    assert_eq!(output.output_type, OutputType::Analysis);
}

#[test]
fn test_work_record_to_output_includes_entities() {
    let mut record = make_record_with_category(Category::Document);
    record.people = vec!["Alice".to_string(), "Bob".to_string()];
    record.tags = vec!["important".to_string()];
    let output = work_record_to_processor_output(&record);
    assert!(output.entities.contains(&"Alice".to_string()));
    assert!(output.entities.contains(&"Bob".to_string()));
    assert!(output.entities.contains(&"important".to_string()));
}

// ─── M2: UserConfirmPush 错误处理测试 ──────────────────────────

#[test]
fn test_review_duplicate_confirm_request_handled_gracefully() {
    let agent = ReviewAgent::new();
    let mut record = make_record(0.9);
    record.task_status = Some("done".to_string());
    record.people = vec!["Alice".to_string()];

    // 第一次 review → 创建 ConfirmRequest
    let result1 = agent.review(&record);
    assert_eq!(result1.verdict, ReviewVerdict::Approved);
    assert_eq!(agent.pending_confirm_count(), 1);

    // 第二次 review 同一记录 → push() 返回 Err（重复 ID）
    // 之前 let _ = 会静默丢弃错误，现在应 warn 并继续
    let result2 = agent.review(&record);
    assert_eq!(result2.verdict, ReviewVerdict::Approved);
    // pending 数量不变（重复请求被拒绝）
    assert_eq!(
        agent.pending_confirm_count(),
        1,
        "重复的 ConfirmRequest 应被拒绝，pending 数量不变"
    );
}
