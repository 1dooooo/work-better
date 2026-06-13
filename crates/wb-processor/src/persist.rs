//! PersistStep —— 审核通过后将 WorkRecord 持久化到 Obsidian
//!
//! 同时提供 Deduplicator，基于 SemanticSearch 实现语义级去重。

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use wb_core::error::Result;
use wb_core::record::WorkRecord;
use wb_storage::obsidian::ObsidianWriter;
use wb_storage::vector::{EmbeddingProvider, InMemoryVectorStore, SemanticSearch, VectorStore};

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
    /// 如果同目录下存在标题相似的已有文件（相似度 ≥ 0.6），则将新事件 ID 追加到已有文件，
    /// 而非创建新文件。否则正常写入。
    ///
    /// 要求 record.obsidian_path 已被设置为非空值。
    /// 返回写入/更新的文件路径。
    pub fn persist(&self, record: &WorkRecord) -> Result<PathBuf> {
        // 去重：在同目录下查找标题相似的已有文件
        if let Some(existing_path) = self.find_similar(record) {
            self.merge_into_existing(&existing_path, record)?;
            return Ok(existing_path);
        }

        self.writer.write_record(record)
    }

    /// 在同日期/分类目录下查找标题相似的已有文件
    ///
    /// 返回匹配文件的完整路径（如果相似度 ≥ 0.6）
    fn find_similar(&self, record: &WorkRecord) -> Option<PathBuf> {
        let relative_path = &record.obsidian_path;
        if relative_path.is_empty() {
            return None;
        }

        let full_path = self.writer.vault_path().join(relative_path);
        let parent = full_path.parent()?;
        let new_stem = full_path.file_stem()?.to_str()?;

        // 同目录下不存在文件 → 无重复
        let entries = fs::read_dir(parent).ok()?;
        let mut best_match: Option<(PathBuf, f64)> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }
            let existing_stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let similarity = Self::title_similarity(new_stem, &existing_stem);
            if similarity >= 0.6 {
                match &best_match {
                    Some((_, best_sim)) if *best_sim >= similarity => {}
                    _ => best_match = Some((path, similarity)),
                }
            }
        }

        best_match.map(|(path, _)| path)
    }

    /// 将新事件的 source_event_ids 追加到已有文件
    ///
    /// 在已有 markdown 文件的 "来源事件:" 部分追加新 ID，
    /// 并更新 task_status（如果新记录有状态信息）。
    fn merge_into_existing(&self, existing_path: &PathBuf, new_record: &WorkRecord) -> Result<()> {
        let mut content =
            fs::read_to_string(existing_path).map_err(wb_core::error::WbError::Io)?;

        // 追加新的 source_event_ids
        for event_id in &new_record.source_event_ids {
            let marker = format!("- `{}`\n", event_id);
            if !content.contains(&marker) {
                // 在 "来源事件" 部分追加，或创建新的来源段落
                if content.contains("**来源事件:**") {
                    content = content.replacen(
                        "**来源事件:**\n",
                        &format!("**来源事件:**\n{}", marker),
                        1,
                    );
                } else {
                    content.push_str(&format!("\n---\n**来源事件:**\n{}", marker));
                }
            }
        }

        // 如果新记录有 task_status 更新，在 frontmatter 中更新或插入
        if let Some(ref new_status) = new_record.task_status {
            let old_line = content
                .lines()
                .find(|l| l.starts_with("task_status:"))
                .map(|l| l.to_string());
            if let Some(old) = old_line {
                let updated = format!("task_status: {}", new_status);
                content = content.replacen(&old, &updated, 1);
            } else {
                // 在 frontmatter 的第一个 "---" 结束标记前插入
                content = content.replacen(
                    "---\n\n",
                    &format!("task_status: {}\n---\n\n", new_status),
                    1,
                );
            }
        }

        fs::write(existing_path, content).map_err(wb_core::error::WbError::Io)?;
        Ok(())
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

    /// 计算两个标题的相似度（基于字符级 LCS）
    ///
    /// 返回 0.0-1.0 的相似度分数
    fn title_similarity(a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower == b_lower {
            return 1.0;
        }

        let a_chars: Vec<char> = a_lower.chars().collect();
        let b_chars: Vec<char> = b_lower.chars().collect();
        let lcs_len = Self::lcs_length(&a_chars, &b_chars);
        let max_len = a_chars.len().max(b_chars.len());

        if max_len == 0 {
            return 1.0;
        }

        lcs_len as f64 / max_len as f64
    }

    /// 计算最长公共子序列长度
    fn lcs_length(a: &[char], b: &[char]) -> usize {
        let m = a.len();
        let n = b.len();
        let mut prev = vec![0usize; n + 1];
        let mut curr = vec![0usize; n + 1];

        for i in 1..=m {
            for j in 1..=n {
                curr[j] = if a[i - 1] == b[j - 1] {
                    prev[j - 1] + 1
                } else {
                    prev[j].max(curr[j - 1])
                };
            }
            std::mem::swap(&mut prev, &mut curr);
            curr.fill(0);
        }

        prev[n]
    }
}

