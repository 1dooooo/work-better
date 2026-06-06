//! wb-storage: 数据存储层

pub mod config;
pub mod freshness;
pub mod obsidian;
pub mod sqlite;
pub mod sync_log;
pub mod vector;

pub use config::{
    AppConfig, CollectorConfig, ModelConfig, SchedulerConfig, ScheduledTaskConfig,
};
pub use freshness::{FreshnessEngine, FreshnessReport};
pub use obsidian::ObsidianWriter;
pub use sqlite::SqliteEventLog;
pub use sync_log::{SyncLayer, SyncLog, SyncLogEntry, SyncStatus};
pub use vector::{
    EmbeddingProvider, InMemoryVectorStore, MockEmbedding, SemanticSearch, SearchResult,
    SyncReport, VectorStore, VectorSync,
};
