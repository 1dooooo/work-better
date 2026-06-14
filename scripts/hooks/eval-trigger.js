#!/usr/bin/env node
// ============================================================
// eval-trigger.js — PostToolUse hook
// 检测代码文件变更后自动运行 eval
// stdin: Claude Code hook JSON { tool_name, tool_input, ... }
// ============================================================

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const EVAL_RELEVANT_PATTERNS = [
  /^crates\//,
  /^src\//,
  /^src-tauri\//,
];

const EVAL_RUNNER = path.resolve(__dirname, '..', 'run-evals.sh');

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name || '';
    const toolInput = data.tool_input || {};

    if (toolName !== 'Edit' && toolName !== 'Write' && toolName !== 'MultiEdit') {
      process.exit(0);
    }

    const filePath = toolInput.file_path || '';
    const isRelevant = EVAL_RELEVANT_PATTERNS.some(p => p.test(filePath));

    if (!isRelevant) {
      process.exit(0);
    }

    // 检查 eval runner 是否存在
    if (!fs.existsSync(EVAL_RUNNER)) {
      console.error(`[eval-trigger] ${EVAL_RUNNER} not found, skipping`);
      process.exit(0);
    }

    // 检查上次运行时间，避免频繁触发（至少间隔 60s）
    const lockFile = path.resolve(__dirname, '..', '..', '.claude', 'evals', '.last-run');
    const now = Date.now();
    if (fs.existsSync(lockFile)) {
      const lastRun = parseInt(fs.readFileSync(lockFile, 'utf8').trim(), 10);
      if (now - lastRun < 60000) {
        process.exit(0);
      }
    }

    // 写入运行时间戳
    const lockDir = path.dirname(lockFile);
    if (!fs.existsSync(lockDir)) {
      fs.mkdirSync(lockDir, { recursive: true });
    }
    fs.writeFileSync(lockFile, String(now));

    console.error(`[eval-trigger] Code change detected in ${filePath}, running evals...`);

    const result = spawnSync('bash', [EVAL_RUNNER], {
      cwd: path.resolve(__dirname, '..', '..'),
      stdio: ['ignore', 'pipe', 'pipe'],
      timeout: 60000,
    });

    if (result.stdout) {
      const output = result.stdout.toString().trim();
      if (output) {
        // 只输出摘要行
        const summary = output.split('\n').filter(l => l.includes('Pass Rate') || l.includes('Total:')).join('\n');
        if (summary) console.error(`[eval-trigger] ${summary}`);
      }
    }

    if (result.stderr) {
      const errors = result.stderr.toString().trim();
      if (errors && !errors.includes('[eval-trigger]')) {
        console.error(`[eval-trigger] stderr: ${errors}`);
      }
    }

    if (result.status !== 0) {
      console.error(`[eval-trigger] Evals failed with exit code ${result.status}`);
    }

    process.exit(0);
  } catch (e) {
    console.error(`[eval-trigger] Error: ${e.message}`);
    process.exit(0);
  }
});
