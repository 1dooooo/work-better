//! 全链路审计 —— 基于 ProcessingAudit 的查询与追踪

use chrono::{DateTime, Utc};
use wb_core::audit::{AuditStep, ProcessingAudit, ReviewVerdict};

/// 优化建议
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: String,
}

/// 审计过滤条件
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub step: Option<AuditStep>,
    pub trace_id: Option<String>,
    pub min_confidence: Option<f64>,
    pub after: Option<DateTime<Utc>>,
    pub has_review_verdict: Option<ReviewVerdict>,
}

/// 全链路审计管道
pub struct AuditPipeline {
    records: Vec<ProcessingAudit>,
}

impl AuditPipeline {
    /// 创建空的审计管道
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 追加一条审计记录（简化接口，自动填充默认字段）
    pub fn record(&mut self, step: &str, trace_id: &str, details: String) {
        let audit_step = match step {
            "Classifier" => AuditStep::Classifier,
            "Extract" => AuditStep::Extract,
            "Upgrade" => AuditStep::Upgrade,
            "Review" => AuditStep::Review,
            "UserConfirm" => AuditStep::UserConfirm,
            "Persist" => AuditStep::Persist,
            _ => AuditStep::Classifier, // fallback
        };

        let audit = ProcessingAudit {
            event_id: String::new(),
            record_id: None,
            trace_id: trace_id.to_string(),
            step: audit_step,
            timestamp: Utc::now(),
            duration_ms: 0,
            model: String::new(),
            model_version: String::new(),
            prompt_id: String::new(),
            prompt_params: serde_json::Value::Null,
            input_summary: details,
            output: serde_json::Value::Null,
            confidence: 0.0,
            token_input: 0,
            token_output: 0,
            cost_estimate: 0.0,
            upgrade_reason: None,
            previous_model: None,
            review_verdict: None,
            review_issues: None,
            user_action: None,
            user_correction: None,
        };

        self.records.push(audit);
    }

    /// 追加一条完整的审计记录
    pub fn push(&mut self, audit: ProcessingAudit) {
        self.records.push(audit);
    }

    /// 按 trace_id 查询关联的所有审计记录
    pub fn trace(&self, trace_id: &str) -> Vec<&ProcessingAudit> {
        self.records
            .iter()
            .filter(|r| r.trace_id == trace_id)
            .collect()
    }

    /// 按过滤条件查询
    pub fn query(&self, filter: AuditFilter) -> Vec<&ProcessingAudit> {
        self.records
            .iter()
            .filter(|r| {
                if let Some(ref step) = filter.step {
                    if r.step != *step {
                        return false;
                    }
                }
                if let Some(ref tid) = filter.trace_id {
                    if r.trace_id != *tid {
                        return false;
                    }
                }
                if let Some(min_conf) = filter.min_confidence {
                    if r.confidence < min_conf {
                        return false;
                    }
                }
                if let Some(after) = filter.after {
                    if r.timestamp < after {
                        return false;
                    }
                }
                if let Some(ref verdict) = filter.has_review_verdict {
                    match &r.review_verdict {
                        Some(v) if v == verdict => {}
                        _ => return false,
                    }
                }
                true
            })
            .collect()
    }

    /// 获取全部审计记录
    pub fn all(&self) -> &[ProcessingAudit] {
        &self.records
    }

    /// 记录总数
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 按 trace_id 统计记录数
    pub fn trace_count(&self, trace_id: &str) -> usize {
        self.records.iter().filter(|r| r.trace_id == trace_id).count()
    }

    /// 基于审计数据生成优化建议
    pub fn generate_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        if self.records.is_empty() {
            return suggestions;
        }

