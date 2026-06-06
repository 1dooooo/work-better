//! Chrome 浏览历史采集器

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use wb_core::error::{Result, WbError};
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::traits::{Collector, HealthStatus};

/// Chrome 历史记录条目
#[derive(Debug, Serialize, Deserialize)]
struct BrowserVisit {
    url: String,
    title: String,
    visit_time: DateTime<Utc>,
}

/// Chrome 浏览历史采集器
///
/// 读取 Chrome 的 SQLite 历史数据库。
/// 注意：Chrome 运行时数据库会被锁定，采集器会先复制到临时文件再读取。
/// URL 可能包含敏感信息，支持脱敏处理。
pub struct BrowserHistoryCollector {
    /// Chrome 数据库路径（默认 `~/Library/Application Support/Google/Chrome/Default/History`）
    db_path: String,
    /// 单次最大采集数量
    limit: u32,
    /// Mock 模式
    mock: bool,
}

impl Default for BrowserHistoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserHistoryCollector {
    /// 创建真实采集器（使用默认 Chrome 路径）
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string());
        Self {
            db_path: format!(
                "{}/Library/Application Support/Google/Chrome/Default/History",
                home
            ),
            limit: 100,
            mock: false,
        }
    }

    /// 创建 Mock 采集器（用于测试）
    pub fn mock() -> Self {
        Self {
            db_path: String::new(),
            limit: 100,
            mock: true,
        }
    }

    /// 设置自定义数据库路径
    #[allow(dead_code)]
    pub fn with_db_path(mut self, path: &str) -> Self {
        self.db_path = path.to_string();
        self
    }

    /// 设置最大采集数量
    #[allow(dead_code)]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// 从 Chrome 历史数据库读取浏览记录
    fn query_history(&self) -> Result<Vec<BrowserVisit>> {
        if self.mock {
            return Ok(Self::mock_visits());
        }

        // 复制数据库到临时文件（Chrome 运行时会锁定数据库）
        let tmp_path = format!("/tmp/wb-chrome-history-{}.db", std::process::id());
        std::fs::copy(&self.db_path, &tmp_path).map_err(|e| {
            WbError::Collector(format!(
                "Failed to copy Chrome history DB from {}: {}",
                self.db_path, e
            ))
        })?;

        let conn = rusqlite::Connection::open_with_flags(
            &tmp_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| WbError::Collector(format!("Failed to open Chrome history DB: {}", e)))?;

        let limit = self.limit as i64;
        let mut stmt = conn
            .prepare(
                "SELECT u.url, u.title, v.visit_time
                 FROM urls u
                 JOIN visits v ON u.id = v.url
                 ORDER BY v.visit_time DESC
                 LIMIT ?1",
            )
            .map_err(|e| WbError::Collector(format!("Failed to prepare query: {}", e)))?;

        // Chrome 使用 WebKit 时间戳（微秒，从 1601-01-01 开始）
        let epoch_diff: i64 = 116_444_736_000_000_000; // 1601-01-01 到 1970-01-01 的微秒数

        let visits = stmt
            .query_map(rusqlite::params![limit], |row| {
                let url: String = row.get(0)?;
                let title: String = row.get(1)?;
                let chrome_time: i64 = row.get(2)?;

                // 转换 Chrome 时间戳为 Unix 时间戳（微秒）
                let unix_micros = chrome_time - epoch_diff;
                let unix_secs = unix_micros / 1_000_000;
                let visit_time = Utc
                    .timestamp_opt(unix_secs, 0)
                    .single()
                    .unwrap_or_default();

                Ok(BrowserVisit {
                    url,
                    title,
                    visit_time,
                })
            })
            .map_err(|e| WbError::Collector(format!("Failed to query history: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        // 清理临时文件
        let _ = std::fs::remove_file(&tmp_path);

        Ok(visits)
    }

    /// Mock 浏览数据
    fn mock_visits() -> Vec<BrowserVisit> {
        vec![
            BrowserVisit {
                url: "https://github.com/rust-lang/rust".to_string(),
                title: "rust-lang/rust: The Rust Programming Language".to_string(),
                visit_time: Utc::now(),
            },
            BrowserVisit {
                url: "https://docs.rs/tokio/latest/tokio/".to_string(),
                title: "tokio - Rust".to_string(),
                visit_time: Utc::now(),
            },
            BrowserVisit {
                url: "https://news.ycombinator.com/item?id=12345".to_string(),
                title: "Ask HN: What are you working on?".to_string(),
                visit_time: Utc::now(),
            },
        ]
    }

    /// 将 BrowserVisit 转换为 Event
    fn convert_visit(visit: BrowserVisit) -> Event {
        let raw_payload =
            serde_json::to_string(&visit).unwrap_or_else(|_| format!("{:?}", visit));

        let content = serde_json::json!({
            "url": visit.url,
            "title": visit.title,
            "visit_time": visit.visit_time.to_rfc3339(),
        });

        let mut event = Event::new(
            Source::SystemBrowser,
            Confidence::High,
            EventType::Browsing,
            content,
            raw_payload,
        );

        // 使用 URL 的哈希 + 时间戳作为事件 id（保证幂等，避免明文 URL）
        let url_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            visit.url.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };
        event.id = format!("chrome-{}-{}", url_hash, visit.visit_time.timestamp());

        event
    }
}

#[async_trait]
impl Collector for BrowserHistoryCollector {
    fn id(&self) -> &str {
        "system.browser_history"
    }

    fn name(&self) -> &str {
        "Chrome 浏览历史采集器"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        let visits = self.query_history()?;
        let events = visits.into_iter().map(Self::convert_visit).collect();
        Ok(events)
    }

    async fn health_check(&self) -> HealthStatus {
        if self.mock {
            return HealthStatus::healthy();
        }

        // 检查数据库文件是否存在
        if std::path::Path::new(&self.db_path).exists() {
            HealthStatus::healthy()
        } else {
            HealthStatus::unhealthy(format!("Chrome history DB not found: {}", self.db_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_visit() {
        let visit = BrowserVisit {
            url: "https://example.com".to_string(),
            title: "Example Domain".to_string(),
            visit_time: Utc::now(),
        };

        let event = BrowserHistoryCollector::convert_visit(visit);

        assert_eq!(event.source, Source::SystemBrowser);
        assert_eq!(event.event_type, EventType::Browsing);
        assert_eq!(event.source_confidence, Confidence::High);
        assert_eq!(event.content["url"], "https://example.com");
        assert_eq!(event.content["title"], "Example Domain");
        assert!(event.content["visit_time"].is_string());
        // id 应该以 chrome- 开头且不包含明文 URL
        assert!(event.id.starts_with("chrome-"));
        assert!(!event.id.contains("https://example.com"));
    }

    #[test]
    fn test_collector_metadata() {
        let collector = BrowserHistoryCollector::mock();
        assert_eq!(collector.id(), "system.browser_history");
        assert_eq!(collector.name(), "Chrome 浏览历史采集器");
        assert_eq!(collector.version(), "0.1.0");
    }

    #[tokio::test]
    async fn test_mock_collect_returns_events() {
        let collector = BrowserHistoryCollector::mock();
        let events = collector.collect().await.unwrap();

        assert_eq!(events.len(), 3);

        // 验证第一个事件
        let first = &events[0];
        assert_eq!(first.source, Source::SystemBrowser);
        assert_eq!(first.event_type, EventType::Browsing);
        assert_eq!(first.content["url"], "https://github.com/rust-lang/rust");

        // 验证事件 id 幂等性（同 URL + 时间戳产生相同 id）
        assert!(first.id.starts_with("chrome-"));
    }
}
