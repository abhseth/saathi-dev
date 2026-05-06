import React from "react";
import { SectionLanding } from "./SectionLanding";
import type { School, TimetableHealthStatus } from "../../types";

type TimetableLandingProps = {
  schools: School[];
  healthData: TimetableHealthStatus[];
  currentUserRole: string;
  onOpenTool: (toolId: string) => void;
};

export function TimetableLanding({ schools, healthData, currentUserRole, onOpenTool }: TimetableLandingProps) {
  const activeSchools = schools.filter((s) => !s.is_dropped).length;
  const gaps = healthData.reduce((sum, h) => sum + (h.gaps_count ?? 0), 0);
  const schoolsWithGaps = healthData.filter((h) => (h.gaps_count ?? 0) > 0).slice(0, 5);

  const attentionNeeded = [
    ...(gaps > 0
      ? [{ label: "Timetable gaps across schools", count: gaps, severity: "warning" as const, onClick: () => onOpenTool("timetable-health") }]
      : []),
  ];

  return (
    <SectionLanding
      section="timetable"
      currentUserRole={currentUserRole}
      onOpenTool={onOpenTool}
      attentionNeeded={attentionNeeded}
      recentItems={
        schoolsWithGaps.length > 0 ? (
          <ul className="landing-recent-list">
            {schoolsWithGaps.map((h) => (
              <li key={h.school_id} className="landing-recent-item">
                <span className="landing-recent-name">{h.school_name}</span>
                <span className="landing-recent-meta">
                  {h.gaps_count} gaps · {h.master_timetable_complete ? "Complete" : "Incomplete"}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="landing-empty">No timetable gaps.</p>
        )
      }
    >
      <div className="landing-metrics">
        <div className="landing-metric">
          <span className="landing-metric-value">{activeSchools}</span>
          <span className="landing-metric-label">Schools Scheduled</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{gaps}</span>
          <span className="landing-metric-label">Timetable Gaps</span>
        </div>
      </div>
    </SectionLanding>
  );
}
