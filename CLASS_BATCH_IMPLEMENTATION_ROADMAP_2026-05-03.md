# Class / Batch Implementation Roadmap — 2026-05-03

## Objective

Correct the Class vs Batch model so SAATHI can support real operational cases like:

- Class Offering: `Class XI JEE Weekdays`
- Batch A: `XI JEE Weekdays A`
- Batch B: `XI JEE Weekdays B`

After implementation:

- Timetable must be created for actual batches, not just delivery pattern.
- Students must belong to actual batches.
- Attendance must be generated only for students in the session batch.
- Reports must roll up from Batch → Class Offering → School.

## Non-Negotiable Domain Rules

1. **Class Offering / Class Plan** is the academic offering.
   Example: `Grade 11 + JEE + Weekday + 3x3 lecture model`.

2. **Delivery Pattern** is not a batch.
   Examples: `Weekday`, `Weekend`, `Hybrid`.

3. **Batch** is a concrete teachable student group.
   Examples: `XI JEE Weekdays A`, `XI JEE Weekdays B`.

4. **Student attendance must be batch-specific.**
   A session for Batch A must not include Batch B students.

5. **No destructive migration.**
   Existing text `batch_id` values must be preserved while new FK columns are introduced and backfilled.

## Current Risk

The current implementation can work only when each class offering has one batch. It becomes incorrect once a school has parallel batches under the same grade/track/delivery pattern.

Primary broken flow:

```text
Timetable slot → school + grade + track + batch_pattern
Attendance expansion → all students in school + grade + track
```

Missing:

```text
Timetable slot → actual batch
Attendance expansion → students in that actual batch only
```

## Roadmap Overview

| Phase | Goal | Risk Level | Stop for Audit |
|---|---|---:|---|
| 0 | Freeze terminology and add safety tests | Low | Yes |
| 1 | Add canonical batch model APIs | Medium | Yes |
| 2 | Add Student → Batch FK while preserving legacy text | Medium | Yes |
| 3 | Add Timetable → Batch FK and UI batch selection | High | Yes |
| 4 | Correct attendance expansion and reports | High | Yes |
| 5 | Import/template cleanup | Medium | Yes |
| 6 | Legacy cleanup and UX polish | Medium | Yes |

Each phase should be implemented and audited before proceeding.

---

# Phase 0 — Terminology Freeze and Regression Fixtures

## Goal

Stop the vocabulary drift before structural changes.

## Changes

### Frontend Labels

Change user-facing labels:

- `Batch Pattern` → `Delivery Pattern`
- `Class Plan` → `Class Offering` or `Class Plan / Offering`
- Keep `Batch` only for concrete groups like `Batch A`, `Batch B`.

Do not rename database columns yet.

### Canonical Grade Format

Define canonical grades:

```text
Grade 6
Grade 7
Grade 8
Grade 9
Grade 10
Grade 11
Grade 12
Dropper
```

Frontend forms must submit canonical values, not raw `11` or `12`.

### Add Regression Fixture

Create a test fixture with:

- School: `Green Valley`
- Class offering: `Grade 11 + JEE + Weekday`
- Batch A: `XI JEE Weekdays A`
- Batch B: `XI JEE Weekdays B`
- Student A1 assigned to Batch A
- Student B1 assigned to Batch B

## Tests

Backend:

- Grade value normalization test.
- Existing class plan creation uses canonical grade.

Frontend:

- Class offering form submits `Grade 11`, not `11`.
- UI label says `Delivery Pattern`, not `Batch Pattern`.

## Acceptance Criteria

- No user-facing screen suggests `Weekday` is a batch.
- New class offering form submits canonical grade.
- All existing tests pass.

## Auditor Checkpoint

Confirm terminology only. No schema behavior should change in this phase.

---

# Phase 1 — Real Batch CRUD

## Goal

Make `batches` a managed first-class entity.

## Backend Schema

Keep existing `batches` table, add columns if missing:

