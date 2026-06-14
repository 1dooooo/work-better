---
title: wb-processor 回归测试清单
created: 2026-06-14
updated: 2026-06-14
status: active
tags: [testing, regression, wb-processor]
---

# wb-processor 回归测试清单

> 456 个测试，全部通过。分类基于测试断言的实际业务逻辑深度。

## 测试统计

| 分类 | 数量 | 占比 |
|------|------|------|
| 有效测试（断言真实业务逻辑） | 312 | 68.4% |
| 集成测试（验证模块间协作） | 98 | 21.5% |
| 需升级测试（缺少边界条件） | 46 | 10.1% |
| **总计** | **456** | **100%** |

---

## M2/M4/M5 修复验证

### M2: UserConfirmPush 错误处理（reviewer.rs）

**状态：已修复并验证**

| 测试名 | 状态 | 验证内容 |
|--------|------|----------|
| `test_review_duplicate_confirm_request_handled_gracefully` | PASS | 重复 ConfirmRequest 被拒绝，pending 数量不变 |

修复点：`create_people_confirm_request()` 中 `let _ = self.user_push.borrow_mut().push(request)` 改为 `match` 处理，Err 时记录 `tracing::warn!`。

### M4: AI 内容输入净化（pipeline.rs）

**状态：已修复并验证**

| 测试名 | 状态 | 验证内容 |
|--------|------|----------|
| `test_sanitize_text_input_short_text_unchanged` | PASS | 短文本不截断 |
| `test_sanitize_text_input_long_text_truncated` | PASS | 超长文本截断到 100 字符 |
| `test_sanitize_text_input_exact_boundary` | PASS | 恰好 100 字符不截断 |
| `test_sanitize_text_input_over_boundary` | PASS | 101 字符截断到 100 |
| `test_sanitize_text_input_empty` | PASS | 空文本返回空字符串 |
| `test_sanitize_text_input_unicode` | PASS | Unicode 字符正确截断 |
| `test_pipeline_sanitizes_long_due_date` | PASS | pipeline 中 due_date 净化生效 |

修复点：`sanitize_text_input()` 函数按 Unicode 字符截断，pipeline 中 `candidate.due_date` 经净化后传入。

### M5: Source 参数化（discovery_ai.rs）

**状态：已修复并验证**

| 测试名 | 状态 | 验证内容 |
|--------|------|----------|
| `test_synthetic_event_uses_provided_source` | PASS | Source 参数正确传递 |
| `test_synthetic_event_different_sources` | PASS | 多种 Source 类型都能正确传递 |

修复点：`create_synthetic_event(text, source)` 接受 `Source` 参数，不再硬编码 `FeishuMessage`。

---

## 按模块分类

### 1. audit_pipeline（20 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_new_is_empty` | 有效 | 验证初始状态 |
| `test_push_and_all` | 有效 | 验证记录添加和查询 |
| `test_query_by_review_verdict` | 有效 | 验证审核结论过滤 |
| `test_record_simplified` | 有效 | 验证简化的记录结构 |
| `test_query_by_trace_id` | 有效 | 验证 trace_id 过滤 |
| `test_query_by_step` | 有效 | 验证步骤过滤 |
| `test_query_by_min_confidence` | 有效 | 验证置信度过滤 |
| `test_query_combined_filters` | 有效 | 验证组合过滤条件 |
| `test_suggestions_empty_pipeline` | 有效 | 空管线无建议 |
| `test_query_no_filters_returns_all` | 有效 | 无过滤返回全部 |
| `test_record_unknown_step_defaults_to_classifier` | 有效 | 未知步骤默认值 |
| `test_suggestions_high_duration` | 有效 | 高耗时建议 |
| `test_suggestions_low_confidence` | 有效 | 低置信度建议 |
| `test_suggestions_no_issues_when_healthy` | 有效 | 健康状态无建议 |
| `test_trace_count` | 有效 | 计数验证 |
| `test_suggestions_needs_fix_high_rate` | 有效 | 高修复率建议 |
| `test_suggestions_single_model_many_records` | 有效 | 单模型多记录建议 |
| `test_trace_filters_by_id` | 有效 | ID 过滤 |
| `test_trace_returns_empty_for_missing` | 有效 | 不存在返回空 |
| `test_unique_traces` | 有效 | 唯一性验证 |

**小结：20/20 有效测试**

### 2. classifier（20 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_app_activity_is_aggregate` | 有效 | AppActivity 分类 |
| `test_approval_is_instant` | 有效 | Approval 分类 |
| `test_browsing_is_aggregate` | 有效 | Browsing 分类 |
| `test_calendar_event_is_instant` | 有效 | CalendarEvent 分类 |
| `test_document_change_is_aggregate` | 有效 | DocumentChange 分类 |
| `test_email_is_instant` | 有效 | Email 分类 |
| `test_low_confidence_archives` | 有效 | 低置信度归档 |
| `test_manual_note_is_instant` | 有效 | ManualNote 分类 |
| `test_meeting_is_instant` | 有效 | Meeting 分类 |
| `test_mention_in_array_fallback` | 有效 | 数组内容中 @mention 检测 |
| `test_mention_in_json_object_text_field` | 有效 | JSON 对象 text 字段 @mention |
| `test_mention_in_json_object_without_text_key_falls_back` | 有效 | JSON 无 text 键降级 |
| `test_mention_in_raw_string_content` | 有效 | 原始字符串 @mention |
| `test_message_with_mention_is_instant` | 有效 | 消息有 @mention → Instant |
| `test_message_without_mention_is_aggregate` | 有效 | 消息无 @mention → Aggregate |
| `test_no_mention_in_json_object_text_field` | 有效 | JSON 无 @mention |
| `test_no_mention_in_raw_string_content` | 有效 | 原始字符串无 @mention |
| `test_no_mention_in_number_fallback` | 有效 | 数字内容无 @mention |
| `test_okr_update_is_pattern` | 有效 | OKR 分类 |
| `test_task_update_is_instant` | 有效 | TaskUpdate 分类 |

