//! 规则定义 —— 用于 ReviewAgent 的质量检查规则

use wb_core::audit::Issue;
use wb_core::record::WorkRecord;

/// 审核规则 trait
pub trait ReviewRule {
    /// 规则名称
    fn name(&self) -> &str;

    /// 检查记录，返回 None 表示通过，Some(Issue) 表示发现问题
    fn check(&self, record: &WorkRecord) -> Option<Issue>;
}

/// 必填字段规则 —— 检查 title、summary、detail 是否为空
pub struct RequiredFieldsRule;

impl ReviewRule for RequiredFieldsRule {
    fn name(&self) -> &str {
        "required_fields"
    }

    fn check(&self, record: &WorkRecord) -> Option<Issue> {
        let mut missing = Vec::new();

        if record.title.trim().is_empty() {
            missing.push("title");
        }
        if record.summary.trim().is_empty() {
            missing.push("summary");
        }
        if record.detail.trim().is_empty() {
            missing.push("detail");
        }

        if missing.is_empty() {
            None
        } else {
            Some(Issue {
                issue_type: "missing_field".to_string(),
                severity: "critical".to_string(),
                description: format!("Required fields are empty: {}", missing.join(", ")),
                suggestion: format!(
                    "Ensure the AI extraction populates: {}",
                    missing.join(", ")
                ),
            })
        }
    }
}

/// 置信度阈值规则 —— 低置信度记录需要人工审核
pub struct ConfidenceThresholdRule {
    /// 置信度阈值，低于此值触发审核
    pub threshold: f64,
}

impl Default for ConfidenceThresholdRule {
    fn default() -> Self {
        Self { threshold: 0.6 }
    }
}

impl ReviewRule for ConfidenceThresholdRule {
    fn name(&self) -> &str {
        "confidence_threshold"
    }

    fn check(&self, record: &WorkRecord) -> Option<Issue> {
        if record.confidence < self.threshold {
            Some(Issue {
                issue_type: "low_confidence".to_string(),
                severity: "high".to_string(),
                description: format!(
                    "Confidence {:.2} is below threshold {:.2}",
                    record.confidence, self.threshold
                ),
                suggestion: "Consider re-processing with a larger model or manual review"
                    .to_string(),
            })
        } else {
            None
        }
    }
}

/// 内容长度规则 —— detail 过短可能表示提取不完整
pub struct ContentLengthRule {
    /// detail 最小字符数
    pub min_length: usize,
}

impl Default for ContentLengthRule {
    fn default() -> Self {
        Self { min_length: 10 }
    }
}

impl ReviewRule for ContentLengthRule {
    fn name(&self) -> &str {
        "content_length"
    }

    fn check(&self, record: &WorkRecord) -> Option<Issue> {
        if record.detail.len() < self.min_length {
            Some(Issue {
                issue_type: "format_error".to_string(),
                severity: "medium".to_string(),
                description: format!(
                    "Detail content is too short ({} chars, minimum {})",
                    record.detail.len(),
                    self.min_length
                ),
                suggestion: "The AI extraction may have produced incomplete output".to_string(),
            })
        } else {
            None
        }
    }
}

/// 分类一致性规则 —— 检查 category 与 task_status 的一致性
pub struct CategoryConsistencyRule;

impl ReviewRule for CategoryConsistencyRule {
    fn name(&self) -> &str {
        "category_consistency"
    }

