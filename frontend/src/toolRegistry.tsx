import React from "react";

/* ── Icons (lightweight inline SVGs) ─────────────────────────────────────── */

function IconDashboard({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
    </svg>
  );
}

function IconSchool({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 21h18M5 21V7l8-4 8 4v14M9 10h.01M15 10h.01M9 14h.01M15 14h.01" />
    </svg>
  );
}

function IconChart({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 20V10M12 20V4M6 20v-6" />
    </svg>
  );
}

function IconMessage({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
    </svg>
  );
}

function IconMapPin({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z" /><circle cx="12" cy="10" r="3" />
    </svg>
  );
}

function IconArchive({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 8v13H3V8M1 3h22v5H1zM10 12h4" />
    </svg>
  );
}

function IconHistory({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
    </svg>
  );
}

function IconClipboard({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2" /><rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
    </svg>
  );
}

function IconRoute({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <circle cx="6" cy="19" r="3" /><circle cx="18" cy="5" r="3" /><path d="M12 19h4.5a3.5 3.5 0 000-7h-5a3.5 3.5 0 010-7H12" />
    </svg>
  );
}

function IconAlert({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" /><line x1="12" y1="9" x2="12" y2="13" /><line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  );
}

function IconSliders({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <line x1="4" y1="21" x2="4" y2="14" /><line x1="4" y1="10" x2="4" y2="3" />
      <line x1="12" y1="21" x2="12" y2="12" /><line x1="12" y1="8" x2="12" y2="3" />
      <line x1="20" y1="21" x2="20" y2="16" /><line x1="20" y1="12" x2="20" y2="3" />
      <line x1="1" y1="14" x2="7" y2="14" /><line x1="9" y1="8" x2="15" y2="8" /><line x1="17" y1="16" x2="23" y2="16" />
    </svg>
  );
}

function IconFileText({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /><polyline points="10 9 9 9 8 9" />
    </svg>
  );
}

function IconRefresh({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10" /><polyline points="1 20 1 14 7 14" /><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
    </svg>
  );
}

function IconDownload({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
    </svg>
  );
}

function IconDatabase({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" /><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
    </svg>
  );
}

function IconUsers({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75" />
    </svg>
  );
}

function IconBriefcase({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="7" width="20" height="14" rx="2" ry="2" /><path d="M16 21V5a2 2 0 00-2-2h-4a2 2 0 00-2 2v16" />
    </svg>
  );
}

function IconBook({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 19.5A2.5 2.5 0 016.5 17H20" /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" />
    </svg>
  );
}

function IconCalendar({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  );
}

function IconGraduationCap({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 10l-10-6-10 6 10 6 10-6z" /><path d="M6 12v5a6 6 0 0012 0v-5" />
    </svg>
  );
}

function IconShuffle({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="16 3 21 3 21 8" /><line x1="4" y1="20" x2="21" y2="3" /><polyline points="21 16 21 21 16 21" /><line x1="15" y1="15" x2="21" y2="21" /><line x1="4" y1="4" x2="9" y2="9" />
    </svg>
  );
}

function IconCalendarOff({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" /><line x1="16" y1="2" x2="16" y2="6" /><line x1="8" y1="2" x2="8" y2="6" /><line x1="3" y1="10" x2="21" y2="10" /><line x1="2" y1="2" x2="22" y2="22" />
    </svg>
  );
}

function IconBell({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 01-3.46 0" />
    </svg>
  );
}

function IconBarChart({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="20" x2="12" y2="10" /><line x1="18" y1="20" x2="18" y2="4" /><line x1="6" y1="20" x2="6" y2="16" />
    </svg>
  );
}

/* ── Tool registry ───────────────────────────────────────────────────────── */

export type AdminView =
  | "master-data"
  | "program-dashboard"
  | "reports"
  | "communications"
  | "directory"
  | "dropped-schools"
  | "region-log"
  | "audit-log"
  | "routing"
  | "escalation"
  | "sla"
  | "templates"
  | "faculty-assignments"
  | "faculty-members"
  | "timetable"
  | "subjects"
  | "substitutions"
  | "leave-swap"
  | "alert-inbox"
  | "control-tower"
  | "compliance-scorecard"
  | "deviation-scoreboard"
  | "holidays"
  | null;

export interface AppTool {
  id: string;
  label: string;
  icon: React.ReactNode;
  /** Allowed roles. Empty = not visible to anyone. */
  roles: string[];
  /** The AdminView this tool opens. Omit for action-only tools. */
  view?: AdminView;
  /** If true, the tool triggers an action instead of opening a view. */
  action?: boolean;
}

export const APP_TOOLS: AppTool[] = [
  { id: "master-data", label: "Master Data", icon: React.createElement(IconDashboard), roles: ["admin", "aom"], view: "master-data" },
  { id: "program-dashboard", label: "Program Dashboard", icon: React.createElement(IconSchool), roles: ["admin"], view: "program-dashboard" },
  { id: "reports", label: "Reports", icon: React.createElement(IconChart), roles: ["admin", "agent", "viewer", "aom"], view: "reports" },
  { id: "communications", label: "Communication Ops", icon: React.createElement(IconMessage), roles: ["admin", "agent", "aom"], view: "communications" },
  { id: "directory", label: "Directory", icon: React.createElement(IconMapPin), roles: ["admin", "agent", "viewer", "aom"], view: "directory" },
  { id: "dropped-schools", label: "Dropped Schools", icon: React.createElement(IconArchive), roles: ["admin", "aom"], view: "dropped-schools" },
  { id: "region-log", label: "Region Log", icon: React.createElement(IconHistory), roles: ["admin"], view: "region-log" },
  { id: "audit-log", label: "Audit Log", icon: React.createElement(IconClipboard), roles: ["admin"], view: "audit-log" },
  { id: "routing", label: "Routing", icon: React.createElement(IconRoute), roles: ["admin"], view: "routing" },
  { id: "escalation", label: "Escalation", icon: React.createElement(IconAlert), roles: ["admin"], view: "escalation" },
  { id: "sla", label: "SLA Settings", icon: React.createElement(IconSliders), roles: ["admin"], view: "sla" },
  { id: "templates", label: "Templates", icon: React.createElement(IconFileText), roles: ["admin"], view: "templates" },
  { id: "export-csv", label: "Export Tickets", icon: React.createElement(IconDownload), roles: ["admin"], action: true },
  { id: "users", label: "Users", icon: React.createElement(IconUsers), roles: ["admin"], action: true },
  { id: "faculty-assignments", label: "Faculty Assignments", icon: React.createElement(IconBriefcase), roles: ["admin", "aom"], view: "faculty-assignments" },
  { id: "subjects", label: "Subjects", icon: React.createElement(IconBook), roles: ["admin", "aom"], view: "subjects" },
  { id: "faculty-members", label: "Faculty Master", icon: React.createElement(IconGraduationCap), roles: ["admin", "aom"], view: "faculty-members" },
  { id: "timetable", label: "Timetable", icon: React.createElement(IconCalendar), roles: ["admin", "aom"], view: "timetable" },
  { id: "substitutions", label: "Substitutions", icon: React.createElement(IconShuffle), roles: ["admin", "aom"], view: "substitutions" },
  { id: "leave-swap", label: "Leave & Swap", icon: React.createElement(IconCalendarOff), roles: ["admin", "aom"], view: "leave-swap" },
  { id: "alert-inbox", label: "Alert Inbox", icon: React.createElement(IconBell), roles: ["admin", "aom"], view: "alert-inbox" },
  { id: "control-tower", label: "Control Tower", icon: React.createElement(IconDashboard), roles: ["admin", "aom"], view: "control-tower" },
  { id: "compliance-scorecard", label: "Compliance", icon: React.createElement(IconClipboard), roles: ["admin", "aom"], view: "compliance-scorecard" },
  { id: "deviation-scoreboard", label: "Deviation", icon: React.createElement(IconBarChart), roles: ["admin", "aom"], view: "deviation-scoreboard" },
  { id: "holidays", label: "Holidays", icon: React.createElement(IconCalendar), roles: ["admin", "aom"], view: "holidays" },
];

export function isToolVisible(tool: AppTool, currentUserRole: string): boolean {
  return tool.roles.length > 0 && tool.roles.includes(currentUserRole);
}

export function canAccessTool(toolId: string, currentUserRole: string): boolean {
  const tool = APP_TOOLS.find((t) => t.id === toolId);
  if (!tool) return false;
  return isToolVisible(tool, currentUserRole);
}
