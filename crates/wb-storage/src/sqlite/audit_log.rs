//! 审计日志统一查询层
//!
//! 封装对 `processing_audits` 和 `execution_logs` 两张表的查询和写入。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use ts_rs::TS;
use uuid::Uuid;

/// 已知的审计步骤枚举（用于验证）
const VALID_STEPS: &[&str] = &["Classifier", "Extract", "Upgrade", "Review", "Persist", "UserConfirm"];

/// 已知的执行状态枚举（用于验证）
const VALID_STATUSES: &[&str] = &["Success", "Failed", "Skipped", "Timeout"];

/// 处理审计日志查询过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditQueryFilter {
    /// 按 step 过滤（Classifier, Extract, Upgrade, Review 等）
    pub step: Option<String>,
    /// 按 trace_id 过滤
    pub trace_id: Option<String>,
    /// 起始时间（ISO 8601 格式）
    pub since: Option<String>,
    /// 结束时间（ISO 8601 格式）
    pub until: Option<String>,
    /// 返回条数限制，默认 100，最大 500
    #[ts(type = "number")]
    pub limit: Option<u32>,
}

impl AuditQueryFilter {
    /// 验证过滤器参数
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref step) = self.step {
            if !VALID_STEPS.contains(&step.as_str()) {
                return Err(format!("无效的 step '{}'，有效值: {:?}", step, VALID_STEPS));
            }
        }
        if let Some(ref since) = self.since {
            if !is_valid_iso8601(since) {
                return Err(format!("无效的 since 格式 '{}'，应为 ISO 8601 格式", since));
            }
        }
        if let Some(ref until) = self.until {
            if !is_valid_iso8601(until) {
                return Err(format!("无效的 until 格式 '{}'，应为 ISO 8601 格式", until));
            }
        }
        Ok(())
    }
}

/// 执行日志查询过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionLogFilter {
    /// 按 task_id 过滤
    pub task_id: Option<String>,
    /// 按 status 过滤（Success, Failed, Skipped, Timeout）
    pub status: Option<String>,
    /// 返回条数限制，默认 100，最大 500
    #[ts(type = "number")]
    pub limit: Option<u32>,
}

impl ExecutionLogFilter {
    /// 验证过滤器参数
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref status) = self.status {
            if !VALID_STATUSES.contains(&status.as_str()) {
                return Err(format!("无效的 status '{}'，有效值: {:?}", status, VALID_STATUSES));
            }
        }
        Ok(())
    }
}

/// 简单的 ISO 8601 格式验证
fn is_valid_iso8601(s: &str) -> bool {
    // 基本格式检查：YYYY-MM-DDTHH:MM:SS
    s.len() >= 19 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-')
}

/// 处理审计日志行（前端 DTO）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProcessingAuditRow {
    pub event_id: String,
    pub record_id: Option<String>,
    pub trace_id: String,
    pub step: String,
    pub timestamp: String,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub model: String,
    pub model_version: String,
    pub prompt_id: String,
    /// prompt 参数（JSON 字符串）
    pub prompt_params: String,
    pub input_summary: String,
    /// 输出结果（JSON 字符串）
    pub output: String,
    pub confidence: f64,
    #[ts(type = "number")]
    pub token_input: u64,
    #[ts(type = "number")]
    pub token_output: u64,
    pub cost_estimate: f64,
    pub upgrade_reason: Option<String>,
    pub previous_model: Option<String>,
    pub review_verdict: Option<String>,
    /// 审核问题（JSON 数组字符串）
    pub review_issues: Option<String>,
    pub user_action: Option<String>,
    pub user_correction: Option<String>,
}

/// 执行日志行（前端 DTO）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionLogRow {
    pub id: String,
    pub task_id: String,
    pub task_name: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
}

