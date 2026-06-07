//! B7: Obsidian Writer Integration Tests
//!
//! Tests the ObsidianWriter, DailyJournal, TemplateEngine, LinkBuilder,
//! TagManager, and ProjectDir modules.

use std::collections::HashMap;
use std::fs;

use chrono::{NaiveDate, Utc};
use wb_core::record::{Category, WorkRecord};
use wb_storage::obsidian::{
    DailyJournal, LinkBuilder, ObsidianWriter, ProjectDir, TagManager, TemplateEngine, VaultManager,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_vault() -> (tempfile::TempDir, VaultManager) {
    let tmp = tempfile::tempdir().unwrap();
    let manager = VaultManager::new(tmp.path().to_str().unwrap()).unwrap();
    (tmp, manager)
}

fn make_record(title: &str, path: &str) -> WorkRecord {
    WorkRecord {
        id: format!("rec-{}", title),
        created_at: Utc::now(),
        source_event_ids: vec!["evt-1".to_string()],
        title: title.to_string(),
        summary: format!("{} summary", title),
        detail: format!("{} detail", title),
        category: Category::Meeting,
        project: Some("test-project".to_string()),
        people: vec!["Alice".to_string(), "Bob".to_string()],
        tags: vec!["meeting".to_string(), "test".to_string()],
        task_status: None,
        task_due: None,
        task_progress: None,
        model_used: "test-model".to_string(),
        confidence: 0.9,
        needs_review: false,
        obsidian_path: path.to_string(),
    }
}

// ---------------------------------------------------------------------------
// B7-01: Write daily note
// ---------------------------------------------------------------------------

#[test]
fn b7_01_daily_journal_creates_file() {
    let (_tmp, vault) = make_vault();
    let journal = DailyJournal::new(&vault);

    let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
    let path = journal.append(date, "今天完成了 Task 1").unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("# 2026-06-06"));
    assert!(content.contains("今天完成了 Task 1"));
    assert!(content.contains("type: daily"));
}

#[test]
fn b7_01_daily_journal_appends_to_existing() {
    let (_tmp, vault) = make_vault();
    let journal = DailyJournal::new(&vault);

    let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
    journal.append(date, "第一条").unwrap();
    journal.append(date, "第二条").unwrap();

    let path = journal.path_for_date(date);
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("第一条"));
    assert!(content.contains("第二条"));
    // Title should appear only once
    assert_eq!(content.matches("# 2026-06-06").count(), 1);
}

#[test]
fn b7_01_daily_journal_path_format() {
    let (_tmp, vault) = make_vault();
    let journal = DailyJournal::new(&vault);

    let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
    let path = journal.path_for_date(date);
    assert!(path.ends_with("Daily/2026-06-06.md"));
}

// ---------------------------------------------------------------------------
// B7-02: Write project note
// ---------------------------------------------------------------------------

#[test]
fn b7_02_project_dir_creates_directory() {
    let (_tmp, vault) = make_vault();
    let pd = ProjectDir::new(&vault);

    let path = pd.ensure("alpha-project").unwrap();
    assert!(path.exists());
    assert!(path.is_dir());
}

#[test]
fn b7_02_project_dir_lists_projects() {
    let (_tmp, vault) = make_vault();
    let pd = ProjectDir::new(&vault);

    pd.ensure("beta").unwrap();
    pd.ensure("alpha").unwrap();

    let mut projects = pd.list_projects().unwrap();
    projects.sort();
    assert_eq!(projects, vec!["alpha", "beta"]);
}

#[test]
fn b7_02_project_dir_sanitizes_names() {
    let (_tmp, vault) = make_vault();
    let pd = ProjectDir::new(&vault);

    // Names with special chars should be sanitized
    let path = pd.ensure("my project").unwrap();
    assert!(path.exists());
    assert!(path.ends_with("Projects/my-project"));
}

#[test]
fn b7_02_project_dir_idempotent() {
    let (_tmp, vault) = make_vault();
    let pd = ProjectDir::new(&vault);

    let path1 = pd.ensure("gamma").unwrap();
    let path2 = pd.ensure("gamma").unwrap();
    assert_eq!(path1, path2);
}

// ---------------------------------------------------------------------------
// B7-03: Template rendering
// ---------------------------------------------------------------------------

#[test]
fn b7_03_render_meeting_template() {
    let engine = TemplateEngine::new();
    let mut ctx = HashMap::new();
    ctx.insert("date", "2026-06-06");
    ctx.insert("title", "周会");
    ctx.insert("project", "work-better");
    ctx.insert("people", "张三, 李四");
    ctx.insert("summary", "讨论进度");
    ctx.insert("agenda", "1. 功能开发\n2. Bug 修复");
    ctx.insert("decisions", "推迟上线");
    ctx.insert("action_items", "- [ ] 完成测试");

    let result = engine.render("meeting.md", &ctx).unwrap();
    assert!(result.contains("# 会议：周会"));
    assert!(result.contains("date: 2026-06-06"));
    assert!(result.contains("[[work-better]]"));
}

#[test]
fn b7_03_render_task_template() {
    let engine = TemplateEngine::new();
    let mut ctx = HashMap::new();
    ctx.insert("date", "2026-06-06");
    ctx.insert("title", "实现登录");
    ctx.insert("status", "进行中");
    ctx.insert("priority", "高");
    ctx.insert("due", "2026-06-10");
    ctx.insert("summary", "用户登录功能");
    ctx.insert("description", "实现 OAuth2 登录");
    ctx.insert("project", "auth");

    let result = engine.render("task.md", &ctx).unwrap();
    assert!(result.contains("# 任务：实现登录"));
    assert!(result.contains("status: 进行中"));
}

