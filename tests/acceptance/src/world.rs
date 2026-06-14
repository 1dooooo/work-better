use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use wb_ai::Extraction;
use wb_collector::manager::CollectorManager;
use wb_core::audit::ReviewResult;
use wb_core::event::Event;
use wb_core::record::WorkRecord;
use wb_processor::classifier::ProcessingRoute;
use wb_processor::task::discovery::PendingTask;
use wb_processor::task::TaskManager;
use wb_scheduler::scheduler::Scheduler;
use wb_storage::SqliteEventLog;

/// Acceptance test world — holds all mutable state across Given/When/Then steps.
#[derive(cucumber::World)]
pub struct AcceptanceWorld {
    // ── Real system instances ──────────────────────────────
    pub event_log: Arc<SqliteEventLog>,
    pub task_manager: Arc<TaskManager>,
    pub collector_manager: Arc<CollectorManager>,
    pub scheduler: Arc<Scheduler>,

    // ── Pending event (created in Given, appended in When) ─
    pub pending_event: Option<Event>,
    pub last_event_id: Option<String>,

    // ── Processing pipeline results (real component outputs) ─
    pub route: Option<ProcessingRoute>,
    pub extraction: Option<Extraction>,
    pub work_record: Option<WorkRecord>,
    pub review_result: Option<ReviewResult>,
    pub discovery_result: Option<Vec<PendingTask>>,

    // ── Existing fields (backward compat with G2-G7 steps) ─
    pub event_type: Option<String>,
    pub event_content: Option<String>,
    pub confidence: Option<f64>,
    pub priority: Option<String>,
    pub processing_result: Option<String>,
    pub model_used: Option<String>,
    pub review_verdict: Option<String>,
    pub task_status: Option<String>,
    pub task_source: Option<String>,
    pub completed_at: Option<String>,
    pub storage_path: Option<PathBuf>,
    pub vault_path: Option<PathBuf>,
    pub stored_records: Vec<String>,
    pub budget_remaining: Option<f64>,
    pub error: Option<String>,
    pub notifications: Vec<String>,
    pub state: HashMap<String, String>,
}

impl std::fmt::Debug for AcceptanceWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcceptanceWorld")
            .field("pending_event", &self.pending_event)
            .field("last_event_id", &self.last_event_id)
            .field("event_type", &self.event_type)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for AcceptanceWorld {
    fn default() -> Self {
        Self {
            event_log: Arc::new(
                SqliteEventLog::new_in_memory().expect("in-memory DB"),
            ),
            task_manager: Arc::new(TaskManager::new()),
            collector_manager: Arc::new(CollectorManager::new()),
            scheduler: Arc::new(Scheduler::new()),
            pending_event: None,
            last_event_id: None,
            // Real component outputs
            route: None,
            extraction: None,
            work_record: None,
            review_result: None,
            discovery_result: None,
            // Backward compat fields
            event_type: None,
            event_content: None,
            confidence: None,
            priority: None,
            processing_result: None,
            model_used: None,
            review_verdict: None,
            task_status: None,
            task_source: None,
            completed_at: None,
            storage_path: None,
            vault_path: None,
            stored_records: Vec::new(),
            budget_remaining: None,
            error: None,
            notifications: Vec::new(),
            state: HashMap::new(),
        }
    }
}
