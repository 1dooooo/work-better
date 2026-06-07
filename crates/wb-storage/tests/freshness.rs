//! B8: Freshness Engine Integration Tests
//!
//! Tests the FreshnessEngine and its sub-tasks (SyncTask, IntegrityTask,
//! QualityTask) against a temporary vault directory.

use std::fs;
use std::path::PathBuf;

use wb_storage::freshness::{
    FreshnessEngine, IntegrityTask, IssueSeverity, QualityTask, SyncTask,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_vault() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_path_buf();
    (tmp, path)
}

fn create_vault_structure(path: &std::path::Path) {
    for dir in &["Daily", "Projects", "People", "Tasks", "Reports", "Knowledge", "System"] {
        fs::create_dir_all(path.join(dir)).unwrap();
    }
}

// ---------------------------------------------------------------------------
// B8-01: Sync detection (document change detect)
// ---------------------------------------------------------------------------

#[test]
fn b8_01_sync_detect_no_daily_dir() {
    let (_tmp, vault) = make_vault();
    let task = SyncTask::new(&vault);
    let issues = task.document_change_detect().unwrap();
    assert!(issues.is_empty());
}

#[test]
fn b8_01_sync_detect_with_today() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    fs::create_dir_all(&daily_dir).unwrap();

    let today = chrono::Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    fs::write(daily_dir.join(format!("{}.md", date_str)), "# Today").unwrap();

    let task = SyncTask::new(&vault);
    let issues = task.document_change_detect().unwrap();
    // Today's log exists, so it should not be reported
    assert!(issues.iter().all(|i| !i.file_path.contains(&date_str)));
}

#[test]
fn b8_01_task_status_sync_done_without_completed() {
    let (_tmp, vault) = make_vault();
    let tasks_dir = vault.join("Tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(
        tasks_dir.join("review-code.md"),
        "---\ntitle: Review Code\nstatus: done\n---\nContent here",
    )
    .unwrap();

    let task = SyncTask::new(&vault);
    let issues = task.task_status_sync().unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("completed"));
    assert_eq!(issues[0].severity, IssueSeverity::Medium);
}

#[test]
fn b8_01_task_status_sync_done_with_completed() {
    let (_tmp, vault) = make_vault();
    let tasks_dir = vault.join("Tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(
        tasks_dir.join("review-code.md"),
        "---\ntitle: Review Code\nstatus: done\ncompleted: 2026-06-06\n---\nContent",
    )
    .unwrap();

    let task = SyncTask::new(&vault);
    let issues = task.task_status_sync().unwrap();
    assert!(issues.is_empty());
}

// ---------------------------------------------------------------------------
// B8-02: Broken links detection
// ---------------------------------------------------------------------------

#[test]
fn b8_02_broken_link_detected() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    fs::create_dir_all(&daily_dir).unwrap();

    fs::write(
        daily_dir.join("2026-06-06.md"),
        "# Daily\n\nSee also [[NonExistent Doc]]",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.link_integrity_check().unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("NonExistent Doc"));
    assert_eq!(issues[0].severity, IssueSeverity::High);
}

#[test]
fn b8_02_valid_link_no_issue() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    fs::create_dir_all(&daily_dir).unwrap();

    // Create target file
    fs::write(vault.join("Daily/meeting.md"), "# Meeting Notes").unwrap();
    // Source links to target
    fs::write(
        daily_dir.join("2026-06-06.md"),
        "# Daily\n\nSee [[meeting]]",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.link_integrity_check().unwrap();
    assert!(issues.is_empty());
}

#[test]
fn b8_02_broken_link_with_alias() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    fs::create_dir_all(&daily_dir).unwrap();

    fs::write(
        daily_dir.join("2026-06-06.md"),
        "# Daily\n\nSee [[meeting|Meeting Notes]]",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.link_integrity_check().unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("meeting"));
}

#[test]
fn b8_02_multiple_broken_links() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    fs::create_dir_all(&daily_dir).unwrap();

    fs::write(
        daily_dir.join("2026-06-06.md"),
        "# Daily\n\n[[Missing A]] and [[Missing B]] and [[Missing C]]",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.link_integrity_check().unwrap();
    assert_eq!(issues.len(), 3);
    assert!(issues.iter().all(|i| i.severity == IssueSeverity::High));
}

