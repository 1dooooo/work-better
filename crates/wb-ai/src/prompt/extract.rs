//! 实体提取提示词模板

/// 构建实体提取提示词
pub fn build_extract_prompt(
    event_type: &str,
    source: &str,
    content: &str,
    raw_payload: &str,
) -> String {
    format!(
        r#"从以下工作事件中提取结构化信息。

事件类型: {}
来源: {}
内容: {}
原始数据: {}

提取要求：
1. title: 简洁明了的标题，不超过 50 字
2. summary: 一句话摘要，不超过 100 字
3. detail: 详细内容，使用 Markdown 格式
4. people: 提及的人名列表（去重）
5. tags: 相关标签（2-5 个，使用 kebab-case 格式）
6. project: 关联的项目名（如果能推断出来）
7. confidence: 提取结果的置信度 0.0-1.0

请以 JSON 格式返回：
{{
  "title": "<title>",
  "summary": "<summary>",
  "detail": "<detail>",
  "people": ["<name>"],
  "tags": ["<tag>"],
  "project": "<project>" or null,
  "confidence": <0.0-1.0>
}}

只返回 JSON，不要其他内容。"#,
        event_type, source, content, raw_payload
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_extract_prompt_contains_all_fields() {
        let prompt = build_extract_prompt(
            "DocumentChange",
            "FeishuDoc",
            "更新了设计文档",
            r#"{"doc_id":"123"}"#,
        );
        assert!(prompt.contains("事件类型: DocumentChange"));
        assert!(prompt.contains("来源: FeishuDoc"));
        assert!(prompt.contains("内容: 更新了设计文档"));
        assert!(prompt.contains(r#"原始数据: {"doc_id":"123"}"#));
    }

    #[test]
    fn test_build_extract_prompt_contains_extraction_requirements() {
        let prompt = build_extract_prompt("Message", "FeishuMessage", "test", "{}");
        assert!(prompt.contains("title:"));
        assert!(prompt.contains("summary:"));
        assert!(prompt.contains("detail:"));
        assert!(prompt.contains("people:"));
        assert!(prompt.contains("tags:"));
        assert!(prompt.contains("project:"));
        assert!(prompt.contains("confidence:"));
    }

    #[test]
    fn test_build_extract_prompt_contains_format_instructions() {
        let prompt = build_extract_prompt("Message", "FeishuMessage", "test", "{}");
        assert!(prompt.contains("kebab-case"));
        assert!(prompt.contains("只返回 JSON，不要其他内容"));
    }
}
