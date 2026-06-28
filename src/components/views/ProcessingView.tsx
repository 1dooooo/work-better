import { useState, useEffect, useCallback } from "react";
import {
  getEvents,
  processEvent,
  type Event,
  type ProcessResult,
} from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
  Zap,
  Database,
  FileText,
} from "lucide-react";
import { toast } from "sonner";

export default function ProcessingView() {
  const [events, setEvents] = useState<Event[]>([]);
  const [processedIds, setProcessedIds] = useState<Set<string>>(new Set());
  const [processing, setProcessing] = useState<Record<string, ProcessResult>>({});
  const [loading, setLoading] = useState(false);
  const [processingEvent, setProcessingEvent] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getEvents(20);
      setEvents(data);
    } catch (err) {
      console.error("Failed to load events:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleProcess = async (eventId: string) => {
    setProcessingEvent(eventId);
    try {
      const result = await processEvent(eventId);
      setProcessing((prev) => ({ ...prev, [eventId]: result }));
      setProcessedIds((prev) => new Set(prev).add(eventId));
      toast.success("处理完成");
    } catch (err) {
      console.error("Process failed:", err);
      toast.error("处理失败");
    } finally {
      setProcessingEvent(null);
    }
  };

  const getReviewStatusBadge = (status: ProcessResult["review_status"]) => {
    if ("Pending" in status) {
      return (
        <Badge variant="secondary" className="gap-1">
          <Clock className="h-3 w-3" />
          待审批
        </Badge>
      );
    }
    if ("Approved" in status) {
      return (
        <Badge variant="default" className="gap-1 bg-success">
          <CheckCircle2 className="h-3 w-3" />
          已批准
        </Badge>
      );
    }
    if ("Rejected" in status) {
      return (
        <Badge variant="destructive" className="gap-1">
          <XCircle className="h-3 w-3" />
          已拒绝
        </Badge>
      );
    }
    return null;
  };

  const getPersistenceStatus = (status: ProcessResult["persistence_status"]) => {
    const items = [
      { key: "obsidian", label: "Obsidian", icon: FileText, value: status.obsidian },
      { key: "vector_db", label: "VectorDB", icon: Database, value: status.vector_db },
      { key: "sqlite", label: "SQLite", icon: Database, value: status.sqlite },
    ];

    return (
      <div className="flex gap-2">
        {items.map((item) => (
          <div
            key={item.key}
            className={`flex items-center gap-1 text-xs ${
              item.value ? "text-success" : "text-muted-foreground"
            }`}
          >
            <item.icon className="h-3 w-3" />
            {item.label}
            {item.value ? (
              <CheckCircle2 className="h-3 w-3" />
            ) : (
              <XCircle className="h-3 w-3" />
            )}
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className="flex h-full flex-col" data-testid="processing-container">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">处理管线</h1>
          <Badge variant="secondary" className="text-xs">
            {events.filter((e) => !processedIds.has(e.id)).length} 待处理
          </Badge>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          className="h-8 gap-1.5"
        >
          <Loader2 className="h-3.5 w-3.5" />
          刷新
        </Button>
      </header>

      {/* Content */}
      <ScrollArea className="flex-1 px-6 py-4">
        {loading && events.length === 0 ? (
          <div className="flex h-40 items-center justify-center text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            加载中...
          </div>
        ) : events.filter((e) => !processedIds.has(e.id)).length === 0 ? (
          <div className="flex h-40 flex-col items-center justify-center gap-2 text-muted-foreground">
            <CheckCircle2 className="h-8 w-8" />
            <span className="text-sm">所有事件已处理</span>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            {events.filter((e) => !processedIds.has(e.id)).map((event) => {
              const result = processing[event.id];
              const isProcessing = processingEvent === event.id;

              return (
                <Card key={event.id} className="border-border" data-testid={`processing-event-${event.id}`}>
                  <CardHeader className="flex flex-row items-center gap-2 px-4 py-3">
                    <Badge variant="outline" className="text-[11px]">
                      {event.type}
                    </Badge>
                    <Badge variant="secondary" className="text-[11px]">
                      {event.source}
                    </Badge>
                    <span className="ml-auto text-[11px] text-muted-foreground">
                      {new Date(event.timestamp).toLocaleString("zh-CN")}
                    </span>
                  </CardHeader>
                  <CardContent className="px-4 pb-4 pt-0">
                    <pre className="whitespace-pre-wrap break-words text-sm text-foreground/90 mb-3">
                      {typeof event.content === "string"
                        ? event.content
                        : JSON.stringify(event.content, null, 2)}
                    </pre>

                    {result ? (
                      <div className="space-y-3">
                        {/* Processing Result */}
                        <div className="grid grid-cols-2 gap-4 rounded-lg bg-muted p-3">
                          <div>
                            <div className="text-xs text-muted-foreground mb-1">
                              分类
                            </div>
                            <div className="text-sm font-medium">
                              {result.category}
                            </div>
                          </div>
                          <div>
                            <div className="text-xs text-muted-foreground mb-1">
                              置信度
                            </div>
                            <div className="text-sm font-medium">
                              {(result.confidence * 100).toFixed(1)}%
                            </div>
                          </div>
                          <div>
                            <div className="text-xs text-muted-foreground mb-1">
                              处理路径
                            </div>
                            <div className="text-sm font-medium">
                              {result.processing_path}
                            </div>
                          </div>
                          <div>
                            <div className="text-xs text-muted-foreground mb-1">
                              使用模型
                            </div>
                            <div className="text-sm font-medium">
                              {result.model_used}
                            </div>
                          </div>
                        </div>

                        {/* Review Status */}
                        <div>
                          <div className="text-xs text-muted-foreground mb-2">
                            审批状态
                          </div>
                          {getReviewStatusBadge(result.review_status)}
                        </div>

                        {/* Persistence Status */}
                        <div>
                          <div className="text-xs text-muted-foreground mb-2">
                            持久化状态
                          </div>
                          {getPersistenceStatus(result.persistence_status)}
                        </div>
                      </div>
                    ) : (
                      <div className="flex justify-end">
                        <Button
                          size="sm"
                          onClick={() => handleProcess(event.id)}
                          disabled={isProcessing}
                          className="gap-1.5"
                          data-testid={`process-button-${event.id}`}
                        >
                          {isProcessing ? (
                            <Loader2 className="h-3.5 w-3.5 animate-spin" />
                          ) : (
                            <Zap className="h-3.5 w-3.5" />
                          )}
                          {isProcessing ? "处理中..." : "处理"}
                        </Button>
                      </div>
                    )}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