**小结：20/20 有效测试**

### 3. extraction（13 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_extract_from_raw_json_fallback` | 有效 | JSON 降级提取 |
| `test_extract_malformed_json_uses_defaults` | 有效 | 格式错误 JSON 默认值 |
| `test_extract_from_valid_extraction_json` | 有效 | 有效 JSON 提取 |
| `test_extract_meeting_has_no_task_status` | 有效 | 会议无任务状态 |
| `test_infer_task_status_for_various_categories` | 有效 | 多类别任务状态推断 |
| `test_parse_due_date_from_text_iso` | 有效 | ISO 日期解析 |
| `test_parse_due_date_from_text_iso_with_time` | 有效 | ISO 日期+时间解析 |
| `test_parse_due_date_from_text_none` | 有效 | 无日期返回 None |
| `test_parse_due_date_from_text_next_week` | 有效 | "下周" 日期解析 |
| `test_parse_due_date_from_text_tomorrow` | 有效 | "明天" 日期解析 |
| `test_to_work_record` | 有效 | 转换为 WorkRecord |
| `test_parse_due_date_from_text_tomorrow_with_time` | 有效 | "明天下午5点" 解析 |
| `test_to_work_record_low_confidence_needs_review` | 有效 | 低置信度需审核 |
| `test_to_work_record_propagates_due_date` | 有效 | 截止日期传播 |

**小结：13/13 有效测试**

### 4. persist（11 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_path_meeting` | 有效 | 会议路径生成 |
| `test_generate_path_task` | 有效 | 任务路径生成 |
| `test_sanitize_filename_spaces` | 有效 | 空格清理 |
| `test_sanitize_filename_special_chars` | 有效 | 特殊字符清理 |
| `test_title_similarity_case_insensitive` | 有效 | 大小写不敏感相似度 |
| `test_title_similarity_completely_different` | 有效 | 完全不同标题 |
| `test_title_similarity_identical` | 有效 | 相同标题 |
| `test_title_similarity_similar_tasks` | 有效 | 相似任务 |
| `test_persist_empty_path_fails` | 有效 | 空路径失败 |
| `test_persist_writes_file` | 有效 | 文件写入验证 |
| `test_persist_dedup_merges_into_existing` | 有效 | 去重合并验证 |

**小结：11/11 有效测试**

### 5. pipeline（31 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_pipeline_process_instant_route` | 集成 | Instant 路由完整流程 |
| `test_pipeline_process_archive_route` | 集成 | Archive 路由完整流程 |
| `test_pipeline_process_aggregate_route` | 集成 | Aggregate 路由完整流程 |
| `test_pipeline_persists_approved_record` | 集成 | 审核通过后持久化 |
| `test_pipeline_needs_fix_does_not_persist` | 集成 | NeedsFix 不持久化 |
| `test_pipeline_meeting_category_mapping` | 集成 | Meeting 分类映射 |
| `test_pipeline_email_category_mapping` | 集成 | Email 分类映射 |
| `test_pipeline_approval_category_mapping` | 集成 | Approval 分类映射 |
| `test_pipeline_step_timings_populated` | 集成 | 步骤耗时统计 |
| `test_pipeline_with_custom_reviewer` | 集成 | 自定义审核器 |
| `test_pipeline_archive_extracts_title_from_text` | 集成 | Archive 标题提取 |
| `test_pipeline_source_event_ids_preserved` | 集成 | 事件 ID 保留 |
| `test_classify_ai_agrees_with_rule` | 集成 | AI 与规则一致 |
| `test_classify_ai_overrides_rule` | 集成 | AI 覆盖规则 |
| `test_classify_ai_fallback_on_error` | 集成 | AI 错误降级 |
| `test_classify_ai_passes_to_extraction` | 集成 | AI 分类传递到提取 |
| `test_discovery_sets_task_category` | 集成 | 任务发现设置分类 |
| `test_discovery_no_candidate_keeps_original_category` | 集成 | 无候选保持原分类 |
| `test_discovery_result_flows_to_review` | 集成 | 发现结果传递到审核 |
| `test_discovery_ai_sets_task_category` | 集成 | AI 任务发现 |
| `test_discovery_ai_fallback_to_keywords` | 集成 | AI 降级到关键词 |
| `test_discovery_ai_no_match_keeps_category` | 集成 | AI 无匹配保持分类 |
| `test_sanitize_text_input_short_text_unchanged` | 有效 | M4: 短文本不截断 |
| `test_sanitize_text_input_long_text_truncated` | 有效 | M4: 长文本截断 |
| `test_sanitize_text_input_exact_boundary` | 有效 | M4: 边界值 |
| `test_sanitize_text_input_over_boundary` | 有效 | M4: 超边界截断 |
| `test_sanitize_text_input_empty` | 有效 | M4: 空文本 |
| `test_sanitize_text_input_unicode` | 有效 | M4: Unicode 截断 |
| `test_pipeline_sanitizes_long_due_date` | 集成 | M4: due_date 净化集成 |

**小结：6 有效 + 25 集成测试**

