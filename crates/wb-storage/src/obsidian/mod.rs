//! Obsidian 文档层 —— 结构化写入 Obsidian vault

pub mod daily;
pub mod links;
pub mod project;
pub mod tags;
pub mod template;
pub mod vault;
pub mod writer;

pub use daily::DailyJournal;
pub use links::LinkBuilder;
pub use project::ProjectDir;
pub use tags::TagManager;
pub use template::TemplateEngine;
pub use vault::VaultManager;
pub use writer::ObsidianWriter;
