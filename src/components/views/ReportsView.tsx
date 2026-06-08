import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { FileText, BarChart3, TrendingUp, Construction } from "lucide-react";
import { cn } from "@/lib/utils";
import type { LucideIcon } from "lucide-react";

type ReportType = "daily" | "weekly" | "monthly";

interface ReportPlaceholder {
  type: ReportType;
  label: string;
  description: string;
  icon: LucideIcon;
}

const REPORT_TYPES: ReportPlaceholder[] = [
  {
    type: "daily",
    label: "日报",
    description: "自动生成每日工作总结，包含完成的任务、关键事件和待办事项",
    icon: FileText,
  },
  {
    type: "weekly",
    label: "周报",
    description: "汇总一周工作内容，分析工作模式和效率趋势",
    icon: BarChart3,
  },
  {
    type: "monthly",
    label: "月报",
    description: "月度工作回顾，包含项目进展、目标达成率和改进建议",
    icon: TrendingUp,
  },
];

export default function ReportsView() {
  const [selectedType, setSelectedType] = useState<ReportType | null>(null);

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-6 py-4">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">报告</h1>
          <Badge variant="outline" className="text-xs">
            即将推出
          </Badge>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 p-6">
        <p className="mb-4 text-sm text-muted-foreground">
          报告功能将在后续版本中启用。选择报告类型查看详情说明。
        </p>

        <div className="grid gap-4 sm:grid-cols-3">
          {REPORT_TYPES.map((report) => {
            const Icon = report.icon;
            const isSelected = selectedType === report.type;
            return (
              <Card
                key={report.type}
                className={cn(
                  "cursor-pointer border-border transition-all hover:shadow-md",
                  isSelected && "ring-2 ring-primary"
                )}
                onClick={() =>
                  setSelectedType(isSelected ? null : report.type)
                }
              >
                <CardHeader className="pb-2">
                  <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                      <Icon className="h-4 w-4 text-primary" />
                    </div>
                    <CardTitle className="text-sm">{report.label}</CardTitle>
                  </div>
                </CardHeader>
                <CardContent>
                  {isSelected && (
                    <p className="text-xs text-muted-foreground leading-relaxed">
                      {report.description}
                    </p>
                  )}
                </CardContent>
              </Card>
            );
          })}
        </div>

        {/* Placeholder Banner */}
        <div className="mt-8 flex items-center gap-3 rounded-lg border border-dashed border-border px-4 py-3">
          <Construction className="h-5 w-5 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">
            即将推出：自动生成日报、周报、月报
          </span>
        </div>
      </div>
    </div>
  );
}