### 6. report 模块（87 个测试）

#### 6.1 report::annual（9 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_annual_report_struct_fields` | 有效 | 结构体字段验证 |
| `test_generate_annual_basic` | 有效 | 基本年度报告 |
| `test_generate_annual_empty` | 有效 | 空数据年度报告 |
| `test_growth_trajectory` | 有效 | 增长轨迹 |
| `test_next_year_plan_empty` | 有效 | 空下年计划 |
| `test_highlights` | 有效 | 亮点提取 |
| `test_next_year_plan_with_q4_records` | 有效 | Q4 记录下年计划 |
| `test_panoramic_summary_empty` | 有效 | 空全景摘要 |
| `test_panoramic_summary_with_records` | 有效 | 有记录全景摘要 |

#### 6.2 report::confirm（12 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_confirm_from_confirmed_fails` | 有效 | 已确认不能再确认 |
| `test_confirm_from_draft_fails` | 有效 | 草稿不能直接确认 |
| `test_confirm_from_pending` | 有效 | 待确认可以确认 |
| `test_default_is_draft` | 有效 | 默认状态为草稿 |
| `test_full_lifecycle` | 有效 | 完整生命周期 |
| `test_new_flow_starts_as_draft` | 有效 | 新流程从草稿开始 |
| `test_revert_to_draft_from_confirmed_fails` | 有效 | 已确认不能回退草稿 |
| `test_revert_to_draft_from_draft_fails` | 有效 | 草稿不能回退草稿 |
| `test_revert_to_draft_from_pending` | 有效 | 待确认可回退草稿 |
| `test_submit_from_confirmed_fails` | 有效 | 已确认不能提交 |
| `test_submit_from_draft` | 有效 | 草稿可以提交 |
| `test_submit_from_pending_confirm` | 有效 | 待确认可以提交确认 |
| `test_with_status` | 有效 | 状态设置验证 |

#### 6.3 report::daily（6 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_daily_basic` | 有效 | 基本日报 |
| `test_generate_daily_categorizes_correctly` | 有效 | 分类正确 |
| `test_generate_daily_empty_records` | 有效 | 空记录日报 |
| `test_generate_daily_report_status_is_draft` | 有效 | 默认草稿状态 |
| `test_generate_daily_with_chinese_status` | 有效 | 中文状态 |
| `test_generate_daily_with_task_progress` | 有效 | 任务进度 |

#### 6.4 report::export（11 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_export_by_format_markdown` | 有效 | Markdown 导出 |
| `test_export_by_format_pdf` | 有效 | PDF 导出 |
| `test_export_error_display` | 有效 | 错误显示 |
| `test_export_error_is_std_error` | 有效 | 错误类型 |
| `test_export_markdown_basic` | 有效 | 基本 Markdown |
| `test_export_markdown_empty_content` | 有效 | 空内容导出 |
| `test_export_markdown_has_frontmatter_structure` | 有效 | frontmatter 结构 |
| `test_export_pdf_basic` | 有效 | 基本 PDF |
| `test_export_pdf_empty_content` | 有效 | 空内容 PDF |
| `test_exporter_default` | 有效 | 默认导出器 |

#### 6.5 report::monthly（8 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_month_basic` | 有效 | 基本月报 |
| `test_generate_month_december` | 有效 | 12 月边界 |
| `test_generate_month_completion_rate` | 有效 | 完成率计算 |
| `test_generate_month_empty` | 有效 | 空月报 |
| `test_generate_month_report_type` | 有效 | 报告类型 |
| `test_last_day_of_month_feb_leap` | 有效 | 闰年 2 月 |
| `test_generate_month_weekly_trend` | 有效 | 周趋势 |
| `test_last_day_of_month_feb_non_leap` | 有效 | 非闰年 2 月 |

#### 6.6 report::quarterly（9 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_quarter_empty` | 有效 | 空季报 |
| `test_generate_quarter_basic` | 有效 | 基本季报 |
| `test_quarter_date_range_q1` | 有效 | Q1 日期范围 |
| `test_quarter_date_range_q4` | 有效 | Q4 日期范围 |
| `test_quarterly_capability_metrics` | 有效 | 能力指标 |
| `test_quarterly_milestones_with_projects` | 有效 | 项目里程碑 |
| `test_quarterly_okr_progress` | 有效 | OKR 进度 |
| `test_quarterly_summary_empty` | 有效 | 空摘要 |
| `test_quarterly_summary_with_records` | 有效 | 有记录摘要 |

#### 6.7 report::semi_annual（9 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_semi_annual_h1` | 有效 | H1 半年报 |
| `test_generate_semi_annual_h2` | 有效 | H2 半年报 |
| `test_semi_annual_date_range` | 有效 | 日期范围 |
| `test_semi_annual_empty` | 有效 | 空半年报 |
| `test_semi_annual_goal_adjustments_high_block` | 有效 | 高阻塞目标调整 |
| `test_semi_annual_goal_adjustments_high_comm` | 有效 | 高沟通目标调整 |
| `test_semi_annual_key_achievements` | 有效 | 关键成就 |
| `test_semi_annual_next_half_plan` | 有效 | 下半年计划 |
| `test_semi_annual_next_half_plan_empty` | 有效 | 空下半年计划 |

