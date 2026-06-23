#!/usr/bin/env node
// ============================================================
// workflow-check.js — PreToolUse hook
// 检查 Edit/Write/Bash 操作是否需要先创建 workflow
// stdin: Claude Code hook JSON { tool_name, tool_input, ... }
// exit 0: 允许（警告或无关）
// exit 2: 阻止（需要先创建 workflow）
// ============================================================

const fs = require('fs');
const path = require('path');

// 需要 workflow 的代码目录模式（用于文件路径匹配，带 ^ 锚点）
const WORKFLOW_REQUIRED_PATTERNS = [
  /^crates\//,
  /^src\//,
  /^src-tauri\//,
];

// 用于 Bash 命令字符串匹配的模式（不带 ^ 锚点，匹配命令中任意位置的引用）
const WORKFLOW_REQUIRED_PATTERNS_IN_COMMAND = [
  /crates\//,
  /src\//,
  /src-tauri\//,
];

// 排除的文件模式（文档、配置、测试等不需要 workflow）
const EXCLUDED_PATTERNS = [
  /\.test\.tsx?$/,        // 测试文件
  /\.spec\.tsx?$/,        // 测试文件
  /\.md$/,                // 文档文件
  /\.json$/,              // JSON 配置文件
  /\.ya?ml$/,             // YAML 配置文件
  /\.css$/,               // 样式文件
];

// Bash 命令中可能写入文件的模式
// 匹配: sed -i, tee, cp/mv 到受保护目录, echo/cat/printf 重定向, awk -i inplace 等
const BASH_WRITE_PATTERNS = [
  /\bsed\s+(-[iI]\b|--in-place)/,           // sed -i or sed --in-place
  /\btee\b/,                                  // tee (writes to file)
  /\bcp\b.*\s+(crates|src|src-tauri)\//,     // cp to protected dir
  /\bmv\b.*\s+(crates|src|src-tauri)\//,     // mv to protected dir
  />\s*(crates|src|src-tauri)\//,             // redirect > to protected dir
  />>\s*(crates|src|src-tauri)\//,            // append >> to protected dir
  /\bawk\b.*-i\s+inplace/,                   // awk -i inplace
  /\binstall\b.*\s+(crates|src|src-tauri)\//, // install to protected dir
  /\bcp\b.*\s+(crates|src|src-tauri)\//,     // cp to protected dir
];

/**
 * 检查文件路径是否需要 workflow
 * @param {string} filePath
 * @returns {{ needsWorkflow: boolean, reason: string }}
 */
function checkFilePath(filePath) {
  if (!filePath) return { needsWorkflow: false, reason: 'empty path' };

  const needsWorkflow = WORKFLOW_REQUIRED_PATTERNS.some(p => p.test(filePath));
  if (!needsWorkflow) return { needsWorkflow: false, reason: 'not in protected dir' };

  const isExcluded = EXCLUDED_PATTERNS.some(p => p.test(filePath));
  if (isExcluded) return { needsWorkflow: false, reason: 'excluded file type' };

  return { needsWorkflow: true, reason: 'protected dir, non-excluded file' };
}

/**
 * 从 Bash 命令中提取可能被写入的文件路径
 * @param {string} command
 * @returns {string[]} 被写入的文件路径列表
 */
