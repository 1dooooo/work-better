#!/usr/bin/env node
// ============================================================
// workflow-check.cjs — PreToolUse hook
// 检查 Edit/Write/Bash 操作是否需要先创建 workflow
// 如果需要且不存在，自动创建 artifact 目录和 dev-output.json
// stdin: Claude Code hook JSON { tool_name, tool_input, ... }
// exit 0: 允许（已有 workflow 或已自动创建）
// exit 2: 阻止（无法自动创建）
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
const BASH_WRITE_PATTERNS = [
  /\bsed\s+(-[iI]\b|--in-place)/,
  /\btee\b/,
  /\bcp\b.*\s+(crates|src|src-tauri)\//,
  /\bmv\b.*\s+(crates|src|src-tauri)\//,
  />\s*(crates|src|src-tauri)\//,
  />>\s*(crates|src|src-tauri)\//,
  /\bawk\b.*-i\s+inplace/,
  /\binstall\b.*\s+(crates|src|src-tauri)\//,
];

/**
 * 检查文件路径是否需要 workflow
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
 */
function extractBashWriteTargets(command) {
  if (!command) return [];
  const targets = [];

  const sedMatch = command.match(/\bsed\s+(?:-[iI]\b|--in-place)\s*(?:'[^']*'|"[^"]*")*\s+(\S+)/g);
  if (sedMatch) {
    for (const m of sedMatch) {
      const filePart = m.match(/\s(\S+)$/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  const redirectMatches = command.match(/>{1,2}\s*([^\s;|&]+)/g);
  if (redirectMatches) {
    for (const m of redirectMatches) {
      const filePart = m.match(/>{1,2}\s*(.+)/);
      if (filePart) targets.push(filePart[1]);
    }
  }

  const teeMatch = command.match(/\btee\s+(?:-[a]+\s+)?(\S+)/g);
  if (teeMatch) {
    for (const m of teeMatch) {
      const filePart = m.match(/\btee\s+(?:-[a]+\s+)?(\S+)/);
      if (filePart) targets.push(filePart[1]);
    }
  }

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
 */
function checkBashCommand(command) {
  if (!command) return { blocked: false, filePath: '' };

  const hasProtectedRef = WORKFLOW_REQUIRED_PATTERNS_IN_COMMAND.some(p => p.test(command));
  if (!hasProtectedRef) return { blocked: false, filePath: '' };

  const hasWritePattern = BASH_WRITE_PATTERNS.some(p => p.test(command));
  if (!hasWritePattern) return { blocked: false, filePath: '' };

  const targets = extractBashWriteTargets(command);
  for (const target of targets) {
    const { needsWorkflow } = checkFilePath(target);
    if (needsWorkflow) {
      return { blocked: true, filePath: target };
    }
  }

  if (targets.length > 0) {
    return { blocked: false, filePath: '' };
  }

  return { blocked: true, filePath: '(detected write to protected dir)' };
}

/**
 * 检查文件路径列表中是否有需要 workflow 的文件
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

/**
 * 查找项目根目录
 */
function findProjectRoot() {
  let dir = process.cwd();
  while (dir !== path.dirname(dir)) {
    if (fs.existsSync(path.join(dir, '.git'))) {
      return dir;
    }
    dir = path.dirname(dir);
  }
  return null;
}

/**
 * 自动创建 workflow artifact 目录和 dev-output.json
 * @param {string} projectRoot
 * @param {string} filePath - 触发的文件路径
 * @returns {{ created: boolean, taskId: string, error: string }}
 */
function autoCreateWorkflow(projectRoot, filePath) {
  const artifactsDir = path.join(projectRoot, '.workflow', 'artifacts');

  // 确保 artifacts 目录存在
  if (!fs.existsSync(artifactsDir)) {
    fs.mkdirSync(artifactsDir, { recursive: true });
  }

  // 检查是否已有活跃 workflow
  if (fs.existsSync(artifactsDir)) {
    const existingTasks = fs.readdirSync(artifactsDir).filter(d => {
      const devOutput = path.join(artifactsDir, d, 'dev-output.json');
      return fs.existsSync(devOutput);
    });

    for (const t of existingTasks) {
      const finalReport = path.join(artifactsDir, t, 'final-report.json');
      if (!fs.existsSync(finalReport)) {
        // 已有活跃 workflow，不需要创建
        return { created: false, taskId: t, error: '' };
      }
    }
  }

  // 生成 task_id：feat-auto-{timestamp}
  const timestamp = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14);
  const taskId = `feat-auto-${timestamp}`;
  const taskDir = path.join(artifactsDir, taskId);

  try {
    fs.mkdirSync(taskDir, { recursive: true });

    // 生成 dev-output.json
    const devOutput = {
      task_id: taskId,
      task_description: `Auto-detected code change: ${filePath}`,
      changed_files: [filePath],
      timestamp: new Date().toISOString(),
      auto_created: true,
    };

    fs.writeFileSync(
      path.join(taskDir, 'dev-output.json'),
      JSON.stringify(devOutput, null, 2)
    );

    console.error(`[workflow-check] ✅ Auto-created workflow: ${taskId}`);
    console.error(`[workflow-check] Artifact dir: ${taskDir}`);
    console.error(`[workflow-check] Please call workflow-advisor to continue.`);

    return { created: true, taskId, error: '' };
  } catch (e) {
    return { created: false, taskId: '', error: e.message };
  }
}

// ─── Main ───────────────────────────────────────────────────

let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name || '';
    const toolInput = data.tool_input || {};

    // 确定要检查的文件路径
    let filePath = '';
    let needsWorkflow = false;

    if (toolName === 'Bash') {
      const command = toolInput.command || '';
      const bashCheck = checkBashCommand(command);
      if (!bashCheck.blocked) {
        process.exit(0);
      }
      filePath = bashCheck.filePath;
      needsWorkflow = true;
    } else if (toolName === 'MultiEdit') {
      const filePaths = toolInput.file_paths || [];
      const { needsWorkflow: nw, filePath: matchedPath } = checkFilePaths(filePaths);
      if (!nw) {
        const singleCheck = checkFilePath(toolInput.file_path || '');
        if (!singleCheck.needsWorkflow) {
          process.exit(0);
        }
        filePath = toolInput.file_path;
      } else {
        filePath = matchedPath;
      }
      needsWorkflow = true;
    } else if (toolName === 'Edit' || toolName === 'Write') {
      filePath = toolInput.file_path || '';
      const check = checkFilePath(filePath);
      if (!check.needsWorkflow) {
        process.exit(0);
      }
      needsWorkflow = true;
    } else {
      process.exit(0);
    }

    if (!needsWorkflow) {
      process.exit(0);
    }

    // 查找项目根目录
    const projectRoot = findProjectRoot();
    if (!projectRoot) {
      console.error('[workflow-check] WARNING: Could not find project root');
      process.exit(2);
    }

    // 自动创建 workflow artifact
    const result = autoCreateWorkflow(projectRoot, filePath);

    if (result.error) {
      console.error(`[workflow-check] ❌ Failed to auto-create workflow: ${result.error}`);
      process.exit(2);
    }

    // 允许操作继续
    process.exit(0);
  } catch (e) {
    console.error(`[workflow-check] Error: ${e.message}`);
    process.exit(2);
  }
});
