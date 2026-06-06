import { useState } from "react";

type ReportType = "daily" | "weekly" | "monthly";

interface ReportPlaceholder {
  type: ReportType;
  label: string;
  description: string;
  icon: string;
}

const REPORT_TYPES: ReportPlaceholder[] = [
  {
    type: "daily",
    label: "日报",
    description: "自动生成每日工作总结，包含完成的任务、关键事件和待办事项",
    icon: "📝",
  },
  {
    type: "weekly",
    label: "周报",
    description: "汇总一周工作内容，分析工作模式和效率趋势",
    icon: "📊",
  },
  {
    type: "monthly",
    label: "月报",
    description: "月度工作回顾，包含项目进展、目标达成率和改进建议",
    icon: "📈",
  },
];

export default function ReportsView() {
  const [selectedType, setSelectedType] = useState<ReportType | null>(null);

  return (
    <div className="view reports-view">
      <header className="view__header">
        <h2 className="view__title">报告</h2>
      </header>

      <div className="view__description">
        <p>报告功能将在后续版本中启用。选择报告类型查看详情说明。</p>
      </div>

      <div className="reports-view__grid">
        {REPORT_TYPES.map((report) => (
          <button
            key={report.type}
            className={`reports-view__card ${
              selectedType === report.type ? "reports-view__card--selected" : ""
            }`}
            onClick={() =>
              setSelectedType(
                selectedType === report.type ? null : report.type
              )
            }
          >
            <span className="reports-view__card-icon">{report.icon}</span>
            <h3 className="reports-view__card-title">{report.label}</h3>
            {selectedType === report.type && (
              <p className="reports-view__card-desc">{report.description}</p>
            )}
          </button>
        ))}
      </div>

      <div className="reports-view__placeholder">
        <p>🚧 即将推出：自动生成日报、周报、月报</p>
      </div>
    </div>
  );
}
