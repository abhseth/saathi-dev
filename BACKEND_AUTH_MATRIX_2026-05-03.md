# Backend Authorization Matrix — 2026-05-03

## Scope

This matrix covers **mutating routes** in the five high-risk route modules:
`admin.rs`, `schools.rs`, `imports.rs`, `export.rs`, `automation.rs`.

Faculty self-service (`faculty.rs`, `substitutions.rs`) and ticket mutations
(`tickets.rs`, handled in Phase 3 Pass 1) are **out of scope** for this pass.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Correct as-is |
| 🔧 | Fixed in this pass |
| ❓ | Needs product decision |
| 🚫 | Denied (returns 403) |

---

## 1. routes/admin.rs

| Handler | Method / Path | Current Guard | Intended Roles | School Scope | Action |
|---------|--------------|---------------|----------------|--------------|--------|
| `list_users` | GET /users | `require_admin` | admin only | n/a | ✅ |
| `create_user` | POST /users | `require_admin` | admin only | n/a | ✅ |
| `update_user` | PUT /users/:id | `require_admin` | admin only | n/a | ✅ |
| `delete_user` | DELETE /users/:id | `require_admin` | admin only | n/a | ✅ |
| `change_password` | PUT /users/password | **none** (self-service) | any authenticated | n/a | ✅ |
| `reset_password` | POST /users/:id/reset-password | `require_admin` | admin only | n/a | ✅ |
| `list_audit_log` | GET /audit-log | `require_admin` | admin only | n/a | ✅ |
| `list_sla_policies` | GET /sla-policies | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `update_sla_policy` | PUT /sla-policies | `require_admin` | admin only | n/a | ✅ |
| `list_assignment_rules` | GET /assignment-rules | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `update_assignment_rule` | PUT /assignment-rules | `require_admin` | admin only | n/a | ✅ |
| `get_escalation_policy` | GET /escalation-policy | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `update_escalation_policy` | PUT /escalation-policy | `require_admin` | admin only | n/a | ✅ |
| `list_communication_templates` | GET /communication-templates | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `update_communication_template` | PUT /communication-templates | `require_admin` | admin only | n/a | ✅ |
| `db_snapshot` | GET /admin/db-snapshot | `require_admin` | admin only | n/a | ✅ |
| `db_restore` | POST /admin/db-restore | `require_admin` | admin only | n/a | ✅ |
| `attendance_summary` | GET /reports/attendance-summary | `require_admin_or_aom` | admin, aom | `scope_filter` | ✅ |
| `chronic_absentees` | GET /reports/chronic-absentees | `require_admin_or_aom` | admin, aom | `scope_filter` | ✅ |
| `subject_attendance` | GET /reports/subject-attendance | `require_admin_or_aom` | admin, aom | `scope_filter` | ✅ |

**Assessment:** All admin-only config/user/backup routes are correctly `require_admin`.
AOM has read-only access to SLA, rules, templates, and reporting where intended.
No changes required.

---

## 2. routes/schools.rs

| Handler | Method / Path | Current Guard | Intended Roles | School Scope | Action |
|---------|--------------|---------------|----------------|--------------|--------|
| `list_schools` | GET /schools | none (read) | any authenticated | `scope_filter` | ✅ |
| `list_dropped_schools` | GET /schools/dropped | none (read) | any authenticated | `scope_filter` | ✅ |
| `create_school` | POST /schools | `require_admin` | admin only | n/a | 🔧 |
| `drop_school` | POST /schools/:id/drop | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `restore_school` | POST /schools/:id/restore | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `delete_school` | DELETE /schools/:id | `require_admin` + `enforce_school_scope` | admin only | `enforce_school_scope` | ✅ |
| `list_regions` | GET /regions | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `upsert_region` | POST /regions | `require_admin` | admin only | n/a | ✅ |
| `delete_region` | DELETE /regions/:id | inline `role != "admin"` | admin only | n/a | 🔧 |
| `list_students` | GET /students | none (read) | any authenticated | `scope_filter` | ✅ |
| `get_student_timeline` | GET /students/:id | none (read) | any authenticated | `enforce_school_scope` after lookup | ✅ |
| `create_student` | POST /students | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `update_student` | PUT /students/:id | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `delete_student` | DELETE /students/:id | `require_admin_or_aom` + `enforce_school_scope` (after lookup) | admin, aom | `enforce_school_scope` | ✅ |
| `list_batches` | GET /batches | none (read) | any authenticated | `scope_filter` | ✅ |
| `list_lecture_models` | GET /lecture-models | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `create_lecture_model` | POST /lecture-models | `require_admin` | admin only | n/a | ✅ |
| `list_class_plans` | GET /class-plans | none (read) | any authenticated | `scope_filter` | ✅ |
| `upsert_class_plan` | POST /class-plans | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `program_dashboard` | GET /program-dashboard | `require_admin` | admin only | n/a | ✅ |
| `school_region_history` | GET /school-region-history | `require_admin` | admin only | n/a | ✅ |

