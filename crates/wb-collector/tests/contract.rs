//! Contract tests (C-layer)
//!
//! C2: Feishu API Schema (httpmock)
//! C3: Platform Contracts

// ============================================================
// C2: Feishu API Schema
// ============================================================

#[cfg(test)]
mod c2_feishu_api {
    use httpmock::prelude::*;

    /// Minimal blocking HTTP GET using stdlib (avoids adding reqwest dep)
    fn blocking_get(url: &str) -> String {
        use std::io::Read;
        use std::net::TcpStream;

        let url = url.trim_start_matches("http://");
        let (host_port, path) = match url.find('/') {
            Some(i) => (&url[..i], &url[i..]),
            None => (url, "/"),
        };

        let mut stream = TcpStream::connect(host_port).expect("failed to connect to mock server");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host_port
        );
        std::io::Write::write_all(&mut stream, request.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        response
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default()
    }

    /// C2-01: Feishu API response schema validation
    #[test]
    fn c2_01_feishu_api_response_schema() {
        let server = MockServer::start();
        let fixture = include_str!("../fixtures/feishu_api_success.json");

        let mock = server.mock(|when, then| {
            when.method(GET).path("/open-apis/im/v1/messages");
            then.status(200)
                .header("content-type", "application/json")
                .body(fixture);
        });

        let resp = blocking_get(&format!("{}/open-apis/im/v1/messages", server.base_url()));
        mock.assert();

        let body: serde_json::Value = serde_json::from_str(&resp).unwrap();

        assert_eq!(body["code"], 0, "code should be 0 for success");
        assert_eq!(body["msg"], "success", "msg should be 'success'");
        assert!(body["data"].is_object(), "data should be an object");
        assert!(
            body["data"]["items"].is_array(),
            "data.items should be an array"
        );
        assert!(
            body["data"]["page_token"].is_string(),
            "data.page_token should be a string"
        );
        assert!(
            body["data"]["has_more"].is_boolean(),
            "data.has_more should be a boolean"
        );

        let items = body["data"]["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert!(item["message_id"].is_string());
        assert!(item["msg_type"].is_string());
        assert!(item["body"].is_object());
        assert!(item["sender"].is_object());
        assert!(item["create_time"].is_string());
        assert!(item["chat_id"].is_string());
    }

    /// C2-02: Feishu API error response format
    #[test]
    fn c2_02_feishu_api_error_response() {
        let server = MockServer::start();
        let fixture = include_str!("../fixtures/feishu_api_error.json");

        let mock = server.mock(|when, then| {
            when.method(GET).path("/open-apis/im/v1/messages");
            then.status(200)
                .header("content-type", "application/json")
                .body(fixture);
        });

        let resp = blocking_get(&format!("{}/open-apis/im/v1/messages", server.base_url()));
        mock.assert();

        let body: serde_json::Value = serde_json::from_str(&resp).unwrap();

        assert_ne!(body["code"], 0, "error code should be non-zero");
        assert!(body["msg"].is_string(), "error msg should be a string");
        assert!(
            !body["msg"].as_str().unwrap().is_empty(),
            "error msg should not be empty"
        );
        assert!(body["data"].is_object(), "data should still be present");
    }
}

// ============================================================
// C3: Platform Contracts
// ============================================================

/// C3-01: macOS screencapture exit code assertion
#[cfg(target_os = "macos")]
#[test]
fn c3_01_macos_screencapture_exit_code() {
    let output = std::process::Command::new("screencapture")
        .arg("-h")
        .output()
        .expect("screencapture should be available on macOS");

    assert!(
        output.status.code().is_some(),
        "screencapture should return an exit code"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "screencapture -h should exit with code 1 (usage)"
    );
}

/// C3-02: HOME config directory creation assertion
#[test]
fn c3_02_home_config_directory_convention() {
    let home = std::env::var("HOME").expect("HOME env var should be set");
    assert!(!home.is_empty(), "HOME should not be empty");

    #[cfg(target_os = "macos")]
    {
        let library_path = format!("{}/Library/Application Support", home);
        let config_path = format!("{}/.config", home);
        assert!(
            std::path::Path::new(&library_path).exists()
                || std::path::Path::new(&config_path).exists(),
            "At least one config directory should exist: {} or {}",
            library_path,
            config_path
        );
    }

    #[cfg(target_os = "linux")]
    {
        let config_path = format!("{}/.config", home);
        assert!(
            std::path::Path::new(&config_path).exists(),
            "XDG config directory should exist: {}",
            config_path
        );
    }
}

/// C3-03: macOS SQLite file locking assertion
#[cfg(target_os = "macos")]
#[test]
fn c3_03_macos_sqlite_file_locking() {
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test_locking.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT);
         INSERT INTO test VALUES (1, 'hello');",
    )
    .unwrap();

    let file_bytes = fs::read(&db_path).unwrap();
    assert!(file_bytes.len() >= 16, "SQLite file should have header");
    assert_eq!(
        &file_bytes[..16],
        b"SQLite format 3\0",
        "File should have SQLite magic header"
    );

    conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode, "wal", "WAL mode should be settable on macOS");

    let conn2 = rusqlite::Connection::open(&db_path).unwrap();
    let value: String = conn2
        .query_row("SELECT value FROM test WHERE id = 1;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "hello", "Concurrent read should succeed");

    drop(conn2);
    drop(conn);
}
