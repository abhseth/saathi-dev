// Drop-in replacement for Tauri's invoke().
//
// Usage:  import { api } from "./api";
//         const tickets = await api<Ticket[]>("list_tickets");
//         const ticket  = await api<Ticket>("create_ticket", { input: draft });
//
// The mapping table below translates every Tauri command name to its HTTP
// counterpart so the rest of the frontend (App.tsx, etc.) can switch from
//   invoke("command_name", args)
// to
//   api("command_name", args)
// with no other changes required.

import type {
  VpCenter,
  CreateVpCenterInput,
  UpdateVpCenterInput,
  VpCenterBuilding,
  CreateVpCenterBuildingInput,
  UpdateVpCenterBuildingInput,
  FacultyProfile,
  UpsertFacultyProfileInput,
} from "./types";

// Empty string = same-origin relative URLs (proxied by Vercel to Railway)
const BASE_URL = import.meta.env.VITE_API_URL ?? "";

// ── Auth token storage ────────────────────────────────────────────────────────

export function getToken(): string | null {
  return sessionStorage.getItem("td:token");
}

export function setToken(token: string) {
  sessionStorage.setItem("td:token", token);
}

export function clearToken() {
  sessionStorage.removeItem("td:token");
}

// ── Core fetch wrapper ────────────────────────────────────────────────────────

async function apiFetch<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const token = getToken();
  const url = `${BASE_URL}/api${path}`;
  const options = {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  };
  console.log("[API] fetch:", url, options);
  let response: Response;
  try {
    response = await fetch(url, options);
  } catch (err) {
    console.error("[API] fetch threw:", err);
    throw err;
  }
  console.log("[API] response:", response.status, response.statusText);

  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    let message = text;
    try {
      message = JSON.parse(text).error ?? text;
    } catch {}
    throw new Error(message);
  }

  // Handle empty 200 responses (delete endpoints return {})
  const text = await response.text();
  return text ? JSON.parse(text) : undefined as T;
}

// ── Command dispatch table ────────────────────────────────────────────────────
// Maps Tauri command names → { method, path(args) } so App.tsx can keep
// calling api("command_name", args) identically to invoke().

interface DispatchEntry {
  method: string;
  path: (args: Record<string, unknown>) => string;
  bodyKey?: string;
  multipart?: boolean;
}

/** All known API command names.  Typo-safe — only keys of the dispatch table are accepted. */
export type ApiCommand = keyof typeof dispatch;

