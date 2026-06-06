//! DailyJournal —— 按日期生成日记文件

use std::path::PathBuf;

use chrono::NaiveDate;

use super::vault::VaultManager;
use wb_core::error::Result;

/// 日记管理器
#[derive(Debug, Clone)]
pub struct DailyJournal {
    vault: VaultManager,
}

impl DailyJournal {
    /// 创建新的日记管理器
    pub fn new(vault: &VaultManager) -> Self {
        Self {
            vault: vault.clone(),
        }
    }

    /// 获取指定日期的日记文件路径: Daily/YYYY-MM-DD.md
    pub fn path_for_date(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("{}.md", date.format("%Y-%m-%d"));
        self.vault.path_for("Daily").join(filename)
    }

    /// 追加日记条目到指定日期的文件
    ///
    /// 如果文件不存在，会创建带有日期标题的新文件。
    /// 如果文件已存在，会在末尾追加内容。
    pub fn append(&self, date: NaiveDate, content: &str) -> Result<PathBuf> {
        let path = self.path_for_date(date);

        // 确保 Daily 目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let entry = if path.exists() {
            // 文件已存在，追加内容
            format!("\n\n{}", content)
        } else {
            // 新文件，添加日期标题
            format!(
                "---\ndate: {}\ntype: daily\n---\n\n# {}\n\n{}",
                date.format("%Y-%m-%d"),
                date.format("%Y-%m-%d"),
                content
            )
        };

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        file.write_all(entry.as_bytes())?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_vault() -> (tempfile::TempDir, VaultManager) {
        let tmp = tempfile::tempdir().unwrap();
        let manager = VaultManager::new(tmp.path().to_str().unwrap()).unwrap();
        (tmp, manager)
    }

    #[test]
    fn test_path_for_date_format() {
        let (_tmp, vault) = make_vault();
        let journal = DailyJournal::new(&vault);

        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let path = journal.path_for_date(date);

        assert!(path.ends_with("Daily/2026-06-06.md"));
    }

    #[test]
    fn test_append_creates_new_file() {
        let (_tmp, vault) = make_vault();
        let journal = DailyJournal::new(&vault);

        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let path = journal.append(date, "今天完成了 Task 1").unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# 2026-06-06"));
        assert!(content.contains("今天完成了 Task 1"));
        assert!(content.contains("type: daily"));
    }

    #[test]
    fn test_append_to_existing_file() {
        let (_tmp, vault) = make_vault();
        let journal = DailyJournal::new(&vault);

        let date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        journal.append(date, "第一条").unwrap();
        journal.append(date, "第二条").unwrap();

        let path = journal.path_for_date(date);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("第一条"));
        assert!(content.contains("第二条"));
        // 标题只出现一次
        assert_eq!(content.matches("# 2026-06-06").count(), 1);
    }

    #[test]
    fn test_path_for_different_dates() {
        let (_tmp, vault) = make_vault();
        let journal = DailyJournal::new(&vault);

        let date1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

        let path1 = journal.path_for_date(date1);
        let path2 = journal.path_for_date(date2);

        assert!(path1.ends_with("Daily/2026-01-01.md"));
        assert!(path2.ends_with("Daily/2026-12-31.md"));
        assert_ne!(path1, path2);
    }
}
