//! 提示词模板模块
//!
//! 为不同任务类型提供结构化的提示词模板。

pub mod analyze;
pub mod classify;
pub mod extract;
pub mod summarize;

pub use analyze::build_analyze_prompt;
pub use classify::build_classify_prompt;
pub use extract::build_extract_prompt;
pub use summarize::build_summarize_prompt;