function extractBashWriteTargets(command) {
  if (!command) return [];

  const targets = [];

  // 匹配 sed -i 's/.../' file 或 sed -i file
  const sedMatch = command.match(/\bsed\s+(?:-[iI]\b|--in-place)\s*(?:'[^']*'|"[^"]*")*\s+(\S+)/g);
  if (sedMatch) {
    for (const m of sedMatch) {
      const filePart = m.match(/\s(\S+)$/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  // 匹配 redirect: > file or >> file (with optional space)
  const redirectMatches = command.match(/>{1,2}\s*([^\s;|&]+)/g);
  if (redirectMatches) {
    for (const m of redirectMatches) {
      const filePart = m.match(/>{1,2}\s*(.+)/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  // 匹配 tee file
  const teeMatch = command.match(/\btee\s+(?:-[a]+\s+)?(\S+)/g);
  if (teeMatch) {
    for (const m of teeMatch) {
      const filePart = m.match(/\btee\s+(?:-[a]+\s+)?(\S+)/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  // 匹配 cp/mv source dest (last arg is dest)
  const cpMvMatch = command.match(/\b(?:cp|mv)\s+(?:-[a-zA-Z]+\s+)?\S+\s+(\S+)\s*$/gm);
  if (cpMvMatch) {
    for (const m of cpMvMatch) {
      const filePart = m.match(/\s(\S+)\s*$/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  return targets;
}

/**
 * 检查 Bash 命令是否可能写入受保护目录的文件
 * @param {string} command
 * @returns {{ blocked: boolean, filePath: string }}
 */
function checkBashCommand(command) {
  if (!command) return { blocked: false, filePath: '' };

  // 快速检查：命令是否包含受保护目录引用（使用不带 ^ 锚点的模式匹配命令中任意位置）
  const hasProtectedRef = WORKFLOW_REQUIRED_PATTERNS_IN_COMMAND.some(p => p.test(command));
  if (!hasProtectedRef) return { blocked: false, filePath: '' };

  // 检查是否有写入操作模式
  const hasWritePattern = BASH_WRITE_PATTERNS.some(p => p.test(command));
  if (!hasWritePattern) return { blocked: false, filePath: '' };

  // 提取具体被写入的文件路径
  const targets = extractBashWriteTargets(command);
  for (const target of targets) {
    const { needsWorkflow } = checkFilePath(target);
    if (needsWorkflow) {
      return { blocked: true, filePath: target };
    }
  }

  // 如果提取到了目标但都被排除，允许执行
  if (targets.length > 0) {
    return { blocked: false, filePath: '' };
  }

  // 如果有写入模式但无法提取具体路径，保守阻止
  return { blocked: true, filePath: '(detected write to protected dir)' };
}

/**
 * 检查文件路径列表中是否有需要 workflow 的文件
 * @param {string[]} filePaths
 * @returns {{ needsWorkflow: boolean, filePath: string }}
 */
function checkFilePaths(filePaths) {
  if (!Array.isArray(filePaths) || filePaths.length === 0) {
    return { needsWorkflow: false, filePath: '' };
  }

  for (const fp of filePaths) {
    const { needsWorkflow } = checkFilePath(fp);
    if (needsWorkflow) {
      return { needsWorkflow: true, filePath: fp };
    }
  }

  return { needsWorkflow: false, filePath: '' };
}

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name || '';
    const toolInput = data.tool_input || {};

    // 确定要检查的文件路径
    let filePath = '';
    let isBlocked = false;

    if (toolName === 'Bash') {
      // Bash 工具：检查命令是否包含文件写入操作
      const command = toolInput.command || '';
      const bashCheck = checkBashCommand(command);
      if (!bashCheck.blocked) {
        process.exit(0);
      }
      filePath = bashCheck.filePath;
      isBlocked = true;
    } else if (toolName === 'MultiEdit') {
      // MultiEdit 工具：检查 file_paths 数组（复数）
      const filePaths = toolInput.file_paths || [];
      const { needsWorkflow, filePath: matchedPath } = checkFilePaths(filePaths);
      if (!needsWorkflow) {
        // 也检查 file_path（单数）作为降级
        const singleCheck = checkFilePath(toolInput.file_path || '');
        if (!singleCheck.needsWorkflow) {
          process.exit(0);
        }
        filePath = toolInput.file_path;
      } else {
        filePath = matchedPath;
      }
    } else if (toolName === 'Edit' || toolName === 'Write') {
      // Edit/Write 工具：检查 file_path
      filePath = toolInput.file_path || '';
      const { needsWorkflow } = checkFilePath(filePath);
      if (!needsWorkflow) {
        process.exit(0);
      }
    } else {
      // 其他工具不检查
      process.exit(0);
    }

    // 查找项目根目录
    const projectRoot = findProjectRoot();
    if (!projectRoot) {
      console.error('[workflow-check] WARNING: Could not find project root, skipping workflow check');
      process.exit(2); // fail-closed: 无法确定项目根目录时阻止
    }

    // 检查是否有活跃的 workflow artifacts
    const artifactsDir = path.join(projectRoot, '.workflow', 'artifacts');
    if (!fs.existsSync(artifactsDir)) {
      console.error('[workflow-check] ❌ BLOCKED: No active workflow found.');
      console.error(`[workflow-check] You are editing code file: ${filePath}`);
      console.error('[workflow-check] According to workflow rules, code changes require a workflow.');
      console.error('[workflow-check] Please create a workflow first:');
      console.error('[workflow-check]   1. Run: ./scripts/create-dev-output.sh <task_id>');
      console.error('[workflow-check]   2. Run: ./scripts/run-workflow.sh <task_id>');
      console.error('[workflow-check] Cannot continue without workflow.');
      process.exit(2); // 强制阻止
    }

    // 检查是否有活跃的任务（有 dev-output.json 但没有 final-report.json）
    const tasks = fs.readdirSync(artifactsDir).filter(d => {
      const devOutput = path.join(artifactsDir, d, 'dev-output.json');
      return fs.existsSync(devOutput);
    });

    if (tasks.length === 0) {
      console.error('[workflow-check] ❌ BLOCKED: No active workflow task found.');
      console.error(`[workflow-check] You are editing code file: ${filePath}`);
      console.error('[workflow-check] According to workflow rules, code changes require a workflow.');
      console.error('[workflow-check] Please create a workflow first:');
      console.error('[workflow-check]   1. Run: ./scripts/create-dev-output.sh <task_id>');
      console.error('[workflow-check]   2. Run: ./scripts/run-workflow.sh <task_id>');
      console.error('[workflow-check] Cannot continue without workflow.');
      process.exit(2); // 强制阻止
    }

    // 使用 mtime 排序获取最新任务（而非字典序）
    const latestTask = tasks.sort((a, b) => {
      const mtimeA = fs.statSync(path.join(artifactsDir, a)).mtimeMs;
      const mtimeB = fs.statSync(path.join(artifactsDir, b)).mtimeMs;
      return mtimeB - mtimeA;
    })[0];

    const finalReport = path.join(artifactsDir, latestTask, 'final-report.json');
    if (fs.existsSync(finalReport)) {
      console.error('[workflow-check] ❌ BLOCKED: Latest workflow task already completed.');
      console.error(`[workflow-check] You are editing code file: ${filePath}`);
      console.error('[workflow-check] According to workflow rules, code changes require a workflow.');
      console.error('[workflow-check] Please create a new workflow:');
      console.error('[workflow-check]   1. Run: ./scripts/create-dev-output.sh <task_id>');
      console.error('[workflow-check]   2. Run: ./scripts/run-workflow.sh <task_id>');
      console.error('[workflow-check] Cannot continue without workflow.');
      process.exit(2); // 强制阻止
    }

    // 有活跃的 workflow，允许继续
    console.error(`[workflow-check] ✅ Active workflow found: ${latestTask}`);
    process.exit(0);
  } catch (e) {
    console.error(`[workflow-check] Error: ${e.message}`);
    process.exit(2); // fail-closed: 出错时阻止（安全关键组件应采用 fail-closed 策略）
  }
});

function findProjectRoot() {
  // 从当前目录向上查找 .git 目录
  let dir = process.cwd();
  while (dir !== path.dirname(dir)) {
    if (fs.existsSync(path.join(dir, '.git'))) {
      return dir;
    }
    dir = path.dirname(dir);
  }
  return null;
}
