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
    const toolInput = data.tool_input || {};

    let shouldTrigger = false;
    let taskId = null;

    // Case 1: Write/Edit 工具写入 dev-output.json
    if (toolName === 'Write' || toolName === 'Edit') {
      const filePath = toolInput.file_path || '';
      const match = filePath.match(/\.workflow\/artifacts\/([^/]+)\/dev-output\.json$/);
      if (match) {
        shouldTrigger = true;
        taskId = match[1];
      }
    }

    // Case 2: Bash 工具执行 git commit（开发任务完成信号）
    if (toolName === 'Bash') {
      const command = toolInput.command || '';
      if (/^git\s+commit\b/.test(command.trim())) {
        // 检查是否有活跃的 workflow artifacts
        const projectRoot = path.resolve(__dirname, '..', '..');
        const artifactsDir = path.join(projectRoot, '.workflow', 'artifacts');
        if (fs.existsSync(artifactsDir)) {
          const tasks = fs.readdirSync(artifactsDir).filter(d => {
            const devOutput = path.join(artifactsDir, d, 'dev-output.json');
            return fs.existsSync(devOutput);
          });
          // 找到最新有 dev-output 但没有 final-report 的任务
          for (const t of tasks.sort().reverse()) {
            const finalReport = path.join(artifactsDir, t, 'final-report.json');
            if (!fs.existsSync(finalReport)) {
              shouldTrigger = true;
              taskId = t;
              break;
            }
          }
        }
      }
    }

    if (!shouldTrigger || !taskId) {
      process.exit(0);
    }

    const projectRoot = path.resolve(__dirname, '..', '..');
    const runWorkflow = path.join(projectRoot, 'scripts', 'run-workflow.sh');

    if (!fs.existsSync(runWorkflow)) {
      console.error(`[workflow-trigger] run-workflow.sh not found at ${runWorkflow}`);
      process.exit(0);
    }

    console.error(`[workflow-trigger] Triggering workflow for task ${taskId} (tool: ${toolName})`);

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

    process.exit(0);
  } catch (e) {
    console.error(`[workflow-trigger] Error: ${e.message}`);
    process.exit(0);
  }
});
