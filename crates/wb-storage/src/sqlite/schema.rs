//! SQLite schema 定义

/// 初始化数据库 schema，创建所有表和索引
pub fn initialize_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            collected_at TEXT NOT NULL,
            source TEXT NOT NULL,
            source_confidence TEXT NOT NULL,
            event_type TEXT NOT NULL,
            content TEXT NOT NULL,
            raw_payload TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            related_ids TEXT NOT NULL DEFAULT '[]',
            attachments TEXT NOT NULL DEFAULT '[]',
            processed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_events_source ON events(source);
        CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
        CREATE INDEX IF NOT EXISTS idx_events_processed ON events(processed);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);

        CREATE TABLE IF NOT EXISTS work_records (
            id TEXT PRIMARY KEY,
            created_at TEXT NOT NULL,
            source_event_ids TEXT NOT NULL DEFAULT '[]',
            title TEXT NOT NULL,
            summary TEXT NOT NULL,
            detail TEXT NOT NULL,
            category TEXT NOT NULL,
            project TEXT,
            people TEXT NOT NULL DEFAULT '[]',
            tags TEXT NOT NULL DEFAULT '[]',
            task_status TEXT,
            task_due TEXT,
            task_progress TEXT,
            model_used TEXT NOT NULL,
            confidence REAL NOT NULL,
            needs_review INTEGER NOT NULL DEFAULT 0,
            obsidian_path TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS processing_audits (
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

        CREATE INDEX IF NOT EXISTS idx_audits_trace ON processing_audits(trace_id);
        CREATE INDEX IF NOT EXISTS idx_audits_event ON processing_audits(event_id);
        ",
    )
}