**Notes:**
- `create_school` is now admin-only. AOMs may continue managing assigned schools through
  scoped update/drop/restore/student/class-plan operations.
- `delete_region` used an inline role check instead of the canonical `require_admin`
  helper. Fixed for consistency (no security change).

### Frontend Alignment

The Master Data panel (`MasterDataPanel`) gates visible controls by `currentUserRole`:
- **Admin-only buttons** (hidden for AOM): Add School, Add Region, Add Lecture Model,
  Import Schools, Import SIP Master.
- **AOM-visible buttons**: Add Class Plan (scoped via `enforce_school_scope`), plus
  all school-scoped student CRUD and import actions.
- This ensures the UI surface matches the backend auth matrix: AOM cannot reach
  admin-only endpoints, and every visible AOM control maps to a
  `require_admin_or_aom` + scope-checked route.

---

## 3. routes/imports.rs

| Handler | Method / Path | Current Guard | Intended Roles | School Scope | Action |
|---------|--------------|---------------|----------------|--------------|--------|
| `import_schools_csv` | POST /imports/schools.csv | `require_admin` | admin only | n/a (global) | ✅ |
| `import_students_csv` | POST /imports/students.csv | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` on uploaded school_id | ✅ |
| `preview_sip_master_import` | POST /imports/sip-master/preview | `require_admin` | admin only | n/a (global) | ✅ |
| `import_sip_master` | POST /imports/sip-master | `require_admin` | admin only | n/a (global) | ✅ |
| `import_timetable_csv` | POST /imports/timetable.csv | `require_admin_or_aom` + per-row `enforce_school_scope` | admin, aom | per-row `enforce_school_scope` | ✅ |

**Assessment:** Global imports are admin-only. School-scoped imports require AOM scope.
No changes required.

---

## 4. routes/export.rs

| Handler | Method / Path | Current Guard | Intended Roles | School Scope | Action |
|---------|--------------|---------------|----------------|--------------|--------|
| `tickets_csv` | GET /export/tickets.csv | `require_admin_or_aom` + `scope_filter` | admin, aom | `scope_filter` | ✅ |
| `communications_csv` | GET /export/communications.csv | `require_admin_or_aom` + `scope_filter` | admin, aom | `scope_filter` | ✅ |
| `sip_master_csv` | GET /export/sip-master.csv | `require_admin_or_aom` + `scope_filter` | admin, aom | `scope_filter` | ✅ |

**Assessment:** All exports are read-only and correctly scoped. No changes required.

---

## 5. routes/automation.rs

| Handler | Method / Path | Current Guard | Intended Roles | School Scope | Action |
|---------|--------------|---------------|----------------|--------------|--------|
| `list_policies` | GET /policies | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `update_policy` | PUT /policies/:key | `require_admin` | admin only | n/a | ✅ |
| `list_escalation_rules` | GET /escalation-rules | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `create_escalation_rule` | POST /escalation-rules | `require_admin` | admin only | n/a | ✅ |
| `update_escalation_rule` | PUT /escalation-rules/:id | `require_admin` | admin only | n/a | ✅ |
| `intervention_digest` | GET /digests/intervention | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `sip_brief` | GET /digests/sip | `require_admin_or_aom` | admin, aom | n/a (read) | ✅ |
| `alert_inbox` | GET /alert-inbox | none (self-service) | any authenticated | `scope_filter` + user filter | ✅ |
| `dismiss_alert` | POST /alert-inbox/:id/dismiss | none (self-service) | any authenticated | user-scoped (claims.sub) | ✅ |
| `bulk_alert_action` | POST /alert-inbox/bulk-action | none (self-service) | any authenticated | user-scoped (claims.sub) | 🔧 |
| `ticket_from_gap` | POST /tickets/from-gap | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `bulk_assign_users` | POST /bulk/assign-users | `require_admin` | admin only | n/a | ✅ |
| `bulk_import_subjects` | POST /bulk/import-subjects | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `bulk_publish_timetables` | POST /bulk/publish-timetables | `require_admin_or_aom` + per-school `enforce_school_scope` | admin, aom | per-school `enforce_school_scope` | ✅ |
| `reassign_faculty` | POST /faculty/reassign | `require_admin_or_aom` + dual `enforce_school_scope` | admin, aom | both source & target schools | ✅ |
| `clone_week_with_check` | POST /week/clone-with-check | `require_admin_or_aom` + `enforce_school_scope` | admin, aom | `enforce_school_scope` | ✅ |
| `list_announcements` | GET /announcements | none (read) | any authenticated | `scope_filter` | ✅ |
| `create_announcement` | POST /announcements | `require_admin_or_aom` + conditional `enforce_school_scope` | admin, aom | if school_id present | ✅ |
| `cross_school_room_conflicts` | GET /room-conflicts/cross-school | `require_admin_or_aom` + `scope_filter` | admin, aom | `scope_filter` | ✅ |

**Notes:**
- `bulk_alert_action` handles three actions: `dismiss`, `snooze`, and `ticket`.
  The `dismiss` and `snooze` branches are self-service (any user can dismiss their
  own alerts). The `ticket` branch calls `repositories::create_ticket`, which
  **bypassed the ticket-writer guard**. A viewer or faculty could create tickets
  through this path. Fixed by adding `require_ticket_writer(&claims)?` inside the
  `ticket` branch only.

---

## Summary of Fixes Applied

| File | Route / Handler | Fix |
|------|----------------|-----|
| `routes/schools.rs` | `create_school` | Changed `require_admin_or_aom` → `require_admin` (AOM cannot create unscoped schools) |
| `routes/schools.rs` | `delete_region` | Replaced inline `role != "admin"` with canonical `require_admin(&claims)?` |
| `routes/automation.rs` | `bulk_alert_action` (`ticket` branch) | Added `require_ticket_writer(&claims)?` before `repositories::create_ticket` call |

---

## Test Coverage Added

New tests in `backend/src/auth.rs`:

| Test | Assertion |
|------|-----------|
| `viewer_cannot_delete_region` | `require_admin` denies `viewer` |
| `agent_cannot_delete_region` | `require_admin` denies `agent` |
| `aom_cannot_delete_region` | `require_admin` denies `aom` |
| `aom_can_access_admin_or_aom_route` | `require_admin_or_aom` allows `aom` |
| `viewer_cannot_access_admin_or_aom_route` | `require_admin_or_aom` denies `viewer` |
| `aom_cannot_access_admin_only_route` | `require_admin` denies `aom` |
| `admin_can_access_admin_only_route` | `require_admin` allows `admin` |
| `aom_scope_allows_assigned_school` | `enforce_school_scope` allows `aom` with matching school |
| `aom_scope_denies_unassigned_school` | `enforce_school_scope` denies `aom` without matching school |
| `ticket_writer_allows_admin_agent_aom` | `require_ticket_writer` allows admin, agent, aom |
| `ticket_writer_denies_viewer_faculty_head` | `require_ticket_writer` denies viewer, faculty, head |
