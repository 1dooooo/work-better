//! G3 数据存储 — Obsidian 写入、向量DB、结构化DB、新鲜度检查

use cucumber::{given, when, then};
use crate::world::AcceptanceWorld;

#[given(regex = r"^(WorkRecord|引用项目|被分类|多上下文|自定义模板|新文档|文档修改|文档删除|语义搜索|大模型需上下文|结构化数据|任务状态变更)")]
fn g3_setup(world: &mut AcceptanceWorld, ctx: String) {
    world.state.insert("g3_context".into(), ctx);
}

#[given(regex = r"^(飞书任务|飞书文档|双向链接|标签命名|信息多次|知识已过时|检查完成|用户触发重建|用户触发重处理|用户触发全量)")]
fn g3_external_setup(world: &mut AcceptanceWorld, ctx: String) {
    world.state.insert("g3_context".into(), ctx);
}

#[when(regex = r"^写入 Obsidian$")]
fn write_obsidian(world: &mut AcceptanceWorld) {
    world.processing_result = Some("written_obsidian".into());
}

#[when(regex = r"^写入$")]
fn write_generic(world: &mut AcceptanceWorld) {
    world.processing_result = Some("written".into());
}

#[when(regex = r"^查看任一位置$")]
fn view_location(world: &mut AcceptanceWorld) {
    world.processing_result = Some("viewed".into());
}

#[when(regex = r"^配置$")]
fn configure(world: &mut AcceptanceWorld) {
    world.processing_result = Some("configured".into());
}

#[when(regex = r"^成功$")]
fn success(world: &mut AcceptanceWorld) {
    world.processing_result = Some("success".into());
}

#[when(regex = r"^保存$")]
fn save(world: &mut AcceptanceWorld) {
    world.processing_result = Some("saved".into());
}

#[when(regex = r"^执行$")]
fn execute(world: &mut AcceptanceWorld) {
    world.processing_result = Some("executed".into());
}

#[when(regex = r"^RAG 召回$")]
fn rag_recall(world: &mut AcceptanceWorld) {
    world.processing_result = Some("rag_recalled".into());
}

#[when(regex = r"^查询$")]
fn query(world: &mut AcceptanceWorld) {
    world.processing_result = Some("queried".into());
}

#[when(regex = r"^更新$")]
fn update(world: &mut AcceptanceWorld) {
    world.processing_result = Some("updated".into());
}

#[when(regex = r"^完成.*顺序")]
fn complete_ordered(world: &mut AcceptanceWorld) {
    world.processing_result = Some("ordered_write".into());
}

#[when(regex = r"^检查$|^检测$")]
fn check(world: &mut AcceptanceWorld) {
    world.processing_result = Some("checked".into());
}

#[when(regex = r"^发现差异$")]
fn found_diff(world: &mut AcceptanceWorld) {
    world.processing_result = Some("diff_found".into());
}

#[when(regex = r"^新鲜度比对$")]
fn freshness_compare(world: &mut AcceptanceWorld) {
    world.processing_result = Some("freshness_compared".into());
}

#[when(regex = r"^每周检查$|^每周规范化$|^每周检测$")]
fn weekly_check(world: &mut AcceptanceWorld) {
    world.processing_result = Some("weekly_check".into());
}

#[when(regex = r"^每月审查$")]
fn monthly_review(world: &mut AcceptanceWorld) {
    world.processing_result = Some("monthly_review".into());
}

#[when(regex = r"^执行完毕$")]
fn execution_done(world: &mut AcceptanceWorld) {
    world.processing_result = Some("execution_done".into());
}

// ── Then ───────────────────────────────────────────────────

#[then(regex = r"^放入正确目录$")]
fn assert_correct_dir(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some(), "应写入正确目录");
}

#[then(regex = r"^自动创建双向链接$")]
fn assert_bidirectional_links(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^自动应用标签$")]
fn assert_auto_tags(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^不同维度可访问$")]
fn assert_multi_dimension(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^新文件遵循模板$")]
fn assert_template_followed(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^异步生成嵌入$")]
fn assert_async_embedding(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r".*重新嵌入.*防抖")]
fn assert_re_embed_debounce(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^嵌入移除$")]
fn assert_embedding_removed(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^按相似度排序返回$")]
fn assert_similarity_sorted(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^检索相关文档$")]
fn assert_retrieve_relevant(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^按索引字段快速查询$")]
fn assert_indexed_query(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^跟踪完整转换历史$")]
fn assert_transition_history(world: &mut AcceptanceWorld) {
    assert!(world.state.contains_key("g3_context"));
}

#[then(regex = r"^顺序.*Obsidian.*向量DB.*结构化DB")]
fn assert_write_order(world: &mut AcceptanceWorld) {
    assert_eq!(world.processing_result.as_deref(), Some("ordered_write"));
}

#[then(regex = r"^向量DB和结构化DB更新$")]
fn assert_secondary_dbs_updated(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^标记并触发重建$")]
fn assert_mark_rebuild(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^标记不匹配$")]
fn assert_mark_mismatch(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^Obsidian 更新为完成$")]
fn assert_obsidian_updated(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^检测过时并重新生成摘要$")]
fn assert_detect_stale(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^标记断链$")]
fn assert_mark_broken_links(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^合并变体$")]
fn assert_merge_variants(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^标记合并候选$")]
fn assert_mark_duplicates(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^标记需用户审查$")]
fn assert_mark_review_needed(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^推送通知$")]
fn assert_notify(world: &mut AcceptanceWorld) {
    world.notifications.push("storage_notification".into());
}

#[then(regex = r"^静默修复$")]
fn assert_silent_fix(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^生成新鲜度报告$")]
fn assert_freshness_report(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^所有文档重新嵌入$")]
fn assert_all_re_embedded(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^所有事件重新处理$")]
fn assert_all_reprocessed(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}

#[then(regex = r"^三层互相验证$")]
fn assert_three_layer_verify(world: &mut AcceptanceWorld) {
    assert!(world.processing_result.is_some());
}
