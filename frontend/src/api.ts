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
  const response = await fetch(`${BASE_URL}/api${path}`, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  });

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

type Dispatch = Record<
  string,
  { method: string; path: (args: Record<string, unknown>) => string; bodyKey?: string }
>;

const dispatch: Dispatch = {
  // Auth
  login_user:          { method: "POST", path: () => "/auth/login",       bodyKey: "input" },
  get_current_user:    { method: "GET",  path: () => "/auth/me" },
  // Tickets
  refresh_escalations: { method: "GET",  path: () => "/tickets" },
  create_ticket:       { method: "POST", path: () => "/tickets",          bodyKey: "input" },
  update_ticket:       { method: "PUT",  path: (a) => `/tickets/${a.id ?? (a.input as {id:number}).id}`, bodyKey: "input" },
  delete_ticket:       { method: "DELETE", path: (a) => `/tickets/${a.id}` },
  list_comments:       { method: "GET",  path: (a) => `/tickets/${a.ticketId}/comments` },
  list_all_comments:   { method: "GET",  path: () => "/comments" },
  add_comment:         { method: "POST", path: (a) => `/tickets/${(a.input as {ticket_id:number}).ticket_id}/comments`, bodyKey: "input" },
  update_comment_status: { method: "PUT", path: (a) => `/comments/${(a.input as {id:number}).id}/status`, bodyKey: "input" },
  list_history:        { method: "GET",  path: (a) => `/tickets/${a.ticketId}/history` },
  // Schools
  list_schools:        { method: "GET",  path: () => "/schools" },
  list_dropped_schools:{ method: "GET",  path: () => "/schools/dropped" },
  create_school:       { method: "POST", path: () => "/schools",          bodyKey: "input" },
  drop_school:         { method: "POST", path: (a) => `/schools/${a.id}/drop`, bodyKey: "body" },
  restore_school:      { method: "POST", path: (a) => `/schools/${a.id}/restore` },
  delete_school:       { method: "DELETE", path: (a) => `/schools/${a.id}` },
  list_regions:        { method: "GET",  path: () => "/regions" },
  upsert_region:       { method: "POST", path: () => "/regions",          bodyKey: "input" },
  delete_region:       { method: "DELETE", path: (a) => `/regions/${a.id}` },
  list_students:       { method: "GET",  path: (a) => `/students${a.schoolId ? `?school_id=${a.schoolId}` : ""}` },
  create_student:      { method: "POST", path: () => "/students",         bodyKey: "input" },
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
  list_audit_log:      { method: "GET",  path: (a) => `/audit-log?limit=${a.limit ?? 150}` },
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
    if (a.facultyUserId) params.set("faculty_user_id", String(a.facultyUserId));
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
};

// ── Public api() function ─────────────────────────────────────────────────────

export async function api<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const entry = dispatch[command];
  if (!entry) {
    throw new Error(`Unknown API command: ${command}`);
  }

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

export async function uploadFile<T>(path: string, file: File): Promise<T> {
  const token = getToken();
  const form = new FormData();
  form.append("file", file);
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
