//! CollectorManager -- 采集器热插拔管理器

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use wb_core::error::Result;
use wb_core::event::Event;

use crate::traits::{Collector, HealthStatus};

/// 采集器分组信息
#[derive(Debug, Clone)]
pub struct CollectorGroupInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub collector_ids: Vec<String>,
}

/// 采集器管理器，支持运行时注册、启停、健康检查和批量采集
pub struct CollectorManager {
    collectors: RwLock<HashMap<String, Arc<dyn Collector>>>,
    enabled: RwLock<HashMap<String, bool>>,
    /// 分组启用状态（key: group_id）
    group_enabled: RwLock<HashMap<String, bool>>,
}

impl CollectorManager {
    pub fn new() -> Self {
        Self {
            collectors: RwLock::new(HashMap::new()),
            enabled: RwLock::new(HashMap::new()),
            group_enabled: RwLock::new(HashMap::new()),
        }
    }

    /// 注册采集器，默认启用
    pub async fn register(&self, collector: Arc<dyn Collector>) {
        let id = collector.id().to_string();
        let group_id = collector.group_id().to_string();

        // 注册采集器
        self.collectors.write().await.insert(id.clone(), collector);
        self.enabled.write().await.insert(id, true);

        // 确保分组存在且默认启用
        let mut groups = self.group_enabled.write().await;
        groups.entry(group_id).or_insert(true);
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

    /// 启用分组
    pub async fn enable_group(&self, group_id: &str) {
        let mut groups = self.group_enabled.write().await;
        groups.insert(group_id.to_string(), true);
    }

    /// 禁用分组
    pub async fn disable_group(&self, group_id: &str) {
        let mut groups = self.group_enabled.write().await;
        groups.insert(group_id.to_string(), false);
    }

    /// 查询分组是否启用
    pub async fn is_group_enabled(&self, group_id: &str) -> bool {
        self.group_enabled.read().await.get(group_id).copied().unwrap_or(true)
    }

    /// 获取所有分组信息
    pub async fn get_groups(&self) -> Vec<CollectorGroupInfo> {
        let collectors = self.collectors.read().await;
        let enabled = self.enabled.read().await;
        let group_enabled = self.group_enabled.read().await;

        let mut groups: HashMap<String, CollectorGroupInfo> = HashMap::new();

        for (id, collector) in collectors.iter() {
            let group_id = collector.group_id().to_string();
            let group_name = collector.group_name().to_string();

            let group = groups.entry(group_id.clone()).or_insert_with(|| {
                CollectorGroupInfo {
                    id: group_id.clone(),
                    name: group_name,
                    enabled: group_enabled.get(&group_id).copied().unwrap_or(true),
                    collector_ids: Vec::new(),
                }
            });

            group.collector_ids.push(id.clone());
        }

        groups.into_values().collect()
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
        let group_enabled = self.group_enabled.read().await;

        let mut results = Vec::new();
        for (id, collector) in collectors.iter() {
            // 检查分组和采集器是否都启用
            let group_ok = group_enabled
                .get(collector.group_id())
                .copied()
                .unwrap_or(true);
            let collector_ok = enabled.get(id).copied().unwrap_or(false);

            if group_ok && collector_ok {
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
    use rstest::rstest;

    /// Test mock collector
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

    // ── A10-01: Register and list ─────────────────────────────────────
    #[tokio::test]
    async fn a10_01_register_and_list() {
        let manager = CollectorManager::new();
        let c1 = Arc::new(MockCollector::new("feishu-msg", "Feishu Messages"));
        let c2 = Arc::new(MockCollector::new("browser", "Browser History"));

        manager.register(c1).await;
        manager.register(c2).await;

        let list = manager.list().await;
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"feishu-msg".to_string()));
        assert!(list.contains(&"browser".to_string()));
    }

    // ── A10-02: Enable/disable ────────────────────────────────────────
    #[tokio::test]
    async fn a10_02_enable_disable_toggle() {
        let manager = CollectorManager::new();
        let collector = Arc::new(MockCollector::new("c1", "Collector 1"));

        manager.register(collector).await;

        // Default is enabled
        assert!(manager.is_enabled("c1").await);

        // Disable
        manager.disable("c1").await;
        assert!(!manager.is_enabled("c1").await);

        // Re-enable
        manager.enable("c1").await;
        assert!(manager.is_enabled("c1").await);
    }

    // ── A10-03: is_enabled check ──────────────────────────────────────
    #[rstest]
    #[case("registered", true)]
    #[case("unregistered", false)]
    #[tokio::test]
    async fn a10_03_is_enabled_check(#[case] id: &str, #[case] expected: bool) {
        let manager = CollectorManager::new();
        if expected {
            let c = Arc::new(MockCollector::new("registered", "Reg"));
            manager.register(c).await;
        }
        assert_eq!(manager.is_enabled(id).await, expected);
    }

    // ── A10-04: Health check ──────────────────────────────────────────
    #[tokio::test]
    async fn a10_04_health_check_registered() {
        let manager = CollectorManager::new();
        let c = Arc::new(MockCollector::new("healthy", "Healthy Collector"));
        manager.register(c).await;

        let status = manager.health_check("healthy").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().level, HealthLevel::Healthy);
    }

    #[tokio::test]
    async fn a10_04_health_check_unhealthy() {
        let manager = CollectorManager::new();
        let c = Arc::new(MockCollector::with_failure("bad", "Bad Collector"));
        manager.register(c).await;

        let status = manager.health_check("bad").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().level, HealthLevel::Unhealthy);
    }

    // ── A10-05: Unregistered returns None ─────────────────────────────
    #[tokio::test]
    async fn a10_05_unregistered_health_returns_none() {
        let manager = CollectorManager::new();
        assert!(manager.health_check("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn a10_05_unregistered_collect_one_returns_none() {
        let manager = CollectorManager::new();
        assert!(manager.collect_one("nonexistent").await.is_none());
    }

    // ── Additional: unregister removes from list ──────────────────────
    #[tokio::test]
    async fn unregister_removes_from_list() {
        let manager = CollectorManager::new();
        let c = Arc::new(MockCollector::new("temp", "Temp"));
        manager.register(c).await;
        assert_eq!(manager.list().await.len(), 1);

        manager.unregister("temp").await;
        assert_eq!(manager.list().await.len(), 0);
    }

    // ── Additional: collect_all skips disabled ────────────────────────
    #[tokio::test]
    async fn collect_all_skips_disabled() {
        let manager = CollectorManager::new();
        let c1 = Arc::new(MockCollector::new("a", "A"));
        let c2 = Arc::new(MockCollector::new("b", "B"));
        manager.register(c1).await;
        manager.register(c2).await;
        manager.disable("b").await;

        let results = manager.collect_all().await;
        assert_eq!(results.len(), 1);
    }

    // ── Additional: health_check_all ──────────────────────────────────
    #[tokio::test]
    async fn health_check_all_mixed() {
        let manager = CollectorManager::new();
        manager
            .register(Arc::new(MockCollector::new("ok", "OK")))
            .await;
        manager
            .register(Arc::new(MockCollector::with_failure("bad", "Bad")))
            .await;

        let results = manager.health_check_all().await;
        assert_eq!(results.len(), 2);

        let statuses: HashMap<String, HealthLevel> =
            results.into_iter().map(|(id, s)| (id, s.level)).collect();

        assert_eq!(statuses.get("ok"), Some(&HealthLevel::Healthy));
        assert_eq!(statuses.get("bad"), Some(&HealthLevel::Unhealthy));
    }
}
