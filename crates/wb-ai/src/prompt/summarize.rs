//! 摘要提示词模板

/// 构建摘要提示词
pub fn build_summarize_prompt(text: &str) -> String {
    format!(
        r#"请对以下内容生成简洁的摘要。

要求：
1. 不超过 100 字
2. 提取关键信息
3. 语言简洁明了
4. 如果是对话，提取主要议题和结论

内容：
{}

只返回摘要文本，不要其他内容。"#,
        text
    )
}

/// 构建批量摘要提示词（多段内容合并摘要）
pub fn build_batch_summarize_prompt(texts: &[&str]) -> String {
    let items: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| format!("{}. {}", i + 1, t))
        .collect();
    let joined = items.join("\n\n");

    format!(
        r#"请对以下 {} 条内容生成一份合并摘要。

要求：
1. 合并后不超过 200 字
2. 突出最重要的信息
3. 去除重复内容
4. 保留关键数字和结论

内容：
{}

只返回摘要文本，不要其他内容。"#,
        texts.len(),
        joined
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_summarize_prompt_contains_text() {
        let prompt = build_summarize_prompt("今天开了产品评审会");
        assert!(prompt.contains("今天开了产品评审会"));
    }

    #[test]
    fn test_build_summarize_prompt_contains_requirements() {
        let prompt = build_summarize_prompt("test");
        assert!(prompt.contains("不超过 100 字"));
        assert!(prompt.contains("只返回摘要文本，不要其他内容"));
    }

    #[test]
    fn test_build_batch_summarize_prompt_multiple_items() {
        let texts = vec!["第一条内容", "第二条内容", "第三条内容"];
        let prompt = build_batch_summarize_prompt(&texts);
        assert!(prompt.contains("3 条内容"));
        assert!(prompt.contains("1. 第一条内容"));
        assert!(prompt.contains("2. 第二条内容"));
        assert!(prompt.contains("3. 第三条内容"));
    }

    #[test]
    fn test_build_batch_summarize_prompt_single_item() {
        let texts = vec!["唯一的内容"];
        let prompt = build_batch_summarize_prompt(&texts);
        assert!(prompt.contains("1 条内容"));
    }
}
