//! 审核代理进阶 —— 分层审核策略 + 用户确认推送
//!
//! 扩展 ReviewAgent 的基础审核能力，提供：
//! - SmallModelReview：小模型一致性检查
//! - LargeModelReview：大模型语义审核
//! - TieredReview：根据输出类型选择审核策略
//! - UserConfirmPush：用户确认推送管理

use std::collections::HashMap;

use wb_core::audit::{Issue, ReviewResult, ReviewVerdict};

// ─── ProcessorOutput ─────────────────────────────────────────────────

/// 处理器输出类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputType {
    Summary,
    Analysis,
    Extraction,
    Report,
}

/// 处理器输出，审核代理的输入
#[derive(Debug, Clone)]
pub struct ProcessorOutput {
    pub output_type: OutputType,
    /// 原始文本内容
    pub content: String,
    /// 输出的置信度
    pub confidence: f64,
    /// 涉及的人名 / 关键实体
    pub entities: Vec<String>,
}

// ─── ReviewModel trait ───────────────────────────────────────────────

/// 模型审核 trait，SmallModelReview 和 LargeModelReview 共同实现
pub trait ReviewModel {
    /// 审核名称，用于标识审核者
    fn reviewer_name(&self) -> &str;

    /// 执行审核
    fn review(&self, output: &ProcessorOutput) -> ReviewResult;
}

// ─── SmallModelReview ────────────────────────────────────────────────

/// 小模型审核 —— 一致性检查 + 关键信息覆盖度
///
/// 适用于 Summary 和 Extraction 类型的输出。
/// 使用轻量级检查，不调用大模型。
pub struct SmallModelReview {
    /// 关键信息最小覆盖比例（0.0 ~ 1.0）
    pub min_coverage: f64,
    /// 内容自相矛盾检测的敏感关键词对
    pub contradiction_pairs: Vec<(String, String)>,
}

impl SmallModelReview {
    pub fn new() -> Self {
        Self {
            min_coverage: 0.5,
            contradiction_pairs: vec![
                ("已批准".to_string(), "已拒绝".to_string()),
                ("已完成".to_string(), "未完成".to_string()),
                ("增加".to_string(), "减少".to_string()),
            ],
        }
    }

    /// 自定义覆盖度阈值
    pub fn with_min_coverage(mut self, coverage: f64) -> Self {
        self.min_coverage = coverage.clamp(0.0, 1.0);
        self
    }

    /// 检查内容是否存在自相矛盾
    fn check_contradiction(&self, content: &str) -> Option<Issue> {
        for (a, b) in &self.contradiction_pairs {
            if content.contains(a) && content.contains(b) {
                return Some(Issue {
                    issue_type: "contradiction".to_string(),
                    severity: "high".to_string(),
                    description: format!("内容中同时包含矛盾表述「{}」和「{}」", a, b),
                    suggestion: "请检查输出是否自相矛盾，并修正不一致的描述".to_string(),
                });
            }
        }
        None
    }

    /// 检查关键信息覆盖度
    fn check_coverage(&self, output: &ProcessorOutput) -> Option<Issue> {
        if output.entities.is_empty() {
            return None;
        }

        let covered = output
            .entities
            .iter()
            .filter(|e| output.content.contains(e.as_str()))
            .count();

        let ratio = covered as f64 / output.entities.len() as f64;
        if ratio < self.min_coverage {
            Some(Issue {
                issue_type: "low_coverage".to_string(),
                severity: "medium".to_string(),
                description: format!(
                    "关键信息覆盖度 {:.0}%（{}/{}），低于阈值 {:.0}%",
                    ratio * 100.0,
                    covered,
                    output.entities.len(),
                    self.min_coverage * 100.0
                ),
                suggestion: "输出可能遗漏了重要信息，请补充缺失的实体".to_string(),
            })
        } else {
            None
        }
    }
}

impl Default for SmallModelReview {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewModel for SmallModelReview {
    fn reviewer_name(&self) -> &str {
        "small_model"
    }

