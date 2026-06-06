//! LinkBuilder —— 双向链接自动构建

/// Obsidian wiki 链接构建器
#[derive(Debug, Clone)]
pub struct LinkBuilder;

impl LinkBuilder {
    /// 创建新的链接构建器
    pub fn new() -> Self {
        Self
    }

    /// 生成 wiki 链接
    ///
    /// - 无 alias: `[[target]]`
    /// - 有 alias: `[[target|alias]]`
    pub fn wikilink(&self, target: &str, alias: Option<&str>) -> String {
        match alias {
            Some(a) => format!("[[{}|{}]]", target, a),
            None => format!("[[{}]]", target),
        }
    }

    /// 生成反向链接提示文本
    ///
    /// 用于在目标文档中添加来源引用：
    /// `反向链接: 来自 [[source]]`
    pub fn backlink(&self, source: &str, target: &str) -> String {
        format!("反向链接: 来自 [[{}]] — 在 {} 中被引用", source, target)
    }
}

impl Default for LinkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wikilink_without_alias() {
        let lb = LinkBuilder::new();
        assert_eq!(lb.wikilink("Project Alpha", None), "[[Project Alpha]]");
    }

    #[test]
    fn test_wikilink_with_alias() {
        let lb = LinkBuilder::new();
        assert_eq!(
            lb.wikilink("Project Alpha", Some("alpha")),
            "[[Project Alpha|alpha]]"
        );
    }

    #[test]
    fn test_wikilink_chinese() {
        let lb = LinkBuilder::new();
        assert_eq!(lb.wikilink("每日站会", None), "[[每日站会]]");
        assert_eq!(lb.wikilink("每日站会", Some("站会")), "[[每日站会|站会]]");
    }

    #[test]
    fn test_backlink_format() {
        let lb = LinkBuilder::new();
        let result = lb.backlink("meeting-2026-06-06", "Task 1");
        assert_eq!(
            result,
            "反向链接: 来自 [[meeting-2026-06-06]] — 在 Task 1 中被引用"
        );
    }

    #[test]
    fn test_backlink_different_sources() {
        let lb = LinkBuilder::new();
        let r1 = lb.backlink("source-a", "target-x");
        let r2 = lb.backlink("source-b", "target-x");
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_default_builder() {
        let lb = LinkBuilder;
        assert_eq!(lb.wikilink("test", None), "[[test]]");
    }
}