        // 1. 分析处理耗时分布 — 高耗时步骤建议优化
        let mut step_durations: std::collections::HashMap<&AuditStep, Vec<u64>> =
            std::collections::HashMap::new();
        for r in &self.records {
            step_durations.entry(&r.step).or_default().push(r.duration_ms);
        }
        for (step, durations) in &step_durations {
            let total: u64 = durations.iter().sum();
            let avg = total as f64 / durations.len() as f64;
            if avg > 5000.0 {
                suggestions.push(OptimizationSuggestion {
                    category: "performance".into(),
                    title: format!("{:?} 步骤平均耗时过高", step),
                    description: format!(
                        "平均耗时 {:.0}ms（共 {} 条记录），建议优化该步骤处理逻辑或考虑缓存",
                        avg,
                        durations.len()
                    ),
                    priority: "high".into(),
                });
            } else if avg > 2000.0 {
                suggestions.push(OptimizationSuggestion {
                    category: "performance".into(),
                    title: format!("{:?} 步骤耗时偏高", step),
                    description: format!("平均耗时 {:.0}ms，可考虑异步处理或批量化", avg),
                    priority: "medium".into(),
                });
            }
        }

        // 2. 分类错误率 — 低置信度和失败审核
        let low_confidence_count = self
            .records
            .iter()
            .filter(|r| r.confidence < 0.6 && r.confidence > 0.0)
            .count();
        let total = self.records.len();
        let low_confidence_rate = low_confidence_count as f64 / total as f64;

        if low_confidence_rate > 0.2 {
            suggestions.push(OptimizationSuggestion {
                category: "coverage".into(),
                title: "低置信度记录占比过高".into(),
                description: format!(
                    "{}/{} 条记录置信度低于 0.6（{:.0}%），建议增加训练样本或优化 prompt",
                    low_confidence_count,
                    total,
                    low_confidence_rate * 100.0
                ),
                priority: "high".into(),
            });
        }

        let needs_fix_count = self
            .records
            .iter()
            .filter(|r| matches!(r.review_verdict, Some(ReviewVerdict::NeedsFix(_))))
            .count();
        let needs_fix_rate = needs_fix_count as f64 / total as f64;

        if needs_fix_rate > 0.15 {
            suggestions.push(OptimizationSuggestion {
                category: "reliability".into(),
                title: "审核需修复比例偏高".into(),
                description: format!(
                    "{}/{} 条记录需修复（{:.0}%），建议检查提取逻辑和数据源质量",
                    needs_fix_count,
                    total,
                    needs_fix_rate * 100.0
                ),
                priority: "high".into(),
            });
        }

        // 3. Token 消耗分析
        let total_tokens: u64 = self
            .records
            .iter()
            .map(|r| r.token_input + r.token_output)
            .sum();
        let avg_tokens = total_tokens as f64 / total as f64;
        if avg_tokens > 4000.0 {
            suggestions.push(OptimizationSuggestion {
                category: "performance".into(),
                title: "Token 消耗偏高".into(),
                description: format!(
                    "平均每次处理消耗 {:.0} tokens，建议优化 prompt 长度或使用更小的模型",
                    avg_tokens
                ),
                priority: "medium".into(),
            });
        }

        // 4. 检查是否缺少升级路径（所有记录都用同一模型）
        let unique_models: std::collections::HashSet<&str> =
            self.records.iter().map(|r| r.model.as_str()).collect();
        if unique_models.len() == 1 && self.records.len() > 10 {
            suggestions.push(OptimizationSuggestion {
                category: "reliability".into(),
                title: "未使用模型升级路径".into(),
                description: "所有记录使用同一模型，建议配置小模型->大模型的升级策略以平衡成本和质量"
                    .into(),
                priority: "low".into(),
            });
        }

        suggestions
    }

    /// 获取所有唯一的 trace_id
    pub fn unique_traces(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .records
            .iter()
            .map(|r| r.trace_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        ids.sort();
        ids
    }
}

