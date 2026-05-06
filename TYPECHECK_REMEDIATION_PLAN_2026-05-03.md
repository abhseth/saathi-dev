# TypeScript Type-Check Remediation Plan

Date: 2026-05-03
Source: `cd frontend && npm run typecheck` (67 errors across 7 files)

---

## 1. App.tsx — Prop Contract Drift (7 errors)

**Root cause:** Active `App.tsx` still wires components using old prop contracts that were refactored but never updated at call sites.

| Line | Component | Problem | Smallest Safe Fix |
|------|-----------|---------|-------------------|
| 316 | `ProgramFilters` | Receives `string[]` school names; component expects `School[]`. | Resolve whether filter state is ID-based or name-based; align prop. |
| 319 | `ProgramFilters` | `onChange` typed as `Dispatch<SetStateAction<ProgramScopeFilters>>`; component expects `(filters: Record<string, string \| string[]>) => void`. | Unify filter shape between state holder and component. |
| 420 | `StudentTimelinePanel` | Missing `student_name` and `school_name` on timeline data object. | Either add fields to backend response / transform layer, or remove them from the component contract if unused. |
| 429 | `UserManagementPanel` | Passes `currentUser`, `onCreateUser`, `onUpdateUser`, `onDeleteUser`, `onChangePassword`; component expects `onAddUser`, `onToggleUser`. | Align to canonical `UserManagementPanel` contract (likely the richer one in App.tsx is correct and the component is stale). |
| 439 | `BottomNav` | Passes old props (`activeFilter`, `showingAdmin`, `onFilterChange`, `onMasterDataClick`, `onMoreClick`); component expects new `BottomNavProps` (`currentSection`, `mobileView`, `onHomeClick`, `onWorkClick`, etc.). | **Decision needed:** adopt old or new mobile state model. If keeping old model, update `BottomNav` to accept legacy props as a compat layer. If adopting new model, update `App.tsx` state and handlers. |
| 443 | `BottomNav` inline callback | Parameter `filter` implicitly has `any` type. | Add explicit type once BottomNav contract is decided. |

**Fix order:** Decide mobile state model first (affects BottomNav), then fix BottomNav, then ProgramFilters, then StudentTimelinePanel, then UserManagementPanel.

---

## 2. components.tsx — Stale Model Field References (23 errors)

**Root cause:** UI code references fields that no longer exist on backend-derived types. Either backend removed them without updating frontend, or frontend types were tightened and UI was not adjusted.

### 2a. School director fields (5 errors)
- Lines 574, 2046–2048: `School.director_name`, `director_mobile`, `director_email`
- **Fix:** Replace with existing contact fields (e.g., `principal_name`, `phone`, `email`) or remove UI rows if data is gone.

### 2b. FacultyAssignment (1 error)
- Line 2199: `FacultyAssignment.grade_levels` — did you mean `grade_level`?
- **Fix:** Change to `grade_level` (singular) or confirm backend shape.

### 2c. LectureSession (2 errors)
- Lines 2776–2777: `LectureSession.subject_name`, `faculty_name`
- **Fix:** Resolve names from related `Subject` / `User` arrays instead of direct fields.

### 2d. Subject (1 error)
- Line 2818: `Subject.code`
- **Fix:** Remove or replace with actual field (e.g., `name`).

### 2e. TimetableSlot / WeeklyTimetableSlot (10 errors)
- Lines 2865, 2868, 2914, 2916, 2919, 2964, 3007, 3047, 3049: `period_number`, `faculty_name`, `week_start`
- **Fix:** Derive `period_number` from slot ordering or time; resolve `faculty_name` from related user array; confirm `week_start` source.

### 2f. EscalationPolicy (3 errors)
- Lines 3113, 3119, 3121: Ad-hoc `{ id, name, rules }` object passed where `EscalationPolicy` expected. Also `name` and `rules` accessed on union type where they don't exist on `EscalationPolicy`.
- **Fix:** Normalize escalation form state to use one consistent type, or narrow the union before property access.

### 2g. SlaPolicy (3 errors)
- Lines 3163, 3164, 3171: `SlaPolicy.priority`
- **Fix:** Remove from UI if field was dropped, or add back to type if backend still returns it.

### 2h. Attendance / ChronicAbsentee (2 errors)
- Line 3289: `AttendanceSummaryRow.date`
- Line 3305: `ChronicAbsentee.absent_days`
- **Fix:** Align with actual backend response shape.

### 2i. Self-referencing type (1 error)
- Line 3342: `rules` referenced in its own type annotation.
- **Fix:** Rename local variable or give explicit type annotation.

### 2j. FacultyTodaySession / FacultyWeeklySlot (2 errors)
- Line 3519: `FacultyTodaySession.room_number`
- Line 3786: `FacultyWeeklySlot.period_number`
- **Fix:** Confirm actual backend shapes and update UI accordingly.

**Fix order:** Start with self-referencing type (quick rename), then batch-update each model family. For each family, first check `types.ts` for the canonical type definition, then adjust the UI reference.

---

## 3. AdminPanelRouter.tsx — Panel Prop Mismatches (17 errors)

**Root cause:** Many admin panels were refactored with new prop contracts, but `AdminPanelRouter.tsx` still renders them with old or wrong props. Some panels appear to be wired to entirely different components.

