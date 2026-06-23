const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

const HOOK_PATH = path.join(__dirname, '..', 'workflow-check.cjs');

// 创建临时目录模拟项目结构
function createTempProject(hasWorkflow = false, hasActiveTask = false, hasCompletedTask = false) {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-check-'));

  // 创建 .git 目录（标记项目根目录）
  fs.mkdirSync(path.join(tmpDir, '.git'));

  if (hasWorkflow) {
    const artifactsDir = path.join(tmpDir, '.workflow', 'artifacts');
    fs.mkdirSync(artifactsDir, { recursive: true });

    if (hasActiveTask) {
      const taskDir = path.join(artifactsDir, 'test-task');
      fs.mkdirSync(taskDir, { recursive: true });
      fs.writeFileSync(path.join(taskDir, 'dev-output.json'), '{}');

      if (hasCompletedTask) {
        fs.writeFileSync(path.join(taskDir, 'final-report.json'), '{}');
      }
    }
  }

  return tmpDir;
}

// 创建没有 .git 目录的临时目录（模拟非项目根目录）
function createNonProjectDir() {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-check-noproot-'));
  return tmpDir;
}

// 运行 hook 并返回结果（使用 spawnSync 避免 shell 注入问题）
function runHook(input, cwd) {
  const result = spawnSync('node', [HOOK_PATH], {
    input: JSON.stringify(input),
    cwd,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe']
  });
  return {
    exitCode: result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || ''
  };
}

