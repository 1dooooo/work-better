//! TokenBudget: Token 预算管理与过载策略

use serde::{Deserialize, Serialize};

/// 过载策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OverloadStrategy {
    /// 排队到明天处理
    QueueTomorrow,
    /// 降级到小模型
    DegradeToSmall,
    /// 允许溢出
    AllowOverflow,
}

/// Token 预算管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// 每日 Token 上限
    pub daily_limit: u64,
    /// 今日已用 Token
    pub daily_used: u64,
    /// 过载策略
    pub overload_strategy: OverloadStrategy,
}

impl TokenBudget {
    /// 创建新的 Token 预算管理器
    pub fn new(daily_limit: u64) -> Self {
        Self {
            daily_limit,
            daily_used: 0,
            overload_strategy: OverloadStrategy::DegradeToSmall,
        }
    }

    /// 设置过载策略
    pub fn with_strategy(mut self, strategy: OverloadStrategy) -> Self {
        self.overload_strategy = strategy;
        self
    }

    /// 剩余可用 Token
    pub fn daily_remaining(&self) -> u64 {
        self.daily_limit.saturating_sub(self.daily_used)
    }

    /// 是否已耗尽预算
    pub fn is_exhausted(&self) -> bool {
        self.daily_used >= self.daily_limit
    }

    /// 记录 Token 使用量
    pub fn record_usage(&mut self, tokens: u64) {
        self.daily_used += tokens;
    }

    /// 重置每日用量
    pub fn reset_daily(&mut self) {
        self.daily_used = 0;
    }

    /// 检查是否能承担一次操作
    ///
    /// 考虑过载策略：AllowOverflow 始终允许，其他策略检查剩余预算
    pub fn can_afford(&self, estimated_tokens: u64) -> bool {
        if self.overload_strategy == OverloadStrategy::AllowOverflow {
            return true;
        }
        self.daily_remaining() >= estimated_tokens
    }

    /// 解析当前请求应采用的策略
    ///
    /// - 紧急请求且预算不足：DegradeToSmall（降级而非丢弃）
    /// - 非紧急请求且预算不足：使用配置的过载策略
    /// - 预算充足：正常处理（返回配置的策略，调用方忽略即可）
    pub fn resolve_strategy(&self, estimated_tokens: u64, is_urgent: bool) -> OverloadStrategy {
        if self.daily_remaining() >= estimated_tokens {
            // 预算充足，返回配置的策略（调用方可忽略）
            return self.overload_strategy.clone();
        }

        // 预算不足
        if is_urgent {
            // 紧急请求：降级到小模型，确保能处理
            OverloadStrategy::DegradeToSmall
        } else {
            // 非紧急请求：按配置的策略执行
            self.overload_strategy.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_budget() {
        let budget = TokenBudget::new(1000);
        assert_eq!(budget.daily_limit, 1000);
        assert_eq!(budget.daily_used, 0);
        assert_eq!(budget.overload_strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_with_strategy() {
        let budget = TokenBudget::new(1000).with_strategy(OverloadStrategy::QueueTomorrow);
        assert_eq!(budget.overload_strategy, OverloadStrategy::QueueTomorrow);
    }

    #[test]
    fn test_daily_remaining() {
        let mut budget = TokenBudget::new(1000);
        assert_eq!(budget.daily_remaining(), 1000);

        budget.record_usage(300);
        assert_eq!(budget.daily_remaining(), 700);

        budget.record_usage(700);
        assert_eq!(budget.daily_remaining(), 0);
    }

    #[test]
    fn test_daily_remaining_saturates_at_zero() {
        let mut budget = TokenBudget::new(100);
        budget.record_usage(200);
        // 不应下溢，saturating_sub 返回 0
        assert_eq!(budget.daily_remaining(), 0);
    }

    #[test]
    fn test_is_exhausted() {
        let mut budget = TokenBudget::new(100);
        assert!(!budget.is_exhausted());

        budget.record_usage(50);
        assert!(!budget.is_exhausted());

        budget.record_usage(50);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_is_exhausted_overflow() {
        let mut budget = TokenBudget::new(100);
        budget.record_usage(150);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_record_usage() {
        let mut budget = TokenBudget::new(1000);
        budget.record_usage(100);
        budget.record_usage(200);
        assert_eq!(budget.daily_used, 300);
    }

    #[test]
    fn test_reset_daily() {
        let mut budget = TokenBudget::new(1000);
        budget.record_usage(500);
        assert_eq!(budget.daily_used, 500);

        budget.reset_daily();
        assert_eq!(budget.daily_used, 0);
        assert_eq!(budget.daily_remaining(), 1000);
    }

    #[test]
    fn test_can_afford_when_budget_sufficient() {
        let budget = TokenBudget::new(1000);
        assert!(budget.can_afford(500));
        assert!(budget.can_afford(1000));
    }

    #[test]
    fn test_can_afford_when_budget_insufficient() {
        let mut budget = TokenBudget::new(1000);
        budget.record_usage(800);
        assert!(budget.can_afford(100));
        assert!(!budget.can_afford(300));
    }

    #[test]
    fn test_can_afford_allow_overflow() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);
        // 预算耗尽但策略为 AllowOverflow，仍可承担
        assert!(budget.can_afford(1000));
    }

    #[test]
    fn test_can_afford_queue_tomorrow() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100);
        // 预算耗尽，策略为排队，不可承担
        assert!(!budget.can_afford(50));
    }

    #[test]
    fn test_resolve_strategy_budget_sufficient() {
        let budget = TokenBudget::new(1000);
        // 预算充足，返回配置的策略
        let strategy = budget.resolve_strategy(100, false);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);

        let strategy = budget.resolve_strategy(100, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_resolve_strategy_budget_exhausted_non_urgent() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100);

        // 非紧急 + 预算耗尽：使用配置策略（QueueTomorrow）
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(strategy, OverloadStrategy::QueueTomorrow);
    }

    #[test]
    fn test_resolve_strategy_budget_exhausted_urgent() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100);

        // 紧急 + 预算耗尽：降级到小模型（确保能处理）
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_resolve_strategy_exhausted_urgent_with_allow_overflow() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);

        // 预算耗尽但策略为 AllowOverflow，remaining < estimated
        // 紧急请求：仍降级到小模型
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_resolve_strategy_exhausted_non_urgent_allow_overflow() {
        let mut budget = TokenBudget::new(100)
            .with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);

        // 非紧急 + 预算耗尽 + AllowOverflow：返回 AllowOverflow
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(strategy, OverloadStrategy::AllowOverflow);
    }
}
