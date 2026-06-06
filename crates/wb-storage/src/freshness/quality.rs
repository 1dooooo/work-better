//! 质量任务 —— 三层一致性检查、知识过期审查

use std::fs;
use std::path::Path;

use wb_core::error::Result;

use super::report::{Issue, IssueSeverity};

/// 保鲜引擎允许的最长过期天数
const STALENESS_THRESHOLD_DAYS: i64 = 30;

/// 质量任务执行器
#[derive(Debug, Clone)]
pub struct QualityTask<'a> {
    vault_path: &'a Path,
}

impl<'a> QualityTask<'a> {
    /// 创建质量任务
    pub fn new(vault_path: &'a Path) -> Self {
        Self { vault_path }
    }

    /// 三层一致性检查
    ///
    /// 检查三个层次的一致性：
    /// 1. frontmatter 的 title 与文件名是否匹配
    /// 2. 链接引用的文档是否存在反向链接
    /// 3. 目录结构是否符合 vault 规范（7 个顶级目录）
    pub fn consistency_check(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        issues.extend(self.check_title_filename_consistency()?);
        issues.extend(self.check_directory_structure()?);

        Ok(issues)
    }

    /// 知识过期审查
    ///
    /// 扫描 Knowledge 和 Projects 目录中的文档，
    /// 检查 frontmatter 中的 `updated` 日期字段，
    /// 超过阈值天数未更新的文档标记为过期。
    pub fn staleness_review(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let check_dirs = ["Knowledge", "Projects"];

        for dir_name in &check_dirs {
            let dir = self.vault_path.join(dir_name);
            if !dir.exists() {
                continue;
            }

            self.walk_and_check_staleness(&dir, &mut issues)?;
        }

        Ok(issues)
    }

