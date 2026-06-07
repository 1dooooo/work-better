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
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);
        // 预算耗尽但策略为 AllowOverflow，仍可承担
        assert!(budget.can_afford(1000));
    }

    #[test]
    fn test_can_afford_queue_tomorrow() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::QueueTomorrow);
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
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100);

        // 非紧急 + 预算耗尽：使用配置策略（QueueTomorrow）
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(strategy, OverloadStrategy::QueueTomorrow);
    }

    #[test]
    fn test_resolve_strategy_budget_exhausted_urgent() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100);

        // 紧急 + 预算耗尽：降级到小模型（确保能处理）
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_resolve_strategy_exhausted_urgent_with_allow_overflow() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);

        // 预算耗尽但策略为 AllowOverflow，remaining < estimated
        // 紧急请求：仍降级到小模型
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn test_resolve_strategy_exhausted_non_urgent_allow_overflow() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100);

        // 非紧急 + 预算耗尽 + AllowOverflow：返回 AllowOverflow
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(strategy, OverloadStrategy::AllowOverflow);
    }
}

#[cfg(test)]
mod a5_token_budget_tests {
    //! A5: Token Budget — rstest parameterized scenarios
    use super::*;
    use rstest::rstest;

    // ── A5-01: Budget initialization ───────────────────────────────────

    #[test]
    fn a5_01_budget_init() {
        let budget = TokenBudget::new(1000);
        assert_eq!(budget.daily_limit, 1000);
        assert_eq!(budget.daily_used, 0);
        assert_eq!(budget.overload_strategy, OverloadStrategy::DegradeToSmall);
    }

    // ── A5-02: Record usage accumulates ────────────────────────────────

    #[test]
    fn a5_02_record_usage() {
        let mut budget = TokenBudget::new(1000);
        budget.record_usage(100);
        budget.record_usage(200);
        assert_eq!(budget.daily_used, 300);
    }

    // ── A5-03: Daily remaining ─────────────────────────────────────────

    #[test]
    fn a5_03_daily_remaining() {
        let mut budget = TokenBudget::new(1000);
        assert_eq!(budget.daily_remaining(), 1000);
        budget.record_usage(300);
        assert_eq!(budget.daily_remaining(), 700);
    }

    // ── A5-04: Saturating subtraction (remaining never goes negative) ──

    #[test]
    fn a5_04_remaining_saturates_at_zero() {
        let mut budget = TokenBudget::new(100);
        budget.record_usage(200);
        assert_eq!(budget.daily_remaining(), 0);
    }

    // ── A5-05: Budget exhausted detection ──────────────────────────────

    #[rstest]
    #[case::not_exhausted(0, false)]
    #[case::half(50, false)]
    #[case::exactly(100, true)]
    #[case::over(150, true)]
    fn a5_05_is_exhausted(#[case] used: u64, #[case] expected: bool) {
        let mut budget = TokenBudget::new(100);
        budget.record_usage(used);
        assert_eq!(
            budget.is_exhausted(),
            expected,
            "Used={used}, limit=100, exhausted should be {expected}"
        );
    }

    // ── A5-06: Budget reset ────────────────────────────────────────────

    #[test]
    fn a5_06_reset_daily() {
        let mut budget = TokenBudget::new(1000);
        budget.record_usage(500);
        assert_eq!(budget.daily_used, 500);
        budget.reset_daily();
        assert_eq!(budget.daily_used, 0);
        assert_eq!(budget.daily_remaining(), 1000);
    }

    // ── A5-07: Builder with_strategy ───────────────────────────────────

    #[rstest]
    #[case::queue(OverloadStrategy::QueueTomorrow)]
    #[case::degrade(OverloadStrategy::DegradeToSmall)]
    #[case::overflow(OverloadStrategy::AllowOverflow)]
    fn a5_07_with_strategy(#[case] strategy: OverloadStrategy) {
        let budget = TokenBudget::new(1000).with_strategy(strategy.clone());
        assert_eq!(budget.overload_strategy, strategy);
    }

    // ── A5-08 ~ A5-09: can_afford basic cases ─────────────────────────