```sql
ALTER TABLE batches ADD COLUMN display_name TEXT NOT NULL DEFAULT '';
ALTER TABLE batches ADD COLUMN status TEXT NOT NULL DEFAULT 'Active';
ALTER TABLE batches ADD COLUMN class_plan_id INTEGER REFERENCES school_class_plans(id);
ALTER TABLE batches ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'));
```

If SQLite limitations require defaults without expressions, use safe default text and update separately.

## Backend Model

Add/extend:

```rust
Batch {
  id,
  school_id,
  school_name,
  class_plan_id,
  batch_id,
  display_name,
  grade_level,
  track,
  delivery_pattern, // still maps to batch_pattern column initially
  capacity,
  status,
  created_at,
  updated_at
}

CreateBatchInput {
  school_id,
  class_plan_id?,
  batch_id,
  display_name,
  grade_level,
  track,
  batch_pattern,
  capacity,
  status
}

UpdateBatchInput { ... }
```

## Backend Routes

Add:

- `POST /batches`
- `PUT /batches/:id`
- `DELETE /batches/:id` or `POST /batches/:id/archive`

Keep:

- `GET /batches`

## Validation

- `batch_id` required.
- `display_name` required or derived from grade/track/pattern/code.
- `school_id` must exist.
- If `class_plan_id` supplied, it must belong to same school.
- For AOM, `school_id` must be in scope.
- Unique: `school_id + batch_id`.

## Frontend

Add a `Batch Management` section inside Master Data or Timetable:

Columns:

- School
- Class Offering
- Batch Code
- Display Name
- Delivery Pattern
- Capacity
- Status

Actions:

- Add Batch
- Edit Batch
- Archive/Delete Batch

## Tests

Backend:

- Admin/AOM can create scoped batch.
- AOM cannot create out-of-scope batch.
- Duplicate batch in same school rejected.
- Archive/delete works safely.

Frontend:

- Batch panel renders.
- Create payload contains batch code/display name.
- Edit updates display name/capacity/status.

## Acceptance Criteria

- Two batches can exist under the same class offering.
- Batch CRUD is reachable and role-scoped.
- No timetable behavior changed yet.

## Auditor Checkpoint

Verify batch CRUD independently before connecting students/timetable.

---

# Phase 2 — Student → Batch FK

## Goal

Students must be attached to a real batch row, while preserving old `batch_id` text.

## Schema

Add:

```sql
ALTER TABLE students ADD COLUMN batch_fk_id INTEGER REFERENCES batches(id);
```

Backfill:

```sql
UPDATE students
SET batch_fk_id = (
  SELECT b.id
  FROM batches b
  WHERE b.school_id = students.school_id
    AND b.batch_id = students.batch_id
  LIMIT 1
)
WHERE batch_fk_id IS NULL
  AND batch_id != '';
```

Do not remove `students.batch_id`.

## Backend Models

Extend:

```rust
Student {
  batch_fk_id: Option<i64>,
  batch_id: String,        // legacy/display
  batch_display_name: String
}

CreateStudentInput {
  batch_fk_id: Option<i64>,
  batch_id: String         // legacy fallback
}
```

## Behavior

On create/update:

- Prefer `batch_fk_id` if provided.
- Validate batch belongs to same school.
- Set legacy `batch_id` from selected batch code for compatibility.
- If only legacy `batch_id` is provided, resolve to batch row if possible.
- If no matching batch exists, return validation error unless explicitly importing in legacy mode.

## Frontend

Student add/edit/import UI should select actual batch from dropdown.

Display:

- Batch display name
- Optional legacy batch code

## Student Import

CSV can accept:

- `batch_id`
- `batch_code`
- `batch_name`

Resolve to `batches.id`.

If ambiguous:

- fail row with clear error.

## Tests

Backend:

- Student can be created with `batch_fk_id`.
- Student cannot reference batch from another school.
- Legacy `batch_id` resolves to `batch_fk_id`.
- Missing/unknown batch fails clearly.

Frontend:

- Student form uses batch dropdown.
- Import error displayed for unknown batch.

## Acceptance Criteria

- Student belongs to exactly one actual batch.
- Existing legacy data remains visible.
- No attendance change yet.