impl Default for AuditPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_audit(trace_id: &str, step: AuditStep, confidence: f64) -> ProcessingAudit {
        ProcessingAudit {
            event_id: "evt-1".into(),
            record_id: Some("rec-1".into()),
            trace_id: trace_id.into(),
            step,
            timestamp: Utc::now(),
            duration_ms: 100,
            model: "mock".into(),
            model_version: "v1".into(),
            prompt_id: "p1".into(),
            prompt_params: json!({}),
            input_summary: "test input".into(),
            output: json!({}),
            confidence,
            token_input: 100,
            token_output: 50,
            cost_estimate: 0.01,
            upgrade_reason: None,
            previous_model: None,
            review_verdict: None,
            review_issues: None,
            user_action: None,
            user_correction: None,
        }
    }

    #[test]
    fn test_new_is_empty() {
        let pipeline = AuditPipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
    }

    #[test]
    fn test_push_and_all() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.8));
        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline.all().len(), 2);
    }

    #[test]
    fn test_record_simplified() {
        let mut pipeline = AuditPipeline::new();
        pipeline.record("Extract", "trace-abc", "extracted entities".into());
        assert_eq!(pipeline.len(), 1);
        let all = pipeline.all();
        assert_eq!(all[0].trace_id, "trace-abc");
        assert_eq!(all[0].step, AuditStep::Extract);
        assert_eq!(all[0].input_summary, "extracted entities");
    }

    #[test]
    fn test_record_unknown_step_defaults_to_classifier() {
        let mut pipeline = AuditPipeline::new();
        pipeline.record("UnknownStep", "t1", "details".into());
        assert_eq!(pipeline.all()[0].step, AuditStep::Classifier);
    }

    #[test]
    fn test_trace_filters_by_id() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t2", AuditStep::Extract, 0.8));
        pipeline.push(make_audit("t1", AuditStep::Review, 0.85));

        let traced = pipeline.trace("t1");
        assert_eq!(traced.len(), 2);
        assert!(traced.iter().all(|r| r.trace_id == "t1"));
    }

    #[test]
    fn test_trace_returns_empty_for_missing() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        assert!(pipeline.trace("nonexistent").is_empty());
    }

    #[test]
    fn test_query_by_step() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.8));
        pipeline.push(make_audit("t2", AuditStep::Classifier, 0.7));

        let filter = AuditFilter {
            step: Some(AuditStep::Classifier),
            ..Default::default()
        };
        let results = pipeline.query(filter);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.step == AuditStep::Classifier));
    }

    #[test]
    fn test_query_by_min_confidence() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.9));
        pipeline.push(make_audit("t2", AuditStep::Extract, 0.5));
        pipeline.push(make_audit("t3", AuditStep::Extract, 0.75));

        let filter = AuditFilter {
            min_confidence: Some(0.7),
            ..Default::default()
        };
        let results = pipeline.query(filter);
        assert_eq!(results.len(), 2); // 0.9 and 0.75
    }

    #[test]
    fn test_query_by_trace_id() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t2", AuditStep::Classifier, 0.9));

        let filter = AuditFilter {
            trace_id: Some("t1".into()),
            ..Default::default()
        };
        let results = pipeline.query(filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].trace_id, "t1");
    }

    #[test]
    fn test_query_by_review_verdict() {
        let mut pipeline = AuditPipeline::new();
        let mut audit = make_audit("t1", AuditStep::Review, 0.9);
        audit.review_verdict = Some(ReviewVerdict::Approved);
        pipeline.push(audit);

        let mut audit2 = make_audit("t2", AuditStep::Review, 0.5);
        audit2.review_verdict = Some(ReviewVerdict::NeedsFix("missing title".into()));
        pipeline.push(audit2);

        let filter = AuditFilter {
            has_review_verdict: Some(ReviewVerdict::Approved),
            ..Default::default()
        };
        let results = pipeline.query(filter);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_combined_filters() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.9));
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.95));
        pipeline.push(make_audit("t2", AuditStep::Extract, 0.9));

        let filter = AuditFilter {
            step: Some(AuditStep::Extract),
            trace_id: Some("t1".into()),
            ..Default::default()
        };
        let results = pipeline.query(filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].step, AuditStep::Extract);
        assert_eq!(results[0].trace_id, "t1");
    }

    #[test]
    fn test_query_no_filters_returns_all() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t2", AuditStep::Extract, 0.8));

        let results = pipeline.query(AuditFilter::default());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_trace_count() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t1", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.8));
        pipeline.push(make_audit("t2", AuditStep::Classifier, 0.7));

        assert_eq!(pipeline.trace_count("t1"), 2);
        assert_eq!(pipeline.trace_count("t2"), 1);
        assert_eq!(pipeline.trace_count("t3"), 0);
    }

    #[test]
    fn test_unique_traces() {
        let mut pipeline = AuditPipeline::new();
        pipeline.push(make_audit("t2", AuditStep::Classifier, 0.9));
        pipeline.push(make_audit("t1", AuditStep::Extract, 0.8));
        pipeline.push(make_audit("t1", AuditStep::Review, 0.85));

        let traces = pipeline.unique_traces();
        assert_eq!(traces, vec!["t1", "t2"]); // sorted
    }

    fn make_audit_with_duration(
        trace_id: &str,
        step: AuditStep,
        confidence: f64,
        duration_ms: u64,
    ) -> ProcessingAudit {
        ProcessingAudit {
            event_id: "evt-1".into(),
            record_id: Some("rec-1".into()),
            trace_id: trace_id.into(),
            step,
            timestamp: Utc::now(),
            duration_ms,
            model: "mock".into(),
            model_version: "v1".into(),
            prompt_id: "p1".into(),
            prompt_params: json!({}),
            input_summary: "test input".into(),
            output: json!({}),
            confidence,
            token_input: 100,
            token_output: 50,
            cost_estimate: 0.01,
            upgrade_reason: None,
            previous_model: None,
            review_verdict: None,
            review_issues: None,
            user_action: None,
            user_correction: None,
        }
    }

    #[test]
    fn test_suggestions_empty_pipeline() {
        let pipeline = AuditPipeline::new();
        let suggestions = pipeline.generate_suggestions();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_suggestions_high_duration() {
        let mut pipeline = AuditPipeline::new();
        for i in 0..5 {
            pipeline.push(make_audit_with_duration(
                &format!("t{}", i),
                AuditStep::Extract,
                0.9,
                6000,
            ));
        }
        let suggestions = pipeline.generate_suggestions();
        let perf: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == "performance")
            .collect();
        assert!(!perf.is_empty());
        assert!(perf.iter().any(|s| s.priority == "high"));
    }

    #[test]
    fn test_suggestions_low_confidence() {
        let mut pipeline = AuditPipeline::new();
        for i in 0..10 {
            pipeline.push(make_audit_with_duration(
                &format!("t{}", i),
                AuditStep::Classifier,
                0.4,
                100,
            ));
        }
        let suggestions = pipeline.generate_suggestions();
        let coverage: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == "coverage")
            .collect();
        assert_eq!(coverage.len(), 1);
        assert!(coverage[0].description.contains("置信度"));
    }

    #[test]
    fn test_suggestions_needs_fix_high_rate() {
        let mut pipeline = AuditPipeline::new();
        for i in 0..20 {
            let mut audit = make_audit_with_duration(
                &format!("t{}", i),
                AuditStep::Review,
                0.8,
                100,
            );
            if i < 5 {
                audit.review_verdict = Some(ReviewVerdict::NeedsFix("issue".into()));
            }
            pipeline.push(audit);
        }
        let suggestions = pipeline.generate_suggestions();
        let reliability: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == "reliability")
            .collect();
        assert!(reliability.iter().any(|s| s.title.contains("需修复")));
    }

    #[test]
    fn test_suggestions_single_model_many_records() {
        let mut pipeline = AuditPipeline::new();
        for i in 0..15 {
            pipeline.push(make_audit_with_duration(
                &format!("t{}", i),
                AuditStep::Classifier,
                0.9,
                100,
            ));
        }
        let suggestions = pipeline.generate_suggestions();
        assert!(suggestions.iter().any(|s| s.title.contains("模型升级")));
    }

    #[test]
    fn test_suggestions_no_issues_when_healthy() {
        let mut pipeline = AuditPipeline::new();
        for i in 0..10 {
            let mut audit = make_audit_with_duration(
                &format!("t{}", i),
                AuditStep::Classifier,
                0.95,
                200,
            );
            audit.token_input = 50;
            audit.token_output = 30;
            pipeline.push(audit);
        }
        // Add records with different models to avoid single-model suggestion
        let mut audit = make_audit_with_duration("t10", AuditStep::Extract, 0.95, 200);
        audit.model = "other-model".into();
        audit.token_input = 50;
        audit.token_output = 30;
        pipeline.push(audit);

        let suggestions = pipeline.generate_suggestions();
        assert!(suggestions.is_empty());
    }
}
