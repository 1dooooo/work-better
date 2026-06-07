//! SLA 管理 —— 四级优先级超时与自动升级

use chrono::{DateTime, Utc};
use wb_core::record::WorkRecord;

/// 优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    P3,
    P2,
    P1,
    P0,
}

/// SLA 配置（各优先级超时阈值，单位毫秒）
#[derive(Debug, Clone)]
pub struct SlaConfig {
    pub p0_timeout_ms: u64, // 5 min = 300_000
    pub p1_timeout_ms: u64, // 30 min = 1_800_000
    pub p2_timeout_ms: u64, // 4 hours = 14_400_000
    pub p3_timeout_ms: u64, // 24 hours = 86_400_000
}

impl Default for SlaConfig {
    fn default() -> Self {
        Self {
            p0_timeout_ms: 300_000,    // 5 min
            p1_timeout_ms: 1_800_000,  // 30 min
            p2_timeout_ms: 14_400_000, // 4 hours
            p3_timeout_ms: 86_400_000, // 24 hours
        }
    }
}

/// SLA 管理器
pub struct SlaManager {
    config: SlaConfig,
}

/// 单条记录的时效信息
#[derive(Debug, Clone)]
pub struct RecordTimeliness {
    pub record_id: String,
    pub priority: Priority,
    pub elapsed_ms: u64,
    pub is_breached: bool,
}

/// 每日时效报告
#[derive(Debug, Clone)]
pub struct TimelinessReport {
    pub generated_at: DateTime<Utc>,
    pub total_records: usize,
    pub breached_count: usize,
    pub on_time_count: usize,
    pub breach_rate: f64,
    pub records: Vec<RecordTimeliness>,
}

impl SlaManager {
    /// 创建新的 SLA 管理器
    pub fn new(config: SlaConfig) -> Self {
        Self { config }
    }

    /// 获取指定优先级的超时阈值（毫秒）
    fn timeout_for(&self, priority: &Priority) -> u64 {
        match priority {
            Priority::P0 => self.config.p0_timeout_ms,
            Priority::P1 => self.config.p1_timeout_ms,
            Priority::P2 => self.config.p2_timeout_ms,
            Priority::P3 => self.config.p3_timeout_ms,
        }
    }

    /// 检查是否已超时
    pub fn check_timeout(&self, priority: &Priority, elapsed_ms: u64) -> bool {
        elapsed_ms > self.timeout_for(priority)
    }

    /// 升级优先级：P3→P2→P1→P0（P0 不再升级）
    pub fn escalate_priority(&self, priority: &Priority) -> Priority {
        match priority {
            Priority::P3 => Priority::P2,
            Priority::P2 => Priority::P1,
            Priority::P1 => Priority::P0,
            Priority::P0 => Priority::P0,
        }
    }

    /// 为记录估算优先级（基于置信度和分类）
    pub fn estimate_priority(record: &WorkRecord) -> Priority {
        if record.needs_review {
            Priority::P1
        } else if record.confidence >= 0.9 {
            Priority::P3
        } else {
            Priority::P2
        }
    }

