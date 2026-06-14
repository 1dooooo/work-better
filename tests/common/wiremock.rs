//! HTTP Mock Server 基础设施
//!
//! 供 L3/L4 测试复用的 HTTP mock 能力。
//! 基于 `httpmock` crate，提供统一的 mock server 管理和预设响应注册。
//!
//! ## 使用方式
//!
//! ```ignore
//! use common::wiremock::{start_mock_server, mock_classify_response};
//!
//! let server = start_mock_server();
//! let _mock = mock_classify_response(&server, "input text", r#"{"category":"task","confidence":0.9}"#);
//! // server.base_url() 可用于配置被测组件的 API 地址
//! ```

use httpmock::{Mock, MockServer};

/// 启动一个 mock HTTP server
///
/// 返回 `MockServer` 实例，其 `base_url()` 可用于配置被测客户端。
/// Server 在 `MockServer` drop 时自动停止。
pub fn start_mock_server() -> MockServer {
    MockServer::start()
}

/// 注册分类 mock 响应
///
/// 当 POST `/api/classify` 的 body 包含 `input` 时，返回 `output` JSON。
///
/// # Arguments
/// * `server` - mock server 实例
/// * `input` - 期望的输入文本片段（子串匹配）
/// * `output` - 返回的 JSON 字符串
///
/// # Returns
/// `Mock` — 可用于断言调用次数、删除等
pub fn mock_classify_response<'a>(server: &'a MockServer, input: &str, output: &str) -> Mock<'a> {
    let input_owned = input.to_string();
    let output_owned = output.to_string();
    server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/api/classify")
            .body_contains(&input_owned);
        then.status(200)
            .header("content-type", "application/json")
            .body(&output_owned);
    })
}

/// 注册提取 mock 响应
///
/// 当 POST `/api/extract` 的 body 包含 `input` 时，返回 `output` JSON。
///
/// # Arguments
/// * `server` - mock server 实例
/// * `input` - 期望的输入文本片段（子串匹配）
/// * `output` - 返回的 JSON 字符串
///
/// # Returns
/// `Mock` — 可用于断言调用次数、删除等
pub fn mock_extract_response<'a>(server: &'a MockServer, input: &str, output: &str) -> Mock<'a> {
    let input_owned = input.to_string();
    let output_owned = output.to_string();
    server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/api/extract")
            .body_contains(&input_owned);
        then.status(200)
            .header("content-type", "application/json")
            .body(&output_owned);
    })
}

/// 注册任务发现 mock 响应
///
/// 当 POST `/api/discover` 的 body 包含 `input` 时，返回 `output` JSON。
///
/// # Arguments
/// * `server` - mock server 实例
/// * `input` - 期望的输入文本片段（子串匹配）
/// * `output` - 返回的 JSON 字符串
///
/// # Returns
/// `Mock` — 可用于断言调用次数、删除等
pub fn mock_discover_response<'a>(server: &'a MockServer, input: &str, output: &str) -> Mock<'a> {
    let input_owned = input.to_string();
    let output_owned = output.to_string();
    server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/api/discover")
            .body_contains(&input_owned);
        then.status(200)
            .header("content-type", "application/json")
            .body(&output_owned);
    })
}

/// 通用 mock 注册 —— 自定义路径和方法
///
/// 用于未被 `mock_*_response` 覆盖的自定义场景。
/// 当 `body_contains` 为 `Some` 时，额外匹配请求 body 中的子串。
pub fn mock_custom_response<'a>(
    server: &'a MockServer,
    method: httpmock::Method,
    path: &str,
    body_contains: Option<&str>,
    status: u16,
    output: &str,
) -> Mock<'a> {
    let path_owned = path.to_string();
    let body_match = body_contains.map(|s| s.to_string());
    let output_owned = output.to_string();
    server.mock(|when, then| {
        // Build the When matcher — method/path consume self, so chain them
        let when = when.method(method).path(path_owned);
        if let Some(body) = body_match {
            when.body_contains(body);
        }
        then.status(status)
            .header("content-type", "application/json")
            .body(output_owned);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_mock_server_returns_address() {
        let server = start_mock_server();
        let url = server.base_url();
        assert!(
            url.starts_with("http://127.0.0.1:"),
            "base_url 应为本地地址: {}",
            url
        );
    }

    #[test]
    fn test_mock_classify_response_registers() {
        let server = start_mock_server();
        let output = r#"{"category":"task","confidence":0.9,"reasoning":"test"}"#;
        let mock = mock_classify_response(&server, "test input", output);
        assert_eq!(mock.hits(), 0, "注册后尚未被调用");
    }

    #[test]
    fn test_mock_extract_response_registers() {
        let server = start_mock_server();
        let output = r#"{"title":"Test","summary":"Sum","detail":"Det"}"#;
        let mock = mock_extract_response(&server, "extract me", output);
        assert_eq!(mock.hits(), 0);
    }

    #[test]
    fn test_mock_discover_response_registers() {
        let server = start_mock_server();
        let output = r#"[{"id":"t-1","title":"Found task"}]"#;
        let mock = mock_discover_response(&server, "find tasks", output);
        assert_eq!(mock.hits(), 0);
    }

    #[test]
    fn test_mock_server_independence() {
        let server1 = start_mock_server();
        let server2 = start_mock_server();
        assert_ne!(
            server1.base_url(),
            server2.base_url(),
            "不同 server 应有不同地址"
        );
    }

    #[test]
    fn test_mock_custom_response() {
        let server = start_mock_server();
        let mock = mock_custom_response(
            &server,
            httpmock::Method::GET,
            "/api/health",
            None,
            200,
            r#"{"status":"ok"}"#,
        );
        assert_eq!(mock.hits(), 0);
    }

    #[test]
    fn test_mock_custom_response_with_body_match() {
        let server = start_mock_server();
        let mock = mock_custom_response(
            &server,
            httpmock::Method::POST,
            "/api/custom",
            Some("match_me"),
            200,
            r#"{"found":true}"#,
        );
        assert_eq!(mock.hits(), 0);
    }
}