// ---------------------------------------------------------------------------
// B8-03: Duplicate detection
// ---------------------------------------------------------------------------

#[test]
fn b8_03_no_duplicates() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    let projects_dir = vault.join("Projects");
    fs::create_dir_all(&daily_dir).unwrap();
    fs::create_dir_all(&projects_dir).unwrap();

    fs::write(daily_dir.join("2026-06-06.md"), "# Daily").unwrap();
    fs::write(projects_dir.join("project-alpha.md"), "# Project").unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.duplicate_detection().unwrap();
    assert!(issues.is_empty());
}

#[test]
fn b8_03_duplicates_detected() {
    let (_tmp, vault) = make_vault();
    let daily_dir = vault.join("Daily");
    let projects_dir = vault.join("Projects");
    fs::create_dir_all(&daily_dir).unwrap();
    fs::create_dir_all(&projects_dir).unwrap();

    // Same filename in different directories
    fs::write(daily_dir.join("meeting.md"), "# Daily Meeting").unwrap();
    fs::write(projects_dir.join("meeting.md"), "# Project Meeting").unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.duplicate_detection().unwrap();
    assert_eq!(issues.len(), 2); // Both locations reported
    assert!(issues.iter().all(|i| i.severity == IssueSeverity::Medium));
}

// ---------------------------------------------------------------------------
// B8-04: Tag normalization
// ---------------------------------------------------------------------------

#[test]
fn b8_04_tags_already_normalized() {
    let (_tmp, vault) = make_vault();
    fs::write(
        vault.join("note.md"),
        "---\ntags:\n  - meeting\n  - project-alpha\n---\nContent",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.tag_normalization().unwrap();
    assert!(issues.is_empty());
}

#[test]
fn b8_04_tags_need_normalization() {
    let (_tmp, vault) = make_vault();
    fs::write(
        vault.join("note.md"),
        "---\ntags:\n  - Meeting\n  - Project Alpha\n---\nContent",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.tag_normalization().unwrap();
    assert_eq!(issues.len(), 2);
    assert!(issues[0].description.contains("应为"));
    assert_eq!(issues[0].severity, IssueSeverity::Low);
}

#[test]
fn b8_04_tags_inline_format() {
    let (_tmp, vault) = make_vault();
    fs::write(
        vault.join("note.md"),
        "---\ntags: [Meeting, Project Alpha]\n---\nContent",
    )
    .unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.tag_normalization().unwrap();
    assert_eq!(issues.len(), 2);
}

#[test]
fn b8_04_no_tags_no_issue() {
    let (_tmp, vault) = make_vault();
    fs::write(vault.join("note.md"), "---\ntitle: Note\n---\nContent").unwrap();

    let task = IntegrityTask::new(&vault);
    let issues = task.tag_normalization().unwrap();
    assert!(issues.is_empty());
}

// ---------------------------------------------------------------------------
// B8-05: Consistency checks
// ---------------------------------------------------------------------------

#[test]
fn b8_05_consistency_check_missing_dirs() {
    let (_tmp, vault) = make_vault();
    // Only create Daily
    fs::create_dir_all(vault.join("Daily")).unwrap();

    let task = QualityTask::new(&vault);
    let issues = task.consistency_check().unwrap();
    // Should report 6 missing directories
    assert!(issues.len() >= 6);
    assert!(issues.iter().any(|i| i.description.contains("Projects")));
    assert!(issues.iter().all(|i| i.severity == IssueSeverity::High));
}

#[test]
fn b8_05_consistency_check_all_dirs_present() {
    let (_tmp, vault) = make_vault();
    create_vault_structure(&vault);

    let task = QualityTask::new(&vault);
    let issues = task.consistency_check().unwrap();
    // No missing dirs, no files -> no issues
    assert!(issues.is_empty());
}

#[test]
fn b8_05_title_filename_mismatch() {
    let (_tmp, vault) = make_vault();
    let knowledge_dir = vault.join("Knowledge");
    fs::create_dir_all(&knowledge_dir).unwrap();

    fs::write(
        knowledge_dir.join("rust-guide.md"),
        "---\ntitle: Rust Programming Guide\n---\nContent",
    )
    .unwrap();

    let task = QualityTask::new(&vault);
    let issues = task.consistency_check().unwrap();
    assert!(issues.iter().any(|i| i.description.contains("标题不一致")));
}

#[test]
fn b8_05_title_filename_match() {
    let (_tmp, vault) = make_vault();
    create_vault_structure(&vault);

    // Write a file where title matches filename (after normalization)
    fs::write(
        vault.join("rust-guide.md"),
        "---\ntitle: Rust Guide\n---\nContent",
    )
    .unwrap();

    let task = QualityTask::new(&vault);
    let issues = task.consistency_check().unwrap();
    // Should have no title mismatch issues (may have other issues from missing dailies etc)
    assert!(!issues.iter().any(|i| i.description.contains("标题不一致")));
}

#[test]
fn b8_05_staleness_review_fresh_document() {
    let (_tmp, vault) = make_vault();
    let knowledge_dir = vault.join("Knowledge");
    fs::create_dir_all(&knowledge_dir).unwrap();

    let today = chrono::Local::now().date_naive().format("%Y-%m-%d");
    fs::write(
        knowledge_dir.join("fresh.md"),
        format!("---\ntitle: Fresh\nupdated: {}\n---\nContent", today),
    )
    .unwrap();

    let task = QualityTask::new(&vault);
    let issues = task.staleness_review().unwrap();
    assert!(issues.is_empty());
}

#[test]
fn b8_05_staleness_review_stale_document() {
    let (_tmp, vault) = make_vault();
    let knowledge_dir = vault.join("Knowledge");
    fs::create_dir_all(&knowledge_dir).unwrap();

    fs::write(
        knowledge_dir.join("old.md"),
        "---\ntitle: Old Doc\nupdated: 2025-01-01\n---\nContent",
    )
    .unwrap();

    let task = QualityTask::new(&vault);
    let issues = task.staleness_review().unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].description.contains("过期"));
    assert_eq!(issues[0].severity, IssueSeverity::Medium);
}

