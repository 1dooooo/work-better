//! 采集器开关功能真实后端测试
//!
//! 测试场景：调用 enable/disable_collector → 采集器状态正确变化
//!
//! 验证目标：
//! 1. 采集器默认启用
//! 2. 禁用后状态正确更新
//! 3. 重新启用后状态正确恢复
//! 4. 禁用的采集器不参与采集

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use wb_collector::manager::CollectorManager;
use wb_collector::traits::{Collector, HealthLevel, HealthStatus};
use wb_core::error::Result;
use wb_core::event::{Event, Source, Confidence, EventType};

/// 测试用的模拟采集器
struct MockCollector {
    id: String,
    name: String,
    collect_count: AtomicU32,
}

impl MockCollector {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            collect_count: AtomicU32::new(0),
        }
    }

    fn collect_count(&self) -> u32 {
        self.collect_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl Collector for MockCollector {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn group_id(&self) -> &str {
        "test"
    }

    fn group_name(&self) -> &str {
        "测试"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        self.collect_count.fetch_add(1, Ordering::SeqCst);
        Ok(vec![Event::new(
            Source::FeishuMessage,
            Confidence::High,
            EventType::Message,
            serde_json::json!({"text": "mock event"}),
            "raw".to_string(),
        )])
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::healthy()
    }
}

/// 测试采集器默认启用
///
/// 场景：注册采集器后，检查默认状态
/// 预期：采集器默认启用
#[tokio::test]
async fn test_collector_default_enabled() {
    let manager = CollectorManager::new();
    let collector = Arc::new(MockCollector::new("test-collector", "Test Collector"));

    manager.register(collector).await;

    // 验证：默认启用
    assert!(
        manager.is_enabled("test-collector").await,
        "采集器应该默认启用"
    );
}

/// 测试禁用采集器
///
/// 场景：禁用采集器后，检查状态
/// 预期：采集器状态为禁用
#[tokio::test]
async fn test_collector_disable() {
    let manager = CollectorManager::new();
    let collector = Arc::new(MockCollector::new("test-collector", "Test Collector"));

    manager.register(collector).await;

    // 禁用采集器
    manager.disable("test-collector").await;

    // 验证：状态为禁用
    assert!(
        !manager.is_enabled("test-collector").await,
        "禁用后采集器应该处于禁用状态"
    );
}

/// 测试重新启用采集器
///
/// 场景：禁用后重新启用，检查状态
/// 预期：采集器状态恢复为启用
#[tokio::test]
async fn test_collector_re_enable() {
    let manager = CollectorManager::new();
    let collector = Arc::new(MockCollector::new("test-collector", "Test Collector"));

    manager.register(collector).await;

    // 禁用后重新启用
    manager.disable("test-collector").await;
    manager.enable("test-collector").await;

    // 验证：状态恢复为启用
    assert!(
        manager.is_enabled("test-collector").await,
        "重新启用后采集器应该处于启用状态"
    );
}

/// 测试禁用的采集器不参与采集
///
/// 场景：禁用采集器后，调用 collect_all
/// 预期：禁用的采集器不被调用
#[tokio::test]
async fn test_disabled_collector_not_collected() {
    let manager = CollectorManager::new();
    let collector = Arc::new(MockCollector::new("test-collector", "Test Collector"));

    manager.register(collector.clone()).await;

    // 禁用采集器
    manager.disable("test-collector").await;

    // 执行采集
    let results = manager.collect_all().await;

    // 验证：没有采集结果
    assert!(
        results.is_empty(),
        "禁用的采集器不应该参与采集"
    );

    // 验证：采集器没有被调用
    assert_eq!(
        collector.collect_count(),
        0,
        "禁用的采集器不应该被调用"
    );
}

/// 测试启用的采集器参与采集
///
/// 场景：启用采集器后，调用 collect_all
/// 预期：启用的采集器被调用
#[tokio::test]
async fn test_enabled_collector_collected() {
    let manager = CollectorManager::new();
    let collector = Arc::new(MockCollector::new("test-collector", "Test Collector"));

    manager.register(collector.clone()).await;

    // 执行采集
    let results = manager.collect_all().await;

    // 验证：有采集结果
    assert_eq!(
        results.len(),
        1,
        "启用的采集器应该参与采集"
    );

    // 验证：采集器被调用
    assert_eq!(
        collector.collect_count(),
        1,
        "启用的采集器应该被调用"
    );
}

/// 测试多个采集器的开关状态
///
/// 场景：注册多个采集器，分别设置不同状态
/// 预期：每个采集器的状态独立
#[tokio::test]
async fn test_multiple_collectors_toggle() {
    let manager = CollectorManager::new();
    let collector1 = Arc::new(MockCollector::new("collector-1", "Collector 1"));
    let collector2 = Arc::new(MockCollector::new("collector-2", "Collector 2"));
    let collector3 = Arc::new(MockCollector::new("collector-3", "Collector 3"));

    manager.register(collector1).await;
    manager.register(collector2).await;
    manager.register(collector3).await;

    // 设置不同状态
    manager.disable("collector-1").await;
    // collector-2 保持启用
    manager.disable("collector-3").await;

    // 验证：状态正确
    assert!(!manager.is_enabled("collector-1").await);
    assert!(manager.is_enabled("collector-2").await);
    assert!(!manager.is_enabled("collector-3").await);

    // 执行采集
    let results = manager.collect_all().await;

    // 验证：只有启用的采集器参与采集
    assert_eq!(
        results.len(),
        1,
        "只有启用的采集器应该参与采集"
    );
}

/// 测试未注册的采集器开关操作
///
/// 场景：对未注册的采集器执行 enable/disable
/// 预期：操作不会报错，状态保持为禁用
#[tokio::test]
async fn test_toggle_unregistered_collector() {
    let manager = CollectorManager::new();

    // 对未注册的采集器执行操作
    manager.enable("nonexistent").await;
    manager.disable("nonexistent").await;

    // 验证：状态为禁用（默认）
    assert!(
        !manager.is_enabled("nonexistent").await,
        "未注册的采集器应该处于禁用状态"
    );
}
