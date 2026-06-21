//! FeishuCollector -- 实现 Collector trait 的飞书消息采集器
//!
//! 将 FeishuMessageCollector 的静态方法封装为符合 Collector trait 的实例，
//! 支持注册到 CollectorManager。

use async_trait::async_trait;
use wb_core::error::Result;
use wb_core::event::Event;

use crate::runner;
use crate::traits::{Collector, HealthStatus};

use super::messages::FeishuMessageCollector;

/// lark-cli 工具路径
const LARK_CLI: &str = "/opt/homebrew/bin/lark-cli";

/// 飞书消息采集器（Collector trait 实现）
///
/// 持有采集所需的配置参数，注册到 CollectorManager 后可统一管理。
pub struct FeishuCollector {
    chat_id: String,
    limit: u32,
}

impl FeishuCollector {
    /// 创建新的飞书采集器实例
    ///
    /// # Arguments
    /// * `chat_id` - 飞书会话 ID
    /// * `limit` - 每次采集的最大消息数量
    pub fn new(chat_id: String, limit: u32) -> Self {
        Self { chat_id, limit }
    }

    /// 更新会话 ID
    pub fn set_chat_id(&mut self, chat_id: String) {
        self.chat_id = chat_id;
    }

    /// 更新采集数量限制
    pub fn set_limit(&mut self, limit: u32) {
        self.limit = limit;
    }
}

#[async_trait]
impl Collector for FeishuCollector {
    fn id(&self) -> &str {
        "feishu"
    }

    fn name(&self) -> &str {
        "消息"
    }

    fn group_id(&self) -> &str {
        "feishu"
    }

    fn group_name(&self) -> &str {
        "飞书"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn collect(&self) -> Result<Vec<Event>> {
        if !runner::check_tool_available(LARK_CLI) {
            return Ok(Vec::new());
        }
        FeishuMessageCollector::collect(&self.chat_id, self.limit)
    }

    async fn health_check(&self) -> HealthStatus {
        if runner::check_tool_available(LARK_CLI) {
            HealthStatus::healthy()
        } else {
            HealthStatus::degraded(format!("lark-cli 未安装: {}", LARK_CLI))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feishu_collector_id_and_name() {
        let collector = FeishuCollector::new("oc_test".into(), 50);
        assert_eq!(collector.id(), "feishu");
        assert_eq!(collector.name(), "消息");
        assert_eq!(collector.group_id(), "feishu");
        assert_eq!(collector.group_name(), "飞书");
        assert_eq!(collector.version(), "0.1.0");
    }

    #[test]
    fn test_feishu_collector_setters() {
        let mut collector = FeishuCollector::new("oc_old".into(), 10);
        collector.set_chat_id("oc_new".into());
        collector.set_limit(100);
        assert_eq!(collector.chat_id, "oc_new");
        assert_eq!(collector.limit, 100);
    }
}
