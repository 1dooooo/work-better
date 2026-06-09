import { useState, useEffect, useCallback, useMemo } from "react";
import {
  getProcessingAudits,
  getExecutionLogs,
  getAuditSummary,
  type ProcessingAuditRow,
  type ExecutionLogRow,
  type AuditSummary,
} from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  RefreshCw,
  Loader2,
  Inbox,
  ChevronDown,
  ChevronRight,
  Cpu,
  Zap,
  DollarSign,
  Activity,
  Clock,
  AlertCircle,
  CheckCircle2,
  XCircle,
  Timer,
} from "lucide-react";

// ─── Types ────────────────────────────────────────────────────────

/** M5: 使用联合类型替代 string */
type AuditStep = "Classifier" | "Extract" | "Upgrade" | "Review" | "Persist" | "UserConfirm";
type ExecutionStatus = "Success" | "Failed" | "Skipped" | "Timeout";

// ─── Utilities ────────────────────────────────────────────────────

/** 格式化毫秒为人类可读的时长 */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

/** 解析 JSON 输出，返回预览文本 */
function parseOutputPreview(output: string, maxLength: number = 120): string {
  try {
    const parsed = JSON.parse(output);
    const text = typeof parsed === "string" ? parsed : JSON.stringify(parsed, null, 2);
    return text.length > maxLength ? text.slice(0, maxLength) + "..." : text;
  } catch {
    return output.length > maxLength ? output.slice(0, maxLength) + "..." : output;
  }
}

// ─── Status Badge ─────────────────────────────────────────────────

const STATUS_CONFIG: Record<ExecutionStatus, { variant: "default" | "secondary" | "destructive" | "outline"; icon: React.ReactNode }> = {
  Success: { variant: "default", icon: <CheckCircle2 className="h-3 w-3" /> },
  Failed: { variant: "destructive", icon: <XCircle className="h-3 w-3" /> },
  Timeout: { variant: "secondary", icon: <Timer className="h-3 w-3" /> },
  Skipped: { variant: "outline", icon: <AlertCircle className="h-3 w-3" /> },
};

function ExecutionStatusBadge({ status }: { status: ExecutionStatus }) {
  const { variant, icon } = STATUS_CONFIG[status] ?? { variant: "outline" as const, icon: null };
  return (
    <Badge variant={variant} className="gap-1 text-[11px]">
      {icon}
      {status}
    </Badge>
  );
}

const STEP_COLORS: Record<AuditStep, string> = {
  Classifier: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
  Extract: "bg-purple-500/10 text-purple-600 dark:text-purple-400",
  Upgrade: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
  Review: "bg-green-500/10 text-green-600 dark:text-green-400",
  Persist: "bg-gray-500/10 text-gray-600 dark:text-gray-400",
  UserConfirm: "bg-pink-500/10 text-pink-600 dark:text-pink-400",
};

function StepBadge({ step }: { step: string }) {
  const color = STEP_COLORS[step as AuditStep] ?? "bg-muted text-muted-foreground";
  return (
    <Badge variant="outline" className={`${color} border-0 text-[11px]`}>
      {step}
    </Badge>
  );
}

// ─── Summary Card ─────────────────────────────────────────────────

