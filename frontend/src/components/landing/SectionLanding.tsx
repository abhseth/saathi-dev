import React from "react";
import type { AppSection } from "../../navigation";
import { SECTION_LABELS, toolsForSection } from "../../navigation";

export type LandingTask = {
  label: string;
  onClick: () => void;
  variant?: "primary" | "secondary" | "danger";
  toolId?: string;
};

export type LandingAlert = {
  label: string;
  count?: number;
  onClick?: () => void;
  severity?: "info" | "warning" | "critical";
};

type SectionLandingProps = {
  section: AppSection;
  currentUserRole: string;
  topTasks?: LandingTask[];
  attentionNeeded?: LandingAlert[];
  recentItems?: React.ReactNode;
  children?: React.ReactNode;
  onOpenTool: (toolId: string) => void;
};

export function SectionLanding({
  section,
  currentUserRole,
  topTasks: topTasksProp,
  attentionNeeded,
  recentItems,
  children,
  onOpenTool,
}: SectionLandingProps) {
  const tools = React.useMemo(
    () => toolsForSection(section, currentUserRole),
    [section, currentUserRole],
  );
  const primary = tools.filter((t) => t.mobile === "card");
  const secondary = tools.filter((t) => t.mobile === "list");

  // Auto-derive top tasks from first 3 card tools when not explicitly provided
  const defaultTopTasks: LandingTask[] = React.useMemo(
    () =>
      primary.slice(0, 3).map((tool, i) => ({
        label: tool.label,
        onClick: () => onOpenTool(tool.id),
        variant: i === 0 ? "primary" : "secondary",
        toolId: tool.id,
      })),
    [primary, onOpenTool],
  );
  const topTasks = topTasksProp ?? defaultTopTasks;
  const topTaskIds = new Set(topTasks.map((t) => t.toolId).filter(Boolean) as string[]);
  const primaryCards = primary.filter((t) => !topTaskIds.has(t.id));

  return (
    <section className="section-landing" aria-label={`${SECTION_LABELS[section]} landing`}>
      <div className="section-landing-header">
        <h1>{SECTION_LABELS[section]}</h1>
      </div>

      {children}

      {topTasks && topTasks.length > 0 && (
        <div className="landing-top-tasks">
          <h2>Top tasks</h2>
          <div className="landing-task-list">
            {topTasks.map((task, i) => (
              <button
                key={i}
                type="button"
                className={`landing-task landing-task--${task.variant ?? "secondary"}`}
                onClick={task.onClick}
              >
                {task.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {attentionNeeded && attentionNeeded.length > 0 && (
        <div className="landing-attention">
          <h2>Attention needed</h2>
          <div className="landing-alert-list">
            {attentionNeeded.map((alert, i) => (
              <button
                key={i}
                type="button"
                className={`landing-alert landing-alert--${alert.severity ?? "info"}`}
                onClick={alert.onClick}
                disabled={!alert.onClick}
              >
                <span className="landing-alert-label">{alert.label}</span>
                {alert.count !== undefined && alert.count > 0 && (
                  <span className="landing-alert-count">{alert.count}</span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {recentItems && (
        <div className="landing-recent">
          <h2>Recent</h2>
          {recentItems}
        </div>
      )}

      {primaryCards.length > 0 && (
        <div className="section-landing-primary">
          {primaryCards.map((tool) => (
            <button
              key={tool.id}
              type="button"
              className="landing-card"
              onClick={() => onOpenTool(tool.id)}
            >
              <span className="landing-card-label">{tool.label}</span>
            </button>
          ))}
        </div>
      )}

      {secondary.length > 0 && (
        <div className="section-landing-secondary">
          <h2>All tools</h2>
          <div className="landing-tool-list">
            {secondary.map((tool) => (
              <button
                key={tool.id}
                type="button"
                className="landing-tool-item"
                onClick={() => onOpenTool(tool.id)}
              >
                {tool.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
