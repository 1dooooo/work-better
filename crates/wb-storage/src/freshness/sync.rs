//! 同步任务 —— 任务状态同步与文档变更检测

use std::fs;
use std::path::Path;

use wb_core::error::Result;

use super::report::{Issue, IssueSeverity};

/// 同步任务执行器
#[derive(Debug, Clone)]
pub struct SyncTask<'a> {
    vault_path: &'a Path,
}

impl<'a> SyncTask<'a> {
    /// 创建同步任务
    pub fn new(vault_path: &'a Path) -> Self {
        Self { vault_path }
    }

    /// 任务状态同步
    ///
    /// 扫描 Tasks 目录下的 markdown 文件，检查 frontmatter 中的 status 字段。
    /// 如果文件存在 `status: done` 但未标记完成日期，则报告问题。
    pub fn task_status_sync(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let tasks_dir = self.vault_path.join("Tasks");

        if !tasks_dir.exists() {
            return Ok(issues);
        }

        for entry in fs::read_dir(&tasks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // 检查 frontmatter 中是否有 status: done
            let has_done_status = content.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == "status: done" || trimmed == "status: \"done\""
            });
            if has_done_status {
                // 如果没有 completed 字段，认为缺少完成日期
                if !content.contains("completed:") {
                    issues.push(Issue {
                        file_path: format!("Tasks/{}", file_name),
                        description: "任务标记为 done 但缺少 completed 日期字段".to_string(),
                        severity: IssueSeverity::Medium,
                        task_name: "task_status_sync".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// 文档变更检测
    ///
    /// 扫描 Daily 目录，检查最近 7 天内是否都有日志文件。
    /// 缺少日志的日期报告为问题。
    pub fn document_change_detect(&self) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let daily_dir = self.vault_path.join("Daily");

        if !daily_dir.exists() {
            return Ok(issues);
        }

        // 收集已有的日期文件
        let mut existing_dates = Vec::new();
        for entry in fs::read_dir(&daily_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "md") {
                if let Some(name) = path.file_stem() {
                    existing_dates.push(name.to_string_lossy().to_string());
                }
            }
        }

        // 检查最近 7 天是否都有日志
        let today = chrono::Local::now().date_naive();
        for i in 0..7 {
            let date = today - chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();

            if !existing_dates.iter().any(|d| d.contains(&date_str)) {
                issues.push(Issue {
                    file_path: format!("Daily/{}.md", date_str),
                    description: format!("缺少日期为 {} 的每日日志", date_str),
                    severity: if i == 0 {
                        IssueSeverity::High
                    } else {
                        IssueSeverity::Low
                    },
                    task_name: "document_change_detect".to_string(),
                });
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_sync_no_tasks_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let sync = SyncTask::new(tmp.path());
        let issues = sync.task_status_sync().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_task_status_sync_done_without_completed() {
        let tmp = tempfile::tempdir().unwrap();
        let tasks_dir = tmp.path().join("Tasks");
        fs::create_dir_all(&tasks_dir).unwrap();

        fs::write(
            tasks_dir.join("review-code.md"),
            "---\ntitle: Review Code\nstatus: done\n---\nContent here",
        )
        .unwrap();

        let sync = SyncTask::new(tmp.path());
        let issues = sync.task_status_sync().unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("completed"));
    }

    #[test]
    fn test_task_status_sync_done_with_completed() {
        let tmp = tempfile::tempdir().unwrap();
        let tasks_dir = tmp.path().join("Tasks");
        fs::create_dir_all(&tasks_dir).unwrap();

        fs::write(
            tasks_dir.join("review-code.md"),
            "---\ntitle: Review Code\nstatus: done\ncompleted: 2026-06-06\n---\nContent here",
        )
        .unwrap();

        let sync = SyncTask::new(tmp.path());
        let issues = sync.task_status_sync().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_document_change_detect_no_daily_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let sync = SyncTask::new(tmp.path());
        let issues = sync.document_change_detect().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_document_change_detect_with_today() {
        let tmp = tempfile::tempdir().unwrap();
        let daily_dir = tmp.path().join("Daily");
        fs::create_dir_all(&daily_dir).unwrap();

        let today = chrono::Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();
        fs::write(daily_dir.join(format!("{}.md", date_str)), "# Today").unwrap();

        let sync = SyncTask::new(tmp.path());
        let issues = sync.document_change_detect().unwrap();
        // 今天的日志已存在，但其他 6 天可能缺失
        assert!(issues.iter().all(|i| !i.file_path.contains(&date_str)));
    }
}
