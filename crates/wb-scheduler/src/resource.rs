//! Resource-awareness for task scheduling.
//!
//! Defers low-priority tasks when the resource budget is running low.

use wb_core::task::Priority;

/// Threshold below which only high-priority tasks (P0, P1) are allowed.
const LOW_BUDGET_THRESHOLD: u32 = 10;

/// Decide whether a task should be deferred based on its priority
/// and the remaining resource budget.
///
/// - P0 tasks are never deferred.
/// - P1 tasks are deferred only when budget is 0.
/// - P2 and P3 tasks are deferred when budget drops below the threshold.
pub fn should_defer(priority: Priority, budget_remaining: u32) -> bool {
    match priority {
        Priority::P0 => false,
        Priority::P1 => budget_remaining == 0,
        Priority::P2 | Priority::P3 => budget_remaining < LOW_BUDGET_THRESHOLD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p0_never_deferred() {
        assert!(!should_defer(Priority::P0, 0));
        assert!(!should_defer(Priority::P0, 100));
    }

    #[test]
    fn test_p1_deferred_only_at_zero() {
        assert!(should_defer(Priority::P1, 0));
        assert!(!should_defer(Priority::P1, 1));
    }

    #[test]
    fn test_p2_deferred_below_threshold() {
        assert!(should_defer(Priority::P2, 0));
        assert!(should_defer(Priority::P2, 9));
        assert!(!should_defer(Priority::P2, LOW_BUDGET_THRESHOLD));
    }

    #[test]
    fn test_p3_deferred_below_threshold() {
        assert!(should_defer(Priority::P3, 5));
        assert!(!should_defer(Priority::P3, LOW_BUDGET_THRESHOLD));
    }
}
