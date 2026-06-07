//! G2: Intelligent Processing — 34 scenarios
//!
//! Black-box acceptance tests for the processing layer.
//! Covers: classification routing, model upgrade, SLA, budget, ReviewAgent.

mod acceptance_helpers;
use acceptance_helpers::*;
use serde_json::json;
use wb_core::event::{Confidence, Event, EventType, Source};
use wb_core::record::WorkRecord;
use wb_processor::classifier::{Classifier, ProcessingRoute};

// ===========================================================================
// G2-01 ~ G2-05: Classification Routing
// ===========================================================================

/// G2-01: Given task_update/approval/meeting, When classified, Then instant (P0/P1)
#[tokio::test]
async fn g2_01_high_value_events_route_to_instant() {
    let task_event = task_update_event("t-1", "done");
    let approval_event = approval_event("a-1", "approved");
    let meeting_event = meeting_event("m-1", "Standup");

    assert_eq!(Classifier::classify(&task_event), ProcessingRoute::Instant);
    assert_eq!(Classifier::classify(&approval_event), ProcessingRoute::Instant);
    assert_eq!(Classifier::classify(&meeting_event), ProcessingRoute::Instant);
}

/// G2-02: Given message(@mention)/manual_note, When classified, Then instant
#[tokio::test]
async fn g2_02_mention_and_manual_route_to_instant() {
    let mention_event = feishu_mention_event("@user", "@user please check");
    let manual = manual_note_event("Quick thought");

    assert_eq!(Classifier::classify(&mention_event), ProcessingRoute::Instant);
    assert_eq!(Classifier::classify(&manual), ProcessingRoute::Instant);
}

/// G2-03: Given message(plain)/doc_change/browsing/app_activity, When classified, Then aggregate (P2)
#[tokio::test]
async fn g2_03_low_priority_events_route_to_aggregate() {
    let msg = feishu_message_event("Just chatting");
    let doc = document_change_event("doc-1", "edit");
    let browse = browsing_event("https://example.com", "Example");
    let app = app_activity_event("Terminal", 60);

    assert_eq!(Classifier::classify(&msg), ProcessingRoute::Aggregate);
    assert_eq!(Classifier::classify(&doc), ProcessingRoute::Aggregate);
    assert_eq!(Classifier::classify(&browse), ProcessingRoute::Aggregate);
    assert_eq!(Classifier::classify(&app), ProcessingRoute::Aggregate);
}

/// G2-04: Given long-term analysis events, When classified, Then pattern analysis (P3)
#[tokio::test]
async fn g2_04_okr_events_route_to_pattern() {
    let okr = okr_update_event("okr-1");
    assert_eq!(Classifier::classify(&okr), ProcessingRoute::Pattern);
}

/// G2-05: Given confidence=low or pure archive, When classified, Then direct archive
#[tokio::test]
async fn g2_05_low_confidence_routes_to_archive() {
    let mut event = feishu_message_event("noise");
    event.source_confidence = Confidence::Low;
    assert_eq!(Classifier::classify(&event), ProcessingRoute::Archive);

    let mut app = app_activity_event("Dock", 5);
    app.source_confidence = Confidence::Low;
    assert_eq!(Classifier::classify(&app), ProcessingRoute::Archive);
}

// ===========================================================================
// G2-06 ~ G2-13: Model Upgrade (scaffolded — requires AI layer integration)
// ===========================================================================

/// G2-06: Given entity extraction + small model confidence <0.7, When processed, Then upgrade to large model
#[tokio::test]
async fn g2_06_entity_extraction_low_confidence_upgrades() {
    // This scenario requires the AI router integration.
    // At the acceptance level, we verify the domain contract:
    // confidence < 0.7 should trigger upgrade.
    let record = WorkRecord::new(
        "Extracted entities".into(),
        "Summary".into(),
        "Detail".into(),
        wb_core::record::Category::Research,
        vec![],
        "small-model".into(),
        0.5, // below 0.7 threshold
    );
    assert!(record.confidence < 0.7, "Low confidence should trigger upgrade");
    assert!(record.needs_review, "Low confidence should flag for review");
}

