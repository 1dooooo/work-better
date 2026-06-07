use std::collections::HashMap;
use std::path::PathBuf;

/// Acceptance test world — holds all mutable state across Given/When/Then steps.
#[derive(Debug, Default, cucumber::World)]
pub struct AcceptanceWorld {
    // Event context
    pub event_type: Option<String>,
    pub event_content: Option<String>,
    pub confidence: Option<f64>,
    pub priority: Option<String>,

    // Processing context
    pub processing_result: Option<String>,
    pub model_used: Option<String>,
    pub review_verdict: Option<String>,

    // Task context
    pub task_status: Option<String>,
    pub task_source: Option<String>,
    pub completed_at: Option<String>,

    // Storage context
    pub storage_path: Option<PathBuf>,
    pub vault_path: Option<PathBuf>,
    pub stored_records: Vec<String>,

    // System context
    pub budget_remaining: Option<f64>,
    pub error: Option<String>,
    pub notifications: Vec<String>,

    // Generic key-value store for ad-hoc state
    pub state: HashMap<String, String>,
}
