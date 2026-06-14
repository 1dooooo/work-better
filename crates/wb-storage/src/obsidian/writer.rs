//! Obsidian Writer —— 将 WorkRecord 写入 Obsidian vault

use std::fs;
use std::path::{Path, PathBuf};

use wb_core::error::Result;
use wb_core::record::WorkRecord;

/// Obsidian vault 写入器
pub struct ObsidianWriter {
    vault_path: PathBuf,
}

impl ObsidianWriter {
    /// 创建新的 ObsidianWriter
    pub fn new(vault_path: impl Into<PathBuf>) -> Self {
        Self {
            vault_path: vault_path.into(),
        }
    }

    /// 获取 vault 根路径
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// 将 WorkRecord 写入 vault
    pub fn write_record(&self, record: &WorkRecord) -> Result<PathBuf> {
        let relative_path = &record.obsidian_path;
        if relative_path.is_empty() {
            return Err(wb_core::error::WbError::Storage(
                "obsidian_path is empty".to_string(),
            ));
        }

        let full_path = self.vault_path.join(relative_path);

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(wb_core::error::WbError::Io)?;
        }

        let content = self.render_record(record);
        fs::write(&full_path, content).map_err(wb_core::error::WbError::Io)?;

        Ok(full_path)
    }

    /// 将 WorkRecord 渲染为 markdown（含 frontmatter）
    pub fn render_record(&self, record: &WorkRecord) -> String {
        let mut out = String::new();

        // YAML frontmatter
        out.push_str("---\n");
        out.push_str(&format!("id: {}\n", record.id));
        out.push_str(&format!(
            "created: {}\n",
            record.created_at.format("%Y-%m-%d %H:%M:%S")
        ));

        // Category: render as lowercase unquoted string
        let category_str = serde_json::to_string(&record.category)
            .unwrap_or_default()
            .trim_matches('"')
            .to_lowercase();
        out.push_str(&format!("category: {}\n", category_str));

        if !record.tags.is_empty() {
            out.push_str("tags:\n");
            for tag in &record.tags {
                out.push_str(&format!("  - {}\n", tag));
            }
        }

        if !record.people.is_empty() {
            out.push_str("people:\n");
            for person in &record.people {
                out.push_str(&format!("  - {}\n", person));
            }
        }

        if let Some(ref project) = record.project {
            out.push_str(&format!("project: {}\n", project));
        }

        if let Some(ref task_status) = record.task_status {
            out.push_str(&format!("task_status: {}\n", task_status));
        }

        out.push_str(&format!("confidence: {}\n", record.confidence));
        out.push_str(&format!("needs_review: {}\n", record.needs_review));
        out.push_str(&format!("model_used: {}\n", record.model_used));
        out.push_str("---\n\n");

        // 标题
        out.push_str(&format!("# {}\n\n", record.title));

        // 摘要
        if !record.summary.is_empty() {
            out.push_str(&format!("> {}\n\n", record.summary));
        }

        // 详情
        if !record.detail.is_empty() {
            out.push_str(&record.detail);
            out.push('\n');
        }

        // 双向链接（Obsidian wiki links）
        let has_links = !record.people.is_empty() || record.project.is_some();
        if has_links {
            out.push_str("\n## 关联\n\n");
            if let Some(ref project) = record.project {
                out.push_str(&format!("- 项目: [[{}]]\n", project));
            }
            for person in &record.people {
                out.push_str(&format!("- 人员: [[{}]]\n", person));
            }
        }

        // 来源追溯
        if !record.source_event_ids.is_empty() {
            out.push_str("\n---\n");
            out.push_str("**来源事件:**\n");
            for id in &record.source_event_ids {
                out.push_str(&format!("- `{}`\n", id));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use wb_core::record::{Category, WorkRecord};

    fn make_test_record() -> WorkRecord {
        WorkRecord {
            id: "test-id-001".to_string(),
            created_at: Utc::now(),
            source_event_ids: vec!["evt-1".to_string(), "evt-2".to_string()],
            title: "测试会议记录".to_string(),
            summary: "讨论了 Q3 产品路线图".to_string(),
            detail: "## 议题\n1. 功能优先级\n2. 资源分配".to_string(),
            category: Category::Meeting,
            project: Some("project-alpha".to_string()),
            people: vec!["张三".to_string(), "李四".to_string()],
            tags: vec!["会议".to_string(), "产品".to_string()],
            task_status: None,
            task_due: None,
            task_priority: None,
            task_progress: None,
            model_used: "claude-sonnet-4-6".to_string(),
            confidence: 0.92,
            needs_review: false,
            obsidian_path: "日记/2026-06-06/test-meeting.md".to_string(),
        }
    }

    #[test]
    fn test_render_record_frontmatter() {
        let writer = ObsidianWriter::new("/tmp/test-vault");
        let record = make_test_record();
        let rendered = writer.render_record(&record);

        assert!(rendered.starts_with("---\n"));
        assert!(rendered.contains("id: test-id-001"));
        assert!(rendered.contains("category: meeting"));
        assert!(rendered.contains("tags:"));
        assert!(rendered.contains("  - 会议"));
        assert!(rendered.contains("  - 产品"));
        assert!(rendered.contains("people:"));
        assert!(rendered.contains("  - 张三"));
        assert!(rendered.contains("confidence: 0.92"));
        assert!(rendered.contains("# 测试会议记录"));
        assert!(rendered.contains("> 讨论了 Q3 产品路线图"));
        // 双向链接
        assert!(rendered.contains("[[project-alpha]]"));
        assert!(rendered.contains("[[张三]]"));
        assert!(rendered.contains("[[李四]]"));
    }

    #[test]
    fn test_write_record_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let writer = ObsidianWriter::new(tmp.path());
        let record = make_test_record();

        let path = writer.write_record(&record).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# 测试会议记录"));
    }

    #[test]
    fn test_write_record_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let writer = ObsidianWriter::new(tmp.path());
        let record = make_test_record();

        let path = writer.write_record(&record).unwrap();
        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_write_record_empty_path_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let writer = ObsidianWriter::new(tmp.path());
        let mut record = make_test_record();
        record.obsidian_path = String::new();

        let result = writer.write_record(&record);
        assert!(result.is_err());
    }
}