    #[rstest]
    #[case::sufficient(1000, 0, 500, true)] // limit=1000, used=0, need=500
    #[case::exact(1000, 0, 1000, true)] // limit=1000, used=0, need=1000
    #[case::insufficient(1000, 800, 300, false)] // limit=1000, used=800, need=300
    #[case::barely_ok(1000, 800, 200, true)] // limit=1000, used=800, need=200
    fn a5_can_afford_default_strategy(
        #[case] limit: u64,
        #[case] used: u64,
        #[case] need: u64,
        #[case] expected: bool,
    ) {
        let mut budget = TokenBudget::new(limit);
        budget.record_usage(used);
        assert_eq!(
            budget.can_afford(need),
            expected,
            "limit={limit}, used={used}, need={need} → can_afford should be {expected}"
        );
    }

    // ── A5-10: can_afford with AllowOverflow ───────────────────────────

    #[test]
    fn a5_10_can_afford_allow_overflow() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::AllowOverflow);
        budget.record_usage(100); // exhausted
                                  // AllowOverflow → always true
        assert!(budget.can_afford(1000));
    }

    // ── A5-11: can_afford with QueueTomorrow ───────────────────────────

    #[test]
    fn a5_11_can_afford_queue_tomorrow() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::QueueTomorrow);
        budget.record_usage(100); // exhausted
                                  // QueueTomorrow does NOT allow overflow
        assert!(!budget.can_afford(50));
    }

    // ── A5-12: resolve_strategy — sufficient budget ────────────────────

    #[rstest]
    #[case::non_urgent(false, OverloadStrategy::DegradeToSmall)]
    #[case::urgent(true, OverloadStrategy::DegradeToSmall)]
    fn a5_12_resolve_sufficient_budget(
        #[case] is_urgent: bool,
        #[case] expected: OverloadStrategy,
    ) {
        let budget = TokenBudget::new(1000); // plenty of budget
        let strategy = budget.resolve_strategy(100, is_urgent);
        assert_eq!(
            strategy, expected,
            "Sufficient budget should return configured strategy regardless of urgency"
        );
    }

    // ── A5-13: resolve_strategy — exhausted + urgent ───────────────────

    #[rstest]
    #[case::queue_tomorrow(OverloadStrategy::QueueTomorrow)]
    #[case::allow_overflow(OverloadStrategy::AllowOverflow)]
    fn a5_13_resolve_exhausted_urgent(#[case] configured: OverloadStrategy) {
        let mut budget = TokenBudget::new(100).with_strategy(configured);
        budget.record_usage(100); // exhausted
                                  // Urgent + exhausted → always DegradeToSmall
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(
            strategy,
            OverloadStrategy::DegradeToSmall,
            "Urgent request with exhausted budget should always degrade to small model"
        );
    }

    // ── A5-14 ~ A5-15: resolve_strategy — exhausted + non-urgent ──────

    #[rstest]
    #[case::queue(OverloadStrategy::QueueTomorrow, OverloadStrategy::QueueTomorrow)] // A5-14
    #[case::overflow(OverloadStrategy::AllowOverflow, OverloadStrategy::AllowOverflow)] // A5-15
    fn a5_resolve_exhausted_non_urgent(
        #[case] configured: OverloadStrategy,
        #[case] expected: OverloadStrategy,
    ) {
        let mut budget = TokenBudget::new(100).with_strategy(configured);
        budget.record_usage(100); // exhausted
                                  // Non-urgent + exhausted → use configured strategy
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(
            strategy, expected,
            "Non-urgent with exhausted budget should follow configured overload strategy"
        );
    }

    // ── A5-16: resolve_strategy combinations with DegradeToSmall ───────

    #[test]
    fn a5_16_resolve_degrade_to_small_exhausted_urgent() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::DegradeToSmall);
        budget.record_usage(100); // exhausted
                                  // Urgent + DegradeToSmall → DegradeToSmall
        let strategy = budget.resolve_strategy(50, true);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }

    #[test]
    fn a5_16_resolve_degrade_to_small_exhausted_non_urgent() {
        let mut budget = TokenBudget::new(100).with_strategy(OverloadStrategy::DegradeToSmall);
        budget.record_usage(100); // exhausted
                                  // Non-urgent + DegradeToSmall → DegradeToSmall (configured strategy)
        let strategy = budget.resolve_strategy(50, false);
        assert_eq!(strategy, OverloadStrategy::DegradeToSmall);
    }
}