    fn review(&self, output: &ProcessorOutput) -> ReviewResult {
        let mut issues = Vec::new();

        if let Some(issue) = self.check_contradiction(&output.content) {
            issues.push(issue);
        }

        if let Some(issue) = self.check_coverage(output) {
            issues.push(issue);
        }

        let has_high = issues
            .iter()
            .any(|i| i.severity == "high" || i.severity == "critical");
        let has_medium = issues.iter().any(|i| i.severity == "medium");

        let verdict = if has_high {
            let summary = issues
                .iter()
                .filter(|i| i.severity == "high" || i.severity == "critical")
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

        let confidence = if issues.is_empty() {
            output.confidence
        } else {
            output.confidence * 0.7
        };

        ReviewResult {
            verdict,
            issues,
            reviewer: self.reviewer_name().to_string(),
            confidence,
        }
    }
}

// ─── LargeModelReview ────────────────────────────────────────────────

/// 大模型审核 —— 语义审核 + 深度分析检查
///
/// 适用于 Analysis 和 Report 类型的输出。
/// 使用更严格的语义检查。
pub struct LargeModelReview {
    /// 最小语义长度（字符数），低于此值认为分析不够深入
    pub min_depth_length: usize,
    /// 必须出现的分析维度关键词
    pub analysis_dimensions: Vec<String>,
}

impl LargeModelReview {
    pub fn new() -> Self {
        Self {
            min_depth_length: 50,
            analysis_dimensions: vec![
                "原因".to_string(),
                "影响".to_string(),
                "建议".to_string(),
                "结论".to_string(),
                "背景".to_string(),
            ],
        }
    }

    /// 自定义深度检查长度
    pub fn with_min_depth_length(mut self, length: usize) -> Self {
        self.min_depth_length = length;
        self
    }

    /// 检查语义深度 —— 分析内容是否足够深入
    fn check_depth(&self, output: &ProcessorOutput) -> Option<Issue> {
        if output.content.len() < self.min_depth_length {
            return Some(Issue {
                issue_type: "shallow_analysis".to_string(),
                severity: "medium".to_string(),
                description: format!(
                    "分析内容过短（{} 字符，最少 {}），可能不够深入",
                    output.content.len(),
                    self.min_depth_length
                ),
                suggestion: "请补充更详细的分析内容".to_string(),
            });
        }
        None
    }

    /// 检查分析维度覆盖
    fn check_dimensions(&self, output: &ProcessorOutput) -> Option<Issue> {
        if self.analysis_dimensions.is_empty() {
            return None;
        }

        let covered = self
            .analysis_dimensions
            .iter()
            .filter(|dim| output.content.contains(dim.as_str()))
            .count();

        // 至少覆盖一个维度
        if covered == 0 {
            Some(Issue {
                issue_type: "missing_dimensions".to_string(),
                severity: "high".to_string(),
                description: "分析未覆盖任何标准维度（原因/影响/建议/结论/背景）".to_string(),
                suggestion: "请确保分析至少覆盖一个关键维度".to_string(),
            })
        } else {
            None
        }
    }
}

impl Default for LargeModelReview {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewModel for LargeModelReview {
    fn reviewer_name(&self) -> &str {
        "large_model"
    }

    fn review(&self, output: &ProcessorOutput) -> ReviewResult {
        let mut issues = Vec::new();

        if let Some(issue) = self.check_depth(output) {
            issues.push(issue);
        }

        if let Some(issue) = self.check_dimensions(output) {
            issues.push(issue);
        }

        let has_high = issues
            .iter()
            .any(|i| i.severity == "high" || i.severity == "critical");

        let verdict = if has_high {
            let summary = issues
                .iter()
                .filter(|i| i.severity == "high" || i.severity == "critical")
                .map(|i| i.description.clone())
                .collect::<Vec<_>>()
                .join("; ");
            ReviewVerdict::NeedsFix(summary)
        } else if issues.iter().any(|i| i.severity == "medium") {
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

        let confidence = if issues.is_empty() {
            output.confidence
        } else {
            output.confidence * 0.6
        };

        ReviewResult {
            verdict,
            issues,
            reviewer: self.reviewer_name().to_string(),
            confidence,
        }
    }
}

// ─── TieredReview ────────────────────────────────────────────────────

/// 分层审核策略 —— 根据输出类型选择合适的审核器
///
/// - Summary / Extraction → SmallModelReview（轻量一致性检查）
/// - Analysis / Report → LargeModelReview（语义深度检查）
pub struct TieredReview {
    small_reviewer: SmallModelReview,
    large_reviewer: LargeModelReview,
}

impl TieredReview {
    pub fn new(small_reviewer: SmallModelReview, large_reviewer: LargeModelReview) -> Self {
        Self {
            small_reviewer,
            large_reviewer,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self {
            small_reviewer: SmallModelReview::new(),
            large_reviewer: LargeModelReview::new(),
        }
    }

    /// 根据输出类型选择审核策略并执行
    pub fn review(&self, output: &ProcessorOutput) -> ReviewResult {
        match output.output_type {
            OutputType::Summary | OutputType::Extraction => self.small_reviewer.review(output),
            OutputType::Analysis | OutputType::Report => self.large_reviewer.review(output),
        }
    }
}

// ─── UserConfirmPush ─────────────────────────────────────────────────

/// 数据范围
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataScope {
    /// 仅用户可见，无需确认
    Private,
    /// 涉及共享数据，需确认
    Shared,
    /// 涉及外部系统，需确认
    External,
}

/// 确认请求
#[derive(Debug, Clone)]
pub struct ConfirmRequest {
    pub id: String,
    pub content: String,
    pub reason: String,
    pub data_scope: DataScope,
    pub created_at: String,
}

/// 用户确认推送管理器
///
/// 管理需要用户确认的推送请求。
/// Private 范围的数据自动放行，Shared / External 需要用户确认。
pub struct UserConfirmPush {
    pending: HashMap<String, ConfirmRequest>,
}

impl UserConfirmPush {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// 判断该数据范围是否需要用户确认
    pub fn should_confirm(&self, data_scope: &DataScope) -> bool {
        matches!(data_scope, DataScope::Shared | DataScope::External)
    }

