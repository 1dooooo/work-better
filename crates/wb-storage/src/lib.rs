//! wb-storage: 数据存储层

pub mod obsidian;
pub mod sqlite;

pub use sqlite::SqliteEventLog;
