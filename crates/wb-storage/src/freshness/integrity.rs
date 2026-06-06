//! 完整性任务 —— 链接完整性检查、重复检测、标签规范化

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use wb_core::error::Result;

use super::report::{Issue, IssueSeverity};
use crate::obsidian::{LinkBuilder, TagManager};

/// 完整性任务执行器
#[derive(Debug, Clone)]
pub struct IntegrityTask<'a> {
    vault_path: &'a Path,
}

impl<'a> IntegrityTask<'a> {
    /// 创建完整性任务
    pub fn new(vault_path: &'a Path) -> Self {
        Self { vault_path }
    }

    /// 链接完整性检查
    ///
    /// 扫描所有 markdown 文件中的 `[[...]]` wiki 链接，
    /// 验证链接目标是否在 vault 中存在对应的文件。
    pub fn link_integrity_check(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let link_builder = LinkBuilder::new();

        // 收集所有已知文件名（不含扩展名）
        let known_files = self.collect_all_md_stems()?;

        // 扫描所有 md 文件中的链接
        let all_files = self.collect_all_md_files()?;
        for file_path in &all_files {
            let content = fs::read_to_string(file_path)?;
            let relative = self.relative_path(file_path);

            // 提取 [[...]] 链接
            for link_target in extract_wikilinks(&content) {
                // 去除 alias 部分: [[target|alias]] -> target
                let target = link_target
                    .split('|')
                    .next()
                    .unwrap_or(&link_target)
                    .trim();

                if target.is_empty() {
                    continue;
                }

                if !known_files.contains(target) {
                    let _ = link_builder.wikilink(target, None); // 使用 API 构建链接格式
                    issues.push(Issue {
                        file_path: relative.clone(),
                        description: format!("断链: [[{}]] 目标文件不存在", target),
                        severity: IssueSeverity::High,
                        task_name: "link_integrity_check".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// 重复检测
    ///
    /// 检测 vault 中是否存在同名的 markdown 文件（跨不同目录）。
    pub fn duplicate_detection(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let all_files = self.collect_all_md_files()?;

        // 按文件名分组
        let mut name_map: HashMap<String, Vec<String>> = HashMap::new();
        for file_path in &all_files {
            let relative = self.relative_path(file_path);
            let stem = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            name_map
                .entry(stem)
                .or_default()
                .push(relative);
        }

        for (name, locations) in &name_map {
            if locations.len() > 1 {
                for location in locations {
                    issues.push(Issue {
                        file_path: location.clone(),
                        description: format!(
                            "文件名重复: \"{}\" 出现在 {} 个位置 ({})",
                            name,
                            locations.len(),
                            locations.join(", ")
                        ),
                        severity: IssueSeverity::Medium,
                        task_name: "duplicate_detection".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// 标签规范化
    ///
    /// 扫描所有 markdown 文件的 frontmatter 中的 tags 字段，
    /// 检测不规范的标签（大写、空格等）。
    pub fn tag_normalization(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let tag_manager = TagManager::new();

        let all_files = self.collect_all_md_files()?;
        for file_path in &all_files {
            let content = fs::read_to_string(file_path)?;
            let relative = self.relative_path(file_path);

            // 提取 frontmatter 中的 tags
            if let Some(tags) = extract_frontmatter_tags(&content) {
                for tag in tags {
                    let normalized = tag_manager.normalize(&tag);
                    if tag != normalized {
                        issues.push(Issue {
                            file_path: relative.clone(),
                            description: format!(
                                "标签不规范: \"{}\" 应为 \"{}\"",
                                tag, normalized
                            ),
                            severity: IssueSeverity::Low,
                            task_name: "tag_normalization".to_string(),
                        });
                    }
                }
            }
        }

        Ok(issues)
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

    /// 收集所有 .md 文件名（不含扩展名）
    fn collect_all_md_stems(&self) -> Result<HashSet<String>> {
        let files = self.collect_all_md_files()?;
        let mut stems = HashSet::new();

        for f in &files {
            if let Some(stem) = f.file_stem() {
                stems.insert(stem.to_string_lossy().to_string());
            }
        }

        Ok(stems)
    }

    /// 获取相对于 vault 根的路径字符串
    fn relative_path(&self, path: &std::path::PathBuf) -> String {
        path.strip_prefix(self.vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}

/// 从 markdown 内容中提取所有 wiki 链接目标
fn extract_wikilinks(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume second [
                let mut link_content = String::new();
                let mut found_close = false;

                while let Some(ch) = chars.next() {
                    if ch == ']' {
                        if chars.peek() == Some(&']') {
                            chars.next();
                            found_close = true;
                            break;
                        }
                    }
                    link_content.push(ch);
                }

                if found_close && !link_content.is_empty() {
                    links.push(link_content);
                }
            }
        }
    }

    links
}

/// 从 frontmatter 中提取 tags 列表
fn extract_frontmatter_tags(content: &str) -> Option<Vec<String>> {
    // 查找 frontmatter 块
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end = after_first.find("---")?;
    let frontmatter = &after_first[..end];

    // 查找 tags 行
    let mut in_tags = false;
    let mut tags = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("tags:") {
            in_tags = true;
            // 如果 tags 在同一行: tags: [a, b, c]
            if let Some(rest) = trimmed.strip_prefix("tags:") {
                let rest = rest.trim();
                if rest.starts_with('[') && rest.ends_with(']') {
                    let inner = &rest[1..rest.len() - 1];
                    for tag in inner.split(',') {
                        let tag = tag.trim().trim_matches('"').trim_matches('\'');
                        if !tag.is_empty() {
                            tags.push(tag.to_string());
                        }
                    }
                    return Some(tags);
                }
            }
            continue;
        }

        if in_tags {
            if trimmed.starts_with("- ") {
                let tag = trimmed[2..].trim().trim_matches('"').trim_matches('\'');
                if !tag.is_empty() {
                    tags.push(tag.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                // 遇到非 tag 行，tags 区域结束
                break;
            }
        }
    }

    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_integrity_check_no_files() {
        let tmp = tempfile::tempdir().unwrap();
        let task = IntegrityTask::new(tmp.path());
        let issues = task.link_integrity_check().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_link_integrity_check_broken_link() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        fs::create_dir_all(&daily_dir).unwrap();

        fs::write(
            daily_dir.join("2026-06-06.md"),
            "# Daily\n\nSee also [[NonExistent Doc]]",
        )
        .unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.link_integrity_check().unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("NonExistent Doc"));
        assert_eq!(issues[0].severity, IssueSeverity::High);
    }

    #[test]
    fn test_link_integrity_check_valid_link() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        fs::create_dir_all(&daily_dir).unwrap();

        // 创建目标文件
        fs::write(tmp.path().join("Daily/meeting.md"), "# Meeting Notes").unwrap();
        // 源文件链接到目标
        fs::write(
            daily_dir.join("2026-06-06.md"),
            "# Daily\n\nSee [[meeting]]",
        )
        .unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.link_integrity_check().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_link_integrity_check_with_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        fs::create_dir_all(&daily_dir).unwrap();

        fs::write(
            daily_dir.join("2026-06-06.md"),
            "# Daily\n\nSee [[meeting|Meeting Notes]]",
        )
        .unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.link_integrity_check().unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("meeting"));
    }

    #[test]
    fn test_duplicate_detection_no_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        let projects_dir = tmp.path().join("Projects");
        fs::create_dir_all(&daily_dir).unwrap();
        fs::create_dir_all(&projects_dir).unwrap();

        fs::write(daily_dir.join("2026-06-06.md"), "# Daily").unwrap();
        fs::write(projects_dir.join("project-alpha.md"), "# Project").unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.duplicate_detection().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_duplicate_detection_with_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        let projects_dir = tmp.path().join("Projects");
        fs::create_dir_all(&daily_dir).unwrap();
        fs::create_dir_all(&projects_dir).unwrap();

        // 两个不同目录下的同名文件
        fs::write(daily_dir.join("meeting.md"), "# Daily Meeting").unwrap();
        fs::write(projects_dir.join("meeting.md"), "# Project Meeting").unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.duplicate_detection().unwrap();
        assert_eq!(issues.len(), 2); // 两个位置都报告
    }

    #[test]
    fn test_tag_normalization_no_issues() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("note.md"),
            "---\ntags:\n  - meeting\n  - project-alpha\n---\nContent",
        )
        .unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.tag_normalization().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_tag_normalization_with_issues() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("note.md"),
            "---\ntags:\n  - Meeting\n  - Project Alpha\n---\nContent",
        )
        .unwrap();

        let task = IntegrityTask::new(tmp.path());
        let issues = task.tag_normalization().unwrap();
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_extract_wikilinks() {
        let content = "See [[Doc A]] and [[Doc B|alias]] and [[Doc C]]";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["Doc A", "Doc B|alias", "Doc C"]);
    }

    #[test]
    fn test_extract_wikilinks_no_links() {
        let content = "No links here, just plain text.";
        let links = extract_wikilinks(content);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_frontmatter_tags_list() {
        let content = "---\ntitle: Note\ntags:\n  - meeting\n  - project\n---\nBody";
        let tags = extract_frontmatter_tags(content);
        assert_eq!(
            tags,
            Some(vec!["meeting".to_string(), "project".to_string()])
        );
    }

    #[test]
    fn test_extract_frontmatter_tags_inline() {
        let content = "---\ntitle: Note\ntags: [meeting, project]\n---\nBody";
        let tags = extract_frontmatter_tags(content);
        assert_eq!(
            tags,
            Some(vec!["meeting".to_string(), "project".to_string()])
        );
    }

    #[test]
    fn test_extract_frontmatter_no_tags() {
        let content = "---\ntitle: Note\n---\nBody";
        let tags = extract_frontmatter_tags(content);
        assert!(tags.is_none());
    }

    #[test]
    fn test_extract_frontmatter_no_frontmatter() {
        let content = "# Just a heading\nSome content";
        let tags = extract_frontmatter_tags(content);
        assert!(tags.is_none());
    }
}
