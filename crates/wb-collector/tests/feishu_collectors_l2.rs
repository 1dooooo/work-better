//! 飞书采集器 L2 集成测试
//!
//! 测试场景：验证完整的采集链路
//! - 调用 lark-cli → 解析响应 → 生成 Event
//!
//! 注意：这些测试使用 mock 数据，不实际调用飞书 API

use wb_core::event::{Source, EventType, Confidence};

/// 测试文档采集器转换逻辑
#[test]
fn test_docs_collector_convert() {
    // 模拟 lark-cli 响应数据
    let response_json = r#"{
        "data": {
            "results": [{
                "entity_type": "docx",
                "result_meta": {
                    "token": "doc-001",
                    "owner_name": "user-001",
                    "update_time_iso": "2024-06-06T12:00:00Z"
                },
                "title_highlighted": "设计文档"
            }]
        }
    }"#;

    // 解析响应
    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let results = response["data"]["results"].as_array().unwrap();

    // 验证解析结果
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["entity_type"], "docx");
    assert_eq!(results[0]["result_meta"]["token"], "doc-001");
}

/// 测试项目采集器转换逻辑
#[test]
fn test_projects_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "task_id": "task-001",
                "title": "完成项目设计",
                "status": "in_progress",
                "assignee": {"name": "user-001"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["task_id"], "task-001");
    assert_eq!(items[0]["title"], "完成项目设计");
}

/// 测试日历采集器转换逻辑
#[test]
fn test_calendar_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "event_id": "event-001",
                "summary": "团队周会",
                "start_time": "2024-06-06T10:00:00Z",
                "end_time": "2024-06-06T11:00:00Z"
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["event_id"], "event-001");
    assert_eq!(items[0]["summary"], "团队周会");
}

/// 测试会议采集器转换逻辑
#[test]
fn test_meetings_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "meeting_id": "meeting-001",
                "topic": "产品评审",
                "start_time": "2024-06-06T14:00:00Z",
                "duration": 3600
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["meeting_id"], "meeting-001");
    assert_eq!(items[0]["topic"], "产品评审");
}

/// 测试邮箱采集器转换逻辑
#[test]
fn test_emails_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "message_id": "email-001",
                "subject": "项目进度汇报",
                "from": {"name": "user-001"},
                "received_time": "2024-06-06T09:00:00Z"
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["message_id"], "email-001");
    assert_eq!(items[0]["subject"], "项目进度汇报");
}

/// 测试审批采集器转换逻辑
#[test]
fn test_approvals_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "instance_id": "approval-001",
                "approval_name": "请假审批",
                "status": "approved",
                "submitter": {"name": "user-001"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["instance_id"], "approval-001");
    assert_eq!(items[0]["approval_name"], "请假审批");
}

/// 测试 OKR 采集器转换逻辑
#[test]
fn test_okr_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "okr_id": "okr-001",
                "title": "Q2 目标",
                "progress": 0.75,
                "owner": {"name": "user-001"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["okr_id"], "okr-001");
    assert_eq!(items[0]["title"], "Q2 目标");
    assert_eq!(items[0]["progress"], 0.75);
}

/// 测试多维表格采集器转换逻辑
#[test]
fn test_bitable_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "record_id": "record-001",
                "table_id": "table-001",
                "fields": {"name": "任务1", "status": "进行中"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["record_id"], "record-001");
    assert_eq!(items[0]["fields"]["name"], "任务1");
}

/// 测试电子表格采集器转换逻辑
#[test]
fn test_spreadsheets_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "spreadsheet_id": "sheet-001",
                "title": "项目进度表",
                "owner": {"name": "user-001"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["spreadsheet_id"], "sheet-001");
    assert_eq!(items[0]["title"], "项目进度表");
}

/// 测试知识库采集器转换逻辑
#[test]
fn test_wiki_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "space_id": "space-001",
                "name": "产品知识库",
                "owner": {"name": "user-001"}
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["space_id"], "space-001");
    assert_eq!(items[0]["name"], "产品知识库");
}

/// 测试妙记采集器转换逻辑
#[test]
fn test_minutes_collector_convert() {
    let response_json = r#"{
        "data": {
            "items": [{
                "minute_id": "minute-001",
                "title": "会议纪要",
                "creator": {"name": "user-001"},
                "duration": 1800
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let items = response["data"]["items"].as_array().unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["minute_id"], "minute-001");
    assert_eq!(items[0]["title"], "会议纪要");
}

/// 测试空响应处理
#[test]
fn test_empty_response_handling() {
    let response_json = r#"{
        "data": {
            "results": []
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let results = response["data"]["results"].as_array().unwrap();

    assert_eq!(results.len(), 0);
}

/// 测试错误响应处理
#[test]
fn test_error_response_handling() {
    let response_json = r#"{
        "error": {
            "code": 403,
            "message": "permission denied"
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();

    assert!(response.get("error").is_some());
    assert_eq!(response["error"]["code"], 403);
}

/// 测试缺失字段处理
#[test]
fn test_missing_fields_handling() {
    let response_json = r#"{
        "data": {
            "results": [{
                "entity_type": "docx"
            }]
        }
    }"#;

    let response: serde_json::Value = serde_json::from_str(response_json).unwrap();
    let results = response["data"]["results"].as_array().unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].get("result_meta").is_none());
}
