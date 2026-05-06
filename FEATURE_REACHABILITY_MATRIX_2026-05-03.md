# Feature Reachability Matrix — 2026-05-03

## Method

- Tools come from `frontend/src/toolRegistry.tsx` `APP_TOOLS`
- UI entry points: `Sidebar` (desktop left rail), `MobileMoreMenu` (mobile overflow), `handleToolClick` in `App.tsx`
- Routed component comes from `AdminPanelRouter.tsx`
- API commands come from `frontend/src/api.ts` dispatch table + hook usage
- Backend authorization comes from `backend/src/routes/*.rs` guards

## Legend

| Status | Meaning |
|--------|---------|
| complete | Real component, real API, auth aligned |
| partial | Reads data but writes are stubbed or UI is incomplete |
| stub | Visible but entirely non-functional |
| misleading | Label or button claims action that does not happen |
| blocked | Backend rejects this role, but UI shows it |

---

## Matrix

| Tool ID | Label | Roles | Component | API Commands Used | Backend Auth | Status | Required Fix |
|---------|-------|-------|-----------|-------------------|--------------|--------|-------------|
| master-data | Master Data | admin, aom | `MasterDataPanel` | `list_schools`, `create_school`, `drop_school`, `restore_school`, `delete_school`, `list_regions`, `upsert_region`, `delete_region`, `list_students`, `create_student`, `update_student`, `delete_student`, `list_lecture_models`, `create_lecture_model`, `list_school_class_plans`, `upsert_school_class_plan`, `import_schools_csv`, `import_students_csv`, `import_sip_master`, `export_sip_master` | `require_admin` (create_school, delete_region, lecture_model, imports), `require_admin_or_aom` + `enforce_school_scope` (rest) | complete | — |
| program-dashboard | Program Dashboard | admin | `ProgramDashboardPanel` | `get_school_program_dashboard` | `require_admin` | complete | Restricted to admin-only. Removed agent, viewer, aom from APP_TOOLS roles. |
| reports | Reports | admin, agent, viewer, aom | `ReportsPanel` | Ticket list (local compute from `loadTickets`) | `require_auth` only (reads tickets) | complete | — |
| communications | Communication Ops | admin, agent, aom | `CommunicationOperationsPanel` | `list_communication_templates`, `update_communication_template` | `require_admin_or_aom` (read), `require_admin` (write) | complete (read-only for non-admin) | Non-admin users see read-only copy; Add/Activate/Deactivate buttons hidden unless role is admin. |
| directory | Directory | admin, agent, viewer, aom | `DirectoryPanel` | `list_schools`, `export_sip_master` | `require_admin_or_aom` + `scope_filter` | complete | — |
| dropped-schools | Dropped Schools | admin, aom | `DroppedSchoolsPanel` | `list_dropped_schools` | `scope_filter` | complete | — |
| region-log | Region Log | admin | `RegionHistoryPanel` | `list_school_region_history` | `require_admin` | complete | — |
| audit-log | Audit Log | admin | `AuditLogPanel` | `list_audit_log` | `require_admin` | complete | — |
| routing | Routing | admin | `AssignmentRulePanel` | `list_assignment_rules`, `update_assignment_rule` | `require_admin_or_aom` (read), `require_admin` (write) | complete | Inline editing implemented for queue, assignee, and condition. Save submits edited draft. |
| escalation | Escalation | admin | `EscalationPolicyPanel` | `get_escalation_policy`, `update_escalation_policy` | `require_admin_or_aom` (read), `require_admin` (write) | complete | — |
| sla | SLA Settings | admin | `SlaPolicyPanel` | `list_sla_policies`, `update_sla_policy` | `require_admin_or_aom` (read), `require_admin` (write) | complete | — |
| templates | Templates | admin | `CommunicationTemplatePanel` | `list_communication_templates`, `update_communication_template` | `require_admin_or_aom` (read), `require_admin` (write) | complete | — |
| sync | Daily Sync | ~~admin~~ | `SyncPanel` | None (all no-op notices) | N/A (no backend routes) | **stub** | **Hidden** — no backend APIs exist for sync/backup operations. |
| export-csv | Export Tickets | admin | Action (no panel) | `export_tickets_csv` | `require_admin_or_aom` + `scope_filter` | **partial** | Frontend expected JSON `{path}` but backend returns raw CSV. Fixed to use `download()` directly. |
| backup | Backup | ~~admin~~ | Action (no panel) | None | N/A | **stub** | **Hidden** — `exportBackup` shows "not available" notice. No backend API wired. |
| users | Users | admin | Action → `UserManagementPanel` | `list_users`, `create_user`, `update_user`, `delete_user`, `reset_password` | `require_admin` | complete | — |
| faculty-assignments | Faculty Assignments | admin, aom | `FacultyAssignmentsPanel` | `list_faculty_assignments`, `create_faculty_assignment`, `delete_faculty_assignment`, `list_subjects`, `list_schools`, `list_faculty_members` | `require_admin_or_aom` + `scope_filter` | complete | Faculty dropdown now sources from `faculty_members` (active + linked). Unlinked faculty cannot be assigned until `faculty_user_id` migration completes. Track hidden for non-track-eligible grades. Two-click delete confirmation. |
| timetable | Timetable | admin, aom | `TimetablePanel` | `list_timetable_slots`, `list_schools` | `require_admin_or_aom` + `scope_filter` | **partial** | Panel displays data. **"Add Slot" button is a no-op.** Backend `POST /timetable-slots` exists. Add button hidden until form is built. |
| subjects | Subjects | admin, aom | `SubjectsPanel` | `list_subjects`, `create_subject`, `update_subject`, `delete_subject` | `require_admin_or_aom` | complete | Inline create, edit, and delete implemented. Add form row with name/track/default/sort_order inputs. Edit inline with same fields. |
| faculty-members | Faculty Master | admin, aom | `FacultyMembersPanel` | `list_faculty_members`, `create_faculty_member`, `update_faculty_member`, `delete_faculty_member`, `list_faculty_school_memberships`, `create_faculty_school_membership`, `delete_faculty_school_membership`, `import_faculty_members_csv` | `require_admin_or_aom` + `scope_filter` | complete | Master data decoupled from users. `user_id` unique when non-null (migration 62). Archive behavior (`is_active = 0`) instead of hard delete. AOM scope enforced on list/update/delete. CSV import creates memberships from `school_name`/`school_id`. Account state badges: No login / Linked / Inactive linked user. |

