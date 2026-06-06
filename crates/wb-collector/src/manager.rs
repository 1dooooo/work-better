//! CollectorManager -- 采集器热插拔管理器

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use wb_core::error::Result;
use wb_core::event::Event;

use crate::traits::{Collector, HealthStatus};

/// 采集器管理器，支持运行时注册、启停、健康检查和批量采集
pub struct CollectorManager {
    collectors: RwLock<HashMap<String, Arc<dyn Collector>>>,
    enabled: RwLock<HashMap<String, bool>>,
}

impl CollectorManager {
    pub fn new() -> Self {
        Self {
            collectors: RwLock::new(HashMap::new()),
            enabled: RwLock::new(HashMap::new()),
        }
    }

    /// 注册采集器，默认启用
    pub async fn register(&self, collector: Arc<dyn Collector>) {
        let id = collector.id().to_string();
        self.collectors
            .write()
            .await
            .insert(id.clone(), collector);
        self.enabled.write().await.insert(id, true);
    }

    /// 注销采集器
    pub async fn unregister(&self, id: &str) {
        self.collectors.write().await.remove(id);
        self.enabled.write().await.remove(id);
    }

    /// 启用采集器
    pub async fn enable(&self, id: &str) {
        if let Some(entry) = self.enabled.write().await.get_mut(id) {
            *entry = true;
        }
    }

    /// 禁用采集器
    pub async fn disable(&self, id: &str) {
        if let Some(entry) = self.enabled.write().await.get_mut(id) {
            *entry = false;
        }
    }

    /// 查询采集器是否启用
    pub async fn is_enabled(&self, id: &str) -> bool {
        self.enabled.read().await.get(id).copied().unwrap_or(false)
    }

    /// 单个采集器健康检查
    pub async fn health_check(&self, id: &str) -> Option<HealthStatus> {
        let collectors = self.collectors.read().await;
        let collector = collectors.get(id)?;
        Some(collector.health_check().await)
    }

    /// 所有采集器健康检查
    pub async fn health_check_all(&self) -> Vec<(String, HealthStatus)> {
        let collectors = self.collectors.read().await;
        let mut results = Vec::with_capacity(collectors.len());
        for (id, collector) in collectors.iter() {
            let status = collector.health_check().await;
            results.push((id.clone(), status));
        }
        results
    }

    /// 从所有已启用的采集器采集数据
    pub async fn collect_all(&self) -> Vec<Result<Vec<Event>>> {
        let collectors = self.collectors.read().await;
        let enabled = self.enabled.read().await;

        let mut results = Vec::new();
        for (id, collector) in collectors.iter() {
            if enabled.get(id).copied().unwrap_or(false) {
                results.push(collector.collect().await);
            }
        }
        results
    }

    /// 从指定 ID 的采集器采集数据
    ///
    /// 返回 `Some(Ok(events))` 表示采集成功，`Some(Err(e))` 表示采集失败，
    /// `None` 表示采集器未注册。
    pub async fn collect_one(&self, id: &str) -> Option<Result<Vec<Event>>> {
        let collectors = self.collectors.read().await;
        let enabled = self.enabled.read().await;

        let collector = collectors.get(id)?;
        if !enabled.get(id).copied().unwrap_or(false) {
            return Some(Err(wb_core::error::WbError::Collector(format!(
                "Collector '{}' is disabled",
                id
            ))));
        }
        Some(collector.collect().await)
    }

    /// 列出所有已注册的采集器 ID
    pub async fn list(&self) -> Vec<String> {
        self.collectors.read().await.keys().cloned().collect()
    }
}

impl Default for CollectorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{HealthLevel, HealthStatus};

    /// 测试用 Mock 采集器
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

    #[async_trait::async_trait]
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
                wb_core::event::Source::FeishuMessage,
                wb_core::event::Confidence::High,
                wb_core::event::EventType::Message,
                serde_json::json!({"text": "mock"}),
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

    #[tokio::test]
    async fn test_register_and_list() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("feishu-msg", "Feishu Messages"));

        manager.register(collector).await;

        let list = manager.list().await;
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"feishu-msg".to_string()));
    }

    #[tokio::test]
    async fn test_unregister() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("feishu-msg", "Feishu Messages"));

        manager.register(collector).await;
        assert_eq!(manager.list().await.len(), 1);

        manager.unregister("feishu-msg").await;
        assert_eq!(manager.list().await.len(), 0);
    }

    #[tokio::test]
    async fn test_enable_disable_toggle() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("feishu-msg", "Feishu Messages"));

        manager.register(collector).await;

        // 默认启用
        assert!(manager.is_enabled("feishu-msg").await);

        // 禁用
        manager.disable("feishu-msg").await;
        assert!(!manager.is_enabled("feishu-msg").await);

        // 重新启用
        manager.enable("feishu-msg").await;
        assert!(manager.is_enabled("feishu-msg").await);
    }

    #[tokio::test]
    async fn test_is_enabled_unknown_returns_false() {
        let manager = CollectorManager::new();
        assert!(!manager.is_enabled("nonexistent").await);
    }

    #[tokio::test]
    async fn test_collect_all_only_from_enabled() {
        let manager = CollectorManager::new();

        let c1 = Arc::new(MockCollector::new("a", "Collector A"));
        let c2 = Arc::new(MockCollector::new("b", "Collector B"));

        manager.register(c1).await;
        manager.register(c2).await;

        // 禁用 b
        manager.disable("b").await;

        let results = manager.collect_all().await;
        assert_eq!(results.len(), 1, "should only collect from enabled collectors");

        let events = results.into_iter().next().unwrap().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_collect_all_returns_error_for_failing_collector() {
        let manager = CollectorManager::new();

        let c = Arc::new(MockCollector::with_failure("fail", "Failing Collector"));
        manager.register(c).await;

        let results = manager.collect_all().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[tokio::test]
    async fn test_collect_one_success() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("test", "Test Collector"));

        manager.register(collector).await;

        let result = manager.collect_one("test").await;
        assert!(result.is_some());
        let events = result.unwrap().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_collect_one_not_registered() {
        let manager = CollectorManager::new();

        let result = manager.collect_one("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_collect_one_disabled() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("test", "Test Collector"));

        manager.register(collector).await;
        manager.disable("test").await;

        let result = manager.collect_one("test").await;
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_health_check_single() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("feishu-msg", "Feishu Messages"));

        manager.register(collector).await;

        let status = manager.health_check("feishu-msg").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().level, HealthLevel::Healthy);

        // 不存在的采集器
        assert!(manager.health_check("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let manager = CollectorManager::new();

        let c1 = Arc::new(MockCollector::new("ok", "OK Collector"));
        let c2 = Arc::new(MockCollector::with_failure("bad", "Bad Collector"));

        manager.register(c1).await;
        manager.register(c2).await;

        let results = manager.health_check_all().await;
        assert_eq!(results.len(), 2);

        let statuses: HashMap<String, HealthLevel> = results
            .into_iter()
            .map(|(id, s)| (id, s.level))
            .collect();

        assert_eq!(statuses.get("ok"), Some(&HealthLevel::Healthy));
        assert_eq!(statuses.get("bad"), Some(&HealthLevel::Unhealthy));
    }
}