    /// 提交一个确认请求
    pub fn push(&mut self, request: ConfirmRequest) -> wb_core::error::Result<()> {
        if !self.should_confirm(&request.data_scope) {
            return Ok(());
        }

        if self.pending.contains_key(&request.id) {
            return Err(wb_core::error::WbError::Ai(format!(
                "确认请求 {} 已存在",
                request.id
            )));
        }

        self.pending.insert(request.id.clone(), request);
        Ok(())
    }

    /// 确认一个请求
    pub fn confirm(&mut self, id: &str) -> wb_core::error::Result<()> {
        self.pending
            .remove(id)
            .ok_or_else(|| wb_core::error::WbError::NotFound(format!("确认请求 {} 不存在", id)))?;
        Ok(())
    }

    /// 拒绝一个请求
    pub fn reject(&mut self, id: &str) -> wb_core::error::Result<()> {
        self.pending
            .remove(id)
            .ok_or_else(|| wb_core::error::WbError::NotFound(format!("确认请求 {} 不存在", id)))?;
        Ok(())
    }

    /// 获取所有待确认请求
    pub fn pending(&self) -> Vec<&ConfirmRequest> {
        self.pending.values().collect()
    }

    /// 待确认数量
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for UserConfirmPush {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── SmallModelReview tests ──────────────────────────────────

    fn make_output(output_type: OutputType, content: &str) -> ProcessorOutput {
        ProcessorOutput {
            output_type,
            content: content.to_string(),
            confidence: 0.9,
            entities: vec![],
        }
    }

    fn make_output_with_entities(
        output_type: OutputType,
        content: &str,
        entities: Vec<&str>,
    ) -> ProcessorOutput {
        ProcessorOutput {
            output_type,
            content: content.to_string(),
            confidence: 0.9,
            entities: entities.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_small_model_clean_output_approved() {
        let reviewer = SmallModelReview::new();
        let output = make_output(OutputType::Summary, "这是一个正常的摘要内容");
        let result = reviewer.review(&output);
        assert_eq!(result.verdict, ReviewVerdict::Approved);
        assert_eq!(result.reviewer, "small_model");
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_small_model_contradiction_detected() {
        let reviewer = SmallModelReview::new();
        let output = make_output(OutputType::Summary, "该项目已批准实施，但预算已拒绝");
        let result = reviewer.review(&output);
        assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "contradiction"));
    }

    #[test]
    fn test_small_model_coverage_below_threshold() {
        let reviewer = SmallModelReview::new();
        let output = make_output_with_entities(
            OutputType::Extraction,
            "只提到了张三",
            vec!["张三", "李四", "王五"],
        );
        let result = reviewer.review(&output);
        // 1/3 = 33% < 50% threshold
        assert!(result.issues.iter().any(|i| i.issue_type == "low_coverage"));
        assert!(matches!(result.verdict, ReviewVerdict::NeedsReview(_)));
    }

    #[test]
    fn test_small_model_coverage_above_threshold() {
        let reviewer = SmallModelReview::new();
        let output = make_output_with_entities(
            OutputType::Summary,
            "张三和李四都参与了讨论",
            vec!["张三", "李四", "王五"],
        );
        let result = reviewer.review(&output);
        // 2/3 = 66% > 50% threshold → no coverage issue
        assert!(!result.issues.iter().any(|i| i.issue_type == "low_coverage"));
    }

    #[test]
    fn test_small_model_custom_coverage_threshold() {
        let reviewer = SmallModelReview::new().with_min_coverage(0.9);
        let output = make_output_with_entities(
            OutputType::Summary,
            "张三和李四参与",
            vec!["张三", "李四", "王五"],
        );
        let result = reviewer.review(&output);
        // 2/3 = 66% < 90% → triggers
        assert!(result.issues.iter().any(|i| i.issue_type == "low_coverage"));
    }

    #[test]
    fn test_small_model_no_entities_skips_coverage() {
        let reviewer = SmallModelReview::new();
        let output = make_output(OutputType::Summary, "普通摘要");
        let result = reviewer.review(&output);
        assert!(result.verdict == ReviewVerdict::Approved);
        assert!(!result.issues.iter().any(|i| i.issue_type == "low_coverage"));
    }

    // ─── LargeModelReview tests ──────────────────────────────────

    #[test]
    fn test_large_model_deep_analysis_approved() {
        let reviewer = LargeModelReview::new();
        let output = make_output(
            OutputType::Analysis,
            "背景：项目进入二期。原因：需求变更。影响：工期延长。建议：调整计划。结论：需增加资源。",
        );
        let result = reviewer.review(&output);
        assert_eq!(result.verdict, ReviewVerdict::Approved);
        assert_eq!(result.reviewer, "large_model");
    }

    #[test]
    fn test_large_model_shallow_analysis() {
        let reviewer = LargeModelReview::new();
        let output = make_output(OutputType::Analysis, "太短了");
        let result = reviewer.review(&output);
        // shallow + no dimensions
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "shallow_analysis"));
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "missing_dimensions"));
    }

