//! 数据库路径解析（共享模块）
//!
//! `resolve_db_path` 被 events.rs 和 audit.rs 共用，避免重复实现。
//! 优先使用配置文件中的 `storage.db_path`，若未配置则回退到 Tauri `app_data_dir`。

use std::path::Path;

/// 配置文件中的默认占位路径（不应作为实际路径使用）。
const PLACEHOLDER_DB_PATH: &str = "~/.work-better/data.db";

/// 解析数据库路径：优先使用配置文件中的 `storage.db_path`，
/// 若未配置或为占位值则回退到默认目录。
///
/// 回退策略：
/// - debug 构建（dev 模式）：使用项目根目录的 `data/work-better.db`
/// - release 构建（生产模式）：使用 Tauri `app_data_dir`
///
/// 返回绝对路径字符串。所有错误均通过 `Err` 返回，不会 panic。
pub fn resolve_db_path(app: &tauri::AppHandle) -> Result<String, Box<dyn std::error::Error>> {
    // 尝试从配置文件读取 db_path
    if let Ok(config) = super::settings::load_config() {
        let db_path = &config.storage.db_path;
        if !db_path.is_empty() && db_path != PLACEHOLDER_DB_PATH {
            return resolve_custom_path(db_path);
        }
    }

    // 配置为默认值或读取失败时，按构建模式选择回退路径
    resolve_fallback_path(app)
}

/// 回退路径：dev 模式用项目目录，生产模式用 Tauri app_data_dir。
fn resolve_fallback_path(app: &tauri::AppHandle) -> Result<String, Box<dyn std::error::Error>> {
    // debug 构建：尝试项目根目录的 data/work-better.db
    #[cfg(debug_assertions)]
    {
        if let Ok(cwd) = std::env::current_dir() {
            let dev_db = cwd.join("data").join("work-better.db");
            // 如果 data/ 目录已存在或项目根有 Cargo.toml（标识 Rust 项目），使用项目目录
            if cwd.join("Cargo.toml").exists() || cwd.join("data").exists() {
                if let Some(parent) = dev_db.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        format!("Failed to create dev db directory '{}': {}", parent.display(), e)
                    })?;
                }
                eprintln!("[db] Dev mode: using project-local {}", dev_db.display());
                return Ok(dev_db.to_string_lossy().to_string());
            }
        }
    }

    // release 构建 或 debug 回退：使用 Tauri app_data_dir
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    Ok(data_dir.join("work-better.db").to_string_lossy().to_string())
}

/// 解析用户自定义的数据库路径（展开 `~`、相对路径、路径遍历防护）。
fn resolve_custom_path(db_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 路径遍历防护：拒绝包含 `..` 的路径
    if db_path.split('/').any(|seg| seg == "..") {
        return Err(format!("Database path must not contain '..': {}", db_path).into());
    }

    let expanded = if db_path.starts_with('~') {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable is not set")?;
        if home.is_empty() {
            return Err("HOME environment variable is empty".into());
        }
        db_path.replacen('~', &home, 1)
    } else if db_path.starts_with("./") {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        cwd.join(&db_path[2..]).to_string_lossy().to_string()
    } else {
        db_path.to_string()
    };

    // 确保父目录存在（传播错误而非静默忽略）
    if let Some(parent) = Path::new(&expanded).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create db directory '{}': {}", parent.display(), e))?;
    }

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_path_traversal() {
        let result = resolve_custom_path("/some/../etc/passwd");
        assert!(result.is_err(), "Should reject path with '..'");
        assert!(result.unwrap_err().to_string().contains("'..'"));
    }

    #[test]
    fn reject_tilde_with_empty_home() {
        // 暂存并清除 HOME
        let original = std::env::var("HOME").ok();
        std::env::set_var("HOME", "");

        let result = resolve_custom_path("~/data.db");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HOME"));

        // 恢复 HOME
        match original {
            Some(val) => std::env::set_var("HOME", val),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn expand_relative_path() {
        let result = resolve_custom_path("./test-data.db");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(Path::new(&path).is_absolute());
        assert!(path.ends_with("test-data.db"));
    }
}
