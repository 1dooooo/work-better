//! Confirm —— 报告确认流程

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 报告状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    /// 草稿（自动生成后默认状态）
    Draft,
    /// 待确认（用户已查看，等待确认）
    PendingConfirm,
    /// 已确认（用户确认无误）
    Confirmed,
}

/// 确认流程管理器
///
/// 管理报告的状态流转：Draft -> PendingConfirm -> Confirmed。
#[derive(Debug, Clone)]
pub struct ConfirmFlow {
    status: ReportStatus,
    confirmed_at: Option<DateTime<Utc>>,
    confirmed_by: Option<String>,
}

impl ConfirmFlow {
    /// 创建新的确认流程，默认 Draft 状态
    pub fn new() -> Self {
        Self {
            status: ReportStatus::Draft,
            confirmed_at: None,
            confirmed_by: None,
        }
    }

    /// 从指定状态创建
    pub fn with_status(status: ReportStatus) -> Self {
        Self {
            status,
            confirmed_at: None,
            confirmed_by: None,
        }
    }

    /// 获取当前状态
    pub fn status(&self) -> ReportStatus {
        self.status
    }

    /// 获取确认时间
    pub fn confirmed_at(&self) -> Option<DateTime<Utc>> {
        self.confirmed_at
    }

    /// 获取确认人
    pub fn confirmed_by(&self) -> Option<&str> {
        self.confirmed_by.as_deref()
    }

    /// 提交确认（Draft / PendingConfirm -> PendingConfirm）
    ///
    /// 草稿或待确认状态可以提交为待确认。
    pub fn submit(&mut self) -> bool {
        match self.status {
            ReportStatus::Draft | ReportStatus::PendingConfirm => {
                self.status = ReportStatus::PendingConfirm;
                true
            }
            ReportStatus::Confirmed => false,
        }
    }

    /// 确认报告（PendingConfirm -> Confirmed）
    ///
    /// 仅待确认状态可以确认。
    pub fn confirm(&mut self, by: &str) -> bool {
        match self.status {
            ReportStatus::PendingConfirm => {
                self.status = ReportStatus::Confirmed;
                self.confirmed_at = Some(Utc::now());
                self.confirmed_by = Some(by.to_string());
                true
            }
            _ => false,
        }
    }

    /// 回退到草稿（PendingConfirm -> Draft）
    pub fn revert_to_draft(&mut self) -> bool {
        match self.status {
            ReportStatus::PendingConfirm => {
                self.status = ReportStatus::Draft;
                self.confirmed_at = None;
                self.confirmed_by = None;
                true
            }
            _ => false,
        }
    }

    /// 是否已确认
    pub fn is_confirmed(&self) -> bool {
        self.status == ReportStatus::Confirmed
    }

    /// 是否为草稿
    pub fn is_draft(&self) -> bool {
        self.status == ReportStatus::Draft
    }
}

impl Default for ConfirmFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_starts_as_draft() {
        let flow = ConfirmFlow::new();
        assert_eq!(flow.status(), ReportStatus::Draft);
        assert!(flow.is_draft());
        assert!(!flow.is_confirmed());
    }

    #[test]
    fn test_with_status() {
        let flow = ConfirmFlow::with_status(ReportStatus::PendingConfirm);
        assert_eq!(flow.status(), ReportStatus::PendingConfirm);
    }

    #[test]
    fn test_submit_from_draft() {
        let mut flow = ConfirmFlow::new();
        assert!(flow.submit());
        assert_eq!(flow.status(), ReportStatus::PendingConfirm);
    }

    #[test]
    fn test_submit_from_pending_confirm() {
        let mut flow = ConfirmFlow::with_status(ReportStatus::PendingConfirm);
        assert!(flow.submit());
        assert_eq!(flow.status(), ReportStatus::PendingConfirm);
    }

    #[test]
    fn test_submit_from_confirmed_fails() {
        let mut flow = ConfirmFlow::with_status(ReportStatus::Confirmed);
        assert!(!flow.submit());
        assert_eq!(flow.status(), ReportStatus::Confirmed);
    }

    #[test]
    fn test_confirm_from_pending() {
        let mut flow = ConfirmFlow::new();
        flow.submit();
        assert!(flow.confirm("user-1"));
        assert!(flow.is_confirmed());
        assert_eq!(flow.confirmed_by(), Some("user-1"));
        assert!(flow.confirmed_at().is_some());
    }

    #[test]
    fn test_confirm_from_draft_fails() {
        let mut flow = ConfirmFlow::new();
        assert!(!flow.confirm("user-1"));
        assert!(!flow.is_confirmed());
    }

    #[test]
    fn test_confirm_from_confirmed_fails() {
        let mut flow = ConfirmFlow::with_status(ReportStatus::Confirmed);
        assert!(!flow.confirm("user-2"));
    }

    #[test]
    fn test_revert_to_draft_from_pending() {
        let mut flow = ConfirmFlow::new();
        flow.submit();
        assert!(flow.revert_to_draft());
        assert!(flow.is_draft());
    }

    #[test]
    fn test_revert_to_draft_from_draft_fails() {
        let mut flow = ConfirmFlow::new();
        assert!(!flow.revert_to_draft());
    }

    #[test]
    fn test_revert_to_draft_from_confirmed_fails() {
        let mut flow = ConfirmFlow::with_status(ReportStatus::Confirmed);
        assert!(!flow.revert_to_draft());
    }

    #[test]
    fn test_full_lifecycle() {
        let mut flow = ConfirmFlow::new();
        assert_eq!(flow.status(), ReportStatus::Draft);

        flow.submit();
        assert_eq!(flow.status(), ReportStatus::PendingConfirm);

        flow.revert_to_draft();
        assert_eq!(flow.status(), ReportStatus::Draft);

        flow.submit();
        flow.confirm("admin");
        assert!(flow.is_confirmed());
        assert!(flow.confirmed_at().is_some());
        assert_eq!(flow.confirmed_by(), Some("admin"));
    }

    #[test]
    fn test_default_is_draft() {
        let flow = ConfirmFlow::default();
        assert!(flow.is_draft());
    }
}
