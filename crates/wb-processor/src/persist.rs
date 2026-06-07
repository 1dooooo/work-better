//! PersistStep —— 审核通过后将 WorkRecord 持久化到 Obsidian

use wb_core::error::Result;
use wb_core::record::WorkRecord;
use wb_storage::obsidian::ObsidianWriter;

/// 持久化步骤：将审核通过的 WorkRecord 写入 Obsidian vault
pub struct PersistStep {
    writer: ObsidianWriter,
}

impl PersistStep {
    /// 创建新的 PersistStep
    ///
    /// `vault_path` 是 Obsidian vault 的根路径
    pub fn new(vault_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            writer: ObsidianWriter::new(vault_path),
        }
    }

    /// 获取内部 ObsidianWriter 的只读引用
    pub fn writer(&self) -> &ObsidianWriter {
        &self.writer
    }

    /// 持久化 WorkRecord 到 Obsidian vault
    ///
    /// 要求 record.obsidian_path 已被设置为非空值。
    /// 返回写入的文件路径。
    pub fn persist(&self, record: &WorkRecord) -> Result<std::path::PathBuf> {
        self.writer.write_record(record)
    }

    /// 为 WorkRecord 生成默认的 Obsidian 路径
    ///
    /// 格式: `journal/YYYY-MM-DD/{category}/{sanitized-title}.md`
    pub fn generate_path(record: &WorkRecord) -> String {
        let date = record.created_at.format("%Y-%m-%d").to_string();
        let category = format!("{:?}", record.category).to_lowercase();
        let title = Self::sanitize_filename(&record.title);
        format!("journal/{}/{}/{}.md", date, category, title)
    }

    /// 清理文件名中的非法字符
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
                ' ' => '-',
                _ => c,
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use wb_core::record::{Category, WorkRecord};

    fn make_record(title: &str, category: Category) -> WorkRecord {
        WorkRecord {
            id: "test-001".to_string(),
            created_at: Utc::now(),
            source_event_ids: vec!["evt-1".to_string()],
            title: title.to_string(),
            summary: "Test summary".to_string(),
            detail: "Test detail content that is long enough".to_string(),
            category,
            project: None,
            people: vec![],
            tags: vec![],
            task_status: None,
            task_due: None,
            task_progress: None,
            model_used: "test-model".to_string(),
            confidence: 0.9,
            needs_review: false,
            obsidian_path: String::new(),
        }
    }

    #[test]
    fn test_generate_path_task() {
        let record = make_record("Fix login bug", Category::Task);
        let path = PersistStep::generate_path(&record);
        assert!(path.starts_with("journal/"));
        assert!(path.contains("/task/"));
        assert!(path.ends_with(".md"));
        assert!(path.contains("Fix-login-bug"));
    }

    #[test]
    fn test_generate_path_meeting() {
        let record = make_record("Weekly Standup", Category::Meeting);
        let path = PersistStep::generate_path(&record);
        assert!(path.contains("/meeting/"));
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(
            PersistStep::sanitize_filename("file/name:test*?"),
            "file-name-test"
        );
    }

    #[test]
    fn test_sanitize_filename_spaces() {
        assert_eq!(PersistStep::sanitize_filename("hello world"), "hello-world");
    }

    #[test]
    fn test_persist_writes_file() {
        let tmp = tempfile::tempdir().unwrap();
        let persist = PersistStep::new(tmp.path());
        let mut record = make_record("Persist Test", Category::Task);
        record.obsidian_path = "test/persist-test.md".to_string();

        let path = persist.persist(&record).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Persist Test"));
    }

    #[test]
    fn test_persist_empty_path_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let persist = PersistStep::new(tmp.path());
        let record = make_record("No Path", Category::Task);
        // obsidian_path is empty → should fail
        assert!(persist.persist(&record).is_err());
    }
}