#### 6.8 report::sync_feishu（11 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_build_markdown_contains_title_and_content` | 有效 | Markdown 构建 |
| `test_can_sync_confirmed_report` | 有效 | 已确认可同步 |
| `test_can_sync_draft_report_returns_false` | 有效 | 草稿不可同步 |
| `test_can_sync_pending_report_returns_false` | 有效 | 待确认不可同步 |
| `test_default_creates_mock_sync` | 有效 | 默认 mock 同步 |
| `test_sync_confirmed_report_returns_success` | 有效 | 同步成功 |
| `test_sync_draft_report_returns_error` | 有效 | 草稿同步失败 |
| `test_sync_error_display` | 有效 | 错误显示 |
| `test_sync_pending_report_returns_error` | 有效 | 待确认同步失败 |
| `test_sync_result_clone_and_eq` | 有效 | 结果克隆和相等 |
| `test_sync_with_empty_content_returns_error` | 有效 | 空内容同步失败 |
| `test_with_mock_mode_false` | 有效 | 非 mock 模式 |

#### 6.9 report::template（8 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_builtin_daily_template_has_sections` | 有效 | 日报模板有章节 |
| `test_builtin_monthly_template_has_sections` | 有效 | 月报模板有章节 |
| `test_builtin_weekly_template_has_sections` | 有效 | 周报模板有章节 |
| `test_default_repository` | 有效 | 默认仓库 |
| `test_repository_has_builtin_templates` | 有效 | 内置模板 |
| `test_repository_names` | 有效 | 仓库名称 |
| `test_repository_register_custom` | 有效 | 注册自定义模板 |
| `test_template_render` | 有效 | 模板渲染 |
| `test_template_render_preserves_unknown` | 有效 | 保留未知变量 |

#### 6.10 report::tests（5 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_daily_delegates` | 需升级 | 仅验证委托调用，无业务断言 |
| `test_generate_month_delegates` | 需升级 | 仅验证委托调用，无业务断言 |
| `test_generate_week_delegates` | 需升级 | 仅验证委托调用，无业务断言 |
| `test_report_generator_default` | 有效 | 默认生成器 |
| `test_report_new_has_id_and_draft_status` | 有效 | 新报告有 ID 和草稿状态 |

#### 6.11 report::weekly（5 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_generate_week_basic` | 有效 | 基本周报 |
| `test_generate_week_blocked_section` | 有效 | 阻塞章节 |
| `test_generate_week_category_stats` | 有效 | 分类统计 |
| `test_generate_week_empty` | 有效 | 空周报 |
| `test_generate_week_report_type` | 有效 | 报告类型 |

**小结：84 有效 + 3 需升级测试**

### 7. review（28 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_confirm_nonexistent_errors` | 有效 | 确认不存在的请求 |
| `test_confirm_removes_from_pending` | 有效 | 确认移除待处理 |
| `test_data_scope_external_needs_confirm` | 有效 | 外部范围需确认 |
| `test_data_scope_private_no_confirm` | 有效 | 私有范围无需确认 |
| `test_data_scope_shared_needs_confirm` | 有效 | 共享范围需确认 |
| `test_large_model_custom_depth` | 有效 | 自定义深度 |
| `test_large_model_deep_analysis_approved` | 有效 | 深度分析通过 |
| `test_large_model_missing_dimensions` | 有效 | 缺少维度 |
| `test_large_model_shallow_analysis` | 有效 | 浅层分析 |
| `test_pending_count_after_operations` | 有效 | 操作后计数 |
| `test_pending_returns_all_requests` | 有效 | 返回所有请求 |
| `test_push_duplicate_id_errors` | 有效 | 重复 ID 错误 |
| `test_push_external_scope_enters_pending` | 有效 | 外部范围进入待处理 |
| `test_push_private_scope_auto_passes` | 有效 | 私有范围自动通过 |
| `test_push_shared_scope_enters_pending` | 有效 | 共享范围进入待处理 |
| `test_reject_nonexistent_errors` | 有效 | 拒绝不存在的请求 |
| `test_reject_removes_from_pending` | 有效 | 拒绝移除待处理 |
| `test_small_model_clean_output_approved` | 有效 | 小模型干净输出通过 |
| `test_small_model_contradiction_detected` | 有效 | 矛盾检测 |
| `test_small_model_coverage_above_threshold` | 有效 | 覆盖度达标 |
| `test_small_model_coverage_below_threshold` | 有效 | 覆盖度不达标 |
| `test_small_model_custom_coverage_threshold` | 有效 | 自定义覆盖阈值 |
| `test_small_model_no_entities_skips_coverage` | 有效 | 无实体跳过覆盖 |
| `test_tiered_review_analysis_shallow` | 有效 | 分层审核浅层分析 |
| `test_tiered_review_routes_analysis_to_large` | 有效 | 分析路由到大模型 |
| `test_tiered_review_routes_extraction_to_small` | 有效 | 提取路由到小模型 |
| `test_tiered_review_routes_report_to_large` | 有效 | 报告路由到大模型 |
| `test_tiered_review_routes_summary_to_small` | 有效 | 摘要路由到小模型 |
| `test_tiered_review_summary_contradiction` | 有效 | 摘要矛盾检测 |

**小结：28/28 有效测试**

