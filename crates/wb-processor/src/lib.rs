//! wb-processor: 事件处理层

pub mod audit_pipeline;
pub mod classifier;
pub mod extraction;
pub mod persist;
pub mod pipeline;
pub mod report;
pub mod review_rules;
pub mod reviewer;
pub mod sla;

pub use audit_pipeline::{AuditFilter, AuditPipeline};
pub use report::{Report, ReportGenerator, ReportType};
pub use classifier::{Classifier, ProcessingRoute};
pub use extraction::{EntityExtractor, ExtractedData};
pub use persist::PersistStep;
pub use pipeline::{ProcessingPipeline, ProcessedResult, StepTimings};
pub use reviewer::ReviewAgent;
pub use sla::{Priority, SlaConfig, SlaManager, TimelinessReport};