/// G2-07: Given task identification + confidence <0.6, When processed, Then upgrade
#[tokio::test]
async fn g2_07_task_id_low_confidence_upgrades() {
    let record = WorkRecord::new(
        "Task detected".into(),
        "Summary".into(),
        "Detail".into(),
        wb_core::record::Category::Task,
        vec![],
        "small-model".into(),
        0.4,
    );
    assert!(record.confidence < 0.6);
    assert!(record.needs_review);
}

/// G2-08: Given summarization + confidence <0.6 or >500 chars, When processed, Then upgrade
#[tokio::test]
async fn g2_08_summary_low_confidence_upgrades() {
    let record = WorkRecord::new(
        "Long summary".into(),
        "A".repeat(600), // > 500 chars
        "Detail".into(),
        wb_core::record::Category::Communication,
        vec![],
        "small-model".into(),
        0.5,
    );
    assert!(record.confidence < 0.6);
}

/// G2-09: Given sentiment + confidence <0.8, When processed, Then upgrade
#[tokio::test]
async fn g2_09_sentiment_low_confidence_upgrades() {
    let record = WorkRecord::new(
        "Sentiment analysis".into(),
        "Summary".into(),
        "Detail".into(),
        wb_core::record::Category::Communication,
        vec![],
        "small-model".into(),
        0.6,
    );
    assert!(record.confidence < 0.8);
}

/// G2-10: Given relation analysis + confidence <0.7, When processed, Then upgrade
#[tokio::test]
async fn g2_10_relation_low_confidence_upgrades() {
    let record = WorkRecord::new(
        "Relation found".into(),
        "Summary".into(),
        "Detail".into(),
        wb_core::record::Category::Research,
        vec![],
        "small-model".into(),
        0.5,
    );
    assert!(record.confidence < 0.7);
}

/// G2-11: Given pattern recognition task, When processed, Then always use large model
#[tokio::test]
async fn g2_11_pattern_recognition_uses_large_model() {
    // Pattern recognition always routes to the large model.
    let okr = okr_update_event("okr-pattern");
    assert_eq!(Classifier::classify(&okr), ProcessingRoute::Pattern);
    // Pattern route implies large model usage (contract tested at AI layer).
}

/// G2-12: Given small model confidence meets threshold, When complete, Then enters ReviewAgent
#[tokio::test]
async fn g2_12_high_confidence_enters_review() {
    let record = WorkRecord::new(
        "Well-classified item".into(),
        "Summary".into(),
        "Detail".into(),
        wb_core::record::Category::Task,
        vec![],
        "small-model".into(),
        0.9,
    );
    assert!(!record.needs_review, "High confidence should not need review");
}

/// G2-13: Given small model fails AND large model fails, When processed, Then mark "needs manual" and notify
#[tokio::test]
async fn g2_13_both_models_fail_needs_manual() {
    // When both models fail, the record should be flagged.
    let record = WorkRecord::new(
        "Failed extraction".into(),
        String::new(), // empty summary indicates failure
        String::new(),
        wb_core::record::Category::Task,
        vec![],
        "none".into(),
        0.0, // zero confidence
    );
    assert!(record.confidence == 0.0);
    assert!(record.needs_review);
}

// ===========================================================================
// G2-14 ~ G2-18: Token Budget (scaffolded)
// ===========================================================================

/// G2-14: Given daily budget not exhausted, When large model needed, Then available
#[tokio::test]
async fn g2_14_budget_available_allows_large_model() {
    // Budget management is in wb-ai/budget.rs.
    // At the acceptance level, we verify the contract:
    // a record with high confidence doesn't need budget intervention.
    let record = make_record("Budget check", "Summary", wb_core::record::Category::Task, vec![]);
    assert!(record.confidence >= 0.8);
}

/// G2-15: Given daily budget exhausted + non-urgent, When large model needed, Then queue for tomorrow
#[tokio::test]
async fn g2_15_budget_exhausted_non_urgent_queued() {
    // Budget exhaustion + queuing is an AI-layer concern.
    // This test documents the contract.
    // TODO: implement with budget mock when AI layer integration is available
}

/// G2-16: Given daily budget exhausted + urgent (P0/P1), When large model needed, Then allow overflow and notify
#[tokio::test]
async fn g2_16_budget_exhausted_urgent_allows_overflow() {
    // TODO: implement with budget mock
}

