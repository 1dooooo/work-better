//! 命令执行器 -- 封装外部 CLI 调用

use std::process::Command;

use serde::de::DeserializeOwned;
use wb_core::error::{Result, WbError};

/// 检查外部工具是否可用（通过执行 `<tool> --version`）
///
/// 返回 `true` 表示工具存在且可执行。
pub fn check_tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 执行外部命令，返回 stdout
pub fn execute(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| WbError::Collector(format!("Failed to execute {}: {}", program, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(WbError::Collector(format!(
            "Command {} failed (exit {}): stderr={}, stdout={}",
            program,
            output.status.code().unwrap_or(-1),
            stderr,
            stdout
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| WbError::Collector(format!("Invalid UTF-8 output from {}: {}", program, e)))
}

/// 执行外部命令，解析 JSON 输出
pub fn execute_json<T: DeserializeOwned>(program: &str, args: &[&str]) -> Result<T> {
    let stdout = execute(program, args)?;

    serde_json::from_str(&stdout).map_err(|e| {
        WbError::Collector(format!(
            "Failed to parse JSON from {}: {}. Output: {}",
            program,
            e,
            &stdout[..stdout.len().min(200)]
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tool_available_existing() {
        // `cargo` should exist in dev environment and supports --version
        assert!(check_tool_available("cargo"));
    }

    #[test]
    fn test_check_tool_available_nonexistent() {
        assert!(!check_tool_available("nonexistent_tool_12345"));
    }
}
