//! ReviewAgent —— 基于规则的质量审核代理
//!
//! 支持分层审核：
//! - 规则层：RequiredFields、ConfidenceThreshold、ContentLength、CategoryConsistency
//! - 小模型层：对 Review/Document 类别追加 SmallModelReview 一致性检查

use std::cell::RefCell;

use tracing;
use wb_core::audit::{Issue, ReviewResult, ReviewVerdict};
use wb_core::record::{Category, WorkRecord};

use crate::review::OutputType;
use crate::review::{
    ConfirmRequest, DataScope, LargeModelReview, ProcessorOutput, ReviewModel, TieredReview,
    UserConfirmPush,
};
use crate::review_rules::{
    CategoryConsistencyRule, ConfidenceThresholdRule, ContentLengthRule, RequiredFieldsRule,
    ReviewRule,
};

/// 审核代理，使用规则链对 WorkRecord 进行质量检查
///
/// 可选地集成 TieredReview，对 Review/Document 类别启用小模型审核。
pub struct ReviewAgent {
    rules: Vec<Box<dyn ReviewRule + Send + Sync>>,
    tiered_review: Option<TieredReview>,
    large_model_reviewer: Option<LargeModelReview>,
    user_push: RefCell<UserConfirmPush>,
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
            tiered_review: None,
            large_model_reviewer: None,
            user_push: RefCell::new(UserConfirmPush::new()),
        }
    }

    /// 追加自定义规则
    pub fn with_rule(mut self, rule: Box<dyn ReviewRule + Send + Sync>) -> Self {
        self.rules.push(rule);
        self
    }

    /// 设置分层审核策略
    ///
    /// 启用后，Review/Document 类别的记录将额外经过 SmallModelReview。
    pub fn with_tiered_review(mut self, tiered: TieredReview) -> Self {
        self.tiered_review = Some(tiered);
        self
    }

    /// 设置大模型审核策略
    ///
    /// 启用后，当 detail > 500 字或 people >= 3 时，追加大模型审核。
    pub fn with_large_model_review(mut self, large: LargeModelReview) -> Self {
        self.large_model_reviewer = Some(large);
        self
    }

    /// 获取待确认推送数量
    pub fn pending_confirm_count(&self) -> usize {
        self.user_push.borrow().pending_count()
    }

    /// 审核 WorkRecord，返回审核结果
    ///
    /// 审核策略：
    /// 1. 规则层审核（所有记录）
    /// 2. 小模型审核（Review/Document 类别，如果配置了 TieredReview）
    /// 3. 大模型审核（detail > 500 或 people >= 3，如果配置了 LargeModelReview）
    ///    大模型审核失败时降级到前一层结果
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
        let rule_issues: Vec<Issue> = self
            .rules
            .iter()
            .filter_map(|rule| rule.check(record))
            .collect();

        // 判断审核策略：
        // 1. 涉及他人（people > 0）→ 规则 + 小模型（轻量一致性检查）
        // 2. Review/Document 类别 → 规则 + TieredReview（根据类型选择）
        // 3. 其他 → 仅规则
        let involving_others = !record.people.is_empty();
        let is_review_or_doc = matches!(record.category, Category::Review | Category::Document);

        let base_result = if involving_others && self.tiered_review.is_some() {
            // 涉及他人：直接使用 SmallModelReview（不经过 TieredReview 路由）
            self.review_with_small_model_direct(record, rule_issues)
        } else if is_review_or_doc && self.tiered_review.is_some() {
            // Review/Document 类别：使用 TieredReview
            self.review_with_small_model(record, rule_issues)
        } else {
            self.review_with_rules_only(rule_issues)
        };

        // 判断是否需要大模型审核
        let result = if self.needs_large_model_review(record) {
            self.review_with_large_model(record, base_result)
        } else {
            base_result
        };

        // 涉及他人 + 非 NeedsFix → 创建确认推送请求
        // Approved 和 NeedsReview 都应触发推送，只有 NeedsFix 不推送
        if !record.people.is_empty() && !matches!(result.verdict, ReviewVerdict::NeedsFix(_)) {
            self.create_people_confirm_request(record);
        }

        result
    }

    /// 判断是否需要大模型审核
    ///
    /// 条件：配置了 LargeModelReview 且 (detail > 500 字 或 people >= 3)
    fn needs_large_model_review(&self, record: &WorkRecord) -> bool {
        self.large_model_reviewer.is_some()
            && (record.detail.chars().count() > 500 || record.people.len() >= 3)
    }

    /// 为涉及他人的记录创建确认推送请求
    fn create_people_confirm_request(&self, record: &WorkRecord) {
        let people_str = record.people.join("、");
        let request = ConfirmRequest {
            id: format!("confirm-{}", record.id),
            content: record.title.clone(),
            reason: format!("涉及人员：{}", people_str),
            data_scope: DataScope::Shared,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        match self.user_push.borrow_mut().push(request) {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("user_confirm_push_failed: {}", e);
            }
        }
    }

    /// 大模型审核（带降级策略）
    ///
    /// 大模型审核失败时，返回前一层（小模型/规则层）的结果。
    fn review_with_large_model(
        &self,
        record: &WorkRecord,
        base_result: ReviewResult,
    ) -> ReviewResult {
        let large = self
            .large_model_reviewer
            .as_ref()
            .expect("large_model_reviewer should be set");

        let output = work_record_to_processor_output(record);
        let large_result = large.review(&output);

        // 降级策略：大模型审核结果 confidence 过低时使用基础结果
        if large_result.confidence < 0.3 {
            return base_result;
        }

        // 合并基础问题和大模型问题
        let mut all_issues = base_result.issues;
        all_issues.extend(large_result.issues);

        let verdict = determine_verdict(&all_issues);

        ReviewResult {
            verdict,
            issues: all_issues,
            reviewer: "large_model".to_string(),
            confidence: large_result.confidence,
        }
    }

    /// 仅规则层审核
    fn review_with_rules_only(&self, issues: Vec<Issue>) -> ReviewResult {
        let verdict = determine_verdict(&issues);
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

    /// 规则层 + 小模型层审核（Review/Document 类别，经过 TieredReview 路由）
    fn review_with_small_model(
        &self,
        record: &WorkRecord,
        rule_issues: Vec<Issue>,
    ) -> ReviewResult {
        let tiered = self.tiered_review.as_ref().expect("tiered_review should be set");
        let output = work_record_to_processor_output(record);
        let small_result = tiered.review(&output);

        // 合并规则层和小模型层的问题
        let mut all_issues = rule_issues;
        all_issues.extend(small_result.issues);

        let verdict = determine_verdict(&all_issues);

        ReviewResult {
            verdict,
            issues: all_issues,
            reviewer: "small_model".to_string(),
            confidence: small_result.confidence,
        }
    }

    /// 规则层 + 直接小模型审核（涉及他人场景，不经过 TieredReview 路由）
    ///
    /// 直接使用 SmallModelReview 进行一致性检查，避免 TieredReview 将 Task 类别
    /// 路由到 LargeModelReview。
    fn review_with_small_model_direct(
        &self,
        record: &WorkRecord,
        rule_issues: Vec<Issue>,
    ) -> ReviewResult {
        let tiered = self.tiered_review.as_ref().expect("tiered_review should be set");
        let output = work_record_to_processor_output(record);
        // 直接使用 small_reviewer，不经过 tiered.review() 路由
        let small_result = tiered.small_reviewer().review(&output);

        // 合并规则层和小模型层的问题
        let mut all_issues = rule_issues;
        all_issues.extend(small_result.issues);

        let verdict = determine_verdict(&all_issues);

        ReviewResult {
            verdict,
            issues: all_issues,
            reviewer: "small_model".to_string(),
            confidence: small_result.confidence,
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

/// 将 WorkRecord 转换为 ProcessorOutput，供 TieredReview 使用
fn work_record_to_processor_output(record: &WorkRecord) -> ProcessorOutput {
    let output_type = match record.category {
        Category::Review | Category::Document => OutputType::Summary,
        _ => OutputType::Analysis,
    };

    let mut entities = record.people.clone();
    entities.extend(record.tags.clone());

    ProcessorOutput {
        output_type,
        content: record.detail.clone(),
        confidence: record.confidence,
        entities,
    }
}

/// 根据问题列表决定审核结论
fn determine_verdict(issues: &[Issue]) -> ReviewVerdict {
    let has_critical = issues.iter().any(|i| i.severity == "critical");
    let has_high = issues.iter().any(|i| i.severity == "high");
    let has_medium = issues.iter().any(|i| i.severity == "medium");

    if has_critical || has_high {
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
    }
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "reviewer_tests.rs"]
mod tests;
