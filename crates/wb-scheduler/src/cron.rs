//! Cron expression parsing and scheduling utilities.

use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CronError {
    #[error("invalid cron expression: {0}")]
    InvalidExpression(String),
}

/// Parse a cron expression and return the next fire time after now.
pub fn next_fire_time(cron_expr: &str) -> Result<DateTime<Utc>, CronError> {
    next_fire_time_from(cron_expr, Utc::now())
}

/// Parse a cron expression and return the next fire time after the given instant.
pub fn next_fire_time_from(
    cron_expr: &str,
    after: DateTime<Utc>,
) -> Result<DateTime<Utc>, CronError> {
    let schedule = Schedule::from_str(cron_expr)
        .map_err(|e| CronError::InvalidExpression(e.to_string()))?;
    schedule
        .after(&after)
        .next()
        .ok_or_else(|| CronError::InvalidExpression("no upcoming fire time".to_string()))
}

/// Check whether a cron-scheduled task is due based on its last run time.
///
/// A task is considered due when the next scheduled fire time after `last_run`
/// is not in the future (i.e., it has already passed or is now).
pub fn is_due(cron_expr: &str, last_run: DateTime<Utc>) -> Result<bool, CronError> {
    let next = next_fire_time_from(cron_expr, last_run)?;
    Ok(next <= Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn test_next_fire_time_valid() {
        let result = next_fire_time("0 * * * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_next_fire_time_invalid() {
        let result = next_fire_time("not a cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_next_fire_time_from_past() {
        let past = Utc::now() - ChronoDuration::hours(1);
        let result = next_fire_time_from("* * * * * *", past);
        assert!(result.is_ok());
        let next = result.unwrap();
        assert!(next > past);
    }

    #[test]
    fn test_is_due_when_past_schedule() {
        // A task that was last run 10 minutes ago with an every-minute schedule
        let last_run = Utc::now() - ChronoDuration::minutes(10);
        let due = is_due("* * * * * *", last_run).unwrap();
        assert!(due);
    }

    #[test]
    fn test_is_not_due_when_just_ran() {
        // A task that just ran with a daily schedule should not be due
        let last_run = Utc::now();
        let due = is_due("0 0 0 * * *", last_run).unwrap();
        assert!(!due);
    }
}