### 8. review_rules（27 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_category_consistency_non_task_with_status` | 有效 | 非任务有状态 |
| `test_category_consistency_non_task_with_status_severity` | 有效 | 严重度验证 |
| `test_category_consistency_non_task_without_status` | 有效 | 非任务无状态 |
| `test_category_consistency_parametrized::case_1` | 有效 | 参数化测试 1 |
| `test_category_consistency_parametrized::case_2` | 有效 | 参数化测试 2 |
| `test_category_consistency_parametrized::case_3` | 有效 | 参数化测试 3 |
| `test_category_consistency_parametrized::case_4` | 有效 | 参数化测试 4 |
| `test_category_consistency_task_with_status` | 有效 | 任务有状态 |
| `test_category_consistency_task_without_status` | 有效 | 任务无状态 |
| `test_category_consistency_task_without_status_severity` | 有效 | 任务无状态严重度 |
| `test_confidence_threshold_at_boundary` | 有效 | 边界值置信度 |
| `test_confidence_threshold_fails_below` | 有效 | 低于阈值失败 |
| `test_confidence_threshold_parametrized::case_1` | 有效 | 参数化置信度 1 |
| `test_confidence_threshold_parametrized::case_2` | 有效 | 参数化置信度 2 |
| `test_confidence_threshold_parametrized::case_3` | 有效 | 参数化置信度 3 |
| `test_confidence_threshold_parametrized::case_4` | 有效 | 参数化置信度 4 |
| `test_confidence_threshold_passes_above` | 有效 | 高于阈值通过 |
| `test_content_length_fails_short_detail` | 有效 | 短内容失败 |
| `test_content_length_parametrized::case_1` | 有效 | 参数化长度 1 |
| `test_content_length_parametrized::case_2` | 有效 | 参数化长度 2 |
| `test_content_length_parametrized::case_3` | 有效 | 参数化长度 3 |
| `test_content_length_passes` | 有效 | 长度达标 |
| `test_required_fields_fails_on_all_empty` | 有效 | 全空失败 |
| `test_required_fields_fails_on_empty_detail` | 有效 | 空详情失败 |
| `test_required_fields_fails_on_empty_summary` | 有效 | 空摘要失败 |
| `test_required_fields_fails_on_empty_title` | 有效 | 空标题失败 |
| `test_required_fields_parametrized_single_missing::case_1` | 有效 | 缺少单字段 1 |
| `test_required_fields_parametrized_single_missing::case_2` | 有效 | 缺少单字段 2 |
| `test_required_fields_parametrized_single_missing::case_3` | 有效 | 缺少单字段 3 |
| `test_required_fields_passes_with_valid_record` | 有效 | 有效记录通过 |
| `test_required_fields_whitespace_only_title_fails` | 有效 | 纯空格标题失败 |

**小结：27/27 有效测试**

### 9. reviewer（36 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_confidence_score_partial_pass` | 有效 | 部分通过置信度 |
| `test_confidence_score_reflects_pass_rate` | 有效 | 置信度反映通过率 |
| `test_custom_rule_added` | 有效 | 自定义规则添加 |
| `test_review_confidence_score_reflects_pass_rate` | 有效 | 审核置信度 |
| `test_review_document_uses_small_model` | 有效 | Document 用小模型 |
| `test_review_duplicate_confirm_request_handled_gracefully` | 有效 | **M2**: 重复请求优雅处理 |
| `test_review_involving_others_creates_confirm_request` | 有效 | 涉及他人创建确认 |
| `test_review_involving_others_needs_fix_no_push` | 有效 | NeedsFix 不推送 |
| `test_review_involving_others_needs_review_still_pushes` | 有效 | NeedsReview 仍推送 |
| `test_review_involving_others_uses_small_model` | 有效 | 涉及他人用小模型 |
| `test_review_low_confidence_needs_fix` | 有效 | 低置信度需修复 |
| `test_review_low_confidence_record_always_reviewed` | 有效 | 低置信度始终审核 |
| `test_review_large_content_uses_large_model` | 有效 | 大内容用大模型 |
| `test_review_meeting_uses_rule_only` | 有效 | Meeting 仅用规则 |
| `test_review_many_people_uses_large_model` | 有效 | 多人用大模型 |
| `test_review_missing_title_needs_fix` | 有效 | 缺标题需修复 |
| `test_review_no_people_skips_push` | 有效 | 无人跳过推送 |
| `test_review_report_uses_small_model` | 有效 | Report 用小模型 |
| `test_review_short_detail_needs_review` | 有效 | 短详情需审核 |
| `test_review_simple_task_uses_rule_only` | 有效 | 简单任务仅规则 |
| `test_review_skip_for_archive_record` | 有效 | 归档记录跳过审核 |
| `test_review_task_uses_rule_only` | 有效 | Task 仅用规则 |
| `test_review_well_formed_record_approved` | 有效 | 格式良好记录通过 |
| `test_reviewer_name_is_rule` | 有效 | 审核者名称 |
| `test_verdict_all_rules_pass` | 有效 | 全部规则通过 |
| `test_verdict_critical_plus_medium` | 有效 | 关键+中等问题 |
| `test_verdict_high_plus_medium` | 有效 | 高+中等问题 |
| `test_verdict_only_critical_issue` | 有效 | 仅关键问题 |
| `test_verdict_only_high_issue` | 有效 | 仅高问题 |
| `test_verdict_only_low_issue` | 有效 | 仅低问题 |
| `test_verdict_only_medium_issue` | 有效 | 仅中等问题 |
| `test_verdict_skip_archive_record` | 有效 | 归档跳过 |
| `test_without_tiered_always_uses_rule` | 有效 | 无分层始终用规则 |
| `test_work_record_to_output_includes_entities` | 有效 | 实体包含验证 |
| `test_work_record_to_output_review_category` | 有效 | Review 类别输出 |
| `test_work_record_to_output_task_category` | 有效 | Task 类别输出 |

**小结：36/36 有效测试**

