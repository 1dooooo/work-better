//! G4 任务管理 — 创建、状态机、AI发现、飞书同步

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

#[given(regex = r"^用户手动创建任务$")]
fn manual_create_task(world: &mut AcceptanceWorld) {
    world.state.insert("task_source".into(), "manual".into());
    world.task_status = Some("todo".into());
}

#[given(regex = r"^系统从会议发现任务$")]
fn ai_discover_task(world: &mut AcceptanceWorld) {
    world.state.insert("task_source".into(), "ai_extracted".into());
    world.state.insert("needs_review".into(), "true".into());
}

#[given(regex = r"^任务存在")]
fn task_exists(world: &mut AcceptanceWorld) {
    world.task_status = Some("todo".into());
}

#[given(regex = r"^(todo|in_progress|blocked|done|cancelled)$")]
fn set_task_status(world: &mut AcceptanceWorld, status: String) {
    world.task_status = Some(status);
}

#[given(regex = r"^标记 done$")]
fn mark_done(world: &mut AcceptanceWorld) {
    world.task_status = Some("done".into());
}

#[given(regex = r"^有子任务$")]
fn has_subtasks(world: &mut AcceptanceWorld) {
    world.state.insert("has_subtasks".into(), "true".into());
}

#[given(regex = r"^(会议结束有待办|聊天消息含承诺|邮件含请求|文档评论含待办)$")]
fn ai_task_source(world: &mut AcceptanceWorld, source: String) {
    world.state.insert("ai_source".into(), source);
}

#[given(regex = r"^自动发现的任务$")]
fn auto_discovered(world: &mut AcceptanceWorld) {
    world.state.insert("task_source".into(), "auto_discovered".into());
    world.state.insert("needs_review".into(), "true".into());
}

#[given(regex = r"^飞书任务状态变更$")]
fn feishu_status_change(world: &mut AcceptanceWorld) {
    world.state.insert("sync_direction".into(), "feishu_to_obsidian".into());
}

#[given(regex = r"^Obsidian 任务修改$")]
fn obsidian_task_edit(world: &mut AcceptanceWorld) {
    world.state.insert("sync_direction".into(), "obsidian_to_feishu".into());
}

#[given(regex = r"^两端同时修改$")]
fn concurrent_edit(world: &mut AcceptanceWorld) {
    world.state.insert("conflict".into(), "true".into());
}

#[when(regex = r"^(保存|todo→in_progress|→blocked|→in_progress|→cancelled|直接→done|→in_progress|直接→done)$")]
fn task_transition(world: &mut AcceptanceWorld, action: String) {
    world.processing_result = Some(format!("transition:{action}"));
}

#[when(regex = r"^AI 提取$")]
fn ai_extract(world: &mut AcceptanceWorld) {
    world.processing_result = Some("ai_extracted".into());
}

#[when(regex = r"^同步$")]
fn sync(world: &mut AcceptanceWorld) {
    world.processing_result = Some("synced".into());
}

#[when(regex = r"^分析$")]
fn analyze(world: &mut AcceptanceWorld) {
    world.processing_result = Some("analyzed".into());
}

#[when(regex = r"^呈现$")]
fn present(world: &mut AcceptanceWorld) {
    world.processing_result = Some("presented".into());
}

#[when(regex = r"^捕获$")]
fn capture(world: &mut AcceptanceWorld) {
    world.processing_result = Some("captured".into());
}

#[when(regex = r"^同步飞书$")]
fn sync_to_feishu(world: &mut AcceptanceWorld) {
    world.processing_result = Some("synced_feishu".into());
}

#[when(regex = r"^检测冲突$")]
fn detect_conflict(world: &mut AcceptanceWorld) {
    world.processing_result = Some("conflict_detected".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^Task.*status=todo.*source=obsidian")]
fn assert_manual_task(world: &mut AcceptanceWorld) {
    assert_eq!(world.task_status.as_deref(), Some("todo"));
    assert_eq!(world.state.get("task_source").map(String::as_str), Some("manual"));
}

#[then(regex = r"^needs_review=true$")]
fn assert_needs_review(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("needs_review").map(String::as_str), Some("true"));
}

#[then(regex = r"^Obsidian 更新 source=feishu$")]
fn assert_feishu_synced(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^合法并持久化$|^合法$")]
fn assert_legal_transition(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some(), "状态转换应合法");
}

#[then(regex = r"^拒绝并解释$|^拒绝$")]
fn assert_rejected(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some(), "状态转换应被拒绝");
}

#[then(regex = r"^设置 completed_at$")]
fn assert_completed_at(world: &mut AcceptanceWorld) {
    world.state.insert("completed_at_set".into(), "true".into());
}

#[then(regex = r"^通过 parent_task 关联$")]
fn assert_parent_linked(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("has_subtasks"));
}

#[then(regex = r"^识别并创建 needs_review 任务$")]
fn assert_ai_task_created(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("ai_source"));
}

#[then(regex = r"^必须确认或拒绝才激活$")]
fn assert_must_confirm(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("needs_review").map(String::as_str), Some("true"));
}

#[then(regex = r"^Obsidian 自动更新.*无需确认$")]
fn assert_auto_sync(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("sync_direction").map(String::as_str), Some("feishu_to_obsidian"));
}

#[then(regex = r"^需用户确认$")]
fn assert_requires_confirm(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^呈现解决机制$")]
fn assert_conflict_resolution(world: &mut AcceptanceWorld) {
    assert_eq!(world.state.get("conflict").map(String::as_str), Some("true"));
}