// ---------------------------------------------------------------------------
// FreshnessEngine full pipeline
// ---------------------------------------------------------------------------

#[test]
fn b8_05_freshness_engine_run_all() {
    let (_tmp, vault) = make_vault();
    create_vault_structure(&vault);

    // Create today's daily log to avoid missing-daily issues
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d");
    fs::write(
        vault.join("Daily").join(format!("{}.md", today)),
        format!("# {}", today),
    )
    .unwrap();

    let engine = FreshnessEngine::new(vault);
    let report = engine.run_all().unwrap();
    assert_eq!(report.tasks_run, 7);
    // issues_found may be > 0 due to missing past daily logs, but tasks_run should be 7
}

#[test]
fn b8_05_freshness_engine_run_sync_tasks() {
    let (_tmp, vault) = make_vault();
    let engine = FreshnessEngine::new(vault);
    let report = engine.run_sync_tasks().unwrap();
    assert_eq!(report.tasks_run, 2);
}

#[test]
fn b8_05_freshness_engine_run_integrity_tasks() {
    let (_tmp, vault) = make_vault();
    let engine = FreshnessEngine::new(vault);
    let report = engine.run_integrity_tasks().unwrap();
    assert_eq!(report.tasks_run, 3);
}

#[test]
fn b8_05_freshness_engine_run_quality_tasks() {
    let (_tmp, vault) = make_vault();
    let engine = FreshnessEngine::new(vault);
    let report = engine.run_quality_tasks().unwrap();
    assert_eq!(report.tasks_run, 2);
}

#[test]
fn b8_05_freshness_engine_generate_report() {
    let (_tmp, vault) = make_vault();
    create_vault_structure(&vault);

    let engine = FreshnessEngine::new(vault);
    let report = engine.generate_report().unwrap();
    assert_eq!(report.tasks_run, 7);
}
