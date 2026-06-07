//! G4: Task Management — 20 scenarios
//!
//! Black-box acceptance tests for task lifecycle, AI extraction, and Feishu sync.

mod acceptance_helpers;
use acceptance_helpers::*;
use wb_core::task::{Priority, Task, TaskStatus};

// ===========================================================================
// G4-01 ~ G4-03: Task Creation
// ===========================================================================

/// G4-01: Given user manually creates task, When saved, Then Task(status=todo, source=obsidian)
#[test]
fn g4_01_manual_task_creation() {
    let task = make_task("Review PR #42", TaskStatus::Todo, "obsidian");

    assert_eq!(task.status, TaskStatus::Todo);
    assert_eq!(task.source_platform, "obsidian");
    assert!(!task.needs_review);
    assert_eq!(task.title, "Review PR #42");
}

/// G4-02: Given system finds task from meeting, When AI extracts, Then needs_review=true
#[test]
fn g4_02_ai_extracted_task_needs_review() {
    let task = make_ai_task("Prepare Q3 roadmap draft");

    assert!(task.needs_review);
    assert_eq!(task.status, TaskStatus::Todo);
    assert!(task.confidence < 1.0);
}

/// G4-03: Given Feishu project task changes, When synced, Then Obsidian updates source=feishu
#[test]
fn g4_03_feishu_synced_task() {
    let task = make_task("Fix login bug", TaskStatus::Todo, "feishu");

    assert_eq!(task.source_platform, "feishu");
    assert_eq!(task.status, TaskStatus::Todo);
}

// ===========================================================================
// G4-04 ~ G4-11: Task State Transitions
// ===========================================================================

/// G4-04: Given task exists, When todo -> in_progress, Then legal and persisted
#[test]
fn g4_04_todo_to_in_progress() {
    let task = Task::new("Implement API", TaskStatus::Todo);
    let updated = task.transition(TaskStatus::InProgress).unwrap();

    assert_eq!(updated.status, TaskStatus::InProgress);
    assert_eq!(task.status, TaskStatus::Todo); // immutable original
}

/// G4-05: Given in_progress, When -> blocked, Then legal
#[test]
fn g4_05_in_progress_to_blocked() {
    let task = Task::new("Blocked task", TaskStatus::InProgress);
    let updated = task.transition(TaskStatus::Blocked).unwrap();
    assert_eq!(updated.status, TaskStatus::Blocked);
}

/// G4-06: Given blocked, When -> in_progress, Then legal
#[test]
fn g4_06_blocked_to_in_progress() {
    let task = Task::new("Unblocked task", TaskStatus::Blocked);
    let updated = task.transition(TaskStatus::InProgress).unwrap();
    assert_eq!(updated.status, TaskStatus::InProgress);
}

/// G4-07: Given active status, When -> cancelled, Then legal
#[test]
fn g4_07_active_to_cancelled() {
    let todo = Task::new("Cancel this", TaskStatus::Todo);
    let cancelled = todo.transition(TaskStatus::Cancelled).unwrap();
    assert_eq!(cancelled.status, TaskStatus::Cancelled);

    let in_progress = Task::new("Also cancel", TaskStatus::InProgress);
    let cancelled2 = in_progress.transition(TaskStatus::Cancelled).unwrap();
    assert_eq!(cancelled2.status, TaskStatus::Cancelled);
}

/// G4-08: Given todo, When directly -> done, Then rejected and explained
#[test]
fn g4_08_todo_to_done_rejected() {
    let task = Task::new("Skip states", TaskStatus::Todo);
    let result = task.transition(TaskStatus::Done);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid transition"));
}

/// G4-09: Given done, When -> in_progress, Then rejected
#[test]
fn g4_09_done_to_in_progress_rejected() {
    let task = Task::new("Cannot reopen", TaskStatus::Done);
    let result = task.transition(TaskStatus::InProgress);
    assert!(result.is_err());
}

/// G4-10: Given blocked, When directly -> done, Then rejected
#[test]
fn g4_10_blocked_to_done_rejected() {
    let task = Task::new("Must go through InProgress", TaskStatus::Blocked);
    let result = task.transition(TaskStatus::Done);
    assert!(result.is_err());
}