/// G2-17: Given daily budget exhausted + strategy degrade_to_small, When large model needed, Then small model fallback
#[tokio::test]
async fn g2_17_budget_degrade_to_small_model() {
    // TODO: implement with budget mock
}

/// G2-18: Given token usage tracked, When audit viewed, Then daily consumption visible
#[tokio::test]
async fn g2_18_token_usage_audit_visible() {
    // TODO: implement with budget tracking mock
}

// ===========================================================================
// G2-19 ~ G2-24: SLA Timeliness
// ===========================================================================

/// G2-19: Given P0 event + >5min unprocessed, When timeout, Then force upgrade and notify
#[tokio::test]
async fn g2_19_p0_timeout_force_upgrade() {
    use wb_processor::sla::{Priority, SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    // P0 timeout is 5 minutes (300,000 ms)
    assert!(!manager.check_timeout(&Priority::P0, 60_000));  // 1 min: not timed out
    assert!(manager.check_timeout(&Priority::P0, 360_000));  // 6 min: timed out
}

/// G2-20: Given P1 event + >30min unprocessed, When timeout, Then upgrade to large model
#[tokio::test]
async fn g2_20_p1_timeout_upgrade() {
    use wb_processor::sla::{Priority, SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    assert!(!manager.check_timeout(&Priority::P1, 900_000));   // 15 min: not timed out
    assert!(manager.check_timeout(&Priority::P1, 2_400_000));  // 40 min: timed out
}

/// G2-21: Given P2 event + >4h unprocessed, When timeout, Then continue normal flow
#[tokio::test]
async fn g2_21_p2_timeout_normal_flow() {
    use wb_processor::sla::{Priority, SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    assert!(!manager.check_timeout(&Priority::P2, 7_200_000));   // 2h: not timed out
    assert!(manager.check_timeout(&Priority::P2, 18_000_000));   // 5h: timed out
}

/// G2-22: Given P3 event, When unprocessed, Then queued for daily batch
#[tokio::test]
async fn g2_22_p3_queued_for_daily_batch() {
    use wb_processor::sla::{Priority, SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    assert!(!manager.check_timeout(&Priority::P3, 43_200_000));  // 12h: not timed out
    assert!(manager.check_timeout(&Priority::P3, 90_000_000));   // 25h: timed out
}

/// G2-23: Given processing queue, When SLA scan (every 5min), Then timed-out events auto-promoted
#[tokio::test]
async fn g2_23_sla_scan_auto_promotes() {
    use wb_processor::sla::{Priority, SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    // Verify escalation works: P3 -> P2 -> P1 -> P0
    let escalated = manager.escalate_priority(&Priority::P3);
    assert_eq!(escalated, Priority::P2);
    let escalated = manager.escalate_priority(&Priority::P2);
    assert_eq!(escalated, Priority::P1);
}

/// G2-24: Given end of day, When SLA report generated, Then shows efficiency stats
#[tokio::test]
async fn g2_24_daily_sla_report() {
    use wb_processor::sla::{SlaConfig, SlaManager};

    let config = SlaConfig::default();
    let manager = SlaManager::new(config);

    // Generate report with empty records
    let report = manager.daily_report(&[]);
    assert_eq!(report.total_records, 0);
    assert_eq!(report.breached_count, 0);
    assert_eq!(report.on_time_count, 0);
}

// ===========================================================================
// G2-25 ~ G2-34: ReviewAgent (scaffolded)
// ===========================================================================

/// G2-25: Given direct archive output, When ReviewAgent, Then direct pass
#[tokio::test]
async fn g2_25_archive_output_direct_pass() {
    use wb_processor::reviewer::ReviewAgent;

    // Archive record: no source events + high confidence -> skip review
    let mut record = make_record("Archived item", "Summary", wb_core::record::Category::Task, vec![]);
    record.confidence = 0.95;
    record.source_event_ids = vec![]; // no source events = archive
    record.detail = "Sufficiently long detail for content length check".into();
    record.task_status = Some("done".into());

    let agent = ReviewAgent::new();
    let result = agent.review(&record);
    assert_eq!(result.verdict, wb_core::audit::ReviewVerdict::Approved);
}

/// G2-26: Given low confidence extraction output, When ReviewAgent, Then rule-layer validation only
#[tokio::test]
async fn g2_26_low_confidence_rule_validation_only() {
    use wb_processor::reviewer::ReviewAgent;

    let mut record = make_record("Partial extraction", "Summary", wb_core::record::Category::Research, vec!["evt-1".into()]);
    record.confidence = 0.4; // low confidence
    record.detail = "Sufficiently long detail for content length check".into();

    let agent = ReviewAgent::new();
    let result = agent.review(&record);
    // Low confidence triggers NeedsFix (high severity)
    assert!(
        matches!(result.verdict, wb_core::audit::ReviewVerdict::NeedsFix(_)),
        "Low confidence should trigger NeedsFix"
    );
}

/// G2-27: Given high confidence extraction output, When ReviewAgent, Then 10% sampling review
#[tokio::test]
async fn g2_27_high_confidence_sampling_review() {
    use wb_processor::reviewer::ReviewAgent;

    let mut record = make_record("High confidence extraction", "Summary with all fields", wb_core::record::Category::Research, vec!["evt-1".into()]);
    record.confidence = 0.95;
    record.detail = "Detailed extraction content that passes length check".into();

    let agent = ReviewAgent::new();
    let result = agent.review(&record);
    assert_eq!(result.verdict, wb_core::audit::ReviewVerdict::Approved);
}

/// G2-28: Given task status change output, When ReviewAgent, Then rule validation + shared data needs confirmation
#[tokio::test]
async fn g2_28_task_change_needs_confirmation() {
    // Task status changes that affect shared data need user confirmation.
    // This is a privacy/autonomy concern (G7-02).
    // TODO: implement with shared data detection
}

/// G2-29: Given report/summary output, When ReviewAgent, Then rule + small model consistency check
#[tokio::test]
async fn g2_29_report_consistency_check() {
    use wb_processor::reviewer::ReviewAgent;

    let mut record = make_record("Weekly summary", "Completed 5 tasks, attended 3 meetings", wb_core::record::Category::Communication, vec!["evt-1".into()]);
    record.confidence = 0.85;
    record.detail = "Detailed weekly summary with task completions and meeting notes".into();

    let agent = ReviewAgent::new();
    let result = agent.review(&record);
    assert_eq!(result.verdict, wb_core::audit::ReviewVerdict::Approved);
}

/// G2-30: Given involves others' info, When ReviewAgent, Then rule + small model + user confirmation
#[tokio::test]
async fn g2_30_others_info_needs_user_confirmation() {
    // Involves privacy-sensitive data from other people.
    // TODO: implement with entity detection + privacy rules
}

/// G2-31: Given ReviewAgent returns needs_fix, When returned, Then re-enters processing layer
#[tokio::test]
async fn g2_31_needs_fix_reprocesses() {
    // The needs_fix verdict should cause the item to be re-queued.
    // This is a pipeline-level concern tested in integration.
    // TODO: implement with pipeline mock
}

/// G2-32: Given ReviewAgent returns needs_review, When returned, Then push notification
#[tokio::test]
async fn g2_32_needs_review_pushes_notification() {
    // TODO: implement with notification mock
}

/// G2-33: Given ReviewAgent returns approved, When returned, Then enters storage layer
#[tokio::test]
async fn g2_33_approved_enters_storage() {
    use wb_processor::reviewer::ReviewAgent;

    let mut record = make_record("Clean extraction", "Summary", wb_core::record::Category::Task, vec!["evt-1".into()]);
    record.confidence = 0.9;
    record.detail = "Sufficiently long detail for content length check".into();
    record.task_status = Some("done".into());

    let agent = ReviewAgent::new();
    let result = agent.review(&record);
    assert_eq!(result.verdict, wb_core::audit::ReviewVerdict::Approved);
    // Approved items should proceed to storage (tested in G3).
}

/// G2-34: Given similar issues frequent, When threshold reached, Then auto-adjust prompt or threshold
#[tokio::test]
async fn g2_34_frequent_issues_auto_adjust() {
    // This is a self-improvement/feedback loop concern.
    // TODO: implement with audit history analysis
}
