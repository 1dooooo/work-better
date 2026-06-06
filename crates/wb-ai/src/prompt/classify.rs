//! 分类提示词模板

/// 构建事件分类提示词
pub fn build_classify_prompt(event_type: &str, source: &str, content: &str) -> String {
    format!(
        r#"你是一个工作事件分类器。请对以下事件进行分类。

事件类型: {}
来源: {}
内容: {}

可选分类标签：
- task: 任务相关（开发、修 bug、写代码等）
- meeting: 会议相关（线上/线下会议、讨论、同步等）
- communication: 沟通相关（聊天、邮件、评论等）
- research: 调研相关（技术调研、方案对比等）
- review: 评审相关（代码审查、文档评审等）
- planning: 规划相关（排期、计划、目标设定等）
- document: 文档相关（写文档、更新文档等）
- decision: 决策相关（技术选型、方案决策等）

请以 JSON 格式返回：
{{"category": "<category>", "confidence": <0.0-1.0>, "reasoning": "<理由>"}}

只返回 JSON，不要其他内容。"#,
        event_type, source, content
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_classify_prompt_contains_all_fields() {
        let prompt = build_classify_prompt("Message", "FeishuMessage", "今天开会讨论项目进度");
        assert!(prompt.contains("事件类型: Message"));
        assert!(prompt.contains("来源: FeishuMessage"));
        assert!(prompt.contains("内容: 今天开会讨论项目进度"));
    }

    #[test]
    fn test_build_classify_prompt_contains_categories() {
        let prompt = build_classify_prompt("Message", "FeishuMessage", "test");
        assert!(prompt.contains("task:"));
        assert!(prompt.contains("meeting:"));
        assert!(prompt.contains("research:"));
        assert!(prompt.contains("decision:"));
    }

    #[test]
    fn test_build_classify_prompt_returns_json_instruction() {
        let prompt = build_classify_prompt("Message", "FeishuMessage", "test");
        assert!(prompt.contains("只返回 JSON，不要其他内容"));
        assert!(prompt.contains("category"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("reasoning"));
    }
}
