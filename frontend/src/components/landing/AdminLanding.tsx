import React from "react";
import { SectionLanding } from "./SectionLanding";
import type { AppUser } from "../../types";

type AdminLandingProps = {
  users: AppUser[];
  currentUserRole: string;
  onOpenTool: (toolId: string) => void;
  onExport?: () => void;
};

export function AdminLanding({ users, currentUserRole, onOpenTool, onExport }: AdminLandingProps) {
  const userCount = users.length;
  const inactiveUsers = users.filter((u) => !u.is_active).length;

  const topTasks = [
    { label: "User management", onClick: () => onOpenTool("users"), variant: "primary" as const, toolId: "users" as const },
    { label: "Audit log", onClick: () => onOpenTool("audit-log"), variant: "secondary" as const, toolId: "audit-log" as const },
    { label: "System sync", onClick: () => onOpenTool("sync"), variant: "secondary" as const, toolId: "sync" as const },
    ...(onExport ? [{ label: "Export data", onClick: onExport, variant: "secondary" as const }] : []),
  ];

  const attentionNeeded = [
    ...(inactiveUsers > 0
      ? [{ label: "Inactive users", count: inactiveUsers, severity: "info" as const, onClick: () => onOpenTool("users") }]
      : []),
  ];

  return (
    <SectionLanding
      section="admin"
      currentUserRole={currentUserRole}
      onOpenTool={onOpenTool}
      topTasks={topTasks}
      attentionNeeded={attentionNeeded}
    >
      <div className="landing-metrics">
        <div className="landing-metric">
          <span className="landing-metric-value">{userCount}</span>
          <span className="landing-metric-label">System Users</span>
        </div>
        <div className="landing-metric">
          <span className="landing-metric-value">{inactiveUsers}</span>
          <span className="landing-metric-label">Inactive</span>
        </div>
      </div>
    </SectionLanding>
  );
}