## Auditor Checkpoint

Create two batches and assign separate students to each. Confirm DB has distinct `batch_fk_id`.

---

# Phase 3 — Timetable → Batch FK

## Goal

Timetable slots must target an actual batch.

## Schema

Add to both timetable tables:

```sql
ALTER TABLE timetable_slots ADD COLUMN batch_fk_id INTEGER REFERENCES batches(id);
ALTER TABLE timetable_weekly_slots ADD COLUMN batch_fk_id INTEGER REFERENCES batches(id);
```

Backfill:

```sql
UPDATE timetable_slots
SET batch_fk_id = (
  SELECT b.id
  FROM batches b
  WHERE b.school_id = timetable_slots.school_id
    AND b.batch_id = timetable_slots.batch_id
  LIMIT 1
)
WHERE batch_fk_id IS NULL
  AND batch_id != '';
```

Do equivalent for weekly timetable.

## Backend Models

Extend:

```rust
TimetableSlot {
  batch_fk_id: Option<i64>,
  batch_id: String,
  batch_display_name: String
}

UpsertTimetableSlotInput {
  batch_fk_id: i64
}
```

Same for weekly timetable.

## Backend Validation

On timetable upsert:

- `batch_fk_id` required for new records.
- Batch must exist.
- Batch must belong to input school.
- Batch grade/track/delivery pattern must match or populate slot grade/track/pattern from batch.
- Faculty-subject eligibility remains school/grade/track/subject based initially.

## Unique Constraint Strategy

SQLite cannot easily alter unique constraints. Use recreate-and-swap migration for timetable tables.

New desired uniqueness:

```text
batch_fk_id + day_of_week + period
```

For weekly:

```text
batch_fk_id + week_start_date + day_of_week + period
```

During migration, preserve old rows.

## Frontend

Timetable UI must select:

1. School
2. Class Offering
3. Batch
4. Day/Period
5. Subject
6. Faculty

Remove or de-emphasize manual `grade/track/batch_pattern` entry once batch is selected.

## Tests

Backend:

- Can create two timetable slots same school/grade/track/day/period for different batches.
- Cannot create duplicate same batch/day/period.
- Cannot use out-of-school batch.

Frontend:

- Timetable form requires batch selection.
- Batch dropdown filters by selected school/class offering.

## Acceptance Criteria

- `XI JEE Weekdays A` and `XI JEE Weekdays B` can both have period 1 Monday.
- Timetable list shows actual batch names.

## Auditor Checkpoint

Manually create same-period timetable for Batch A and Batch B. Confirm both persist and display separately.

---

# Phase 4 — Attendance Correction

## Goal

Attendance must be generated and reported by actual batch.

## Backend Changes

Update `ensure_session_students`.

Current:

```sql
WHERE school_id = ?1
  AND grade_level = ?2
  AND (?3 = '' OR track = ?3)
```

Target:

```sql
WHERE batch_fk_id = ?1
```

Fallback:

- For legacy sessions where `batch_fk_id IS NULL`, retain old behavior temporarily.
- Log/flag legacy fallback in health report.

## Reports

Update:

- attendance summary
- subject attendance
- chronic absentee
- batch utilization
- any dashboard using batch totals

All should join by `students.batch_fk_id = timetable_slots.batch_fk_id`.

## Tests

Backend:

- Batch A session creates attendance only for Batch A students.
- Batch B student is excluded.
- Same class/track students in different batches do not leak across attendance.
- Reports show correct counts by batch.

Frontend:

- Attendance UI shows batch name.
- Empty batch warning appears when a batch has no students.

## Acceptance Criteria

- Attendance for Batch A excludes Batch B.
- Reports roll up correctly.

## Auditor Checkpoint

Use two-batch fixture. Generate sessions and attendance. Verify row counts exactly.

---

# Phase 5 — Import and Template Cleanup

## Goal

Make imports express class offerings and batches clearly.

## Student CSV Template

Replace ambiguous fields:

Current:

```text
batch_id
```

Preferred:

```text
batch_code,batch_name
```