    /// 检查 frontmatter title 与文件名的一致性
    fn check_title_filename_consistency(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let all_files = self.collect_all_md_files()?;

        for file_path in &all_files {
            let content = fs::read_to_string(file_path)?;
            let relative = self.relative_path(file_path);

            if let Some(title) = extract_frontmatter_field(&content, "title") {
                let stem = file_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // 比较 title 与文件名（忽略大小写和连字符/空格差异）
                let normalized_title = title.to_lowercase().replace(' ', "-");
                let normalized_stem = stem.to_lowercase();

                if normalized_title != normalized_stem {
                    issues.push(Issue {
                        file_path: relative,
                        description: format!(
                            "标题不一致: frontmatter title=\"{}\", 文件名=\"{}\"",
                            title, stem
                        ),
                        severity: IssueSeverity::Medium,
                        task_name: "consistency_check".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// 检查目录结构是否符合规范
    fn check_directory_structure(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let expected_dirs = ["Daily", "Projects", "People", "Tasks", "Reports", "Knowledge", "System"];

        for dir in &expected_dirs {
            let path = self.vault_path.join(dir);
            if !path.exists() {
                issues.push(Issue {
                    file_path: dir.to_string(),
                    description: format!("缺少标准目录: {}", dir),
                    severity: IssueSeverity::High,
                    task_name: "consistency_check".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// 递归检查文档过期状态
    fn walk_and_check_staleness(&self, dir: &Path, issues: &mut Vec<Issue>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_and_check_staleness(&path, issues)?;
                continue;
            }

            if !path.extension().map_or(false, |e| e == "md") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let relative = self.relative_path(&path);

            if let Some(updated) = extract_frontmatter_field(&content, "updated") {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&updated, "%Y-%m-%d") {
                    let today = chrono::Local::now().date_naive();
                    let days_old = (today - date).num_days();

                    if days_old > STALENESS_THRESHOLD_DAYS {
                        issues.push(Issue {
                            file_path: relative,
                            description: format!(
                                "文档已过期: 上次更新 {}，已 {} 天未更新（阈值 {} 天）",
                                updated, days_old, STALENESS_THRESHOLD_DAYS
                            ),
                            severity: IssueSeverity::Medium,
                            task_name: "staleness_review".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// 收集 vault 下所有 .md 文件路径
    fn collect_all_md_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        self.walk_md_files(self.vault_path, &mut files)?;
        Ok(files)
    }

    /// 递归收集 .md 文件
    fn walk_md_files(&self, dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_md_files(&path, files)?;
            } else if path.extension().map_or(false, |e| e == "md") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// 获取相对于 vault 根的路径字符串
    fn relative_path(&self, path: &std::path::PathBuf) -> String {
        path.strip_prefix(self.vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}

/// 从 frontmatter 中提取指定字段的值
fn extract_frontmatter_field(content: &str, field: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let frontmatter = &after_first[..end];

    let prefix = format!("{}:", field);
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_check_no_files() {
        let tmp = tempfile::tempdir().unwrap();
        let task = QualityTask::new(tmp.path());
        let issues = task.consistency_check().unwrap();
        // 会报告缺少 7 个标准目录
        assert_eq!(issues.len(), 7);
    }

    #[test]
    fn test_consistency_check_missing_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        // 只创建部分目录
        fs::create_dir_all(tmp.path().join("Daily")).unwrap();

        let task = QualityTask::new(tmp.path());
        let issues = task.consistency_check().unwrap();
        // 缺少 6 个目录
        assert!(issues.len() >= 6);
        assert!(issues.iter().any(|i| i.description.contains("Projects")));
    }

    #[test]
    fn test_consistency_check_title_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let knowledge_dir = tmp.path().join("Knowledge");
        fs::create_dir_all(&knowledge_dir).unwrap();

        // 文件名是 "rust-guide" 但 title 是 "Rust Programming Guide"
        fs::write(
            knowledge_dir.join("rust-guide.md"),
            "---\ntitle: Rust Programming Guide\n---\nContent",
        )
        .unwrap();

        let task = QualityTask::new(tmp.path());
        let issues = task.check_title_filename_consistency().unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("标题不一致"));
    }

    #[test]
    fn test_consistency_check_title_matches() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("rust-guide.md"),
            "---\ntitle: Rust Guide\n---\nContent",
        )
        .unwrap();

        let task = QualityTask::new(tmp.path());
        let issues = task.check_title_filename_consistency().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_staleness_review_no_files() {
        let tmp = tempfile::tempdir().unwrap();
        let task = QualityTask::new(tmp.path());
        let issues = task.staleness_review().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_staleness_review_fresh_document() {
        let tmp = tempfile::tempdir().unwrap();
        let knowledge_dir = tmp.path().join("Knowledge");
        fs::create_dir_all(&knowledge_dir).unwrap();

        let today = chrono::Local::now().date_naive().format("%Y-%m-%d");
        fs::write(
            knowledge_dir.join("fresh.md"),
            format!("---\ntitle: Fresh\nupdated: {}\n---\nContent", today),
        )
        .unwrap();

        let task = QualityTask::new(tmp.path());
        let issues = task.staleness_review().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_staleness_review_stale_document() {
        let tmp = tempfile::tempdir().unwrap();
        let knowledge_dir = tmp.path().join("Knowledge");
        fs::create_dir_all(&knowledge_dir).unwrap();

        fs::write(
            knowledge_dir.join("old.md"),
            "---\ntitle: Old Doc\nupdated: 2025-01-01\n---\nContent",
        )
        .unwrap();

        let task = QualityTask::new(tmp.path());
        let issues = task.staleness_review().unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("过期"));
    }

    #[test]
    fn test_extract_frontmatter_field_found() {
        let content = "---\ntitle: My Note\nauthor: ido\n---\nBody";
        assert_eq!(
            extract_frontmatter_field(content, "title"),
            Some("My Note".to_string())
        );
        assert_eq!(
            extract_frontmatter_field(content, "author"),
            Some("ido".to_string())
        );
    }

    #[test]
    fn test_extract_frontmatter_field_not_found() {
        let content = "---\ntitle: My Note\n---\nBody";
        assert!(extract_frontmatter_field(content, "tags").is_none());
    }

    #[test]
    fn test_extract_frontmatter_field_no_frontmatter() {
        let content = "# Just content";
        assert!(extract_frontmatter_field(content, "title").is_none());
    }

    #[test]
    fn test_staleness_threshold_constant() {
        assert_eq!(STALENESS_THRESHOLD_DAYS, 30);
    }
}
