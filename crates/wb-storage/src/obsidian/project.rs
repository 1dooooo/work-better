//! ProjectDir —— 项目目录管理

use std::path::PathBuf;

use super::vault::VaultManager;
use wb_core::error::Result;

/// 项目目录管理器
#[derive(Debug, Clone)]
pub struct ProjectDir {
    vault: VaultManager,
}

impl ProjectDir {
    /// 创建新的项目目录管理器
    pub fn new(vault: &VaultManager) -> Self {
        Self {
            vault: vault.clone(),
        }
    }

    /// 确保指定项目的目录存在，返回项目目录路径
    pub fn ensure(&self, name: &str) -> Result<PathBuf> {
        let sanitized = sanitize_project_name(name);
        let path = self.vault.path_for("Projects").join(&sanitized);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// 列出所有已有项目名称
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let projects_dir = self.vault.path_for("Projects");

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut projects = Vec::new();
        for entry in std::fs::read_dir(&projects_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    projects.push(name.to_string());
                }
            }
        }

        projects.sort();
        Ok(projects)
    }
}

/// 规范化项目名称：保留字母、数字、连字符、下划线和中文
fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ' ' => '-',
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault() -> (tempfile::TempDir, VaultManager) {
        let tmp = tempfile::tempdir().unwrap();
        let manager = VaultManager::new(tmp.path().to_str().unwrap()).unwrap();
        (tmp, manager)
    }

    #[test]
    fn test_ensure_creates_project_dir() {
        let (_tmp, vault) = make_vault();
        let pd = ProjectDir::new(&vault);

        let path = pd.ensure("alpha-project").unwrap();
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn test_list_projects_empty() {
        let (_tmp, vault) = make_vault();
        let pd = ProjectDir::new(&vault);

        let projects = pd.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_list_projects_after_creating() {
        let (_tmp, vault) = make_vault();
        let pd = ProjectDir::new(&vault);

        pd.ensure("beta").unwrap();
        pd.ensure("alpha").unwrap();

        let mut projects = pd.list_projects().unwrap();
        projects.sort();
        assert_eq!(projects, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_sanitize_project_name() {
        assert_eq!(sanitize_project_name("my project"), "my-project");
        assert_eq!(sanitize_project_name("path/to/project"), "path_to_project");
        assert_eq!(sanitize_project_name("normal"), "normal");
    }

    #[test]
    fn test_ensure_idempotent() {
        let (_tmp, vault) = make_vault();
        let pd = ProjectDir::new(&vault);

        let path1 = pd.ensure("gamma").unwrap();
        let path2 = pd.ensure("gamma").unwrap();
        assert_eq!(path1, path2);
    }
}
