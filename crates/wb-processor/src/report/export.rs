//! Export —— 报告导出
//!
//! 支持 Markdown 和 PDF（placeholder）两种格式导出。

use super::Report;

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Pdf,
}

/// 报告导出器
///
/// 将 Report 导出为指定格式。
/// - Markdown：直接返回报告内容字符串
/// - PDF：placeholder，返回 UTF-8 编码的 PDF 文本骨架（后续接入真实 PDF 库）
pub struct ReportExporter;

impl ReportExporter {
    pub fn new() -> Self {
        Self
    }

    /// 导出为 Markdown 字符串
    ///
    /// 返回完整的 Markdown 文本，包含元数据头和正文。
    pub fn export_markdown(&self, report: &Report) -> Result<String, ExportError> {
        if report.content.is_empty() {
            return Err(ExportError::EmptyContent);
        }

        let mut md = String::new();

        // YAML frontmatter
        md.push_str("---\n");
        md.push_str(&format!("id: {}\n", report.id));
        md.push_str(&format!("type: {:?}\n", report.report_type));
        md.push_str(&format!("title: \"{}\"\n", report.title));
        md.push_str(&format!(
            "period: {} ~ {}\n",
            report.period_start, report.period_end
        ));
        md.push_str(&format!(
            "generated_at: {}\n",
            report.generated_at.format("%Y-%m-%dT%H:%M:%SZ")
        ));
        md.push_str(&format!("status: {:?}\n", report.status));
        md.push_str("---\n\n");

        md.push_str(&report.content);

        Ok(md)
    }

    /// 导出为 PDF 字节
    ///
    /// 当前为 placeholder 实现，将 Markdown 内容包装为简单文本。
    /// 后续接入 `printpdf` 或类似库生成真实 PDF。
    pub fn export_pdf(&self, report: &Report) -> Result<Vec<u8>, ExportError> {
        if report.content.is_empty() {
            return Err(ExportError::EmptyContent);
        }

        // Placeholder: 生成一个包含报告标题和内容的文本字节流
        // 实际实现应使用 PDF 库生成二进制 PDF
        let mut pdf_text = String::new();
        pdf_text.push_str("PDF PLACEHOLDER\n");
        pdf_text.push_str("================\n\n");
        pdf_text.push_str(&format!("Title: {}\n", report.title));
        pdf_text.push_str(&format!(
            "Period: {} ~ {}\n",
            report.period_start, report.period_end
        ));
        pdf_text.push_str(&format!(
            "Generated: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        pdf_text.push_str("\n---\n\n");
        pdf_text.push_str(&report.content);

        Ok(pdf_text.into_bytes())
    }

    /// 按格式导出
    pub fn export(&self, format: ExportFormat, report: &Report) -> Result<Vec<u8>, ExportError> {
        match format {
            ExportFormat::Markdown => {
                let md = self.export_markdown(report)?;
                Ok(md.into_bytes())
            }
            ExportFormat::Pdf => self.export_pdf(report),
        }
    }
}

impl Default for ReportExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// 导出错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// 报告内容为空
    EmptyContent,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::EmptyContent => write!(f, "报告内容为空，无法导出"),
        }
    }
}

impl std::error::Error for ExportError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::confirm::ReportStatus;
    use crate::report::{Report, ReportType};
    use chrono::NaiveDate;

    fn make_test_report() -> Report {
        Report {
            id: "test-id-001".to_string(),
            report_type: ReportType::Daily,
            title: "测试日报".to_string(),
            content: "# 日报\n\n## 完成事项\n\n- Task A\n".to_string(),
            status: ReportStatus::Draft,
            generated_at: chrono::DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            period_start: NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
        }
    }

    #[test]
    fn test_export_markdown_basic() {
        let exporter = ReportExporter::new();
        let report = make_test_report();
        let md = exporter.export_markdown(&report).unwrap();

        assert!(md.starts_with("---\n"));
        assert!(md.contains("id: test-id-001"));
        assert!(md.contains("type: Daily"));
        assert!(md.contains("title: \"测试日报\""));
        assert!(md.contains("period: 2026-06-06 ~ 2026-06-06"));
        assert!(md.contains("# 日报"));
        assert!(md.contains("- Task A"));
    }

    #[test]
    fn test_export_markdown_empty_content() {
        let exporter = ReportExporter::new();
        let mut report = make_test_report();
        report.content = String::new();

        let result = exporter.export_markdown(&report);
        assert_eq!(result.unwrap_err(), ExportError::EmptyContent);
    }

    #[test]
    fn test_export_pdf_basic() {
        let exporter = ReportExporter::new();
        let report = make_test_report();
        let pdf = exporter.export_pdf(&report).unwrap();

        assert!(!pdf.is_empty());
        let text = String::from_utf8(pdf).unwrap();
        assert!(text.contains("PDF PLACEHOLDER"));
        assert!(text.contains("测试日报"));
        assert!(text.contains("Task A"));
    }

    #[test]
    fn test_export_pdf_empty_content() {
        let exporter = ReportExporter::new();
        let mut report = make_test_report();
        report.content = String::new();

        let result = exporter.export_pdf(&report);
        assert_eq!(result.unwrap_err(), ExportError::EmptyContent);
    }

    #[test]
    fn test_export_by_format_markdown() {
        let exporter = ReportExporter::new();
        let report = make_test_report();
        let bytes = exporter.export(ExportFormat::Markdown, &report).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("---\n"));
        assert!(text.contains("# 日报"));
    }

    #[test]
    fn test_export_by_format_pdf() {
        let exporter = ReportExporter::new();
        let report = make_test_report();
        let bytes = exporter.export(ExportFormat::Pdf, &report).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("PDF PLACEHOLDER"));
    }

    #[test]
    fn test_exporter_default() {
        let _exporter = ReportExporter::default();
    }

    #[test]
    fn test_export_error_display() {
        let err = ExportError::EmptyContent;
        assert_eq!(format!("{}", err), "报告内容为空，无法导出");
    }

    #[test]
    fn test_export_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ExportError::EmptyContent);
        assert!(err.to_string().contains("空"));
    }

    #[test]
    fn test_export_markdown_has_frontmatter_structure() {
        let exporter = ReportExporter::new();
        let report = make_test_report();
        let md = exporter.export_markdown(&report).unwrap();

        // 确保 frontmatter 闭合
        let parts: Vec<&str> = md.splitn(3, "---\n").collect();
        assert!(parts.len() >= 3, "should have opening and closing ---");
        // frontmatter 区域包含必要字段
        let frontmatter = parts[1];
        assert!(frontmatter.contains("id:"));
        assert!(frontmatter.contains("type:"));
        assert!(frontmatter.contains("title:"));
        assert!(frontmatter.contains("period:"));
        assert!(frontmatter.contains("generated_at:"));
        assert!(frontmatter.contains("status:"));
    }
}