    /// 生成每日时效报告
    pub fn daily_report(&self, records: &[(WorkRecord, u64)]) -> TimelinessReport {
        let mut record_timeliness = Vec::with_capacity(records.len());
        let mut breached_count = 0usize;

        for (record, elapsed_ms) in records {
            let priority = Self::estimate_priority(record);
            let is_breached = self.check_timeout(&priority, *elapsed_ms);
            if is_breached {
                breached_count += 1;
            }
            record_timeliness.push(RecordTimeliness {
                record_id: record.id.clone(),
                priority,
                elapsed_ms: *elapsed_ms,
                is_breached,
            });
        }

        let total = records.len();
        let on_time = total - breached_count;
        let breach_rate = if total > 0 {
            breached_count as f64 / total as f64
        } else {
            0.0
        };

        TimelinessReport {
            generated_at: Utc::now(),
            total_records: total,
            breached_count,
            on_time_count: on_time,
            breach_rate,
            records: record_timeliness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> SlaManager {
        SlaManager::new(SlaConfig::default())
    }

    #[test]
    fn test_check_timeout_within_limit() {
        let mgr = default_manager();
        // P0: 5min, elapsed 1min → not timed out
        assert!(!mgr.check_timeout(&Priority::P0, 60_000));
    }

    #[test]
    fn test_check_timeout_exceeded() {
        let mgr = default_manager();
        // P0: 5min, elapsed 6min → timed out
        assert!(mgr.check_timeout(&Priority::P0, 360_000));
    }

    #[test]
    fn test_check_timeout_p3_within_limit() {
        let mgr = default_manager();
        // P3: 24h, elapsed 12h → not timed out
        assert!(!mgr.check_timeout(&Priority::P3, 43_200_000));
    }

    #[test]
    fn test_escalate_p3_to_p2() {
        let mgr = default_manager();
        assert_eq!(mgr.escalate_priority(&Priority::P3), Priority::P2);
    }

    #[test]
    fn test_escalate_p2_to_p1() {
        let mgr = default_manager();
        assert_eq!(mgr.escalate_priority(&Priority::P2), Priority::P1);
    }

    #[test]
    fn test_escalate_p1_to_p0() {
        let mgr = default_manager();
        assert_eq!(mgr.escalate_priority(&Priority::P1), Priority::P0);
    }

    #[test]
    fn test_escalate_p0_stays_p0() {
        let mgr = default_manager();
        assert_eq!(mgr.escalate_priority(&Priority::P0), Priority::P0);
    }

    #[test]
    fn test_full_escalation_chain() {
        let mgr = default_manager();
        let mut p = Priority::P3;
        p = mgr.escalate_priority(&p);
        assert_eq!(p, Priority::P2);
        p = mgr.escalate_priority(&p);
        assert_eq!(p, Priority::P1);
        p = mgr.escalate_priority(&p);
        assert_eq!(p, Priority::P0);
        p = mgr.escalate_priority(&p);
        assert_eq!(p, Priority::P0);
    }

    fn make_record(confidence: f64, needs_review: bool) -> WorkRecord {
        let mut r = WorkRecord::new(
            "Test".into(),
            "Summary".into(),
            "Detail".into(),
            wb_core::record::Category::Task,
            vec![],
            "mock".into(),
            confidence,
        );
        r.needs_review = needs_review;
        r
    }

    #[test]
    fn test_estimate_priority_needs_review() {
        let r = make_record(0.5, true);
        assert_eq!(SlaManager::estimate_priority(&r), Priority::P1);
    }

    #[test]
    fn test_estimate_priority_high_confidence() {
        let r = make_record(0.95, false);
        assert_eq!(SlaManager::estimate_priority(&r), Priority::P3);
    }

    #[test]
    fn test_estimate_priority_medium_confidence() {
        let r = make_record(0.7, false);
        assert_eq!(SlaManager::estimate_priority(&r), Priority::P2);
    }

    #[test]
    fn test_daily_report_empty() {
        let mgr = default_manager();
        let report = mgr.daily_report(&[]);
        assert_eq!(report.total_records, 0);
        assert_eq!(report.breach_rate, 0.0);
    }

    #[test]
    fn test_daily_report_all_on_time() {
        let mgr = default_manager();
        let r1 = make_record(0.95, false); // P3, 24h limit
        let r2 = make_record(0.9, false); // P3
        let records: Vec<(WorkRecord, u64)> = vec![(r1, 1000), (r2, 2000)];
        let report = mgr.daily_report(&records);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.breached_count, 0);
        assert_eq!(report.on_time_count, 2);
        assert_eq!(report.breach_rate, 0.0);
    }

    #[test]
    fn test_daily_report_some_breached() {
        let mgr = default_manager();
        let r1 = make_record(0.5, true); // P1, 30min limit
        let r2 = make_record(0.95, false); // P3, 24h limit
        let records: Vec<(WorkRecord, u64)> = vec![
            (r1, 3_600_000), // 1 hour > 30min P1 limit → breached
            (r2, 1000),      // 1s < 24h P3 limit → on time
        ];
        let report = mgr.daily_report(&records);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.breached_count, 1);
        assert_eq!(report.on_time_count, 1);
        assert!((report.breach_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_sla_config() {
        let config = SlaConfig {
            p0_timeout_ms: 1000,
            p1_timeout_ms: 2000,
            p2_timeout_ms: 3000,
            p3_timeout_ms: 4000,
        };
        let mgr = SlaManager::new(config);
        // P0 with 1500ms elapsed → breached (limit is 1000ms)
        assert!(mgr.check_timeout(&Priority::P0, 1500));
        // P3 with 1500ms elapsed → not breached (limit is 4000ms)
        assert!(!mgr.check_timeout(&Priority::P3, 1500));
    }

    // ─── A4-01~08: Parametrized P0-P3 within/over limit ───────────
    use rstest::rstest;

    // (priority, elapsed_ms, expected_breached)
    #[rstest]
    #[case(Priority::P0, 60_000, false)] // A4-01: P0 within limit (1min < 5min)
    #[case(Priority::P0, 360_000, true)] // A4-02: P0 over limit (6min > 5min)
    #[case(Priority::P1, 900_000, false)] // A4-03: P1 within limit (15min < 30min)
    #[case(Priority::P1, 2_400_000, true)] // A4-04: P1 over limit (40min > 30min)
    #[case(Priority::P2, 7_200_000, false)] // A4-05: P2 within limit (2h < 4h)
    #[case(Priority::P2, 18_000_000, true)] // A4-06: P2 over limit (5h > 4h)
    #[case(Priority::P3, 43_200_000, false)] // A4-07: P3 within limit (12h < 24h)
    #[case(Priority::P3, 90_000_000, true)] // A4-08: P3 over limit (25h > 24h)
    fn test_sla_timeout_boundary(
        #[case] priority: Priority,
        #[case] elapsed_ms: u64,
        #[case] expected_breached: bool,
    ) {
        let mgr = default_manager();
        assert_eq!(mgr.check_timeout(&priority, elapsed_ms), expected_breached);
    }

    // ─── A4-09: Full escalation chain P3→P2→P1→P0 ─────────────────
    // (already covered by test_full_escalation_chain, adding rstest version)

    #[rstest]
    #[case(Priority::P3, Priority::P2)]
    #[case(Priority::P2, Priority::P1)]
    #[case(Priority::P1, Priority::P0)]
    #[case(Priority::P0, Priority::P0)]
    fn test_escalate_parametrized(#[case] input: Priority, #[case] expected: Priority) {
        let mgr = default_manager();
        assert_eq!(mgr.escalate_priority(&input), expected);
    }

    // ─── A4-10: Escalation ceiling — P0 does not escalate further ──

    #[test]
    fn test_escalation_ceiling_p0_never_exceeds() {
        let mgr = default_manager();
        let mut p = Priority::P0;
        for _ in 0..10 {
            p = mgr.escalate_priority(&p);
        }
        assert_eq!(p, Priority::P0);
    }

    // ─── A4-11~13: Priority estimation rules (parametrized) ────────

    #[rstest]
    #[case(0.5, true, Priority::P1)] // A4-11: needs_review → P1
    #[case(0.95, false, Priority::P3)] // A4-12: high confidence, no review → P3
    #[case(0.7, false, Priority::P2)] // A4-13: medium confidence, no review → P2
    fn test_estimate_priority_parametrized(
        #[case] confidence: f64,
        #[case] needs_review: bool,
        #[case] expected: Priority,
    ) {
        let r = make_record(confidence, needs_review);
        assert_eq!(SlaManager::estimate_priority(&r), expected);
    }

    // ─── A4-14: Daily report — all breached ────────────────────────

    #[test]
    fn test_daily_report_all_breached() {
        let mgr = default_manager();
        let records: Vec<(WorkRecord, u64)> = vec![
            (make_record(0.5, true), 3_600_000),     // P1, 1h > 30min
            (make_record(0.95, false), 100_000_000), // P3, ~28h > 24h
        ];
        let report = mgr.daily_report(&records);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.breached_count, 2);
        assert_eq!(report.on_time_count, 0);
        assert!((report.breach_rate - 1.0).abs() < f64::EPSILON);
    }

    // ─── A4-15: Daily report — single record ───────────────────────

    #[test]
    fn test_daily_report_single_record_on_time() {
        let mgr = default_manager();
        let records = vec![(make_record(0.7, false), 1000)]; // P2, 1s < 4h
        let report = mgr.daily_report(&records);
        assert_eq!(report.total_records, 1);
        assert_eq!(report.breached_count, 0);
        assert_eq!(report.on_time_count, 1);
        assert_eq!(report.records.len(), 1);
        assert!(!report.records[0].is_breached);
    }

    // ─── A4-16: Daily report — record_ids are preserved ────────────

    #[test]
    fn test_daily_report_preserves_record_ids() {
        let mgr = default_manager();
        let r1 = make_record(0.95, false);
        let r2 = make_record(0.5, true);
        let id1 = r1.id.clone();
        let id2 = r2.id.clone();
        let records = vec![(r1, 1000), (r2, 3_600_000)];
        let report = mgr.daily_report(&records);
        assert_eq!(report.records[0].record_id, id1);
        assert_eq!(report.records[1].record_id, id2);
    }
}