const dispatch = {
  // Auth
  login_user:          { method: "POST", path: () => "/auth/login",       bodyKey: "input" },
  get_current_user:    { method: "GET",  path: () => "/auth/me" },
  // Tickets
  refresh_escalations: { method: "GET",  path: (a) => `/tickets?page=${a.page ?? 1}&page_size=${a.page_size ?? 50}` },
  refresh_sla_status: { method: "POST", path: () => "/tickets/refresh-escalations" },
  create_ticket:       { method: "POST", path: () => "/tickets",          bodyKey: "input" },
  update_ticket:       { method: "PUT",  path: (a) => `/tickets/${a.id ?? (a.input as {id:number}).id}`, bodyKey: "input" },
  delete_ticket:       { method: "DELETE", path: (a) => `/tickets/${a.id}` },
  list_comments:       { method: "GET",  path: (a) => `/tickets/${a.ticketId}/comments` },
  list_all_comments:   { method: "GET",  path: (a) => `/comments?page=${a.page ?? 1}&page_size=${a.page_size ?? 50}` },
  add_comment:         { method: "POST", path: (a) => `/tickets/${(a.input as {ticket_id:number}).ticket_id}/comments`, bodyKey: "input" },
  update_comment_status: { method: "PUT", path: (a) => `/comments/${(a.input as {id:number}).id}/status`, bodyKey: "input" },
  list_history:        { method: "GET",  path: (a) => `/tickets/${a.ticketId}/history` },
  // Schools
  list_schools:        { method: "GET",  path: () => "/schools" },
  list_dropped_schools:{ method: "GET",  path: () => "/schools/dropped" },
  create_school:       { method: "POST", path: () => "/schools",          bodyKey: "input" },
  update_school:       { method: "PUT",  path: (a) => `/schools/${(a.input as {id:number}).id}`, bodyKey: "input" },
  drop_school:         { method: "POST", path: (a) => `/schools/${a.id}/drop`, bodyKey: "body" },
  restore_school:      { method: "POST", path: (a) => `/schools/${a.id}/restore` },
  get_school_delete_impact: { method: "GET", path: (a) => `/schools/${a.id}/delete-impact` },
  delete_school:       { method: "DELETE", path: (a) => `/schools/${a.id}` },
  list_regions:        { method: "GET",  path: () => "/regions" },
  upsert_region:       { method: "POST", path: () => "/regions",          bodyKey: "input" },
  delete_region:       { method: "DELETE", path: (a) => `/regions/${a.id}` },
  list_students:       { method: "GET",  path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    if (a.page) p.set("page", String(a.page));
    if (a.pageSize) p.set("page_size", String(a.pageSize));
    if (a.search) p.set("search", String(a.search));
    const q = p.toString();
    return q ? `/students?${q}` : "/students";
  } },
  create_student:      { method: "POST", path: () => "/students",         bodyKey: "input" },
  update_student:      { method: "PUT",  path: (a) => `/students/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_student:      { method: "DELETE", path: (a) => `/students/${a.id}` },
  get_student_timeline:{ method: "GET",  path: (a) => `/students/${a.id}` },
  list_batches:        { method: "GET",  path: (a) => `/batches${a.schoolId ? `?school_id=${a.schoolId}` : ""}` },
  get_batch:           { method: "GET",  path: (a) => `/batches/${a.id}` },
  get_batch_students:  { method: "GET",  path: (a) => `/batches/${a.id}/students` },
  batch_analytics:     { method: "GET",  path: () => "/batch-analytics" },
  create_batch:        { method: "POST", path: () => "/batches", bodyKey: "input" },
  update_batch:        { method: "PUT",  path: (a) => `/batches/${(a.input as {id:number}).id}`, bodyKey: "input" },
  archive_batch:       { method: "DELETE", path: (a) => `/batches/${a.id}` },
  import_students_csv: { method: "POST", path: () => "/imports/students.csv", multipart: true },
  import_timetable_csv: { method: "POST", path: () => "/imports/timetable.csv", multipart: true },
  list_lecture_models: { method: "GET",  path: () => "/lecture-models" },
  create_lecture_model:{ method: "POST", path: () => "/lecture-models",   bodyKey: "input" },
  list_school_class_plans: { method: "GET", path: (a) => `/class-plans${a.schoolId ? `?school_id=${a.schoolId}` : ""}` },
  upsert_school_class_plan: { method: "POST", path: () => "/class-plans", bodyKey: "input" },
  get_school_program_dashboard: { method: "GET", path: () => "/program-dashboard" },
  list_school_region_history:   { method: "GET", path: () => "/school-region-history" },
  // Admin
  list_users:          { method: "GET",  path: () => "/users" },
  create_user:         { method: "POST", path: () => "/users",            bodyKey: "input" },
  update_user:         { method: "PUT",  path: (a) => `/users/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_user:         { method: "DELETE", path: (a) => `/users/${a.id}` },
  change_password:     { method: "PUT",  path: () => "/users/password",   bodyKey: "input" },
  reset_password:      { method: "POST", path: (a) => `/users/${a.id}/reset-password`, bodyKey: "input" },
  list_audit_log:      { method: "GET",  path: (a) => `/audit-log?page=${a.page ?? 1}&page_size=${a.page_size ?? 50}` },
  list_sla_policies:   { method: "GET",  path: () => "/sla-policies" },
  update_sla_policy:   { method: "PUT",  path: () => "/sla-policies",     bodyKey: "input" },
  list_assignment_rules:   { method: "GET", path: () => "/assignment-rules" },
  update_assignment_rule:  { method: "PUT", path: () => "/assignment-rules", bodyKey: "input" },
  get_escalation_policy:   { method: "GET", path: () => "/escalation-policy" },
  update_escalation_policy:{ method: "PUT", path: () => "/escalation-policy", bodyKey: "input" },
  list_communication_templates:   { method: "GET", path: () => "/communication-templates" },
  update_communication_template:  { method: "PUT", path: () => "/communication-templates", bodyKey: "input" },
  // Faculty / subjects / timetable (Phase 1)
  list_subjects:        { method: "GET",  path: () => "/subjects" },
  create_subject:       { method: "POST", path: () => "/subjects",          bodyKey: "input" },
  update_subject:       { method: "PUT",  path: (a) => `/subjects/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_subject:       { method: "DELETE", path: (a) => `/subjects/${a.id}` },
  list_effective_subjects: { method: "GET", path: (a) => `/schools/${a.schoolId}/effective-subjects?track=${encodeURIComponent(String(a.track))}` },
  set_school_optional_subject: { method: "POST", path: (a) => `/schools/${a.schoolId}/optional-subjects`, bodyKey: "input" },
  list_faculty_assignments: { method: "GET", path: (a) => {
    const params = new URLSearchParams();
    if (a.schoolId) params.set("school_id", String(a.schoolId));
    if (a.facultyId) params.set("faculty_id", String(a.facultyId));
    const q = params.toString();
    return q ? `/faculty-assignments?${q}` : "/faculty-assignments";
  } },
  create_faculty_assignment: { method: "POST", path: () => "/faculty-assignments", bodyKey: "input" },
  delete_faculty_assignment: { method: "DELETE", path: (a) => `/faculty-assignments/${a.id}` },
  list_timetable_slots: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    if (a.gradeLevel) p.set("grade_level", String(a.gradeLevel));
    if (a.track !== undefined && a.track !== null) p.set("track", String(a.track));
    if (a.batchPattern) p.set("batch_pattern", String(a.batchPattern));
    const q = p.toString();
    return q ? `/timetable-slots?${q}` : "/timetable-slots";
  } },
  upsert_timetable_slot: { method: "POST", path: () => "/timetable-slots", bodyKey: "input" },
  delete_timetable_slot: { method: "DELETE", path: (a) => `/timetable-slots/${a.id}` },
  // Faculty app (Phase 2)
  faculty_today_sessions: { method: "GET", path: () => "/faculty/today-sessions" },
  faculty_session_attendance: { method: "GET", path: (a) => `/faculty/sessions/${a.sessionId}/attendance` },
  mark_attendance: { method: "POST", path: (a) => `/faculty/sessions/${a.sessionId}/attendance`, bodyKey: "input" },
  // Faculty app (Phase 3)
  admin_today_sessions: { method: "GET", path: () => "/faculty/admin-sessions" },
  substitute_session: { method: "POST", path: (a) => `/faculty/sessions/${a.sessionId}/substitute`, bodyKey: "input" },
  cancel_session: { method: "POST", path: (a) => `/faculty/sessions/${a.sessionId}/cancel` },
  restore_session: { method: "POST", path: (a) => `/faculty/sessions/${a.sessionId}/restore` },
  create_makeup_session: { method: "POST", path: () => "/faculty/makeup-sessions", bodyKey: "input" },
  // Weekly timetable (Phase 4)
  list_weekly_timetable: { method: "GET", path: (a) => `/timetable-weekly?school_id=${a.schoolId}&week_start=${a.weekStart}` },
  upsert_weekly_timetable: { method: "POST", path: () => "/timetable-weekly", bodyKey: "input" },
  delete_weekly_timetable: { method: "DELETE", path: (a) => `/timetable-weekly/${a.id}` },
  clone_week: { method: "POST", path: () => "/timetable-weekly/clone", bodyKey: "input" },
  // Faculty schedule & substitutions
  list_faculty_schedule: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.weekStart) p.set("week_start", String(a.weekStart));
    const q = p.toString();
    return q ? `/faculty-schedule/${a.facultyUserId}?${q}` : `/faculty-schedule/${a.facultyUserId}`;
  } },
  list_substitutions: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    if (a.facultyUserId) p.set("faculty_user_id", String(a.facultyUserId));
    if (a.weekStart) p.set("week_start", String(a.weekStart));
    const q = p.toString();
    return q ? `/substitutions?${q}` : "/substitutions";
  } },
  pending_substitutions: { method: "GET", path: (a) => `/pending-substitutions?school_id=${a.schoolId || ""}&week_start=${a.weekStart}` },
  accept_substitution: { method: "POST", path: (a) => `/substitutions/${a.sessionId}/accept` },
  decline_substitution: { method: "POST", path: (a) => `/substitutions/${a.sessionId}/decline`, bodyKey: "input" },
  // Holidays
  list_holidays: { method: "GET", path: () => "/holidays" },
  create_holiday: { method: "POST", path: () => "/holidays", bodyKey: "input" },
  create_holiday_bulk: { method: "POST", path: () => "/holidays/bulk", bodyKey: "input" },
  delete_holiday: { method: "DELETE", path: (a) => `/holidays/${a.id}` },
  // Reporting
  attendance_summary: { method: "GET", path: (a) => `/reports/attendance-summary?date=${a.date ?? ""}` },
  das_report: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.startDate) p.set("start_date", String(a.startDate));
    if (a.endDate) p.set("end_date", String(a.endDate));
    if (a.groupBy) p.set("group_by", String(a.groupBy));
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    const q = p.toString();
    return q ? `/reports/das?${q}` : "/reports/das";
  } },
  chronic_absentees: { method: "GET", path: () => "/reports/chronic-absentees" },
  subject_attendance: { method: "GET", path: (a) => `/reports/subject-attendance?date=${a.date ?? ""}` },
  // Exports
  export_tickets_csv: { method: "GET", path: () => "/export/tickets.csv" },
  export_communications_csv: { method: "GET", path: () => "/export/communications.csv" },
  export_sip_master: { method: "GET", path: () => "/export/sip-master.csv" },
  // Timetable analytics / health (Phase 5)
  list_timetable_health: { method: "GET", path: () => "/timetable-health" },
  list_compliance_metrics: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    const q = p.toString();
    return q ? `/compliance-metrics?${q}` : "/compliance-metrics";
  } },
  get_deviation_score: { method: "GET", path: (a) => `/deviation-score/${a.schoolId}` },
  list_room_conflicts: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    if (a.weekStart) p.set("week_start", String(a.weekStart));
    const q = p.toString();
    return q ? `/room-conflicts?${q}` : "/room-conflicts";
  } },
  // Alerts
  get_alerts: { method: "GET", path: () => "/alerts" },
  get_faculty_alerts: { method: "GET", path: () => "/faculty/alerts" },
  // Notifications (Phase 6)
  list_notifications: { method: "GET", path: () => "/notifications" },
  unread_notification_count: { method: "GET", path: () => "/notifications/unread-count" },
  mark_notification_read: { method: "POST", path: (a) => `/notifications/${a.id}/read` },
  mark_all_notifications_read: { method: "POST", path: () => "/notifications/mark-all-read" },
  create_notification: { method: "POST", path: () => "/notifications", bodyKey: "input" },
  lecture_sessions: { method: "GET", path: (a) => `/lecture-sessions?school_id=${a.school_id}&grade_level=${a.grade_level || ""}&from=${a.from}&to=${a.to}` },
  // Phase 6: Substitution & Leave Engine
  suggest_substitutes: { method: "POST", path: () => "/suggest-substitutes", bodyKey: "input" },
  create_leave_request: { method: "POST", path: () => "/leave-requests", bodyKey: "input" },
  list_leave_requests: { method: "GET", path: () => "/leave-requests" },
  approve_leave_request: { method: "POST", path: (a) => `/leave-requests/${a.id}/approve` },
  reject_leave_request: { method: "POST", path: (a) => `/leave-requests/${a.id}/reject`, bodyKey: "input" },
  get_leave_impact: { method: "GET", path: (a) => `/leave-requests/${a.id}/impact` },
  list_leave_audit: { method: "GET", path: (a) => `/leave-requests/${a.id}/audit` },
  create_swap_request: { method: "POST", path: () => "/swap-requests", bodyKey: "input" },
  list_swap_requests: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.facultyUserId) p.set("faculty_user_id", String(a.facultyUserId));
    const q = p.toString();
    return q ? `/swap-requests?${q}` : "/swap-requests";
  }},
  accept_swap_request: { method: "POST", path: (a) => `/swap-requests/${a.id}/accept` },
  today_substitutions: { method: "GET", path: () => "/today-substitutions" },
  substitution_detail: { method: "GET", path: (a) => `/substitution-detail/${a.sessionId}` },
  substitution_balance: { method: "GET", path: (a) => `/substitution-balance/${a.facultyUserId}` },
  substitution_reports: { method: "GET", path: (a) => `/substitution-reports?month=${encodeURIComponent(String(a.month))}` },
  bulk_attendance: { method: "POST", path: () => "/bulk-attendance", bodyKey: "input" },
  mark_attendance_quick: { method: "POST", path: () => "/mark-attendance-quick", bodyKey: "input" },
  assign_substitute: { method: "POST", path: (a) => `/assign-substitute/${a.sessionId}`, bodyKey: "input" },
  // Analytics & Dashboards (Phase 6)
  compliance_scorecard: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    const q = p.toString();
    return q ? `/analytics/compliance-scorecard?${q}` : "/analytics/compliance-scorecard";
  }},
  control_tower: { method: "GET", path: () => "/analytics/control-tower" },
  faculty_utilization: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.facultyId) p.set("faculty_id", String(a.facultyId));
    if (a.weeks) p.set("weeks", String(a.weeks));
    const q = p.toString();
    return q ? `/analytics/faculty-utilization?${q}` : "/analytics/faculty-utilization";
  }},
  deviation_scoreboard: { method: "GET", path: () => "/analytics/deviation-scoreboard" },
  session_type_breakdown: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    const q = p.toString();
    return q ? `/analytics/session-type-breakdown?${q}` : "/analytics/session-type-breakdown";
  }},
  faculty_stability: { method: "GET", path: () => "/analytics/faculty-stability" },
  subject_coverage_heatmap: { method: "GET", path: () => "/analytics/subject-coverage-heatmap" },
  health_trends: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.weeks) p.set("weeks", String(a.weeks));
    const q = p.toString();
    return q ? `/analytics/health-trends?${q}` : "/analytics/health-trends";
  }},
  substitution_trends: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.weeks) p.set("weeks", String(a.weeks));
    const q = p.toString();
    return q ? `/analytics/substitution-trends?${q}` : "/analytics/substitution-trends";
  }},
  region_heatmap: { method: "GET", path: () => "/analytics/region-heatmap" },
  room_conflicts_radar: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.schoolId) p.set("school_id", String(a.schoolId));
    if (a.weekStart) p.set("week_start", String(a.weekStart));
    const q = p.toString();
    return q ? `/analytics/room-conflicts-radar?${q}` : "/analytics/room-conflicts-radar";
  }},
  adherence_comparison: { method: "GET", path: () => "/analytics/adherence-comparison" },
  week_diff: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    p.set("school_id", String(a.schoolId));
    p.set("week_a", String(a.weekA));
    p.set("week_b", String(a.weekB));
    return `/analytics/week-diff?${p.toString()}`;
  }},
  compliance_pivot: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    p.set("pivot", String(a.pivot));
    return `/analytics/compliance-pivot?${p.toString()}`;
  }},
  // Automation & Policy Engine (Phase 6)
  list_policies: { method: "GET", path: () => "/policies" },
  update_policy: { method: "PUT", path: (a) => `/policies/${a.key}`, bodyKey: "input" },
  list_escalation_rules: { method: "GET", path: () => "/escalation-rules" },
  create_escalation_rule: { method: "POST", path: () => "/escalation-rules", bodyKey: "input" },
  update_escalation_rule: { method: "PUT", path: (a) => `/escalation-rules/${a.id}`, bodyKey: "input" },
  intervention_digest: { method: "GET", path: () => "/digests/intervention" },
  sip_brief: { method: "GET", path: () => "/digests/sip" },
  alert_inbox: { method: "GET", path: (a) => {
    const p = new URLSearchParams();
    if (a.userId) p.set("user_id", String(a.userId));
    const q = p.toString();
    return q ? `/alert-inbox?${q}` : "/alert-inbox";
  }},
  dismiss_alert: { method: "POST", path: (a) => `/alert-inbox/${a.hash}/dismiss` },
  bulk_alert_action: { method: "POST", path: () => "/alert-inbox/bulk-action", bodyKey: "input" },
  ticket_from_gap: { method: "POST", path: () => "/tickets/from-gap", bodyKey: "input" },
  bulk_assign_users: { method: "POST", path: () => "/bulk/assign-users", bodyKey: "input" },
  bulk_import_subjects: { method: "POST", path: () => "/bulk/import-subjects", bodyKey: "input" },
  bulk_publish_timetables: { method: "POST", path: () => "/bulk/publish-timetables", bodyKey: "input" },
  reassign_faculty: { method: "POST", path: () => "/faculty/reassign", bodyKey: "input" },
  clone_week_with_check: { method: "POST", path: () => "/week/clone-with-check", bodyKey: "input" },
  list_announcements: { method: "GET", path: () => "/announcements" },
  create_announcement: { method: "POST", path: () => "/announcements", bodyKey: "input" },
  cross_school_room_conflicts: { method: "GET", path: (a) => `/room-conflicts/cross-school?week_start=${a.weekStart}` },
  // VP Centers & Faculty Profiles
  list_vp_centers: { method: "GET", path: () => "/vp-centers" },
  create_vp_center: { method: "POST", path: () => "/vp-centers", bodyKey: "input" },
  update_vp_center: { method: "PUT", path: (a) => `/vp-centers/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_vp_center: { method: "DELETE", path: (a) => `/vp-centers/${a.id}` },
  list_vp_center_buildings: { method: "GET", path: (a) => `/vp-centers/${a.vpCenterId}/buildings` },
  create_vp_center_building: { method: "POST", path: () => "/vp-center-buildings", bodyKey: "input" },
  update_vp_center_building: { method: "PUT", path: (a) => `/vp-center-buildings/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_vp_center_building: { method: "DELETE", path: (a) => `/vp-center-buildings/${a.id}` },
  list_faculty_profiles: { method: "GET", path: () => "/faculty-profiles" },
  get_faculty_profile: { method: "GET", path: (a) => `/faculty-profiles/${a.facultyUserId}` },
  upsert_faculty_profile: { method: "POST", path: () => "/faculty-profiles", bodyKey: "input" },
  // Faculty Members (master data, optional login)
  list_faculty_members: { method: "GET", path: () => "/faculty-members" },
  create_faculty_member: { method: "POST", path: () => "/faculty-members", bodyKey: "input" },
  update_faculty_member: { method: "PUT", path: (a) => `/faculty-members/${(a.input as {id:number}).id}`, bodyKey: "input" },
  delete_faculty_member: { method: "DELETE", path: (a) => `/faculty-members/${a.id}` },
  list_faculty_school_memberships: { method: "GET", path: (a) => `/faculty-members/${a.facultyId}/memberships` },
  create_faculty_school_membership: { method: "POST", path: () => "/faculty-school-memberships", bodyKey: "input" },
  delete_faculty_school_membership: { method: "DELETE", path: (a) => `/faculty-school-memberships/${a.id}` },
  create_faculty_login: { method: "POST", path: (a) => `/faculty-members/${a.facultyId}/create-login`, bodyKey: "input" },
  link_faculty_user: { method: "POST", path: (a) => `/faculty-members/${a.facultyId}/link-user`, bodyKey: "input" },
  import_faculty_members_csv: { method: "POST", path: () => "/imports/faculty-members.csv", bodyKey: "file", multipart: true },
} satisfies Record<string, DispatchEntry>;

// ── Public api() function ─────────────────────────────────────────────────────

export async function api<T = unknown>(
  command: ApiCommand,
  args: Record<string, unknown> = {},
): Promise<T> {
  const entry = dispatch[command] as DispatchEntry;

  const path = entry.path(args);
  const body = entry.bodyKey ? args[entry.bodyKey] : undefined;

  return apiFetch<T>(entry.method, path, body);
}

// ── File download helper ──────────────────────────────────────────────────────

export async function download(path: string, filename: string): Promise<void> {
  const token = getToken();
  const response = await fetch(`${BASE_URL}/api${path}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    throw new Error(text);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

// ── File upload helper ────────────────────────────────────────────────────────

export async function uploadFile<T>(path: string, file: File, extraFields?: Record<string, string>): Promise<T> {
  const token = getToken();
  const form = new FormData();
  form.append("file", file);
  if (extraFields) {
    for (const [key, value] of Object.entries(extraFields)) {
      form.append(key, value);
    }
  }
  const response = await fetch(`${BASE_URL}/api${path}`, {
    method: "POST",
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: form,
  });
  if (!response.ok) {
    const text = await response.text().catch(() => response.statusText);
    let message = text;
    try {
      message = JSON.parse(text).error ?? text;
    } catch {}
    throw new Error(message);
  }
  const text = await response.text();
  return text ? JSON.parse(text) : (undefined as T);
}

// ── Auth helpers for App.tsx ──────────────────────────────────────────────────

export async function login(username: string, password: string) {
  const result = await api<{ token: string; user: unknown }>("login_user", {
    input: { username, password },
  });
  setToken(result.token);
  return result.user;
}

export function logout() {
  clearToken();
}

// ── VP Centers & Faculty Profiles ─────────────────────────────────────────────

export async function listVpCenters() {
  return api<VpCenter[]>("list_vp_centers");
}

export async function createVpCenter(input: CreateVpCenterInput) {
  return api<VpCenter>("create_vp_center", { input });
}

export async function updateVpCenter(input: UpdateVpCenterInput) {
  return api<VpCenter>("update_vp_center", { input });
}

export async function deleteVpCenter(id: number) {
  return api<void>("delete_vp_center", { id });
}

export async function listVpCenterBuildings(vpCenterId: number) {
  return api<VpCenterBuilding[]>("list_vp_center_buildings", { vpCenterId });
}

export async function createVpCenterBuilding(input: CreateVpCenterBuildingInput) {
  return api<VpCenterBuilding>("create_vp_center_building", { input });
}

export async function updateVpCenterBuilding(input: UpdateVpCenterBuildingInput) {
  return api<VpCenterBuilding>("update_vp_center_building", { input });
}

export async function deleteVpCenterBuilding(id: number) {
  return api<void>("delete_vp_center_building", { id });
}

export async function listFacultyProfiles() {
  return api<FacultyProfile[]>("list_faculty_profiles");
}

export async function getFacultyProfile(facultyUserId: number) {
  return api<FacultyProfile>("get_faculty_profile", { facultyUserId });
}

export async function upsertFacultyProfile(input: UpsertFacultyProfileInput) {
  return api<FacultyProfile>("upsert_faculty_profile", { input });
}