| Line | Panel | Router passes | Component expects |
|------|-------|---------------|-------------------|
| 57 | `SlaPolicyPanel` | `(issueCategory, hours) => Promise<void>` | `(policies: SlaPolicy[]) => void` |
| 59 | `AssignmentRulesPanel` | `AssignmentRule[]`; `(queue, assignee, isActive) => Promise<void>` | `{ id, queue, assignee, condition }[]`; `(rules) => void` |
| 62 | `EscalationPolicyEditor` | `onClose` + save callback with partial shape | No `onClose`; save takes full `EscalationPolicy` |
| 74 | `ImportStudentsPanel` | `(conflictAction) => Promise<void>` | `() => void` |
| 85 | `RegionReassignmentPanel` | `(regionId, mappings[]) => Promise<void>` | `(oldId, newId) => void` |
| 91 | `SchoolProgramDashboard` | `SchoolProgramDashboard` (missing fields) | Needs `total_batches`, `total_students`, `active_tickets` |
| 95 | `AnalyticsDashboard` | `comments`, `dashboard`, `droppedSchools`, `schools`, `tickets`, `onClose`, `onExportCsv`, `onExportSipMaster` | `{ tickets_by_status, tickets_by_school } \| null` |
| 108 | `CommunicationLogPanel` | `comments`, `schools`, `tickets`, `onClose`, `onExport`, `onOpenTicket`, `onUpdateCommentStatus` | `CommunicationTemplate[]`, `onAddTemplate`, `onToggleTemplate` |
| 118 | `SchoolRegionMapPanel` | `regions`, `schools`, `onClose` | `schools`, `onExport` |
| 120 | `SchoolRecycleBinPanel` | `schools`, `onClose`, `onRestore` | `schools` only |
| 122 | `SchoolRegionHistoryPanel` | `SchoolRegionHistory[]` | Needs `changed_by` |
| 124 | `AuditLogPanel` | `AuditLogEntry[]` | Needs `details` |
| 126 | `TemplateLibraryPanel` | `templates`, `onClose`, `onSave` | `templates`, `onAddTemplate` |
| 146 | `FacultyAssignmentPanel` | `schools`, `users`, `subjects`, `assignments`, `onClose`, `onCreate`, `onDelete` | `assignments`, `schools`, `users`, `subjects`, `onAddAssignment` |
| 155 | `TimetableSlotPanel` | Rich props (`users`, `onClose`, `onLoad`, `onUpsert`, `onDelete`) | `slots`, `schools`, `onAddSlot` |
| 167 | `SubjectPolicyPanel` | `schools`, `subjects`, `onClose`, `onLoadEffective`, `onToggleOptional` | `subjects`, `onAddSubject` |

**Fix approach:**
1. For each panel, determine which contract is canonical — the richer one in the router (with `onClose`, `onCreate`, etc.) or the simpler one in the component definition.
2. If the router contract is the intended production shape, update the component to accept the richer props.
3. If the component contract is canonical and the router is stale, update the router to match.
4. For panels that are entirely mismatched (e.g., `AnalyticsDashboard`, `CommunicationLogPanel`), verify whether the router is pointing at the wrong component entirely.

**Fix order:** Tackle panels in order of production importance: SLA, Assignment Rules, Faculty Assignment, Timetable, then remaining admin surfaces.

---

## 4. Missing Shared Types (3 errors)

| File | Missing type | Context |
|------|--------------|---------|
| `hooks/useFacultyState.ts` (x2) | `CreateMakeupSessionInput` | Used in `handleCreateMakeupSession` and a type annotation for `facultyActions`. |
| `types.ts` | `SubjectGap` | Referenced at line 788. |

**Fix:**
- `CreateMakeupSessionInput`: Check if backend has a matching creation payload. If yes, mirror it in `types.ts`. If no, determine if the makeup session feature is implemented; if not, remove from `useFacultyState.ts`.
- `SubjectGap`: Check if this type exists in any backend schema or if it was renamed. Either restore it or remove the reference.

---

## 5. Master Data Draft Mismatch (1 error)

| File | Line | Problem |
|------|------|---------|
| `hooks/useMasterDataState.ts` | 20 | `SchoolProfileDraft.vp_tagging` is required but missing from the object literal being constructed. |

**Fix:** Add `vp_tagging` field to the draft object, or make it optional in `SchoolProfileDraft` if backend accepts it as optional.

---

## Recommended Fix Order (Smallest Safe Path)

1. **Step A — Missing types** (3 errors): Resolve `CreateMakeupSessionInput` and `SubjectGap` existence. Quick win.
2. **Step B — Self-reference & master data** (2 errors): Rename `rules` local; add `vp_tagging`. Quick win.
3. **Step C — App.tsx mobile model decision** (7 errors): Decide `showMobileDetail` vs `mobileView`, then fix `BottomNav`, `ProgramFilters`, `StudentTimelinePanel`, `UserManagementPanel`. This unblocks the main shell.
4. **Step D — components.tsx stale fields** (23 errors): Batch by model family. Start with `School` director fields, then `SlaPolicy`, `EscalationPolicy`, then timetable/lecture/session fields.
5. **Step E — AdminPanelRouter** (17 errors): Fix panel-by-panel, preferring updating components to match router's richer contracts since those are more likely the intended production surface.

---

## Verification Commands

```bash
cd frontend && npm run typecheck   # should pass after all fixes
cd frontend && npm run test        # must still pass
cd frontend && npm run build       # must still pass
bash scripts/ci-gate.sh            # must pass after Step 0.1 is complete
```

---

## Notes

- Do not use `any`, broad casts, or `@ts-ignore` as primary fixes per development plan constraint.
- If a temporary cast is unavoidable, document it inline with a `TODO` referencing this plan.
- The `BottomNav` mobile model decision affects both App.tsx and potentially useTicketState.ts (which references `onSetMobileView`).