    fn check(&self, record: &WorkRecord) -> Option<Issue> {
        use wb_core::record::Category;

        // Task 类型应有 task_status
        if record.category == Category::Task && record.task_status.is_none() {
            return Some(Issue {
                issue_type: "invalid_state".to_string(),
                severity: "medium".to_string(),
                description: "Category is Task but task_status is missing".to_string(),
                suggestion: "Ensure task_status is set for task-type records".to_string(),
            });
        }

        // Non-task 类型不应有 task_status
        if record.category != Category::Task && record.task_status.is_some() {
            return Some(Issue {
                issue_type: "invalid_state".to_string(),
                severity: "low".to_string(),
                description: format!(
                    "Category is {:?} but task_status is set",
                    record.category
                ),
                suggestion: "Consider if category should be Task, or remove task_status"
                    .to_string(),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wb_core::record::{Category, WorkRecord};

    fn make_record(confidence: f64) -> WorkRecord {
        WorkRecord::new(
            "Test Title".to_string(),
            "Test summary with enough content".to_string(),
            "Detail content that is long enough to pass the length check".to_string(),
            Category::Task,
            vec!["evt-1".to_string()],
            "gpt-4".to_string(),
            confidence,
        )
    }

    // --- RequiredFieldsRule ---

    #[test]
    fn test_required_fields_passes_with_valid_record() {
        let rule = RequiredFieldsRule;
        let record = make_record(0.9);
        assert!(rule.check(&record).is_none());
    }

    #[test]
    fn test_required_fields_fails_on_empty_title() {
        let rule = RequiredFieldsRule;
        let mut record = make_record(0.9);
        record.title = String::new();
        let issue = rule.check(&record).unwrap();
        assert_eq!(issue.issue_type, "missing_field");
        assert_eq!(issue.severity, "critical");
        assert!(issue.description.contains("title"));
    }

    #[test]
    fn test_required_fields_fails_on_empty_summary() {
        let rule = RequiredFieldsRule;
        let mut record = make_record(0.9);
        record.summary = "  ".to_string();
        let issue = rule.check(&record).unwrap();
        assert!(issue.description.contains("summary"));
    }

    #[test]
    fn test_required_fields_fails_on_empty_detail() {
        let rule = RequiredFieldsRule;
        let mut record = make_record(0.9);
        record.detail = String::new();
        let issue = rule.check(&record).unwrap();
        assert!(issue.description.contains("detail"));
    }

    #[test]
    fn test_required_fields_fails_on_all_empty() {
        let rule = RequiredFieldsRule;
        let mut record = make_record(0.9);
        record.title = String::new();
        record.summary = String::new();
        record.detail = String::new();
        let issue = rule.check(&record).unwrap();
        assert!(issue.description.contains("title"));
        assert!(issue.description.contains("summary"));
        assert!(issue.description.contains("detail"));
    }

    // --- ConfidenceThresholdRule ---

    #[test]
    fn test_confidence_threshold_passes_above() {
        let rule = ConfidenceThresholdRule::default();
        let record = make_record(0.7);
        assert!(rule.check(&record).is_none());
    }

    #[test]
    fn test_confidence_threshold_fails_below() {
        let rule = ConfidenceThresholdRule::default();
        let record = make_record(0.3);
        let issue = rule.check(&record).unwrap();
        assert_eq!(issue.issue_type, "low_confidence");
        assert_eq!(issue.severity, "high");
    }

    #[test]
    fn test_confidence_threshold_at_boundary() {
        let rule = ConfidenceThresholdRule::default();
        let record = make_record(0.6);
        assert!(rule.check(&record).is_none(), "0.6 should not trigger (threshold is < 0.6)");
    }

    // --- ContentLengthRule ---

    #[test]
    fn test_content_length_passes() {
        let rule = ContentLengthRule::default();
        let record = make_record(0.9);
        assert!(rule.check(&record).is_none());
    }

    #[test]
    fn test_content_length_fails_short_detail() {
        let rule = ContentLengthRule::default();
        let mut record = make_record(0.9);
        record.detail = "short".to_string();
        let issue = rule.check(&record).unwrap();
        assert_eq!(issue.issue_type, "format_error");
        assert_eq!(issue.severity, "medium");
    }

    // --- CategoryConsistencyRule ---

    #[test]
    fn test_category_consistency_task_with_status() {
        let rule = CategoryConsistencyRule;
        let mut record = make_record(0.9);
        record.task_status = Some("in_progress".to_string());
        assert!(rule.check(&record).is_none());
    }

    #[test]
    fn test_category_consistency_task_without_status() {
        let rule = CategoryConsistencyRule;
        let record = make_record(0.9);
        let issue = rule.check(&record).unwrap();
        assert_eq!(issue.issue_type, "invalid_state");
        assert_eq!(issue.severity, "medium");
    }

    #[test]
    fn test_category_consistency_non_task_with_status() {
        let rule = CategoryConsistencyRule;
        let mut record = make_record(0.9);
        record.category = Category::Meeting;
        record.task_status = Some("done".to_string());
        let issue = rule.check(&record).unwrap();
        assert_eq!(issue.issue_type, "invalid_state");
        assert_eq!(issue.severity, "low");
    }

    #[test]
    fn test_category_consistency_non_task_without_status() {
        let rule = CategoryConsistencyRule;
        let mut record = make_record(0.9);
        record.category = Category::Meeting;
        assert!(rule.check(&record).is_none());
    }
}
