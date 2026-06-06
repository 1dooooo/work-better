//! wb-storage: 数据存储层

pub mod freshness;
pub mod obsidian;
pub mod sqlite;
pub mod vector;

pub use freshness::{FreshnessEngine, FreshnessReport};
pub use obsidian::ObsidianWriter;
pub use sqlite::SqliteEventLog;
pub use vector::{
    EmbeddingProvider, InMemoryVectorStore, MockEmbedding, SemanticSearch, SearchResult,
    SyncReport, VectorStore, VectorSync,
};
