//! Collector trait -- 采集器的统一抽象

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use wb_core::error::Result;
use wb_core::event::Event;

/// 采集器健康级别
#[derive(Debug, Clone, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 采集器健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub level: HealthLevel,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
    pub last_success: Option<DateTime<Utc>>,
    pub error_count: u32,
}

impl HealthStatus {
    /// 创建健康状态
    pub fn healthy() -> Self {
        Self {
            level: HealthLevel::Healthy,
            message: None,
            last_check: Utc::now(),
            last_success: Some(Utc::now()),
            error_count: 0,
        }
    }

    /// 创建不健康状态
    pub fn unhealthy(msg: String) -> Self {
        Self {
            level: HealthLevel::Unhealthy,
            message: Some(msg),
            last_check: Utc::now(),
            last_success: None,
            error_count: 1,
        }
    }
}

/// 采集器统一 trait
///
/// 所有采集器实现此 trait，支持热插拔注册到 CollectorManager。
#[async_trait]
pub trait Collector: Send + Sync {
    /// 采集器唯一标识
    fn id(&self) -> &str;

    /// 采集器可读名称
    fn name(&self) -> &str;

    /// 采集器版本
    fn version(&self) -> &str;

    /// 执行采集，返回事件列表
    async fn collect(&self) -> Result<Vec<Event>>;

    /// 健康检查
    async fn health_check(&self) -> HealthStatus;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::healthy();
        assert_eq!(status.level, HealthLevel::Healthy);
        assert!(status.message.is_none());
        assert!(status.last_success.is_some());
        assert_eq!(status.error_count, 0);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::unhealthy("connection timeout".to_string());
        assert_eq!(status.level, HealthLevel::Unhealthy);
        assert_eq!(status.message.as_deref(), Some("connection timeout"));
        assert!(status.last_success.is_none());
        assert_eq!(status.error_count, 1);
    }
}
