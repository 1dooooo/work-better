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
    use rstest::rstest;

    // ── A8-01: P0 never deferred ──────────────────────────────────────
    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(5)]
    #[case(9)]
    #[case(10)]
    #[case(100)]
    fn a8_01_p0_never_deferred(#[case] budget: u32) {
        assert!(!should_defer(Priority::P0, budget));
    }

    // ── A8-02: P1 only deferred at 0 resources ───────────────────────
    #[test]
    fn a8_02_p1_deferred_at_zero() {
        assert!(should_defer(Priority::P1, 0));
    }

    #[rstest]
    #[case(1)]
    #[case(5)]
    #[case(9)]
    #[case(10)]
    #[case(100)]
    fn a8_02_p1_not_deferred_above_zero(#[case] budget: u32) {
        assert!(!should_defer(Priority::P1, budget));
    }

    // ── A8-03: P2 threshold check ─────────────────────────────────────
    #[rstest]
    #[case(0)]
    #[case(5)]
    #[case(9)]
    fn a8_03_p2_deferred_below_threshold(#[case] budget: u32) {
        assert!(should_defer(Priority::P2, budget));
    }

    #[rstest]
    #[case(10)]
    #[case(11)]
    #[case(100)]
    fn a8_03_p2_not_deferred_at_or_above_threshold(#[case] budget: u32) {
        assert!(!should_defer(Priority::P2, budget));
    }

    // ── A8-04: P3 threshold check ─────────────────────────────────────
    #[rstest]
    #[case(0)]
    #[case(5)]
    #[case(9)]
    fn a8_04_p3_deferred_below_threshold(#[case] budget: u32) {
        assert!(should_defer(Priority::P3, budget));
    }

    #[rstest]
    #[case(10)]
    #[case(11)]
    #[case(100)]
    fn a8_04_p3_not_deferred_at_or_above_threshold(#[case] budget: u32) {
        assert!(!should_defer(Priority::P3, budget));
    }

    // ── A8-05: Resource recovery ──────────────────────────────────────
    // Simulates budget draining from 15 → 0 → 15, verifying defer behavior
    // transitions correctly at each boundary.
    #[test]
    fn a8_05_resource_recovery() {
        // Budget 15: P2 not deferred
        assert!(!should_defer(Priority::P2, 15));

        // Budget drops to 9: P2 now deferred
        assert!(should_defer(Priority::P2, 9));

        // Budget drops to 0: P1 now deferred too
        assert!(should_defer(Priority::P1, 0));
        assert!(should_defer(Priority::P2, 0));

        // Budget recovers to 1: P1 no longer deferred, P2 still deferred
        assert!(!should_defer(Priority::P1, 1));
        assert!(should_defer(Priority::P2, 1));

        // Budget recovers to 10: P2 no longer deferred
        assert!(!should_defer(Priority::P2, 10));
    }

    // ── A8-06: Multiple resource types (all priorities at each level) ─
    //
    // Verifies the full 4x3 matrix: each priority level at budget 0,
    // budget below threshold, and budget at/above threshold.
    #[rstest]
    // budget = 0 (all except P0 deferred)
    #[case::p0_at_zero(Priority::P0, 0, false)]
    #[case::p1_at_zero(Priority::P1, 0, true)]
    #[case::p2_at_zero(Priority::P2, 0, true)]
    #[case::p3_at_zero(Priority::P3, 0, true)]
    // budget = 5 (below threshold; P2/P3 deferred)
    #[case::p0_below(Priority::P0, 5, false)]
    #[case::p1_below(Priority::P1, 5, false)]
    #[case::p2_below(Priority::P2, 5, true)]
    #[case::p3_below(Priority::P3, 5, true)]
    // budget = 10 (at threshold; nothing deferred)
    #[case::p0_at(Priority::P0, 10, false)]
    #[case::p1_at(Priority::P1, 10, false)]
    #[case::p2_at(Priority::P2, 10, false)]
    #[case::p3_at(Priority::P3, 10, false)]
    // budget = 100 (well above; nothing deferred)
    #[case::p0_above(Priority::P0, 100, false)]
    #[case::p1_above(Priority::P1, 100, false)]
    #[case::p2_above(Priority::P2, 100, false)]
    #[case::p3_above(Priority::P3, 100, false)]
    fn a8_06_all_priority_budget_matrix(
        #[case] priority: Priority,
        #[case] budget: u32,
        #[case] expected_defer: bool,
    ) {
        let actual = should_defer(priority.clone(), budget);
        assert_eq!(
            actual, expected_defer,
            "should_defer({:?}, {}) should be {} but was {}",
            priority, budget, expected_defer, actual,
        );
    }
}