function SummaryCards({ summary }: { summary: AuditSummary | null }) {
  if (!summary) return null;

  const cards = [
    { label: "处理审计", value: summary.total_processing_audits.toLocaleString(), icon: <Cpu className="h-4 w-4 text-muted-foreground" /> },
    { label: "执行日志", value: summary.total_execution_logs.toLocaleString(), icon: <Activity className="h-4 w-4 text-muted-foreground" /> },
    { label: "总 Token", value: summary.total_tokens.toLocaleString(), icon: <Zap className="h-4 w-4 text-muted-foreground" /> },
    { label: "总成本", value: `$${summary.total_cost.toFixed(4)}`, icon: <DollarSign className="h-4 w-4 text-muted-foreground" /> },
    { label: "成功率", value: `${(summary.success_rate * 100).toFixed(1)}%`, icon: <CheckCircle2 className="h-4 w-4 text-muted-foreground" /> },
  ];

  return (
    // M9: 添加响应式断点
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
      {cards.map((card) => (
        <Card key={card.label} className="border-border">
          <CardContent className="flex items-center gap-3 p-3">
            {card.icon}
            <div>
              <p className="text-[11px] text-muted-foreground">{card.label}</p>
              <p className="text-sm font-semibold">{card.value}</p>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

// ─── Processing Audit Item ────────────────────────────────────────

function ProcessingAuditItem({ audit }: { audit: ProcessingAuditRow }) {
  const [expanded, setExpanded] = useState(false);

  const toggleExpanded = useCallback(() => setExpanded(prev => !prev), []);

  // L2: 键盘事件处理
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggleExpanded();
    }
  }, [toggleExpanded]);

  const outputPreview = useMemo(() => parseOutputPreview(audit.output), [audit.output]);

  return (
    <Card className="border-border transition-colors hover:bg-muted/30">
      <CardHeader
        className="flex cursor-pointer flex-row items-center gap-2 px-4 py-2.5"
        onClick={toggleExpanded}
        onKeyDown={handleKeyDown}
        role="button"
        tabIndex={0}
        aria-expanded={expanded}
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        )}
        <StepBadge step={audit.step} />
        <Badge variant="secondary" className="text-[11px]">
          {audit.model}
        </Badge>
        <span className="text-[11px] text-muted-foreground">
          {formatDuration(audit.duration_ms)}
        </span>
        <span className="text-[11px] text-muted-foreground">
          {audit.token_input + audit.token_output} tokens
        </span>
        {audit.cost_estimate > 0 && (
          <span className="text-[11px] text-muted-foreground">
            ${audit.cost_estimate.toFixed(4)}
          </span>
        )}
        <span className="ml-auto text-[11px] text-muted-foreground">
          {new Date(audit.timestamp).toLocaleString("zh-CN")}
        </span>
      </CardHeader>
      {expanded && (
        <CardContent className="px-4 pb-3 pt-0">
          <div className="space-y-2 text-xs">
            <div className="flex gap-4">
              <span className="text-muted-foreground">Trace ID:</span>
              <span className="font-mono">{audit.trace_id}</span>
            </div>
            <div className="flex gap-4">
              <span className="text-muted-foreground">Prompt:</span>
              <span>{audit.prompt_id}</span>
            </div>
            <div className="flex gap-4">
              <span className="text-muted-foreground">置信度:</span>
              <span>{(audit.confidence * 100).toFixed(1)}%</span>
            </div>
            {audit.input_summary && (
              <div>
                <span className="text-muted-foreground">输入摘要:</span>
                <pre className="mt-1 whitespace-pre-wrap break-words rounded bg-muted p-2 text-[11px]">
                  {audit.input_summary}
                </pre>
              </div>
            )}
            <div>
              <span className="text-muted-foreground">输出:</span>
              {/* M7: 展开视图显示完整内容 */}
              <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded bg-muted p-2 text-[11px]">
                {expanded ? audit.output : outputPreview}
              </pre>
            </div>
            {audit.review_verdict && (
              <div className="flex gap-4">
                <span className="text-muted-foreground">审核结论:</span>
                <Badge variant={audit.review_verdict === "Approved" ? "default" : "secondary"}>
                  {audit.review_verdict}
                </Badge>
              </div>
            )}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

// ─── Execution Log Item ───────────────────────────────────────────

function ExecutionLogItem({ log }: { log: ExecutionLogRow }) {
  const [expanded, setExpanded] = useState(false);

  const toggleExpanded = useCallback(() => setExpanded(prev => !prev), []);

  // L2: 键盘事件处理
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggleExpanded();
    }
  }, [toggleExpanded]);

  return (
    <Card className="border-border transition-colors hover:bg-muted/30">
      <CardHeader
        className="flex cursor-pointer flex-row items-center gap-2 px-4 py-2.5"
        onClick={toggleExpanded}
        onKeyDown={handleKeyDown}
        role="button"
        tabIndex={0}
        aria-expanded={expanded}
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        )}
        <ExecutionStatusBadge status={log.status as ExecutionStatus} />
        <span className="text-sm font-medium">{log.task_name}</span>
        <span className="text-[11px] text-muted-foreground">
          {formatDuration(log.duration_ms)}
        </span>
        <span className="ml-auto text-[11px] text-muted-foreground">
          {new Date(log.created_at).toLocaleString("zh-CN")}
        </span>
      </CardHeader>
      {expanded && (
        <CardContent className="px-4 pb-3 pt-0">
          <div className="space-y-2 text-xs">
            <div className="flex gap-4">
              <span className="text-muted-foreground">Task ID:</span>
              <span className="font-mono">{log.task_id}</span>
            </div>
            <div className="flex gap-4">
              <span className="text-muted-foreground">开始:</span>
              <span>{new Date(log.started_at).toLocaleString("zh-CN")}</span>
            </div>
            <div className="flex gap-4">
              <span className="text-muted-foreground">结束:</span>
              <span>{new Date(log.finished_at).toLocaleString("zh-CN")}</span>
            </div>
            {log.output && (
              <div>
                <span className="text-muted-foreground">输出:</span>
                <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded bg-muted p-2 text-[11px]">
                  {log.output}
                </pre>
              </div>
            )}
            {log.error && (
              <div>
                <span className="text-destructive">错误:</span>
                <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded bg-destructive/10 p-2 text-[11px] text-destructive">
                  {log.error}
                </pre>
              </div>
            )}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

// ─── Main View ────────────────────────────────────────────────────

type AuditTab = "processing" | "execution";

export default function AuditView() {
  const [tab, setTab] = useState<AuditTab>("processing");
  const [processingAudits, setProcessingAudits] = useState<ProcessingAuditRow[]>([]);
  const [executionLogs, setExecutionLogs] = useState<ExecutionLogRow[]>([]);
  const [summary, setSummary] = useState<AuditSummary | null>(null);
  const [loading, setLoading] = useState(false);
  // M1: 添加 error state
  const [error, setError] = useState<string | null>(null);
  const [stepFilter, setStepFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [audits, logs, summaryData] = await Promise.all([
        getProcessingAudits({ limit: 100 }),
        getExecutionLogs({ limit: 100 }),
        getAuditSummary(),
      ]);
      setProcessingAudits(audits);
      setExecutionLogs(logs);
      setSummary(summaryData);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("Failed to load audit data:", err);
      setError(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // M3: 使用 useMemo 优化派生数据
  const filteredProcessing = useMemo(() =>
    stepFilter === "all"
      ? processingAudits
      : processingAudits.filter((a) => a.step === stepFilter),
    [processingAudits, stepFilter]
  );

  const filteredExecution = useMemo(() =>
    statusFilter === "all"
      ? executionLogs
      : executionLogs.filter((l) => l.status === statusFilter),
    [executionLogs, statusFilter]
  );

  const steps = useMemo(() =>
    Array.from(new Set(processingAudits.map((a) => a.step))),
    [processingAudits]
  );

  const statuses = useMemo(() =>
    Array.from(new Set(executionLogs.map((l) => l.status))),
    [executionLogs]
  );

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">审计日志</h1>
          <Badge variant="secondary" className="text-xs">
            <Clock className="mr-1 h-3 w-3" />
            Developer Mode
          </Badge>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          disabled={loading}
          className="h-8 gap-1.5"
        >
          {loading ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <RefreshCw className="h-3.5 w-3.5" />
          )}
          刷新
        </Button>
      </header>

      {/* Summary */}
      <div className="px-6 py-4">
        <SummaryCards summary={summary} />
      </div>

      {/* Tabs */}
      <Tabs
        value={tab}
        onValueChange={(v) => setTab(v as AuditTab)}
        className="flex-1 flex flex-col overflow-hidden"
      >
        <div className="flex items-center justify-between px-6">
          <TabsList>
            <TabsTrigger value="processing" className="gap-1.5">
              <Cpu className="h-3.5 w-3.5" />
              处理日志
              <Badge variant="secondary" className="ml-1 h-5 min-w-5 justify-center rounded-full px-1 text-[10px]">
                {filteredProcessing.length}
              </Badge>
            </TabsTrigger>
            <TabsTrigger value="execution" className="gap-1.5">
              <Activity className="h-3.5 w-3.5" />
              执行日志
              <Badge variant="secondary" className="ml-1 h-5 min-w-5 justify-center rounded-full px-1 text-[10px]">
                {filteredExecution.length}
              </Badge>
            </TabsTrigger>
          </TabsList>

          {tab === "processing" ? (
            <Select value={stepFilter} onValueChange={(v) => v !== null && setStepFilter(v)}>
              <SelectTrigger className="h-8 w-[130px] text-xs">
                <SelectValue placeholder="全部步骤" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部步骤</SelectItem>
                {steps.map((s) => (
                  <SelectItem key={s} value={s}>{s}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            <Select value={statusFilter} onValueChange={(v) => v !== null && setStatusFilter(v)}>
              <SelectTrigger className="h-8 w-[130px] text-xs">
                <SelectValue placeholder="全部状态" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                {statuses.map((s) => (
                  <SelectItem key={s} value={s}>{s}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>

        <TabsContent value="processing" className="flex-1 overflow-hidden mt-0">
          <ScrollArea className="h-full px-6 py-4">
            {loading && processingAudits.length === 0 ? (
              <div className="flex h-40 items-center justify-center text-muted-foreground">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                加载中...
              </div>
            ) : error ? (
              // M1: 显示错误状态
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-destructive">
                <AlertCircle className="h-8 w-8" />
                <span className="text-sm">{error}</span>
                <Button variant="outline" size="sm" onClick={refresh}>
                  重试
                </Button>
              </div>
            ) : filteredProcessing.length === 0 ? (
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Inbox className="h-8 w-8" />
                <span className="text-sm">暂无处理日志</span>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                {filteredProcessing.map((audit) => (
                  <ProcessingAuditItem
                    key={`${audit.event_id}-${audit.trace_id}-${audit.step}`}
                    audit={audit}
                  />
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>

        <TabsContent value="execution" className="flex-1 overflow-hidden mt-0">
          <ScrollArea className="h-full px-6 py-4">
            {loading && executionLogs.length === 0 ? (
              <div className="flex h-40 items-center justify-center text-muted-foreground">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                加载中...
              </div>
            ) : error ? (
              // M1: 显示错误状态
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-destructive">
                <AlertCircle className="h-8 w-8" />
                <span className="text-sm">{error}</span>
                <Button variant="outline" size="sm" onClick={refresh}>
                  重试
                </Button>
              </div>
            ) : filteredExecution.length === 0 ? (
              <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Inbox className="h-8 w-8" />
                <span className="text-sm">暂无执行日志</span>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                {filteredExecution.map((log) => (
                  <ExecutionLogItem key={log.id} log={log} />
                ))}
              </div>
            )}
          </ScrollArea>
        </TabsContent>
      </Tabs>
    </div>
  );
}
