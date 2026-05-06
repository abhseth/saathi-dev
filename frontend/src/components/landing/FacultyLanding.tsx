import React from "react";
import { SectionLanding } from "./SectionLanding";
import type { AppUser, SubstitutionRecord, LeaveRequest } from "../../types";

type FacultyLandingProps = {
  users: AppUser[];
  substitutions: SubstitutionRecord[];
  leaveRequests: LeaveRequest[];
  currentUserRole: string;
  onOpenTool: (toolId: string) => void;
};

export function FacultyLanding({ users, substitutions, leaveRequests, currentUserRole, onOpenTool }: FacultyLandingProps) {
  const facultyCount = users.filter((u) => u.role === "faculty").length;
  const pendingSubs = substitutions.filter((s) => s.status === "Pending");
  const pendingLeaves = leaveRequests.filter((l) => l.status === "Pending");

  const attentionNeeded = [
    ...(pendingSubs.length > 0
      ? [{ label: "Pending substitutions", count: pendingSubs.length, severity: "warning" as const, onClick: () => onOpenTool("substitution-command-center") }]
      : []),
    ...(pendingLeaves.length > 0
      ? [{ label: "Leave requests awaiting approval", count: pendingLeaves.length, severity: "info" as const, onClick: () => onOpenTool("leave-swap") }]
      : []),
  ];

  return (
    <SectionLanding
      section="faculty"
      currentUserRole={currentUserRole}
      onOpenTool={onOpenTool}
      attentionNeeded={attentionNeeded}
      recentItems={
        pendingSubs.length > 0 ? (
          <ul className="landing-recent-list">
            {pendingSubs.slice(0, 5).map((s, i) => (
              <li key={i} className="landing-recent-item">
                <span className="landing-recent-name">{s.original_faculty_name}</span>
                <span className="landing-recent-meta">
                  {s.subject_name} · {s.grade_level} · {s.session_date}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="landing-empty">No pending substitutions.</p>
        )
      }
    >
      <div className="landing-metrics">
        <div className="landing-metric">
          <span className="landing-metric-value">{facultyCount}</span>
          <span className="landing-metric-label">Faculty</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{pendingSubs.length}</span>
          <span className="landing-metric-label">Pending Subs</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{pendingLeaves.length}</span>
          <span className="landing-metric-label">Pending Leave</span>
        </div>
      </div>
    </SectionLanding>
  );
}