/// 语义去重器
///
/// 基于 SemanticSearch 实现语义级任务去重。
/// 每次 persist 后自动将新记录索引到向量存储中，
/// 后续记录可通过语义相似度匹配到已有记录。
pub struct Deduplicator {
    search: SemanticSearch<InMemoryVectorStore>,
    /// 语义相似度阈值（0.0-1.0），超过此值视为重复
    threshold: f32,
}

impl Deduplicator {
    /// 创建新的去重器
    ///
    /// `embedding` 用于将文本转为向量，`threshold` 为相似度阈值（推荐 0.7-0.85）
    pub fn new(embedding: Arc<dyn EmbeddingProvider>, threshold: f32) -> Self {
        let store = InMemoryVectorStore::new(embedding);
        let search = SemanticSearch::new(store);
        Self { search, threshold }
    }

    /// 在向量库中查找与给定记录语义相似的已有记录
    ///
    /// 返回匹配的 doc_id（即 WorkRecord.id），如果相似度 ≥ threshold
    pub async fn find_similar(&self, record: &WorkRecord) -> Option<String> {
        let query = format!("{} {}", record.title, record.summary);
        let results = self.search.search_with_threshold(&query, 3, self.threshold).await.ok()?;
        results.into_iter().next().map(|r| r.doc_id)
    }

    /// 将记录索引到向量存储中（persist 成功后调用）
    pub async fn index_record(&self, record: &WorkRecord) {
        let content = format!("{} {} {}", record.title, record.summary, record.detail);
        let _ = self.search.store().upsert(&record.id, &content).await;
    }

    /// 获取底层 search 引擎的引用（用于测试）
    pub fn search(&self) -> &SemanticSearch<InMemoryVectorStore> {
        &self.search
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

    #[test]
    fn test_title_similarity_identical() {
        assert_eq!(PersistStep::title_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_title_similarity_case_insensitive() {
        assert_eq!(PersistStep::title_similarity("Hello", "hello"), 1.0);
    }

    #[test]
    fn test_title_similarity_completely_different() {
        let sim = PersistStep::title_similarity("abc", "xyz");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_title_similarity_similar_tasks() {
        // "给Bob发送邮件" vs "给Bob发送邮件-已完成" — 高相似度
        let sim = PersistStep::title_similarity(
            "给Bob发送邮件",
            "给Bob发送邮件-已完成",
        );
        assert!(sim >= 0.6, "similarity {} should be >= 0.6", sim);
    }

    #[test]
    fn test_persist_dedup_merges_into_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let persist = PersistStep::new(tmp.path());

        // 写入第一条记录
        let mut record1 = make_record("给Bob发送邮件", Category::Task);
        record1.obsidian_path = "test/给Bob发送邮件.md".to_string();
        let path1 = persist.persist(&record1).unwrap();
        let content1 = std::fs::read_to_string(&path1).unwrap();
        assert!(content1.contains("evt-1"));

        // 写入第二条标题相似的记录 — 应该合并而非新建
        let mut record2 = make_record("给Bob发送邮件-已完成", Category::Task);
        record2.obsidian_path = "test/给Bob发送邮件-已完成.md".to_string();
        record2.source_event_ids = vec!["evt-2".to_string()];
        record2.task_status = Some("done".to_string());
        let path2 = persist.persist(&record2).unwrap();

        // 应该返回同一个文件路径
        assert_eq!(path1, path2);

        // 文件内容应包含两个事件 ID
        let content2 = std::fs::read_to_string(&path2).unwrap();
        assert!(content2.contains("evt-1"));
        assert!(content2.contains("evt-2"));
        // task_status 应被更新
        assert!(content2.contains("task_status: done"));
    }
}
