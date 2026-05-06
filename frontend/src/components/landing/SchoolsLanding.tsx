import React from "react";
import { SectionLanding } from "./SectionLanding";
import type { School, TimetableHealthStatus } from "../../types";

type SchoolsLandingProps = {
  schools: School[];
  healthData: TimetableHealthStatus[];
  currentUserRole: string;
  onOpenTool: (toolId: string) => void;
};

export function SchoolsLanding({ schools, healthData, currentUserRole, onOpenTool }: SchoolsLandingProps) {
  const activeSchools = schools.filter((s) => !s.is_dropped).length;
  const redSchools = healthData.filter((h) => h.status === "Red");
  const amberSchools = healthData.filter((h) => h.status === "Amber");
  const atRiskSchools = [...redSchools, ...amberSchools].slice(0, 5);

  const attentionNeeded = [
    ...(redSchools.length > 0
      ? [{ label: "Schools in critical health", count: redSchools.length, severity: "critical" as const, onClick: () => onOpenTool("timetable-health") }]
      : []),
    ...(amberSchools.length > 0
      ? [{ label: "Schools need attention", count: amberSchools.length, severity: "warning" as const, onClick: () => onOpenTool("timetable-health") }]
      : []),
  ];

  return (
    <SectionLanding
      section="schools"
      currentUserRole={currentUserRole}
      onOpenTool={onOpenTool}
      attentionNeeded={attentionNeeded}
      recentItems={
        atRiskSchools.length > 0 ? (
          <ul className="landing-recent-list">
            {atRiskSchools.map((h) => (
              <li key={h.school_id} className={`landing-recent-item landing-recent-item--${h.status?.toLowerCase() ?? "info"}`}>
                <span className="landing-recent-name">{h.school_name}</span>
                <span className="landing-recent-meta">
                  {h.gaps_count ?? 0} gaps · {h.aom_name || "No AOM"}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="landing-empty">All schools are healthy.</p>
        )
      }
    >
      <div className="landing-metrics">
        <div className="landing-metric">
          <span className="landing-metric-value">{activeSchools}</span>
          <span className="landing-metric-label">Active Schools</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{redSchools.length}</span>
          <span className="landing-metric-label">Critical</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{amberSchools.length}</span>
          <span className="landing-metric-label">At Risk</span>
        </div>
      </div>
    </SectionLanding>
  );
}
