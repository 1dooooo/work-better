#!/usr/bin/env node
// SessionStart hook: 检查 setup 是否完成
// 未完成则阻止 Claude Code 正常工作

const fs = require('fs');
const path = require('path');

const projectRoot = process.env.CLAUDE_PROJECT_ROOT || process.cwd();
const markerPath = path.join(projectRoot, '.claude', '.setup-done');

if (!fs.existsSync(markerPath)) {
  console.error('');
  console.error('╔══════════════════════════════════════════════════╗');
  console.error('║  ⛔ 开发环境尚未初始化                           ║');
  console.error('║                                                  ║');
  console.error('║  请先运行:                                       ║');
  console.error('║    bash .claude/scripts/setup-dev.sh             ║');
  console.error('║                                                  ║');
  console.error('║  详见: CONTRIBUTING.md                           ║');
  console.error('╚══════════════════════════════════════════════════╝');
  console.error('');
  process.exit(2);
}
