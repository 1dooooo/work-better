use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 处理步骤
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum AuditStep {
    Classifier,
    Extract,
    Upgrade,
    Review,
    UserConfirm,
    Persist,
}

/// 审核结论
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum ReviewVerdict {
    Approved,
    NeedsFix,
    NeedsReview,
}

/// 处理审计记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct ProcessingAudit {
    pub event_id: String,
    pub record_id: Option<String>,
    pub trace_id: String,
    pub step: AuditStep,
    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub model: String,
    pub model_version: String,
    pub prompt_id: String,
    #[ts(type = "unknown")]
    pub prompt_params: serde_json::Value,
    pub input_summary: String,
    #[ts(type = "unknown")]
    pub output: serde_json::Value,
    pub confidence: f64,
    pub token_input: u64,
    pub token_output: u64,
    pub cost_estimate: f64,
    pub upgrade_reason: Option<String>,
    pub previous_model: Option<String>,
    pub review_verdict: Option<ReviewVerdict>,
    pub review_issues: Option<Vec<String>>,
    pub user_action: Option<String>,
    pub user_correction: Option<String>,
}