## Role / Backend Mismatches

| Tool | Frontend Shows To | Backend Requires | Risk |
|------|-------------------|------------------|------|
| routing | admin | admin (write) | OK — read is admin_or_aom but tool is admin-only |
| escalation | admin | admin (write) | OK — same as above |
| sla | admin | admin (write) | OK — same as above |
| templates | admin | admin (write) | OK — same as above |

## Resolved Mismatches (Phase 4 Pass 2)

| Tool | Issue | Resolution |
|------|-------|------------|
| program-dashboard | Frontend showed to agent/viewer/aom; backend required admin | Restricted APP_TOOLS roles to `["admin"]` only. |
| communications | Non-admin could see Save/Add/Activate controls; backend write requires admin | `CommunicationOperationsPanel` now receives `currentUserRole`. Add/Activate/Deactivate buttons hidden for non-admin. Read-only copy shown. |

## Orphan API Commands (backend exposed, frontend not using)

From contract check (29 total). Notable gaps relevant to visible tools:

| API Command | Backend Route | Why It Matters |
|-------------|--------------|----------------|

| `upsert_timetable_slot` | POST /timetable-slots | Timetable panel has no Add form |
| `delete_timetable_slot` | DELETE /timetable-slots/:id | No delete UI |
| `create_announcement` | POST /announcements | No announcement UI in shell |
| `reset_password` | POST /users/:id/reset-password | Users panel may not expose |
| `import_timetable_csv` | POST /imports/timetable.csv | Master Data has no timetable import button |
| `get_student_timeline` | GET /students/:id | Timeline exists but not as a tool |
| `health_trends` | GET /analytics/health-trends | No analytics tools in shell |
| `deviation_scoreboard` | GET /analytics/deviation-scoreboard | No analytics tools in shell |

## ApproverDashboard Gaps

All 5 drill-down callbacks in `App.tsx` are `() => {}`:
- `onOpenLeaveSwap`
- `onOpenSubstitutions`
- `onOpenAlerts`
- `onOpenDayAtGlance`
- `onOpenTimetableHealth`

Backend APIs exist for all of them. Deferred to Phase 4/5.

## Dead Components (exported but never routed)

| Component | Backend API? | Status |
|-----------|-------------|--------|
| `SessionManagerPanel` | Partial (no generic create) | Unused |
| `WeeklyTimetablePanel` | Yes (`upsert_weekly_timetable`) | Unused |
| `SchoolMasterTimetablePanel` | Yes (`upsert_weekly_timetable`) | Unused |
| `HolidaysPanel` | Yes (`create_holiday`) | Unused |
| `VpCentersPanel` | Yes (full CRUD) | Unused — routed to `FacultyApp` instead |

## Fixes Applied In This Pass

1. **Hidden tools**: `sync`, `backup` removed from `APP_TOOLS` (no backend support).
2. **Fixed export-csv**: Changed `exportTicketCsvBundle` from `api()` (expects JSON) to `download()` (handles raw CSV). Renamed label from "Export CSV" to "Export Tickets".
3. **Hidden no-op Add buttons**: `SubjectsPanel`, `TimetablePanel`, `FacultyAssignmentsPanel` now only render their Add button when a real handler is provided. `AdminPanelRouter` no longer passes `() => {}` for these.
4. **Program Dashboard mismatch**: Documented — backend is admin-only, frontend shows to all. Fix deferred (needs product decision: open to aom or restrict frontend).

## Fixes Applied In Phase 4 Pass 2