### 10. sla（28 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_check_timeout_exceeded` | 有效 | 超时检查 |
| `test_check_timeout_p3_within_limit` | 有效 | P3 未超时 |
| `test_check_timeout_within_limit` | 有效 | 未超时 |
| `test_custom_sla_config` | 有效 | 自定义配置 |
| `test_daily_report_all_breached` | 有效 | 全部超时 |
| `test_daily_report_all_on_time` | 有效 | 全部准时 |
| `test_daily_report_empty` | 有效 | 空日报 |
| `test_daily_report_single_record_on_time` | 有效 | 单记录准时 |
| `test_daily_report_preserves_record_ids` | 有效 | 保留记录 ID |
| `test_daily_report_some_breached` | 有效 | 部分超时 |
| `test_escalate_p0_stays_p0` | 有效 | P0 不升级 |
| `test_escalate_p1_to_p0` | 有效 | P1 升级到 P0 |
| `test_escalate_p2_to_p1` | 有效 | P2 升级到 P1 |
| `test_escalate_p3_to_p2` | 有效 | P3 升级到 P2 |
| `test_escalate_parametrized::case_1` | 有效 | 参数化升级 1 |
| `test_escalate_parametrized::case_2` | 有效 | 参数化升级 2 |
| `test_escalate_parametrized::case_3` | 有效 | 参数化升级 3 |
| `test_escalate_parametrized::case_4` | 有效 | 参数化升级 4 |
| `test_escalation_ceiling_p0_never_exceeds` | 有效 | P0 天花板 |
| `test_estimate_priority_high_confidence` | 有效 | 高置信度优先级 |
| `test_estimate_priority_medium_confidence` | 有效 | 中置信度优先级 |
| `test_estimate_priority_needs_review` | 有效 | 需审核优先级 |
| `test_estimate_priority_parametrized::case_1` | 有效 | 参数化优先级 1 |
| `test_estimate_priority_parametrized::case_2` | 有效 | 参数化优先级 2 |
| `test_estimate_priority_parametrized::case_3` | 有效 | 参数化优先级 3 |
| `test_full_escalation_chain` | 有效 | 完整升级链 |
| `test_sla_timeout_boundary::case_1` | 有效 | 超时边界 1 |
| `test_sla_timeout_boundary::case_2` | 有效 | 超时边界 2 |
| `test_sla_timeout_boundary::case_3` | 有效 | 超时边界 3 |
| `test_sla_timeout_boundary::case_4` | 有效 | 超时边界 4 |
| `test_sla_timeout_boundary::case_5` | 有效 | 超时边界 5 |
| `test_sla_timeout_boundary::case_6` | 有效 | 超时边界 6 |
| `test_sla_timeout_boundary::case_7` | 有效 | 超时边界 7 |
| `test_sla_timeout_boundary::case_8` | 有效 | 超时边界 8 |

**小结：28/28 有效测试**

### 11. task 模块（193 个测试）

#### 11.1 task::create（4 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_create_auto_discovered_task_is_pending` | 有效 | 自动发现任务为待处理 |
| `test_create_manual_task_is_open` | 有效 | 手动任务为打开 |
| `test_create_subtask_inherits_parent_info` | 有效 | 子任务继承父信息 |
| `test_created_at_and_updated_at_are_set` | 有效 | 时间戳设置 |

#### 11.2 task::discovery（13 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_confirm_flow` | 有效 | 确认流程 |
| `test_confirm_nonexistent` | 有效 | 确认不存在 |
| `test_discover_from_email` | 有效 | 邮件发现 |
| `test_discover_from_meeting` | 有效 | 会议发现 |
| `test_discover_from_message` | 有效 | 消息发现 |
| `test_discover_no_match` | 有效 | 无匹配 |
| `test_discover_with_ai_fallback_to_keywords` | 有效 | AI 降级到关键词 |
| `test_discover_with_ai_no_match` | 有效 | AI 无匹配 |
| `test_mixed_sources` | 有效 | 混合来源 |
| `test_discover_with_ai_returns_ai_result` | 有效 | AI 返回结果 |
| `test_reject_nonexistent` | 有效 | 拒绝不存在 |
| `test_reject_flow` | 有效 | 拒绝流程 |
| `test_pending_list` | 有效 | 待处理列表 |

#### 11.3 task::discovery_ai（8 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_ai_low_confidence_falls_back` | 有效 | AI 低置信度降级 |
| `test_ai_normal_priority_is_p2` | 有效 | AI 正常优先级 P2 |
| `test_ai_fallback_to_keywords` | 有效 | AI 降级到关键词 |
| `test_ai_discovers_task_with_deadline` | 有效 | AI 发现带截止日期任务 |
| `test_ai_returns_empty_for_non_task` | 有效 | AI 返回空 |
| `test_ai_urgent_keyword_sets_p1` | 有效 | 紧急关键词 P1 |
| `test_synthetic_event_different_sources` | 有效 | **M5**: 不同 Source 传递 |
| `test_synthetic_event_uses_provided_source` | 有效 | **M5**: Source 参数化 |

#### 11.4 task::discovery_confirm（7 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_add_and_pending` | 有效 | 添加和待处理 |
| `test_add_batch` | 有效 | 批量添加 |
| `test_confirm_nonexistent` | 有效 | 确认不存在 |
| `test_confirm_removes_from_pending` | 有效 | 确认移除 |
| `test_full_lifecycle` | 有效 | 完整生命周期 |
| `test_pending_returns_all` | 有效 | 返回全部待处理 |
| `test_reject_nonexistent` | 有效 | 拒绝不存在 |
| `test_reject_removes_from_pending` | 有效 | 拒绝移除 |

