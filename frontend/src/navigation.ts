// ── Section-based navigation registry ───────────────────────────────────────
// This file is the single source of truth for the new shell architecture.
// Every legacy adminView string is mapped here to its section, label, and
// role visibility. This prevents migration mistakes across 50+ tools.

export type AppSection = "work" | "schools" | "faculty" | "timetable" | "reports" | "admin";

export type ToolDef = {
  id: string; // matches legacy adminView string
  section: AppSection;
  label: string;
  roles: string[];
  mobile: "card" | "list" | "hidden";
};

export const SECTION_LABELS: Record<AppSection, string> = {
  work: "Work",
  schools: "Schools",
  faculty: "Faculty",
  timetable: "Timetable",
  reports: "Reports",
  admin: "Admin",
};

export const SECTION_ORDER: AppSection[] = [
  "work",
  "schools",
  "faculty",
  "timetable",
  "reports",
  "admin",
];

export const TOOL_REGISTRY: ToolDef[] = [
  // ── Work ──
  {
    id: "communications",
    section: "work",
    label: "Communication Ops",
    roles: ["admin", "aom", "head", "agent"],
    mobile: "card",
  },
  {
    id: "alert-inbox",
    section: "work",
    label: "Alert Inbox",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "list",
  },
  {
    id: "bulk-ops",
    section: "work",
    label: "Bulk Operations",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "reassign-wizard",
    section: "work",
    label: "Reassign Wizard",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "ticket-from-gap",
    section: "work",
    label: "Ticket from Gap",
    roles: ["admin", "aom"],
    mobile: "list",
  },

  // ── Schools ──
  {
    id: "master-data",
    section: "schools",
    label: "Master Data",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "card",
  },
  {
    id: "directory",
    section: "schools",
    label: "Directory",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "card",
  },
  {
    id: "program-dashboard",
    section: "schools",
    label: "Program Dashboard",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "dropped-schools",
    section: "schools",
    label: "Dropped Schools",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "region-log",
    section: "schools",
    label: "Region Log",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "list",
  },

  // ── Faculty ──
  {
    id: "faculty-assignments",
    section: "faculty",
    label: "Faculty Assignments",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "cross-school-faculty",
    section: "faculty",
    label: "Cross-School Faculty",
    roles: ["admin", "aom", "head"],
    mobile: "list",
  },
  {
    id: "faculty-directory",
    section: "faculty",
    label: "Faculty Directory",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "session-manager",
    section: "faculty",
    label: "Session Manager",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "bulk-quick-attendance",
    section: "faculty",
    label: "Bulk Attendance",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "substitution-command-center",
    section: "faculty",
    label: "Substitution Center",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "leave-swap",
    section: "faculty",
    label: "Leave & Swap",
    roles: ["admin", "aom", "head", "faculty"],
    mobile: "card",
  },
  {
    id: "substitution-analytics",
    section: "faculty",
    label: "Substitution Analytics",
    roles: ["admin", "aom"],
    mobile: "list",
  },

  // ── Timetable ──
  {
    id: "timetable",
    section: "timetable",
    label: "Timetable",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "weekly-timetable",
    section: "timetable",
    label: "Weekly Timetable",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "day-at-glance",
    section: "timetable",
    label: "Day at a Glance",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "multi-day-glance",
    section: "timetable",
    label: "Multi-School Day",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "school-master-timetable",
    section: "timetable",
    label: "Master Timetable",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "grade-timetable",
    section: "timetable",
    label: "Grade Timetable",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "faculty-timetable",
    section: "timetable",
    label: "Faculty Timetable",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "holidays",
    section: "timetable",
    label: "Holidays",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "subjects",
    section: "timetable",
    label: "Subjects",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "week-clone",
    section: "timetable",
    label: "Clone Week",
    roles: ["admin", "aom"],
    mobile: "list",
  },

  // ── Reports ──
  {
    id: "reports",
    section: "reports",
    label: "Reports",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "card",
  },
  {
    id: "attendance-reports",
    section: "reports",
    label: "Attendance Reports",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "control-tower",
    section: "reports",
    label: "Control Tower",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "compliance-scorecard",
    section: "reports",
    label: "Compliance Scorecard",
    roles: ["admin", "aom"],
    mobile: "card",
  },
  {
    id: "deviation-scoreboard",
    section: "reports",
    label: "Deviation Scoreboard",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "trend-charts",
    section: "reports",
    label: "Trend Charts",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "session-type-breakdown",
    section: "reports",
    label: "Session Types",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "faculty-stability",
    section: "reports",
    label: "Faculty Stability",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "subject-coverage",
    section: "reports",
    label: "Subject Coverage",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "region-heatmap",
    section: "reports",
    label: "Region Heatmap",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "room-conflicts",
    section: "reports",
    label: "Room Conflicts",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "adherence-comparison",
    section: "reports",
    label: "Adherence Comparison",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "week-diff",
    section: "reports",
    label: "Week Diff",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "compliance-pivot",
    section: "reports",
    label: "Compliance Pivot",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "timetable-health",
    section: "reports",
    label: "Health Dashboard",
    roles: ["admin", "aom", "head"],
    mobile: "card",
  },
  {
    id: "compliance",
    section: "reports",
    label: "Compliance",
    roles: ["admin", "aom"],
    mobile: "list",
  },
  {
    id: "substitution-analytics",
    section: "reports",
    label: "Substitution Analytics",
    roles: ["admin", "aom"],
    mobile: "list",
  },

  // ── Admin ──
  {
    id: "users",
    section: "admin",
    label: "Users",
    roles: ["admin"],
    mobile: "card",
  },
  {
    id: "sla",
    section: "admin",
    label: "SLA Settings",
    roles: ["admin"],
    mobile: "card",
  },
  {
    id: "routing",
    section: "admin",
    label: "Routing Rules",
    roles: ["admin"],
    mobile: "card",
  },
  {
    id: "escalation",
    section: "admin",
    label: "Escalation Policy",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "templates",
    section: "admin",
    label: "Templates",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "policies",
    section: "admin",
    label: "Policies",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "escalation-rules",
    section: "admin",
    label: "Escalation Rules",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "audit-log",
    section: "admin",
    label: "Audit Log",
    roles: ["admin", "aom", "head", "agent", "viewer"],
    mobile: "list",
  },
  {
    id: "sync",
    section: "admin",
    label: "Daily Sync",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "digests",
    section: "admin",
    label: "Digests",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "cross-school-rooms",
    section: "admin",
    label: "Cross-School Rooms",
    roles: ["admin"],
    mobile: "list",
  },
  {
    id: "vp-centers",
    section: "admin",
    label: "VP Centers",
    roles: ["admin", "aom"],
    mobile: "card",
  },
];

// ── Helpers ──

export function toolsForSection(
  section: AppSection,
  role: string,
): ToolDef[] {
  return TOOL_REGISTRY.filter(
    (t) => t.section === section && t.roles.includes(role),
  );
}

export function sectionForToolId(toolId: string): AppSection | null {
  const tool = TOOL_REGISTRY.find((t) => t.id === toolId);
  return tool?.section ?? null;
}

export function labelForToolId(toolId: string): string | null {
  const tool = TOOL_REGISTRY.find((t) => t.id === toolId);
  return tool?.label ?? null;
}

export function canAccessSection(section: AppSection, role: string): boolean {
  return TOOL_REGISTRY.some(
    (t) => t.section === section && t.roles.includes(role),
  );
}

export function defaultSectionForRole(role: string): AppSection {
  if (role === "faculty") return "faculty";
  if (canAccessSection("work", role)) return "work";
  if (canAccessSection("reports", role)) return "reports";
  return "admin";
}