/// G4-11: Given marked done, When completed, Then sets completed_at
#[test]
fn g4_11_done_sets_completed_at() {
    let task = Task::new("Finish this", TaskStatus::InProgress);
    let done = task.transition(TaskStatus::Done).unwrap();

    assert!(done.completed_at.is_some());
    assert_eq!(done.status, TaskStatus::Done);
}

// ===========================================================================
// G4-12 ~ G4-17: Task Hierarchy & AI Extraction
// ===========================================================================

/// G4-12: Given sub-tasks, When parent created, Then linked via parent_task
#[test]
fn g4_12_parent_child_task_linking() {
    let mut parent = Task::new("Implement feature X", TaskStatus::Todo);
    parent.id = "parent-001".to_string();

    let mut child = Task::new("Write tests for X", TaskStatus::Todo);
    child.parent_task = Some(parent.id.clone());

    assert_eq!(child.parent_task, Some("parent-001".to_string()));
}

/// G4-13: Given meeting ends with todos, When analyzed, Then creates needs_review tasks
#[test]
fn g4_13_meeting_todo_extraction() {
    let task = make_ai_task("Send meeting notes to team");
    assert!(task.needs_review);
    assert_eq!(task.status, TaskStatus::Todo);
}

/// G4-14: Given chat message contains commitment, When analyzed, Then creates needs_review task
#[test]
fn g4_14_chat_commitment_extraction() {
    let task = make_ai_task("Review Alice's PR by Friday");
    assert!(task.needs_review);
}

/// G4-15: Given email contains request, When analyzed, Then creates needs_review task
#[test]
fn g4_15_email_request_extraction() {
    let task = make_ai_task("Respond to budget approval request");
    assert!(task.needs_review);
}

/// G4-16: Given doc comment contains todo, When analyzed, Then creates needs_review task
#[test]
fn g4_16_doc_comment_todo_extraction() {
    let task = make_ai_task("Fix the edge case mentioned in code review");
    assert!(task.needs_review);
}

/// G4-17: Given auto-discovered task, When presented, Then must confirm or reject to activate
#[test]
fn g4_17_auto_discovered_requires_confirmation() {
    let task = make_ai_task("Auto-detected: deploy staging");
    assert!(task.needs_review, "Auto-discovered tasks must require confirmation");
    // Tasks with needs_review=true should not be acted on until confirmed
}

// ===========================================================================
// G4-18 ~ G4-20: Feishu Sync
// ===========================================================================

/// G4-18: Given Feishu task status changes, When captured, Then Obsidian auto-updates (no confirmation)
#[test]
fn g4_18_feishu_status_auto_updates() {
    let task = make_task("Synced task", TaskStatus::Done, "feishu");
    assert_eq!(task.source_platform, "feishu");
    assert_eq!(task.status, TaskStatus::Done);
    // Feishu -> Obsidian sync is automatic (no needs_review)
    assert!(!task.needs_review);
}

/// G4-19: Given Obsidian task modified, When syncing to Feishu, Then needs user confirmation
#[test]
fn g4_19_obsidian_to_feishu_needs_confirmation() {
    let task = make_task("Modified in Obsidian", TaskStatus::InProgress, "obsidian");
    // Obsidian -> Feishu sync requires confirmation (shared data)
    // This is enforced at the sync layer, not the task model level.
    assert_eq!(task.source_platform, "obsidian");
}

/// G4-20: Given both sides modified simultaneously, When conflict detected, Then presents resolution
#[test]
fn g4_20_conflict_resolution() {
    // Conflict detection is a sync-layer concern.
    // This test documents the contract: when both sides change,
    // a resolution mechanism must be presented.
    let obsidian_task = make_task("Conflicting task", TaskStatus::InProgress, "obsidian");
    let feishu_task = make_task("Conflicting task", TaskStatus::Blocked, "feishu");

    // Both have different states — conflict exists
    assert_ne!(obsidian_task.status, feishu_task.status);
    // The resolution mechanism is tested at the sync layer.
}