- **program-dashboard**: Restricted to admin-only in `APP_TOOLS`.
- **communications**: `CommunicationOperationsPanel` now gated by `currentUserRole`; non-admins see read-only copy with no write controls.

## Fixes Applied In Phase 4 Pass 3

- **routing**: `AssignmentRulePanel` now has inline editable inputs for queue, assignee, and condition. Save button hidden when no rules exist. Draft state syncs with prop changes via `useEffect`.

## Fixes Applied In Phase 4 Pass 4

- **subjects**: `SubjectsPanel` now supports full CRUD. Added `createSubject`, `updateSubject`, `deleteSubject` to `useFacultyState`. Panel has inline add row form, inline edit mode per row, and delete buttons. Empty state handled. Role access unchanged (admin, aom).

## Fixes Applied In Phase 4 Pass 4A

- **subjects matrix cleanup**: Removed `create_subject`, `update_subject`, `delete_subject` from orphan API commands table. Subject CRUD is now fully routed through `SubjectsPanel`.
- **subjects delete safety**: Delete button in `SubjectsPanel` now requires a two-click confirmation (text changes to "Confirm Delete" on first click) to prevent accidental deletion.
- **hook dependency**: `deleteFacultyAssignment` in `useFacultyState` already correctly depends on `loadFacultyAssignments`; no stale dependency found at current HEAD.

## Fixes Applied In Phase 4 Pass 5

- **faculty-assignments**: `FacultyAssignmentsPanel` now supports full CRUD. Inline create form with selects for school, faculty (filtered to role === faculty), subject, grade, and track (hidden for non-track-eligible grades). Save is disabled until required fields are populated. Two-click delete confirmation. Routed through `AdminPanelRouter` with real `createFacultyAssignment` and `deleteFacultyAssignment` handlers.
- **matrix cleanup**: Removed `create_faculty_assignment` and `delete_faculty_assignment` from orphan API commands table.

## Fixes Applied In Phase 4 Pass 5A

- **faculty-assignments data-load**: `App.tsx` now calls `admin.loadUsers()` when opening the Faculty Assignments tool, ensuring the faculty selector is populated even when Users was not opened first.
- **empty-faculty UX guard**: `FacultyAssignmentsPanel` shows a read-only notice when the create form is available but no faculty users exist in the system.

## Fixes Applied In Phase 4 Pass 6

- **faculty-members master data**: Introduced `faculty_members` table (migration 61) decoupled from `users`. Faculty can be added as master data without login credentials, then linked to user accounts later via nullable `user_id`.
- **faculty_school_memberships**: New join table linking faculty members to schools with role and primary flag.
- **backfill**: Migration 61 backfills existing `users.role = 'faculty'` into `faculty_members` and creates corresponding `faculty_school_memberships` from `user_schools`.
- **backend CRUD**: Full repo + routes for `faculty_members` (list, create, update, delete) and `faculty_school_memberships` (list, create, delete). Auth: `require_admin_or_aom`.
- **backend import**: `POST /imports/faculty-members.csv` endpoint added.
- **frontend panel**: `FacultyMembersPanel` with inline create/edit/delete, expandable school memberships, and two-click delete confirmation.
- **frontend hook**: `useFacultyState` extended with `facultyMembers`, `facultyMemberMemberships`, loaders, and mutations.
- **tool registry**: New "Faculty Master" tool added to `APP_TOOLS` for admin and aom roles.
- **tests**: 2 backend repo tests + 7 frontend component tests. CI gate green.

## Fixes Applied In Phase 4 Pass 6A

- **Scope enforcement**: AOM can only list/update/archive faculty members linked to their scoped schools. AOM can only create/delete memberships for schools in their scope.
- **Data integrity**: Migration 62 adds partial unique index `idx_faculty_members_unique_user` on `user_id WHERE user_id IS NOT NULL`. Duplicate user linkage rejected at create and update time.
- **Archive behavior**: `delete_faculty_member` now sets `is_active = 0` instead of hard deleting. Preserves historical memberships and referential integrity.
- **Name validation**: Blank faculty name rejected by backend with clear error message.
- **Membership upsert fix**: `create_faculty_school_membership` readback now queries by `(faculty_id, school_id)` natural key instead of `last_insert_rowid()`, which was incorrect on UPDATE branch of upsert.
- **CSV import enhancement**: `import_faculty_members_csv` parses optional `school_name`/`school_id` and auto-creates `faculty_school_memberships`. School scope enforced.
- **Faculty Assignments integration**: Panel now sources faculty dropdown from `facultyMembers` (active + linked with `user_id`). `App.tsx` loads `facultyMembers` when opening Faculty Assignments. Unlinked faculty shown in Master but not assignable until `faculty_user_id` → `faculty_id` migration.
- **Faculty Master UI**: Added CSV import button with hidden file input. Account state badges show "No login", "Linked", or "Inactive linked user".
- **tests**: 5 new backend tests (scope filtering, blank name, duplicate user_id, archive behavior, membership upsert readback) + updated frontend tests. 62 backend tests, 61 frontend tests. CI gate green.