/// 审计日志统计摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditSummary {
    #[ts(type = "number")]
    pub total_processing_audits: u64,
    #[ts(type = "number")]
    pub total_execution_logs: u64,
    #[ts(type = "number")]
    pub total_tokens: u64,
    pub total_cost: f64,
    pub success_rate: f64,
}

/// 处理审计日志写入记录（来自事件处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingAuditInsert {
    pub event_id: String,
    pub record_id: Option<String>,
    pub trace_id: String,
    pub step: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub model: String,
    pub model_version: String,
    pub prompt_id: String,
    pub prompt_params: String,
    pub input_summary: String,
    pub output: String,
    pub confidence: f64,
    pub token_input: u64,
    pub token_output: u64,
    pub cost_estimate: f64,
}

/// 执行日志写入记录（来自 scheduler）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogInsert {
    pub task_id: String,
    pub task_name: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// 审计日志存储
pub struct AuditLogStore {
    conn: Mutex<Connection>,
}

impl AuditLogStore {
    /// 从已有连接创建
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    /// 查询处理审计日志
    pub fn query_processing_audits(
        &self,
        filter: &AuditQueryFilter,
    ) -> Result<Vec<ProcessingAuditRow>, String> {
        // M2: 输入验证
        filter.validate()?;

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let limit = filter.limit.unwrap_or(100).min(500);

        let mut sql = String::from(
            "SELECT event_id, record_id, trace_id, step, timestamp, duration_ms,
                    model, model_version, prompt_id, prompt_params, input_summary,
                    output, confidence, token_input, token_output, cost_estimate,
                    upgrade_reason, previous_model, review_verdict, review_issues,
                    user_action, user_correction
             FROM processing_audits WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref step) = filter.step {
            sql.push_str(" AND step = ?");
            param_values.push(Box::new(step.clone()));
        }
        if let Some(ref trace_id) = filter.trace_id {
            sql.push_str(" AND trace_id = ?");
            param_values.push(Box::new(trace_id.clone()));
        }
        if let Some(ref since) = filter.since {
            sql.push_str(" AND timestamp >= ?");
            param_values.push(Box::new(since.clone()));
        }
        if let Some(ref until) = filter.until {
            sql.push_str(" AND timestamp <= ?");
            param_values.push(Box::new(until.clone()));
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
        param_values.push(Box::new(limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| format!("Prepare error: {}", e))?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| {
                // M1: 使用列名访问替代硬编码索引
                Ok(ProcessingAuditRow {
                    event_id: row.get("event_id")?,
                    record_id: row.get("record_id")?,
                    trace_id: row.get("trace_id")?,
                    step: row.get("step")?,
                    timestamp: row.get("timestamp")?,
                    duration_ms: row.get("duration_ms")?,
                    model: row.get("model")?,
                    model_version: row.get("model_version")?,
                    prompt_id: row.get("prompt_id")?,
                    prompt_params: row.get("prompt_params")?,
                    input_summary: row.get("input_summary")?,
                    output: row.get("output")?,
                    confidence: row.get("confidence")?,
                    token_input: row.get("token_input")?,
                    token_output: row.get("token_output")?,
                    cost_estimate: row.get("cost_estimate")?,
                    upgrade_reason: row.get("upgrade_reason")?,
                    previous_model: row.get("previous_model")?,
                    review_verdict: row.get("review_verdict")?,
                    review_issues: row.get("review_issues")?,
                    user_action: row.get("user_action")?,
                    user_correction: row.get("user_correction")?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }

    /// 查询执行日志
    pub fn query_execution_logs(
        &self,
        filter: &ExecutionLogFilter,
    ) -> Result<Vec<ExecutionLogRow>, String> {
        // M2: 输入验证
        filter.validate()?;

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let limit = filter.limit.unwrap_or(100).min(500);

        let mut sql = String::from(
            "SELECT id, task_id, task_name, status, started_at, finished_at,
                    duration_ms, output, error, created_at
             FROM execution_logs WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref task_id) = filter.task_id {
            sql.push_str(" AND task_id = ?");
            param_values.push(Box::new(task_id.clone()));
        }
        if let Some(ref status) = filter.status {
            sql.push_str(" AND status = ?");
            param_values.push(Box::new(status.clone()));
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ?");
        param_values.push(Box::new(limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| format!("Prepare error: {}", e))?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| {
                // M1: 使用列名访问替代硬编码索引
                Ok(ExecutionLogRow {
                    id: row.get("id")?,
                    task_id: row.get("task_id")?,
                    task_name: row.get("task_name")?,
                    status: row.get("status")?,
                    started_at: row.get("started_at")?,
                    finished_at: row.get("finished_at")?,
                    duration_ms: row.get("duration_ms")?,
                    output: row.get("output")?,
                    error: row.get("error")?,
                    created_at: row.get("created_at")?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }

    /// 插入执行日志
    pub fn insert_execution_log(&self, record: &ExecutionLogInsert) -> Result<(), String> {
        // 验证 status
        if !VALID_STATUSES.contains(&record.status.as_str()) {
            return Err(format!("无效的 status '{}'，有效值: {:?}", record.status, VALID_STATUSES));
        }

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO execution_logs (id, task_id, task_name, status, started_at, finished_at, duration_ms, output, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                record.task_id,
                record.task_name,
                record.status,
                record.started_at,
                record.finished_at,
                record.duration_ms as i64,
                record.output,
                record.error,
            ],
        )
        .map_err(|e| format!("Insert error: {}", e))?;

        Ok(())
    }

    /// 插入处理审计日志
    pub fn insert_processing_audit(&self, record: &ProcessingAuditInsert) -> Result<(), String> {
        // 验证 step
        if !VALID_STEPS.contains(&record.step.as_str()) {
            return Err(format!("无效的 step '{}'，有效值: {:?}", record.step, VALID_STEPS));
        }

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO processing_audits
             (event_id, record_id, trace_id, step, timestamp, duration_ms,
              model, model_version, prompt_id, prompt_params, input_summary,
              output, confidence, token_input, token_output, cost_estimate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                record.event_id,
                record.record_id,
                record.trace_id,
                record.step,
                record.timestamp,
                record.duration_ms as i64,
                record.model,
                record.model_version,
                record.prompt_id,
                record.prompt_params,
                record.input_summary,
                record.output,
                record.confidence,
                record.token_input as i64,
                record.token_output as i64,
                record.cost_estimate,
            ],
        )
        .map_err(|e| format!("Insert error: {}", e))?;

        Ok(())
    }

    /// 获取审计日志统计摘要
    pub fn get_summary(&self) -> Result<AuditSummary, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // M3: 合并查询 - 一次性获取所有统计
        let (total_processing, total_tokens, total_cost, total_execution, success_count): (u64, u64, f64, u64, u64) = conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM processing_audits),
                    (SELECT COALESCE(SUM(token_input + token_output), 0) FROM processing_audits),
                    (SELECT COALESCE(SUM(cost_estimate), 0.0) FROM processing_audits),
                    (SELECT COUNT(*) FROM execution_logs),
                    (SELECT COUNT(*) FROM execution_logs WHERE status = 'Success')",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|e| format!("Summary query error: {}", e))?;

        let success_rate = if total_execution > 0 {
            success_count as f64 / total_execution as f64
        } else {
            0.0
        };

        Ok(AuditSummary {
            total_processing_audits: total_processing,
            total_execution_logs: total_execution,
            total_tokens,
            total_cost,
            success_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_store() -> AuditLogStore {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS processing_audits (
                event_id TEXT NOT NULL,
                record_id TEXT,
                trace_id TEXT NOT NULL,
                step TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                model TEXT NOT NULL,
                model_version TEXT NOT NULL,
                prompt_id TEXT NOT NULL,
                prompt_params TEXT NOT NULL,
                input_summary TEXT NOT NULL,
                output TEXT NOT NULL,
                confidence REAL NOT NULL,
                token_input INTEGER NOT NULL,
                token_output INTEGER NOT NULL,
                cost_estimate REAL NOT NULL,
                upgrade_reason TEXT,
                previous_model TEXT,
                review_verdict TEXT,
                review_issues TEXT,
                user_action TEXT,
                user_correction TEXT,
                PRIMARY KEY(event_id, trace_id, step)
            );
            CREATE TABLE IF NOT EXISTS execution_logs (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                task_name TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                output TEXT,
                error TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();

        AuditLogStore::new(conn)
    }

    #[test]
    fn test_insert_and_query_execution_log() {
        let store = setup_store();

        let insert = ExecutionLogInsert {
            task_id: "task1".to_string(),
            task_name: "Test Task".to_string(),
            status: "Success".to_string(),
            started_at: "2026-06-09T10:00:00Z".to_string(),
            finished_at: "2026-06-09T10:00:05Z".to_string(),
            duration_ms: 5000,
            output: Some("done".to_string()),
            error: None,
        };

        store.insert_execution_log(&insert).unwrap();

        let filter = ExecutionLogFilter {
            task_id: Some("task1".to_string()),
            ..Default::default()
        };
        let results = store.query_execution_logs(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_name, "Test Task");
        assert_eq!(results[0].status, "Success");
    }

    #[test]
    fn test_query_execution_logs_with_status_filter() {
        let store = setup_store();

        store
            .insert_execution_log(&ExecutionLogInsert {
                task_id: "t1".to_string(),
                task_name: "Task 1".to_string(),
                status: "Success".to_string(),
                started_at: "2026-06-09T10:00:00Z".to_string(),
                finished_at: "2026-06-09T10:00:01Z".to_string(),
                duration_ms: 1000,
                output: None,
                error: None,
            })
            .unwrap();

        store
            .insert_execution_log(&ExecutionLogInsert {
                task_id: "t2".to_string(),
                task_name: "Task 2".to_string(),
                status: "Failed".to_string(),
                started_at: "2026-06-09T10:01:00Z".to_string(),
                finished_at: "2026-06-09T10:01:02Z".to_string(),
                duration_ms: 2000,
                output: None,
                error: Some("timeout".to_string()),
            })
            .unwrap();

        let filter = ExecutionLogFilter {
            status: Some("Failed".to_string()),
            ..Default::default()
        };
        let results = store.query_execution_logs(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "Failed");
    }

    #[test]
    fn test_get_summary_empty() {
        let store = setup_store();
        let summary = store.get_summary().unwrap();
        assert_eq!(summary.total_processing_audits, 0);
        assert_eq!(summary.total_execution_logs, 0);
        assert_eq!(summary.total_tokens, 0);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.success_rate, 0.0);
    }

    #[test]
    fn test_invalid_step_filter() {
        let store = setup_store();
        let filter = AuditQueryFilter {
            step: Some("InvalidStep".to_string()),
            ..Default::default()
        };
        let result = store.query_processing_audits(&filter);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的 step"));
    }

    #[test]
    fn test_invalid_status_filter() {
        let store = setup_store();
        let filter = ExecutionLogFilter {
            status: Some("InvalidStatus".to_string()),
            ..Default::default()
        };
        let result = store.query_execution_logs(&filter);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的 status"));
    }

    #[test]
    fn test_invalid_insert_status() {
        let store = setup_store();
        let insert = ExecutionLogInsert {
            task_id: "t1".to_string(),
            task_name: "Task 1".to_string(),
            status: "InvalidStatus".to_string(),
            started_at: "2026-06-09T10:00:00Z".to_string(),
            finished_at: "2026-06-09T10:00:01Z".to_string(),
            duration_ms: 1000,
            output: None,
            error: None,
        };
        let result = store.insert_execution_log(&insert);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的 status"));
    }
}
