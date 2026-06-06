//! ReviewAgent —— 基于规则的质量审核代理

use wb_core::audit::{ReviewResult, ReviewVerdict};
use wb_core::record::WorkRecord;

use crate::review_rules::{
    CategoryConsistencyRule, ConfidenceThresholdRule, ContentLengthRule, RequiredFieldsRule,
    ReviewRule,
};

/// 审核代理，使用规则链对 WorkRecord 进行质量检查
pub struct ReviewAgent {
    rules: Vec<Box<dyn ReviewRule + Send + Sync>>,
}

impl ReviewAgent {
    /// 创建默认规则集的审核代理
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(RequiredFieldsRule),
                Box::new(ConfidenceThresholdRule::default()),
                Box::new(ContentLengthRule::default()),
                Box::new(CategoryConsistencyRule),
            ],
        }
    }

    /// 追加自定义规则
    pub fn with_rule(mut self, rule: Box<dyn ReviewRule + Send + Sync>) -> Self {
        self.rules.push(rule);
        self
    }

    /// 审核 WorkRecord，返回审核结果
    pub fn review(&self, record: &WorkRecord) -> ReviewResult {
        // 判断是否跳过审核
        if !self.should_review(record) {
            return ReviewResult {
                verdict: ReviewVerdict::Approved,
                issues: Vec::new(),
                reviewer: "rule".to_string(),
                confidence: 1.0,
            };
        }

        // 运行所有规则，收集问题
        let issues: Vec<_> = self
            .rules
            .iter()
            .filter_map(|rule| rule.check(record))
            .collect();

        // 根据问题严重程度决定 verdict
        let has_critical = issues.iter().any(|i| i.severity == "critical");
        let has_high = issues.iter().any(|i| i.severity == "high");
        let has_medium = issues.iter().any(|i| i.severity == "medium");

        let verdict = if has_critical || has_high {
            let summary = issues
                .iter()
                .filter(|i| i.severity == "critical" || i.severity == "high")
                .map(|i| i.description.clone())
                .collect::<Vec<_>>()
                .join("; ");
            ReviewVerdict::NeedsFix(summary)
        } else if has_medium {
            let summary = issues
                .iter()
                .filter(|i| i.severity == "medium")
                .map(|i| i.description.clone())
                .collect::<Vec<_>>()
                .join("; ");
            ReviewVerdict::NeedsReview(summary)
        } else {
            ReviewVerdict::Approved
        };

        // 计算审核置信度：基于规则通过率
        let passed = self.rules.len() - issues.len();
        let confidence = if self.rules.is_empty() {
            1.0
        } else {
            passed as f64 / self.rules.len() as f64
        };

        ReviewResult {
            verdict,
            issues,
            reviewer: "rule".to_string(),
            confidence,
        }
    }

    /// 判断是否需要审核
    ///
    /// - 直接归档的记录（置信度极高且 source_event_ids 为空）跳过审核
    /// - 其他记录均需审核
    fn should_review(&self, record: &WorkRecord) -> bool {
        // 直接归档：无来源事件且高置信度 → 认为是手动创建的归档记录
        if record.source_event_ids.is_empty() && record.confidence >= 0.9 {
            return false;
        }
        true
    }
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        assert!(result.issues.iter().any(|i| i.issue_type == "missing_field"));
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
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "format_error"));
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
}
