//! 命令执行器 -- 封装外部 CLI 调用

use std::process::Command;

use serde::de::DeserializeOwned;
use wb_core::error::{Result, WbError};

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