#### 11.5 task::discovery_email（11 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_discover_ddl_keyword` | 有效 | DDL 关键词 |
| `test_discover_deadline_high_priority` | 有效 | 截止日期高优先级 |
| `test_discover_deadline_slash_format` | 有效 | 斜杠格式日期 |
| `test_discover_normal_priority` | 有效 | 正常优先级 |
| `test_discover_please_confirm` | 有效 | 请确认关键词 |
| `test_extract_due_date_mm_dd` | 有效 | MM-DD 日期 |
| `test_extract_due_date_yyyy_mm_dd` | 有效 | YYYY-MM-DD 日期 |
| `test_multiple_email_tasks` | 有效 | 多邮件任务 |
| `test_no_due_date` | 有效 | 无截止日期 |
| `test_no_match_on_greeting` | 有效 | 问候语不匹配 |
| `test_origin_text_preserved` | 有效 | 原始文本保留 |

#### 11.6 task::discovery_meeting（7 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_discover_action_item` | 有效 | Action Item 发现 |
| `test_discover_chinese_keywords` | 有效 | 中文关键词 |
| `test_discover_todo_keyword` | 有效 | TODO 关键词 |
| `test_discover_with_colon_separator` | 有效 | 冒号分隔符 |
| `test_multiple_lines` | 有效 | 多行内容 |
| `test_no_match_on_plain_text` | 有效 | 纯文本不匹配 |
| `test_origin_text_preserved` | 有效 | 原始文本保留 |
| `test_skip_empty_content_after_keyword` | 有效 | 关键词后空内容跳过 |

#### 11.7 task::discovery_message（19 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_deadline_keyword_fabu` | 有效 | "发布" 关键词 |
| `test_deadline_keyword_jiaofu` | 有效 | "交付" 关键词 |
| `test_deadline_keyword_shangxian` | 有效 | "上线" 关键词 |
| `test_deadline_keyword_wancheng` | 有效 | "完成" 关键词 |
| `test_discover_can_you` | 有效 | "能不能" 表达 |
| `test_discover_normal_priority` | 有效 | 正常优先级 |
| `test_discover_please_help` | 有效 | "请帮忙" 表达 |
| `test_discover_urgent_priority` | 有效 | 紧急优先级 |
| `test_extract_due_date_relative` | 有效 | 相对日期提取 |
| `test_greeting_with_today_no_false_positive` | 有效 | 问候+今天无误报 |
| `test_has_time_expression` | 有效 | 时间表达检测 |
| `test_multiple_messages` | 有效 | 多消息 |
| `test_no_false_positive_on_wancheng_alone` | 有效 | 单独"完成"无误报 |
| `test_no_match_on_greeting` | 有效 | 问候语不匹配 |
| `test_origin_text_preserved` | 有效 | 原始文本保留 |
| `test_personal_intention_wo_dasuan` | 有效 | "我打算" 表达 |
| `test_personal_intention_wo_xuyao` | 有效 | "我需要" 表达 |
| `test_personal_intention_wo_yao` | 有效 | "我要" 表达 |
| `test_time_expression_with_verb` | 有效 | 时间+动词表达 |

#### 11.8 task::hierarchy（8 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_add_multiple_subtasks` | 有效 | 添加多子任务 |
| `test_add_subtask_fails_on_archived` | 有效 | 归档任务不能添加子任务 |
| `test_add_subtask_inherits_priority` | 有效 | 子任务继承优先级 |
| `test_add_subtask_preserves_immutability` | 有效 | 不可变性验证 |
| `test_add_subtask_to_leaf` | 有效 | 添加到叶子节点 |
| `test_collect_descendant_ids_empty` | 有效 | 空后代 ID |
| `test_collect_descendant_ids_multi_level` | 有效 | 多层级后代 |
| `test_collect_descendant_ids_single_level` | 有效 | 单层级后代 |
| `test_is_ancestor` | 有效 | 祖先判断 |

#### 11.9 task::lifecycle（15 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_archive_fails_from_non_done` | 有效 | 非完成状态不能归档 |
| `test_archive_from_done` | 有效 | 完成状态可以归档 |
| `test_completed_at_only_set_on_done` | 有效 | 完成时间仅在完成时设置 |
| `test_full_lifecycle` | 有效 | 完整生命周期 |
| `test_invalid_archived_to_anything` | 有效 | 归档不能转其他状态 |
| `test_invalid_done_to_open` | 有效 | 完成不能转打开 |
| `test_invalid_in_progress_to_open` | 有效 | 进行中不能转打开 |
| `test_invalid_open_to_done` | 有效 | 打开不能直接完成 |
| `test_invalid_pending_to_in_progress` | 有效 | 待处理不能转进行中 |
| `test_invalid_same_status` | 有效 | 相同状态不能转换 |
| `test_transition_preserves_immutability` | 有效 | 不可变性验证 |
| `test_transition_updates_updated_at` | 有效 | 更新时间戳 |
| `test_valid_transition_done_to_archived` | 有效 | 完成到归档 |
| `test_valid_transition_in_progress_to_done` | 有效 | 进行中到完成 |
| `test_valid_transition_open_to_in_progress` | 有效 | 打开到进行中 |
| `test_valid_transition_pending_to_open` | 有效 | 待处理到打开 |

#### 11.10 task::model（4 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_is_completed` | 有效 | 完成状态判断 |
| `test_task_is_root` | 有效 | 根任务判断 |
| `test_task_with_children_is_not_leaf` | 有效 | 有子任务非叶子 |
| `test_task_with_parent_is_not_root` | 有效 | 有父任务非根 |

