//! B4: Tauri Collectors Management Integration Tests
//!
//! Tests the CollectorManager logic that the Tauri collector commands use.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use wb_collector::manager::CollectorManager;
use wb_collector::traits::{Collector, HealthLevel, HealthStatus};
use wb_core::event::{Confidence, Event, EventType, Source};
use wb_core::error::Result;

// ---------------------------------------------------------------------------
// Mock collector
// ---------------------------------------------------------------------------

struct MockCollector {
    id: String,
    name: String,
    should_fail: bool,
}

impl MockCollector {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            should_fail: false,
        }
    }

    fn with_failure(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            should_fail: true,
        }
    }
}

#[async_trait]
impl Collector for MockCollector {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        if self.should_fail {
            return Err(wb_core::error::WbError::Collector(
                "mock failure".to_string(),
            ));
        }
        Ok(vec![Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            json!({"text": "mock"}),
            "raw".to_string(),
        )])
    }

    async fn health_check(&self) -> HealthStatus {
        if self.should_fail {
            HealthStatus::unhealthy("mock unhealthy".to_string())
        } else {
            HealthStatus::healthy()
        }
    }
}

// ---------------------------------------------------------------------------
// B4-01: List collectors
// ---------------------------------------------------------------------------

/// Mirrors Tauri `list_collectors` command: `manager.list().await`
#[tokio::test]
async fn b4_01_list_collectors_empty() {
    let manager = CollectorManager::new();
    let list = manager.list().await;
    assert!(list.is_empty());
}

#[tokio::test]
async fn b4_01_list_collectors_after_register() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("feishu", "飞书")))
        .await;
    manager
        .register(Arc::new(MockCollector::new("browser", "浏览器")))
        .await;

    let list = manager.list().await;
    assert_eq!(list.len(), 2);
    assert!(list.contains(&"feishu".to_string()));
    assert!(list.contains(&"browser".to_string()));
}

// ---------------------------------------------------------------------------
// B4-02: Enable/disable collectors
// ---------------------------------------------------------------------------

/// Mirrors Tauri `enable_collector`/`disable_collector` commands
#[tokio::test]
async fn b4_02_enable_disable_toggle() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("c1", "Collector 1")))
        .await;

    // Default is enabled
    assert!(manager.is_enabled("c1").await);

    // Disable
    manager.disable("c1").await;
    assert!(!manager.is_enabled("c1").await);

    // Re-enable
    manager.enable("c1").await;
    assert!(manager.is_enabled("c1").await);
}

#[tokio::test]
async fn b4_02_disable_affects_collect() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("c1", "Collector 1")))
        .await;

    manager.disable("c1").await;

    // collect_one on disabled collector should return error
    let result = manager.collect_one("c1").await;
    assert!(result.is_some());
    assert!(result.unwrap().is_err());
}

#[tokio::test]
async fn b4_02_enable_unregistered_noop() {
    let manager = CollectorManager::new();
    // Enabling non-existent collector should not panic
    manager.enable("nonexistent").await;
    assert!(!manager.is_enabled("nonexistent").await);
}

// ---------------------------------------------------------------------------
// B4-03: Health check
// ---------------------------------------------------------------------------

/// Mirrors Tauri `check_collector_health` command
#[tokio::test]
async fn b4_03_health_check_healthy() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("ok", "OK Collector")))
        .await;

    let status = manager.health_check("ok").await;
    assert!(status.is_some());
    assert_eq!(status.unwrap().level, HealthLevel::Healthy);
}

#[tokio::test]
async fn b4_03_health_check_unhealthy() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::with_failure("bad", "Bad Collector")))
        .await;

    let status = manager.health_check("bad").await;
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.level, HealthLevel::Unhealthy);
    assert!(status.message.is_some());
    assert_eq!(status.error_count, 1);
}

#[tokio::test]
async fn b4_03_health_check_nonexistent() {
    let manager = CollectorManager::new();
    assert!(manager.health_check("nonexistent").await.is_none());
}

#[tokio::test]
async fn b4_03_health_check_all() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("ok", "OK")))
        .await;
    manager
        .register(Arc::new(MockCollector::with_failure("bad", "Bad")))
        .await;

    let results = manager.health_check_all().await;
    assert_eq!(results.len(), 2);

    let statuses: std::collections::HashMap<String, HealthLevel> =
        results.into_iter().map(|(id, s)| (id, s.level)).collect();

    assert_eq!(statuses.get("ok"), Some(&HealthLevel::Healthy));
    assert_eq!(statuses.get("bad"), Some(&HealthLevel::Unhealthy));
}

// ---------------------------------------------------------------------------
// B4-04: Collect from enabled collectors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn b4_04_collect_all_from_enabled() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("a", "A")))
        .await;
    manager
        .register(Arc::new(MockCollector::new("b", "B")))
        .await;
    manager.disable("b").await;

    let results = manager.collect_all().await;
    assert_eq!(results.len(), 1, "Should only collect from enabled");
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn b4_04_collect_one_success() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("c1", "C1")))
        .await;

    let result = manager.collect_one("c1").await;
    assert!(result.is_some());
    let events = result.unwrap().unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn b4_04_collect_one_nonexistent() {
    let manager = CollectorManager::new();
    assert!(manager.collect_one("nope").await.is_none());
}

#[tokio::test]
async fn b4_04_unregister_removes_collector() {
    let manager = CollectorManager::new();
    manager
        .register(Arc::new(MockCollector::new("temp", "Temp")))
        .await;
    assert_eq!(manager.list().await.len(), 1);

    manager.unregister("temp").await;
    assert_eq!(manager.list().await.len(), 0);
    assert!(manager.health_check("temp").await.is_none());
}