describe('workflow-check hook', () => {
  let tmpDir;

  afterEach(() => {
    if (tmpDir && fs.existsSync(tmpDir)) {
      fs.rmSync(tmpDir, { recursive: true });
    }
  });

  // ========== 工具类型检查 ==========

  test('should exit 0 for non-Edit/Write/Bash tools (Read)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Read',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for non-Edit/Write/Bash tools (Glob)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Glob',
      tool_input: { pattern: 'crates/**/*.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  // ========== Bash 工具检查 ==========

  test('should exit 2 for Bash sed -i on protected file without workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: "sed -i 's/old/new/' crates/wb-core/src/lib.rs" }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 2 for Bash redirect to protected file without workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: 'echo "hello" > crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 2 for Bash append redirect to protected file without workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: 'echo "hello" >> src/app.tsx' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 0 for Bash command that does not write to protected dirs', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: 'cat crates/wb-core/src/lib.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for Bash command targeting excluded files', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: "sed -i 's/old/new/' crates/README.md" }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for Bash command outside protected dirs', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: "sed -i 's/old/new/' docs/test.md" }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for Bash sed -i on protected file with active workflow', () => {
    tmpDir = createTempProject(true, true);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: "sed -i 's/old/new/' crates/wb-core/src/lib.rs" }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for Bash with empty command', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: { command: '' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for Bash with no command field', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Bash',
      tool_input: {}
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  // ========== MultiEdit 工具检查 ==========

  test('should exit 2 for MultiEdit on code file without workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 0 for MultiEdit on code file with active workflow', () => {
    tmpDir = createTempProject(true, true);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 2 for MultiEdit with file_paths array containing protected file', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_paths: ['docs/readme.md', 'crates/wb-core/src/lib.rs'] }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 0 for MultiEdit with file_paths array of excluded files only', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_paths: ['crates/README.md', 'src/styles.css'] }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for MultiEdit with file_paths array of non-protected files', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_paths: ['docs/guide.md', 'scripts/build.sh'] }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 2 for MultiEdit with file_paths array when no workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_paths: ['src/app.tsx', 'src/components/Button.tsx'] }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
  });

  test('should exit 0 for MultiEdit with empty file_paths array', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'MultiEdit',
      tool_input: { file_paths: [] }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  // ========== 排除文件模式检查 ==========

  test('should exit 0 for excluded files (test files .test.ts)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.test.ts' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (spec files .spec.tsx)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'src/components/Button.spec.tsx' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (markdown .md)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Write',
      tool_input: { file_path: 'crates/README.md' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (JSON .json)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/config.json' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (YAML .yaml)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/config.yaml' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (YAML .yml)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/config.yml' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (CSS .css)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Write',
      tool_input: { file_path: 'src/styles/global.css' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for excluded files (codemap .codemap.md)', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/wb-core.codemap.md' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  // ========== 目录检查 ==========

  test('should exit 0 for files outside workflow-required directories', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'docs/test.md' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for scripts/ directory files', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'scripts/test.js' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  // ========== Workflow 状态检查 ==========

  test('should exit 2 when no workflow artifacts directory exists', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('No active workflow found');
    expect(result.stderr).toContain('Cannot continue without workflow');
  });

  test('should exit 2 when workflow artifacts exist but no active task', () => {
    tmpDir = createTempProject(true, false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('No active workflow task found');
    expect(result.stderr).toContain('Cannot continue without workflow');
  });

  test('should exit 0 when active workflow exists', () => {
    tmpDir = createTempProject(true, true);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 for src/ files when workflow is active', () => {
    tmpDir = createTempProject(true, true);
    const result = runHook({
      tool_name: 'Write',
      tool_input: { file_path: 'src/app.tsx' }
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });

  test('should exit 2 for src-tauri/ files when no workflow', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'src-tauri/src/main.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('No active workflow found');
  });

  // ========== 已完成任务检查 ==========

  test('should exit 2 when latest task is completed (final-report.json exists)', () => {
    tmpDir = createTempProject(true, true, true);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('BLOCKED');
    expect(result.stderr).toContain('already completed');
  });

  // ========== 错误处理 (fail-closed 策略) ==========

  test('should exit 2 for malformed stdin JSON input (fail-closed)', () => {
    tmpDir = createTempProject(false);
    const result = spawnSync('node', [HOOK_PATH], {
      input: 'not valid json',
      cwd: tmpDir,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    // JSON parse error should exit 2 (fail-closed: security-critical component)
    expect(result.status).toBe(2);
    expect(result.stderr).toContain('Error');
  });

  test('should exit 2 for empty stdin input (fail-closed)', () => {
    tmpDir = createTempProject(false);
    const result = spawnSync('node', [HOOK_PATH], {
      input: '',
      cwd: tmpDir,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    // Empty input causes JSON parse error, should exit 2 (fail-closed)
    expect(result.status).toBe(2);
  });

  // ========== 项目根目录查找 (fail-closed 策略) ==========

  test('should exit 2 when project root not found (no .git directory, fail-closed)', () => {
    tmpDir = createNonProjectDir();
    const result = spawnSync('node', [HOOK_PATH], {
      input: JSON.stringify({
        tool_name: 'Edit',
        tool_input: { file_path: 'crates/test.rs' }
      }),
      cwd: tmpDir,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    // When project root not found, should exit 2 (fail-closed)
    expect(result.status).toBe(2);
    expect(result.stderr).toContain('WARNING');
  });

  // ========== 边界场景：多任务 ==========

  test('should check latest task when multiple tasks exist', () => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-check-multi-'));
    fs.mkdirSync(path.join(tmpDir, '.git'));

    const artifactsDir = path.join(tmpDir, '.workflow', 'artifacts');
    fs.mkdirSync(artifactsDir, { recursive: true });

    // 创建一个已完成的旧任务
    const oldTaskDir = path.join(artifactsDir, 'aaa-old-task');
    fs.mkdirSync(oldTaskDir, { recursive: true });
    fs.writeFileSync(path.join(oldTaskDir, 'dev-output.json'), '{}');
    fs.writeFileSync(path.join(oldTaskDir, 'final-report.json'), '{}');

    // 创建一个新的活跃任务
    const newTaskDir = path.join(artifactsDir, 'bbb-new-task');
    fs.mkdirSync(newTaskDir, { recursive: true });
    fs.writeFileSync(path.join(newTaskDir, 'dev-output.json'), '{}');

    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    // 最新任务是活跃的，应该允许
    expect(result.exitCode).toBe(0);
  });

  test('should block when all tasks are completed', () => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-check-all-done-'));
    fs.mkdirSync(path.join(tmpDir, '.git'));

    const artifactsDir = path.join(tmpDir, '.workflow', 'artifacts');
    fs.mkdirSync(artifactsDir, { recursive: true });

    // 创建两个已完成的任务
    for (const taskName of ['task-a', 'task-b']) {
      const taskDir = path.join(artifactsDir, taskName);
      fs.mkdirSync(taskDir, { recursive: true });
      fs.writeFileSync(path.join(taskDir, 'dev-output.json'), '{}');
      fs.writeFileSync(path.join(taskDir, 'final-report.json'), '{}');
    }

    const result = runHook({
      tool_name: 'Edit',
      tool_input: { file_path: 'crates/test.rs' }
    }, tmpDir);
    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain('already completed');
  });

  // ========== 边界场景：缺少 tool_input ==========

  test('should exit 0 when tool_input is missing file_path', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit',
      tool_input: {}
    }, tmpDir);
    // filePath will be empty string, won't match any workflow pattern
    expect(result.exitCode).toBe(0);
  });

  test('should exit 0 when tool_input is undefined', () => {
    tmpDir = createTempProject(false);
    const result = runHook({
      tool_name: 'Edit'
    }, tmpDir);
    expect(result.exitCode).toBe(0);
  });
});