#[test]
fn b7_03_render_unknown_template_errors() {
    let engine = TemplateEngine::new();
    let ctx = HashMap::new();
    let result = engine.render("nonexistent.md", &ctx);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// B7-04: Link builder
// ---------------------------------------------------------------------------

#[test]
fn b7_04_wikilink_without_alias() {
    let lb = LinkBuilder::new();
    assert_eq!(lb.wikilink("Project Alpha", None), "[[Project Alpha]]");
}

#[test]
fn b7_04_wikilink_with_alias() {
    let lb = LinkBuilder::new();
    assert_eq!(
        lb.wikilink("Project Alpha", Some("alpha")),
        "[[Project Alpha|alpha]]"
    );
}

#[test]
fn b7_04_wikilink_chinese() {
    let lb = LinkBuilder::new();
    assert_eq!(lb.wikilink("每日站会", None), "[[每日站会]]");
    assert_eq!(
        lb.wikilink("每日站会", Some("站会")),
        "[[每日站会|站会]]"
    );
}

#[test]
fn b7_04_backlink_format() {
    let lb = LinkBuilder::new();
    let result = lb.backlink("meeting-2026-06-06", "Task 1");
    assert_eq!(
        result,
        "反向链接: 来自 [[meeting-2026-06-06]] — 在 Task 1 中被引用"
    );
}

// ---------------------------------------------------------------------------
// B7-05: Tags
// ---------------------------------------------------------------------------

#[test]
fn b7_05_tag_normalize() {
    let tm = TagManager::new();
    assert_eq!(tm.normalize("Meeting"), "meeting");
    assert_eq!(tm.normalize("project alpha"), "project-alpha");
    assert_eq!(tm.normalize("  trimmed  "), "trimmed");
    assert_eq!(tm.normalize("产品 规划"), "产品-规划");
}

#[test]
fn b7_05_format_tags() {
    let tm = TagManager::new();
    let tags = vec!["Meeting".to_string(), "Project Alpha".to_string()];
    let result = tm.format_tags(&tags);
    assert_eq!(result, "tags:\n  - meeting\n  - project-alpha\n");
}

#[test]
fn b7_05_format_tags_empty() {
    let tm = TagManager::new();
    assert_eq!(tm.format_tags(&[] as &[String]), "");
}

// ---------------------------------------------------------------------------
// B7-06: ObsidianWriter full flow
// ---------------------------------------------------------------------------

#[test]
fn b7_06_write_record_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    let writer = ObsidianWriter::new(tmp.path());
    let record = make_record("测试会议", "日记/2026-06-06/test.md");

    let path = writer.write_record(&record).unwrap();
    assert!(path.exists());

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("# 测试会议"));
    assert!(content.contains("---\n"));
    assert!(content.contains("id: rec-测试会议"));
    assert!(content.contains("category: meeting"));
}

#[test]
fn b7_06_write_record_creates_parent_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let writer = ObsidianWriter::new(tmp.path());
    let record = make_record("deep", "a/b/c/deep.md");

    let path = writer.write_record(&record).unwrap();
    assert!(path.exists());
    assert!(path.parent().unwrap().exists());
}

#[test]
fn b7_06_write_record_empty_path_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let writer = ObsidianWriter::new(tmp.path());
    let mut record = make_record("empty", "");
    record.obsidian_path = String::new();

    let result = writer.write_record(&record);
    assert!(result.is_err());
}

#[test]
fn b7_06_render_record_frontmatter() {
    let writer = ObsidianWriter::new("/tmp/test-vault");
    let record = make_record("渲染测试", "test/render.md");
    let rendered = writer.render_record(&record);

    assert!(rendered.starts_with("---\n"));
    assert!(rendered.contains("id: rec-渲染测试"));
    assert!(rendered.contains("category: meeting"));
    assert!(rendered.contains("tags:"));
    assert!(rendered.contains("  - meeting"));
    assert!(rendered.contains("people:"));
    assert!(rendered.contains("  - Alice"));
    assert!(rendered.contains("confidence: 0.9"));
    assert!(rendered.contains("# 渲染测试"));
    assert!(rendered.contains("> 渲染测试 summary"));
    assert!(rendered.contains("**来源事件:**"));
    assert!(rendered.contains("- `evt-1`"));
}

#[test]
fn b7_06_render_record_with_project() {
    let writer = ObsidianWriter::new("/tmp/test-vault");
    let record = make_record("项目笔记", "Projects/alpha/note.md");
    let rendered = writer.render_record(&record);

    assert!(rendered.contains("project: test-project"));
}

#[test]
fn b7_06_render_record_with_task_status() {
    let writer = ObsidianWriter::new("/tmp/test-vault");
    let mut record = make_record("任务记录", "Tasks/task.md");
    record.task_status = Some("进行中".to_string());

    let rendered = writer.render_record(&record);
    assert!(rendered.contains("task_status: 进行中"));
}

#[test]
fn b7_06_vault_structure_created() {
    let (_tmp, vault) = make_vault();

    // VaultManager should create 7 top-level directories
    for dir in &["Daily", "Projects", "People", "Tasks", "Reports", "Knowledge", "System"] {
        let path = vault.root().join(dir);
        assert!(path.exists(), "Directory {} should exist", dir);
    }
}
