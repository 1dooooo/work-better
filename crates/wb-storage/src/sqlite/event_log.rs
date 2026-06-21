//! SqliteEventLog 实现

use std::sync::Mutex;

use rusqlite::{params, Connection};
use wb_core::error::Result;
use wb_core::event::{Event, EventFilter, EventLog};

use super::schema;

pub struct SqliteEventLog {
    conn: Mutex<Connection>,
}

impl SqliteEventLog {
    /// 创建新的 SqliteEventLog（使用内存数据库）
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            wb_core::error::WbError::Storage(format!("Failed to open memory DB: {}", e))
        })?;
        schema::initialize_schema(&conn).map_err(|e| {
            wb_core::error::WbError::Storage(format!("Failed to initialize schema: {}", e))
        })?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 创建新的 SqliteEventLog（使用文件数据库）
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| {
            wb_core::error::WbError::Storage(format!("Failed to open DB at {}: {}", path, e))
        })?;
        schema::initialize_schema(&conn).map_err(|e| {
            wb_core::error::WbError::Storage(format!("Failed to initialize schema: {}", e))
        })?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default()
}

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
    Ok(Event {
        id: row.get(0)?,
        timestamp: parse_datetime(&row.get::<_, String>(1)?),
        collected_at: parse_datetime(&row.get::<_, String>(2)?),
        source: serde_json::from_str(&row.get::<_, String>(3)?)
            .unwrap_or(wb_core::event::Source::UserCapture),
        source_confidence: serde_json::from_str(&row.get::<_, String>(4)?)
            .unwrap_or(wb_core::event::Confidence::Low),
        event_type: serde_json::from_str(&row.get::<_, String>(5)?)
            .unwrap_or(wb_core::event::EventType::ManualNote),
        content: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or(serde_json::Value::Null),
        raw_payload: row.get(7)?,
        tags: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
        related_ids: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
        attachments: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
        processed: row.get::<_, i32>(11)? != 0,
    })
}

const SELECT_COLUMNS: &str =
    "id, timestamp, collected_at, source, source_confidence, event_type, content, raw_payload, tags, related_ids, attachments, processed";