    #[test]
    fn test_large_model_missing_dimensions() {
        let reviewer = LargeModelReview::new();
        let output = make_output(
            OutputType::Report,
            "这是一段足够长的报告内容，但没有覆盖任何标准分析维度，只是在描述一些事实而已",
        );
        let result = reviewer.review(&output);
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "missing_dimensions"));
        assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
    }

    #[test]
    fn test_large_model_custom_depth() {
        let reviewer = LargeModelReview::new().with_min_depth_length(200);
        let output = make_output(
            OutputType::Analysis,
            "原因：预算不足。影响：项目延期。建议：追加预算。",
        );
        let result = reviewer.review(&output);
        // 有维度覆盖但长度不够 → medium severity → NeedsReview
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == "shallow_analysis"));
        assert!(matches!(result.verdict, ReviewVerdict::NeedsReview(_)));
    }

    // ─── TieredReview tests ──────────────────────────────────────

    #[test]
    fn test_tiered_review_routes_summary_to_small() {
        let tiered = TieredReview::default_config();
        let output = make_output(OutputType::Summary, "正常摘要");
        let result = tiered.review(&output);
        assert_eq!(result.reviewer, "small_model");
    }

    #[test]
    fn test_tiered_review_routes_extraction_to_small() {
        let tiered = TieredReview::default_config();
        let output = make_output(OutputType::Extraction, "正常提取");
        let result = tiered.review(&output);
        assert_eq!(result.reviewer, "small_model");
    }

    #[test]
    fn test_tiered_review_routes_analysis_to_large() {
        let tiered = TieredReview::default_config();
        let output = make_output(
            OutputType::Analysis,
            "背景：项目进入二期。原因：需求变更。影响：工期延长。建议：调整计划。结论：需增加资源。",
        );
        let result = tiered.review(&output);
        assert_eq!(result.reviewer, "large_model");
    }

    #[test]
    fn test_tiered_review_routes_report_to_large() {
        let tiered = TieredReview::default_config();
        let output = make_output(
            OutputType::Report,
            "背景：项目进入二期。原因：需求变更。影响：工期延长。建议：调整计划。结论：需增加资源。",
        );
        let result = tiered.review(&output);
        assert_eq!(result.reviewer, "large_model");
    }

    #[test]
    fn test_tiered_review_summary_contradiction() {
        let tiered = TieredReview::default_config();
        let output = make_output(OutputType::Summary, "该项目已批准实施，但预算已拒绝");
        let result = tiered.review(&output);
        assert!(matches!(result.verdict, ReviewVerdict::NeedsFix(_)));
        assert_eq!(result.reviewer, "small_model");
    }

    #[test]
    fn test_tiered_review_analysis_shallow() {
        let tiered = TieredReview::default_config();
        let output = make_output(OutputType::Analysis, "短");
        let result = tiered.review(&output);
        assert_eq!(result.reviewer, "large_model");
        assert!(!result.issues.is_empty());
    }

    // ─── UserConfirmPush tests ───────────────────────────────────

    #[test]
    fn test_data_scope_private_no_confirm() {
        let pusher = UserConfirmPush::new();
        assert!(!pusher.should_confirm(&DataScope::Private));
    }

    #[test]
    fn test_data_scope_shared_needs_confirm() {
        let pusher = UserConfirmPush::new();
        assert!(pusher.should_confirm(&DataScope::Shared));
    }

    #[test]
    fn test_data_scope_external_needs_confirm() {
        let pusher = UserConfirmPush::new();
        assert!(pusher.should_confirm(&DataScope::External));
    }

    #[test]
    fn test_push_private_scope_auto_passes() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-1".to_string(),
            content: "test".to_string(),
            reason: "test".to_string(),
            data_scope: DataScope::Private,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request).unwrap();
        // Private 范围不应进入 pending
        assert_eq!(pusher.pending_count(), 0);
    }

    #[test]
    fn test_push_shared_scope_enters_pending() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-2".to_string(),
            content: "shared data".to_string(),
            reason: "涉及共享数据".to_string(),
            data_scope: DataScope::Shared,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request).unwrap();
        assert_eq!(pusher.pending_count(), 1);
    }

    #[test]
    fn test_push_external_scope_enters_pending() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-3".to_string(),
            content: "external data".to_string(),
            reason: "涉及外部系统".to_string(),
            data_scope: DataScope::External,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request).unwrap();
        assert_eq!(pusher.pending_count(), 1);
    }

    #[test]
    fn test_push_duplicate_id_errors() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-dup".to_string(),
            content: "test".to_string(),
            reason: "test".to_string(),
            data_scope: DataScope::Shared,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request.clone()).unwrap();
        let err = pusher.push(request).unwrap_err();
        assert!(err.to_string().contains("已存在"));
    }

    #[test]
    fn test_confirm_removes_from_pending() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-c".to_string(),
            content: "test".to_string(),
            reason: "test".to_string(),
            data_scope: DataScope::Shared,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request).unwrap();
        assert_eq!(pusher.pending_count(), 1);

        pusher.confirm("req-c").unwrap();
        assert_eq!(pusher.pending_count(), 0);
    }

    #[test]
    fn test_reject_removes_from_pending() {
        let mut pusher = UserConfirmPush::new();
        let request = ConfirmRequest {
            id: "req-r".to_string(),
            content: "test".to_string(),
            reason: "test".to_string(),
            data_scope: DataScope::External,
            created_at: "2026-01-01".to_string(),
        };
        pusher.push(request).unwrap();
        assert_eq!(pusher.pending_count(), 1);

        pusher.reject("req-r").unwrap();
        assert_eq!(pusher.pending_count(), 0);
    }

    #[test]
    fn test_confirm_nonexistent_errors() {
        let mut pusher = UserConfirmPush::new();
        let err = pusher.confirm("nonexistent").unwrap_err();
        assert!(err.to_string().contains("不存在"));
    }

    #[test]
    fn test_reject_nonexistent_errors() {
        let mut pusher = UserConfirmPush::new();
        let err = pusher.reject("nonexistent").unwrap_err();
        assert!(err.to_string().contains("不存在"));
    }

    #[test]
    fn test_pending_returns_all_requests() {
        let mut pusher = UserConfirmPush::new();
        pusher
            .push(ConfirmRequest {
                id: "a".to_string(),
                content: "a".to_string(),
                reason: "a".to_string(),
                data_scope: DataScope::Shared,
                created_at: "2026-01-01".to_string(),
            })
            .unwrap();
        pusher
            .push(ConfirmRequest {
                id: "b".to_string(),
                content: "b".to_string(),
                reason: "b".to_string(),
                data_scope: DataScope::External,
                created_at: "2026-01-01".to_string(),
            })
            .unwrap();

        let pending = pusher.pending();
        assert_eq!(pending.len(), 2);
        let ids: Vec<&str> = pending.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn test_pending_count_after_operations() {
        let mut pusher = UserConfirmPush::new();
        assert_eq!(pusher.pending_count(), 0);

        pusher
            .push(ConfirmRequest {
                id: "x".to_string(),
                content: "x".to_string(),
                reason: "x".to_string(),
                data_scope: DataScope::Shared,
                created_at: "2026-01-01".to_string(),
            })
            .unwrap();
        pusher
            .push(ConfirmRequest {
                id: "y".to_string(),
                content: "y".to_string(),
                reason: "y".to_string(),
                data_scope: DataScope::External,
                created_at: "2026-01-01".to_string(),
            })
            .unwrap();
        assert_eq!(pusher.pending_count(), 2);

        pusher.confirm("x").unwrap();
        assert_eq!(pusher.pending_count(), 1);

        pusher.reject("y").unwrap();
        assert_eq!(pusher.pending_count(), 0);
    }
}