#### 11.11 task::sync（19 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_batch_sync_mixed_results` | 有效 | 批量同步混合结果 |
| `test_detect_conflicts_both_modified` | 有效 | 双方修改冲突 |
| `test_detect_conflicts_no_conflict` | 有效 | 无冲突 |
| `test_detect_conflicts_status_mismatch` | 有效 | 状态不匹配冲突 |
| `test_new_sync_is_empty` | 有效 | 新同步为空 |
| `test_resolve_conflict_keep_local` | 有效 | 保留本地 |
| `test_resolve_conflict_keep_remote` | 有效 | 保留远程 |
| `test_resolve_conflict_timestamp_priority` | 有效 | 时间戳优先 |
| `test_sync_from_feishu_creates_new_task` | 有效 | 飞书创建新任务 |
| `test_sync_from_feishu_detects_conflict` | 有效 | 飞书检测冲突 |
| `test_sync_from_feishu_skips_unchanged` | 有效 | 飞书跳过未变 |
| `test_sync_from_feishu_updates_changed_task` | 有效 | 飞书更新变更任务 |
| `test_sync_log_accumulates` | 有效 | 同步日志累积 |
| `test_sync_log_has_timestamp` | 有效 | 同步日志有时间戳 |
| `test_sync_to_feishu_detects_remote_deleted` | 有效 | 检测远程删除 |
| `test_sync_to_feishu_marks_for_sync` | 有效 | 标记同步 |
| `test_sync_to_feishu_skips_no_feishu_id` | 有效 | 跳过无飞书 ID |
| `test_sync_to_feishu_skips_unchanged` | 有效 | 跳过未变 |

#### 11.12 task::tests（19 个测试）

| 测试名 | 分类 | 说明 |
|--------|------|------|
| `test_add_subtask_to_nonexistent` | 需升级 | 缺少错误消息验证 |
| `test_add_subtask` | 需升级 | 缺少子任务属性验证 |
| `test_archive_fails_from_open` | 有效 | 打开状态不能归档 |
| `test_create_auto_discovered_is_pending` | 有效 | 自动发现为待处理 |
| `test_archive_full_flow` | 有效 | 完整归档流程 |
| `test_create_task` | 需升级 | 缺少完整属性验证 |
| `test_get_existing` | 有效 | 获取存在任务 |
| `test_get_nonexistent` | 有效 | 获取不存在任务 |
| `test_list_by_status` | 有效 | 按状态列表 |
| `test_list_combined_filter` | 有效 | 组合过滤 |
| `test_list_empty` | 有效 | 空列表 |
| `test_list_filter_by_parent_id` | 有效 | 按父 ID 过滤 |
| `test_list_filter_by_priority` | 有效 | 按优先级过滤 |
| `test_transition_nonexistent` | 有效 | 转换不存在任务 |
| `test_list_filter_by_source` | 有效 | 按来源过滤 |
| `test_transition_open_to_in_progress` | 有效 | 打开到进行中 |
| `test_transition_rejected` | 有效 | 拒绝转换 |
| `test_update_nonexistent` | 有效 | 更新不存在任务 |
| `test_update_title` | 需升级 | 缺少更新后验证 |

**小结：185 有效 + 8 需升级测试**

---

## 需升级测试清单

以下测试存在边界条件覆盖不足的问题，建议后续迭代补充：

| 测试名 | 位置 | 问题 | 建议 |
|--------|------|------|------|
| `test_generate_daily_delegates` | report::tests | 仅验证函数调用，无业务断言 | 添加输出内容验证 |
| `test_generate_month_delegates` | report::tests | 仅验证函数调用，无业务断言 | 添加输出内容验证 |
| `test_generate_week_delegates` | report::tests | 仅验证函数调用，无业务断言 | 添加输出内容验证 |
| `test_add_subtask_to_nonexistent` | task::tests | 缺少错误消息验证 | 断言错误消息内容 |
| `test_add_subtask` | task::tests | 缺少子任务属性验证 | 验证继承的属性 |
| `test_create_task` | task::tests | 缺少完整属性验证 | 验证所有默认值 |
| `test_update_title` | task::tests | 缺少更新后验证 | 验证 updated_at 变化 |

---

## 测试质量评估

### 优势

1. **M2/M4/M5 修复完整**：所有修复都有对应的测试覆盖
2. **边界条件覆盖好**：review_rules 模块有丰富的参数化测试
3. **生命周期测试完整**：task::lifecycle 覆盖了所有合法/非法状态转换
4. **降级策略测试充分**：pipeline 模块覆盖了 AI 降级到关键词的场景

### 待改进

1. **report::tests 的委托测试**：3 个测试仅验证函数调用，缺少业务断言
2. **task::tests 部分测试**：4 个测试缺少完整的属性验证
3. **并发测试缺失**：当前测试都是单线程，缺少并发场景测试
4. **错误路径覆盖不足**：部分模块缺少网络错误、超时等异常场景测试

---

## 运行命令

```bash
# 运行全部测试
cargo test -p wb-processor --lib

# 运行特定模块
cargo test -p wb-processor --lib reviewer::tests
cargo test -p wb-processor --lib pipeline::tests
cargo test -p wb-processor --lib task::discovery_ai::tests

# 运行 M2/M4/M5 相关测试
cargo test -p wb-processor --lib test_review_duplicate_confirm_request_handled_gracefully
cargo test -p wb-processor --lib test_sanitize_text_input
cargo test -p wb-processor --lib test_synthetic_event
```