#[async_trait::async_trait]
impl EventLog for SqliteEventLog {
    async fn append(&self, event: &Event) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| wb_core::error::WbError::Storage(format!("Lock poisoned: {}", e)))?;

        let content = serde_json::to_string(&event.content)?;
        let tags = serde_json::to_string(&event.tags)?;
        let related_ids = serde_json::to_string(&event.related_ids)?;
        let attachments = serde_json::to_string(&event.attachments)?;

        conn.execute(
            "INSERT OR IGNORE INTO events (id, timestamp, collected_at, source, source_confidence, event_type, content, raw_payload, tags, related_ids, attachments, processed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event.id,
                event.timestamp.to_rfc3339(),
                event.collected_at.to_rfc3339(),
                serde_json::to_string(&event.source)?,
                serde_json::to_string(&event.source_confidence)?,
                serde_json::to_string(&event.event_type)?,
                content,
                event.raw_payload,
                tags,
                related_ids,
                attachments,
                event.processed as i32,
            ],
        )
        .map_err(|e| wb_core::error::WbError::Storage(format!("Failed to append event: {}", e)))?;

        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Event>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| wb_core::error::WbError::Storage(format!("Lock poisoned: {}", e)))?;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM events WHERE id = ?1",
                SELECT_COLUMNS
            ))
            .map_err(|e| wb_core::error::WbError::Storage(format!("Prepare failed: {}", e)))?;

        let result = stmt.query_row(params![id], row_to_event);

        match result {
            Ok(event) => Ok(Some(event)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(wb_core::error::WbError::Storage(format!(
                "Query failed: {}",
                e
            ))),
        }
    }

    async fn query(&self, filter: &EventFilter) -> Result<Vec<Event>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| wb_core::error::WbError::Storage(format!("Lock poisoned: {}", e)))?;

        let mut sql = format!("SELECT {} FROM events WHERE 1=1", SELECT_COLUMNS);
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref source) = filter.source {
            sql.push_str(&format!(" AND source = ?{}", param_idx));
            param_values.push(Box::new(serde_json::to_string(source).unwrap_or_default()));
            param_idx += 1;
        }

        if let Some(ref event_type) = filter.event_type {
            sql.push_str(&format!(" AND event_type = ?{}", param_idx));
            param_values.push(Box::new(
                serde_json::to_string(event_type).unwrap_or_default(),
            ));
            param_idx += 1;
        }

        if let Some(processed) = filter.processed {
            sql.push_str(&format!(" AND processed = ?{}", param_idx));
            param_values.push(Box::new(if processed { 1 } else { 0 }));
            param_idx += 1;
        }

        if let Some(ref since) = filter.since {
            sql.push_str(&format!(" AND timestamp >= ?{}", param_idx));
            param_values.push(Box::new(since.to_rfc3339()));
            param_idx += 1;
        }

        if let Some(ref until) = filter.until {
            sql.push_str(&format!(" AND timestamp <= ?{}", param_idx));
            param_values.push(Box::new(until.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| wb_core::error::WbError::Storage(format!("Prepare failed: {}", e)))?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let events = stmt
            .query_map(param_refs.as_slice(), row_to_event)
            .map_err(|e| wb_core::error::WbError::Storage(format!("Query failed: {}", e)))?;

        let mut result = Vec::new();
        for event in events {
            result.push(event.map_err(|e| {
                wb_core::error::WbError::Storage(format!("Row mapping failed: {}", e))
            })?);
        }

        Ok(result)
    }

    async fn mark_processed(&self, id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| wb_core::error::WbError::Storage(format!("Lock poisoned: {}", e)))?;

        let updated = conn
            .execute("UPDATE events SET processed = 1 WHERE id = ?1", params![id])
            .map_err(|e| wb_core::error::WbError::Storage(format!("Update failed: {}", e)))?;

        if updated == 0 {
            return Err(wb_core::error::WbError::NotFound(format!(
                "Event not found: {}",
                id
            )));
        }

        Ok(())
    }

    async fn get_unprocessed(&self, limit: Option<usize>) -> Result<Vec<Event>> {
        let filter = EventFilter {
            processed: Some(false),
            limit,
            ..Default::default()
        };
        self.query(&filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wb_core::event::{Confidence, Event, EventType, Source};

    fn make_test_event(source: Source, event_type: EventType) -> Event {
        Event::new(
            source,
            Confidence::High,
            event_type,
            json!({"text": "test content"}),
            r#"{"raw": "data"}"#.to_string(),
        )
    }

    #[tokio::test]
    async fn test_append_and_get() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let event = make_test_event(Source::FeishuMessage, EventType::Message);

        log.append(&event).await.unwrap();
        let retrieved = log.get(&event.id).await.unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.source, Source::FeishuMessage);
        assert_eq!(retrieved.event_type, EventType::Message);
    }

    #[tokio::test]
    async fn test_get_nonexistent_returns_none() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let result = log.get("nonexistent-id").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mark_processed() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let event = make_test_event(Source::FeishuDoc, EventType::DocumentChange);

        log.append(&event).await.unwrap();
        log.mark_processed(&event.id).await.unwrap();

        let unprocessed = log.get_unprocessed(None).await.unwrap();
        assert!(unprocessed.is_empty());
    }

    #[tokio::test]
    async fn test_processed_field_persists() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let event = make_test_event(Source::FeishuMessage, EventType::Message);

        // New event should be unprocessed
        log.append(&event).await.unwrap();
        let fetched = log.get(&event.id).await.unwrap().unwrap();
        assert!(!fetched.processed, "newly appended event should be unprocessed");

        // After marking processed, the field should persist
        log.mark_processed(&event.id).await.unwrap();
        let fetched = log.get(&event.id).await.unwrap().unwrap();
        assert!(fetched.processed, "event should be processed after mark_processed");
    }

    #[tokio::test]
    async fn test_get_unprocessed() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let event1 = make_test_event(Source::FeishuMessage, EventType::Message);
        let event2 = make_test_event(Source::FeishuDoc, EventType::DocumentChange);

        log.append(&event1).await.unwrap();
        log.append(&event2).await.unwrap();

        let unprocessed = log.get_unprocessed(None).await.unwrap();
        assert_eq!(unprocessed.len(), 2);

        log.mark_processed(&event1.id).await.unwrap();
        let unprocessed = log.get_unprocessed(None).await.unwrap();
        assert_eq!(unprocessed.len(), 1);
    }

    #[tokio::test]
    async fn test_query_by_source() {
        let log = SqliteEventLog::new_in_memory().unwrap();
        let event1 = make_test_event(Source::FeishuMessage, EventType::Message);
        let event2 = make_test_event(Source::FeishuDoc, EventType::DocumentChange);

        log.append(&event1).await.unwrap();
        log.append(&event2).await.unwrap();

        let filter = EventFilter {
            source: Some(Source::FeishuMessage),
            ..Default::default()
        };
        let results = log.query(&filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, Source::FeishuMessage);
    }
}
