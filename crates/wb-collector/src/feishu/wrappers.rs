//! 飞书子采集器的 Collector trait 包装
//!
//! 为没有实现 Collector trait 的飞书子模块创建包装结构体。
//! 已实现 trait 的模块（bitable, meetings, emails, minutes, okr, wiki, spreadsheets）
//! 可直接注册，无需包装。

use async_trait::async_trait;
use wb_core::error::Result;
use wb_core::event::Event;

use crate::traits::{Collector, HealthStatus};

/// lark-cli 工具路径
const LARK_CLI: &str = "/opt/homebrew/bin/lark-cli";

/// 检查 lark-cli 是否可用（不阻塞，仅检查文件是否存在）
fn is_lark_cli_available() -> bool {
    std::path::Path::new(LARK_CLI).exists()
}

/// 飞书文档采集器
pub struct FeishuDocsCollectorWrapper {
    limit: u32,
}

impl FeishuDocsCollectorWrapper {
    pub fn new(limit: u32) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Collector for FeishuDocsCollectorWrapper {
    fn id(&self) -> &str {
        "feishu.docs"
    }

    fn name(&self) -> &str {
        "文档"
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
        if !is_lark_cli_available() {
            return Ok(Vec::new());
        }
        super::docs::FeishuDocsCollector::collect(self.limit)
    }

    async fn health_check(&self) -> HealthStatus {
        if is_lark_cli_available() {
            HealthStatus::healthy()
        } else {
            HealthStatus::degraded(format!("lark-cli 未安装: {}", LARK_CLI))
        }
    }
}

/// 飞书项目/任务采集器
pub struct FeishuProjectsCollectorWrapper {
    limit: u32,
}

impl FeishuProjectsCollectorWrapper {
    pub fn new(limit: u32) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Collector for FeishuProjectsCollectorWrapper {
    fn id(&self) -> &str {
        "feishu.projects"
    }

    fn name(&self) -> &str {
        "项目"
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
        if !is_lark_cli_available() {
            return Ok(Vec::new());
        }
        super::projects::FeishuProjectsCollector::collect(self.limit)
    }

    async fn health_check(&self) -> HealthStatus {
        if is_lark_cli_available() {
            HealthStatus::healthy()
        } else {
            HealthStatus::degraded(format!("lark-cli 未安装: {}", LARK_CLI))
        }
    }
}

/// 飞书日历采集器
pub struct FeishuCalendarCollectorWrapper {
    limit: u32,
}

impl FeishuCalendarCollectorWrapper {
    pub fn new(limit: u32) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Collector for FeishuCalendarCollectorWrapper {
    fn id(&self) -> &str {
        "feishu.calendar"
    }

    fn name(&self) -> &str {
        "日历"
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
        if !is_lark_cli_available() {
            return Ok(Vec::new());
        }
        super::calendar::FeishuCalendarCollector::collect(self.limit)
    }

    async fn health_check(&self) -> HealthStatus {
        if is_lark_cli_available() {
            HealthStatus::healthy()
        } else {
            HealthStatus::degraded(format!("lark-cli 未安装: {}", LARK_CLI))
        }
    }
}

/// 飞书审批采集器
pub struct FeishuApprovalsCollectorWrapper {
    limit: u32,
}

impl FeishuApprovalsCollectorWrapper {
    pub fn new(limit: u32) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Collector for FeishuApprovalsCollectorWrapper {
    fn id(&self) -> &str {
        "feishu.approvals"
    }

    fn name(&self) -> &str {
        "审批"
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
        if !is_lark_cli_available() {
            return Ok(Vec::new());
        }
        super::approvals::FeishuApprovalsCollector::collect(self.limit)
    }

    async fn health_check(&self) -> HealthStatus {
        if is_lark_cli_available() {
            HealthStatus::healthy()
        } else {
            HealthStatus::degraded(format!("lark-cli 未安装: {}", LARK_CLI))
        }
    }
}
