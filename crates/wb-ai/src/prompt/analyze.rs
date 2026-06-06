//! 深度分析提示词模板

/// 构建深度分析提示词
///
/// 用于大模型对复杂事件进行深度分析，包括模式识别、关系分析等。
pub fn build_analyze_prompt(events_context: &str, question: &str) -> String {
    format!(
        r#"你是一个资深的工作效率分析师。请对以下工作事件进行深度分析。

事件上下文：
{events_context}

分析问题：
{question}

分析要求：
1. 基于事实进行分析，不猜测
2. 识别模式和趋势
3. 给出可操作的建议
4. 标注分析的置信度

请以 JSON 格式返回：
{{
  "analysis": "详细分析（Markdown 格式）",
  "patterns": ["识别到的模式"],
  "suggestions": ["可操作的建议"],
  "confidence": <0.0-1.0>,
  "data_points": ["支撑分析的关键数据点"]
}}

只返回 JSON，不要其他内容。"#
    )
}

/// 构建跨事件关联分析提示词
pub fn build_relation_prompt(events: &[&str]) -> String {
    let items: Vec<String> = events
        .iter()
        .enumerate()
        .map(|(i, e)| format!("事件 {}: {}", i + 1, e))
        .collect();
    let joined = items.join("\n\n");

    format!(
        r#"请分析以下工作事件之间的关联关系。

事件列表：
{joined}

分析要求：
1. 找出事件之间的因果关系或时间关联
2. 识别涉及的共同人物或项目
3. 发现潜在的工作流模式

请以 JSON 格式返回：
{{
  "relations": [
    {{
      "event_indices": [<关联事件编号>],
      "relation_type": "causal|temporal|collaborative|thematic",
      "description": "关联描述"
    }}
  ],
  "common_actors": ["共同涉及的人物"],
  "common_projects": ["共同涉及的项目"]
}}

只返回 JSON，不要其他内容。"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_analyze_prompt_contains_context() {
        let prompt = build_analyze_prompt("事件列表...", "分析效率趋势");
        assert!(prompt.contains("事件列表..."));
        assert!(prompt.contains("分析效率趋势"));
    }

    #[test]
    fn test_build_analyze_prompt_contains_requirements() {
        let prompt = build_analyze_prompt("ctx", "question");
        assert!(prompt.contains("基于事实进行分析"));
        assert!(prompt.contains("可操作的建议"));
        assert!(prompt.contains("confidence"));
    }

    #[test]
    fn test_build_relation_prompt_multiple_events() {
        let events = vec!["开会讨论方案", "编写设计文档", "提交代码"];
        let prompt = build_relation_prompt(&events);
        assert!(prompt.contains("事件 1: 开会讨论方案"));
        assert!(prompt.contains("事件 2: 编写设计文档"));
        assert!(prompt.contains("事件 3: 提交代码"));
    }

    #[test]
    fn test_build_relation_prompt_contains_relation_types() {
        let events = vec!["event1", "event2"];
        let prompt = build_relation_prompt(&events);
        assert!(prompt.contains("causal"));
        assert!(prompt.contains("temporal"));
        assert!(prompt.contains("collaborative"));
    }

    #[test]
    fn test_build_analyze_prompt_contains_json_structure() {
        let prompt = build_analyze_prompt("ctx", "q");
        assert!(prompt.contains("patterns"));
        assert!(prompt.contains("suggestions"));
        assert!(prompt.contains("data_points"));
    }
}
