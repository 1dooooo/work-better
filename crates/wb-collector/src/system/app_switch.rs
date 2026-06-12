//! 前台应用切换采集器

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wb_core::error::{Result, WbError};
use wb_core::event::{Confidence, Event, EventType, Source};

use crate::runner;
use crate::traits::{Collector, HealthStatus};

/// osascript 输出的应用信息
#[derive(Debug, Serialize, Deserialize)]
struct AppInfo {
    name: String,
    bundle_id: String,
}

/// 前台应用切换采集器
///
/// 通过 macOS `osascript` 获取当前前台应用的名称和 Bundle ID。
/// 由于 macOS 安全权限限制，首次运行可能需要用户授权辅助功能权限。
pub struct AppSwitchCollector {
    /// Mock 模式：跳过真实系统调用，返回测试数据
    mock: bool,
}

impl Default for AppSwitchCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl AppSwitchCollector {
    /// 创建真实采集器
    pub fn new() -> Self {
        Self { mock: false }
    }

    /// 创建 Mock 采集器（用于测试）
    pub fn mock() -> Self {
        Self { mock: true }
    }

    /// 获取当前前台应用信息
    fn get_frontmost_app(&self) -> Result<AppInfo> {
        if self.mock {
            return Ok(AppInfo {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
            });
        }

        let script = r#"tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appName to name of frontApp
            set appBundle to bundle identifier of frontApp
            return appName & "|||" & appBundle
        end tell"#;

        let output = runner::execute("osascript", &["-e", script])?;
        let trimmed = output.trim();

        let parts: Vec<&str> = trimmed.split("|||").collect();
        if parts.len() < 2 {
            return Err(WbError::Collector(format!(
                "Unexpected osascript output format: {}",
                trimmed
            )));
        }

        Ok(AppInfo {
            name: parts[0].to_string(),
            bundle_id: parts[1].to_string(),
        })
    }

    /// 将 AppInfo 转换为 Event
    fn convert_app_info(info: AppInfo) -> Event {
        let raw_payload = serde_json::to_string(&info).unwrap_or_else(|_| format!("{:?}", info));

        let content = serde_json::json!({
            "app_name": info.name,
            "bundle_id": info.bundle_id,
        });

        Event::new(
            Source::SystemAppSwitch,
            Confidence::High,
            EventType::AppActivity,
            content,
            raw_payload,
        )
    }
}

#[async_trait]
impl Collector for AppSwitchCollector {
    fn id(&self) -> &str {
        "system.app_switch"
    }

    fn name(&self) -> &str {
        "前台应用"
    }

    fn group_id(&self) -> &str {
        "system"
    }

    fn group_name(&self) -> &str {
        "系统"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        let info = self.get_frontmost_app()?;
        let event = Self::convert_app_info(info);
        Ok(vec![event])
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_app_info() {
        let info = AppInfo {
            name: "Safari".to_string(),
            bundle_id: "com.apple.Safari".to_string(),
        };

        let event = AppSwitchCollector::convert_app_info(info);

        assert_eq!(event.source, Source::SystemAppSwitch);
        assert_eq!(event.event_type, EventType::AppActivity);
        assert_eq!(event.source_confidence, Confidence::High);
        assert_eq!(event.content["app_name"], "Safari");
        assert_eq!(event.content["bundle_id"], "com.apple.Safari");
    }

    #[test]
    fn test_collector_metadata() {
        let collector = AppSwitchCollector::new();
        assert_eq!(collector.id(), "system.app_switch");
        assert_eq!(collector.name(), "前台应用");
        assert_eq!(collector.group_id(), "system");
        assert_eq!(collector.group_name(), "系统");
        assert_eq!(collector.version(), "0.1.0");
    }

    #[tokio::test]
    async fn test_mock_collect_returns_event() {
        let collector = AppSwitchCollector::mock();
        let events = collector.collect().await.unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.source, Source::SystemAppSwitch);
        assert_eq!(event.event_type, EventType::AppActivity);
        assert_eq!(event.content["app_name"], "Safari");
        assert_eq!(event.content["bundle_id"], "com.apple.Safari");
    }
}
