//! VaultManager —— vault 路径、目录结构管理

use std::path::{Path, PathBuf};

use wb_core::error::Result;

/// vault 顶级目录列表
const TOP_LEVEL_DIRS: &[&str] = &[
    "Daily",
    "Projects",
    "People",
    "Tasks",
    "Reports",
    "Knowledge",
    "System",
];

/// Obsidian vault 目录管理器
#[derive(Debug, Clone)]
pub struct VaultManager {
    vault_path: PathBuf,
}

impl VaultManager {
    /// 创建新的 VaultManager 并确保目录结构存在
    pub fn new(vault_path: &str) -> Result<Self> {
        let manager = Self {
            vault_path: PathBuf::from(vault_path),
        };
        manager.ensure_structure()?;
        Ok(manager)
    }

    /// 确保 7 个顶级目录存在
    pub fn ensure_structure(&self) -> Result<()> {
        for dir in TOP_LEVEL_DIRS {
            let path = self.vault_path.join(dir);
            std::fs::create_dir_all(&path)?;
        }
        Ok(())
    }

    /// 获取分类对应的目录路径
    pub fn path_for(&self, category: &str) -> PathBuf {
        self.vault_path.join(category)
    }

    /// 获取 vault 根路径
    pub fn root(&self) -> &Path {
        &self.vault_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_all_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("test-vault");
        let manager = VaultManager::new(vault_path.to_str().unwrap()).unwrap();

        for dir in TOP_LEVEL_DIRS {
            let expected = vault_path.join(dir);
            assert!(expected.exists(), "directory {} should exist", dir);
        }

        assert_eq!(manager.root(), vault_path.as_path());
    }

    #[test]
    fn test_ensure_structure_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("test-vault");

        let _ = VaultManager::new(vault_path.to_str().unwrap()).unwrap();
        // 第二次调用不应报错
        let _ = VaultManager::new(vault_path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_path_for() {
        let tmp = tempfile::tempdir().unwrap();
        let manager = VaultManager::new(tmp.path().to_str().unwrap()).unwrap();

        let daily_path = manager.path_for("Daily");
        assert_eq!(daily_path, tmp.path().join("Daily"));

        let projects_path = manager.path_for("Projects");
        assert_eq!(projects_path, tmp.path().join("Projects"));
    }

    #[test]
    fn test_top_level_dirs_count() {
        assert_eq!(TOP_LEVEL_DIRS.len(), 7);
    }
}