Allow legacy `batch_id` temporarily as alias.

## Timetable CSV Template

Current:

```text
school_name,grade_level,track,batch_pattern,...
```

Preferred:

```text
school_name,batch_code,day_of_week,period,subject_name,faculty_username,start_time,end_time
```

Batch resolves grade/track/delivery pattern.

## SIP Master Import

Current SIP master captures class plan counts/admissions.

Add optional batch columns:

```text
grade_11_jee_weekday_batch_count
grade_11_jee_weekday_batch_names
```

Simpler acceptable first pass:

```text
grade_11_jee_batch_names
```

Example value:

```text
A;B
```

## Tests

- Student import resolves batch code.
- Timetable import resolves batch code.
- SIP master import can create batches under class offerings.

## Acceptance Criteria

- New templates make Class Offering and Batch distinct.
- Old templates still work where practical, with warnings.

## Auditor Checkpoint

Import one school with XI JEE Weekdays A/B, students split across batches, and timetable for each batch.

---

# Phase 6 — Legacy Cleanup and UX Polish

## Goal

Remove confusion after core flows are stable.

## Cleanup

- Stop displaying legacy `students.batch_id` as primary value.
- Stop generating synthetic batch IDs like `Grade 11|JEE|Weekday` for new records.
- Replace `batch_pattern` user-facing labels with `delivery_pattern`.
- Add health warnings for:
  - students without `batch_fk_id`
  - timetable slots without `batch_fk_id`
  - batches without class offering

## Optional Later Migration

Only after production data is verified:

- Make `students.batch_fk_id` NOT NULL for active students.
- Make `timetable_slots.batch_fk_id` NOT NULL.
- Consider removing/deprecating legacy text `batch_id`.

Do not do this until there is confidence in imports and backfills.

## Acceptance Criteria

- No operational UI confuses delivery pattern with batch.
- Health dashboard reports zero legacy batch warnings for clean data.

## Auditor Checkpoint

Run full manual scenario plus CI:

1. Create class offering.
2. Create two batches.
3. Import/create students into each batch.
4. Create timetable for both batches same period.
5. Generate attendance.
6. Confirm report rollups.

---

# Required Tests Across the Roadmap

## Backend Critical Tests

1. `batch_crud_supports_multiple_batches_per_class`
2. `student_batch_fk_rejects_cross_school_batch`
3. `student_legacy_batch_id_resolves_to_batch_fk`
4. `timetable_allows_same_period_for_different_batches`
5. `timetable_rejects_duplicate_same_batch_period`
6. `attendance_expansion_uses_batch_fk`
7. `attendance_reports_are_batch_specific`
8. `student_import_resolves_batch_code`
9. `timetable_import_resolves_batch_code`

## Frontend Critical Tests

1. Class offering form submits canonical grade.
2. Batch management creates and edits batch.
3. Student form requires/selects actual batch.
4. Timetable form requires/selects actual batch.
5. Attendance UI displays actual batch.

---

# Manual QA Scenario

Use this scenario after Phases 3 and 4.

## Setup

School:

```text
Green Valley
```

Class Offering:

```text
Grade 11 / JEE / Weekday
```

Batches:

```text
XI JEE Weekdays A
XI JEE Weekdays B
```

Students:

```text
Arjun A → Batch A
Bhavya B → Batch B
```

Faculty:

```text
Physics Faculty assigned to Grade 11 JEE Physics
```

Timetable:

```text
Monday Period 1 Physics → Batch A
Monday Period 1 Physics → Batch B
```

Expected:

- Both timetable slots save.
- Batch A attendance contains Arjun only.
- Batch B attendance contains Bhavya only.
- Attendance reports show separate batch totals.

---

# Recommended Immediate Next Pass

Start with **Phase 0** only.

Reason:

- It is low-risk.
- It fixes confusing UI terminology immediately.
- It creates the fixture/test language needed for deeper migrations.
- It prevents future agents from making the same `Batch Pattern = Batch` mistake.

Do not start Phase 3 or Phase 4 directly. They require schema migration and careful backward compatibility.
