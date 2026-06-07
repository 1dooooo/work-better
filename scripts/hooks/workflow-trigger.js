#!/usr/bin/env node
// ============================================================
// workflow-trigger.js — PostToolUse hook
// 检测 dev-output.json 写入后自动触发 run-workflow.sh
// stdin: Claude Code hook JSON { tool_name, tool_input, ... }
// ============================================================

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name || '';

    // 只关心 Write / Edit 工具
    if (toolName !== 'Write' && toolName !== 'Edit') {
      process.exit(0);
    }

    const filePath = data.tool_input?.file_path || '';
    // 匹配 .workflow/artifacts/{task_id}/dev-output.json
    const match = filePath.match(/\.workflow\/artifacts\/([^/]+)\/dev-output\.json$/);
    if (!match) {
      process.exit(0);
    }

    const taskId = match[1];
    const projectRoot = path.resolve(__dirname, '..', '..');
    const runWorkflow = path.join(projectRoot, 'scripts', 'run-workflow.sh');

    if (!fs.existsSync(runWorkflow)) {
      console.error(`[workflow-trigger] run-workflow.sh not found at ${runWorkflow}`);
      process.exit(0);
    }

    console.error(`[workflow-trigger] dev-output.json detected for task ${taskId}, triggering workflow...`);

    const result = spawnSync('bash', [runWorkflow, taskId], {
      cwd: projectRoot,
      stdio: ['ignore', 'pipe', 'pipe'],
      timeout: 300000, // 5 min
    });

    if (result.stdout) process.stderr.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);

    if (result.status !== 0) {
      console.error(`[workflow-trigger] Workflow exited with code ${result.status}`);
    } else {
      console.error(`[workflow-trigger] Workflow completed successfully for task ${taskId}`);
    }

    // Hook 不阻塞 agent，输出到 stderr 让 agent 看到结果
    process.exit(0);
  } catch (e) {
    // 解析失败不阻塞
    console.error(`[workflow-trigger] Error: ${e.message}`);
    process.exit(0);
  }
});
