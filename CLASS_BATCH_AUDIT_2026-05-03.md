# Class vs Batch Audit — 2026-05-03

## Executive Summary

Class and Batch are not consistently separated today.

The intended domain should be:

- **Class**: academic offering at a school, e.g. `Class XI JEE Weekdays`.
- **Batch**: concrete teachable student group inside that class, e.g. `XI JEE Weekdays A`, `XI JEE Weekdays B`.

Current code partially supports batches, but the operational flows still mostly schedule and mark attendance by `school_id + grade_level + track + batch_pattern`. This means two batches inside the same class, such as `XI JEE Weekdays A` and `XI JEE Weekdays B`, cannot be reliably separated in timetable and attendance workflows.

## Current Model

### School Class Plans

Backend table: `school_class_plans`

Purpose today:

- Stores school-level academic offering and admission plan.
- Fields include `school_id`, `grade_level`, `track`, `lecture_model_id`, `batch_pattern`, `aop_admissions`, `registrations`, `actual_admissions`.
- Unique key is `school_id + grade_level + track`.

Assessment:

- This is closer to a **Class Plan** or **Program Offering**, not a real class section.
- It does not represent multiple actual batches under the same class.
- It stores `batch_pattern`, but not number of batches or batch instances.

### Batches

Backend table: `batches`

Purpose today:

- Has `school_id`, `batch_id`, `grade_level`, `track`, `batch_pattern`, `capacity`.
- It was introduced in migration 35.

Assessment:

- This is the closest current table to the correct `Batch` entity.
- It is underused.
- There are only list APIs for batches; no proper create/update/delete API.
- `batch_id` is a text code, not a first-class foreign-key target in operational tables.

### Students

Backend table: `students`

Relevant fields:

- `grade_level`
- `program_track`
- `track`
- `batch_id`

Assessment:

- Student `batch_id` is plain text, not a foreign key to `batches.id`.
- Student import accepts `batch_id`, `batch`, `batch_alloted`, `batch_allotted`.
- Student creation/update does not validate that the batch exists.
- This allows students to reference non-existent or misspelled batches.

### Timetable

Backend tables:

- `timetable_slots`
- `timetable_weekly_slots`

Relevant fields:

- `grade_level`
- `track`
- `batch_pattern`
- `batch_id` exists in schema after migrations.

Assessment:

- Backend model `TimetableSlot` does **not expose `batch_id`**.
- Upsert input `UpsertTimetableSlotInput` does **not accept `batch_id`**.
- Insert/upsert still writes by `batch_pattern`, not by actual batch.
- Unique constraint remains `school_id + grade_level + track + batch_pattern + day_of_week + period`.
- Therefore two batches with same class and same pattern cannot have separate timetable entries for the same period.

This is the most serious mismatch.

### Attendance

Backend function: `ensure_session_students`

Current student matching:

```sql
WHERE school_id = ?1
  AND grade_level = ?2
  AND (?3 = '' OR track = ?3)
```

Assessment:

- Attendance expansion ignores `batch_id`.
- A session for `XI JEE Weekdays A` would pull all students in `XI JEE`, including `XI JEE Weekdays B`.
- Attendance reports group by `ts.batch_id`, but the student join still does not restrict by student batch.

This will produce incorrect attendance whenever multiple batches exist for the same school/class/track.

## Functional Bugs / Gaps

### 1. Multiple Batches Per Class Are Not Operationally Supported

Example:

- Class: `XI JEE Weekdays`
- Batches: `XI JEE Weekdays A`, `XI JEE Weekdays B`

Current risk:

- Timetable cannot reliably create separate same-period slots for A and B.
- Attendance will include students from both batches.
- Reports may appear batch-specific but calculate from a wider student pool.

Severity: Critical

### 2. `batch_pattern` Is Being Used As If It Were Batch

`batch_pattern` means delivery pattern: `Weekday`, `Weekend`, etc.

It should not identify a student group. But today it is used in timetable uniqueness and legacy generated `batch_id` values like:

```text
Grade 11|JEE|Weekday
```

This collapses all weekday batches into one logical batch.

Severity: Critical

### 3. `batches` Table Has No Full Management Surface

Routes expose only:

- `GET /batches`

Missing:

- `POST /batches`
- `PUT /batches/:id`
- `DELETE /batches/:id` or archive
- validations against class plan

Frontend can list batches, but users cannot properly create or maintain batch entities except indirectly through imports or timetable-derived seed data.

Severity: High

### 4. Student Batch Link Is Not Referentially Safe

Student `batch_id` is text.

Problems:

- Typo creates silent orphan batch reference.
- Renaming a batch breaks student association.
- Reports and timetable cannot reliably join students to batches.

Severity: High

### 5. Timetable Types Hide `batch_id`

Backend and frontend types expose `batch_pattern`, but not actual `batch_id` for timetable slots.

Result:

- UI cannot target actual batch.
- APIs cannot persist actual batch selection.
- Existing schema column is effectively dormant.

Severity: High

### 6. Class Plan Naming Is Ambiguous

The UI currently has “Add Class Plan.”

This is not wrong, but the user-facing vocabulary should be clearer:

- `Class Offering` or `Class Plan`: school offers Class XI JEE Weekdays.
- `Batch`: actual teachable group A/B/C inside that offering.

Severity: Medium

### 7. Grade Format Is Inconsistent

Examples found:

- Class plan import uses `Grade 11`.
- UI class plan form uses values like `11`, `12`, `Dropper`.
- Other parts use `Grade 11`.

This can break joins and filters when one table stores `11` and another stores `Grade 11`.

Severity: High

## Recommended Target Model

### Class Offering

Represent academic offering:

- `id`
- `school_id`
- `grade_level`
- `track`
- `delivery_pattern`
- `lecture_model_id`
- admissions/planning fields

Current `school_class_plans` can continue serving this role, but UI should explain it as class offering/planning.

### Batch

Represent concrete teachable group:

- `id`
- `class_plan_id` or `school_id + grade_level + track + delivery_pattern`
- `batch_code`, e.g. `A`, `B`
- `display_name`, e.g. `XI JEE Weekdays A`
- `capacity`
- `status`

Preferred schema: add `class_plan_id` to `batches`.

### Students

Students should reference:

- `batch_fk_id INTEGER REFERENCES batches(id)`

Keep legacy `batch_id TEXT` temporarily during migration, but operational code should use integer batch FK.

### Timetable

Timetable slots should reference:

- `batch_fk_id INTEGER REFERENCES batches(id)`

Unique key should become:

```text
school_id + batch_fk_id + day_of_week + period
```

or simply:

```text
batch_fk_id + day_of_week + period
```

depending on whether batch IDs are globally unique.

### Attendance

Session student expansion must filter by batch:

```sql
WHERE students.batch_fk_id = timetable_slots.batch_fk_id
```

For legacy sessions without batch FK, fallback can use old grade/track behavior only temporarily.

## Proposed Implementation Plan

### Phase 1 — Vocabulary and Guardrails

- Rename UI labels where needed:
  - `Batch Pattern` → `Delivery Pattern`.
  - `Class Plan` → `Class Offering / Plan` if acceptable.
  - Keep `Batch` only for actual batch groups.
- Normalize grade values across frontend/backend to one canonical format, preferably `Grade 11`, `Grade 12`, `Dropper`.
- Add audit tests proving `Grade 11` and `11` do not diverge silently.

### Phase 2 — Batch CRUD

- Add backend batch mutation APIs:
  - create batch
  - update batch
  - archive/delete batch
- Add validation:
  - batch must belong to a valid class offering.
  - no duplicate batch code/display name in same class offering.
- Add frontend Batch Management panel under Master Data or Timetable.

### Phase 3 — Student Batch FK Migration

- Add nullable `students.batch_fk_id`.
- Backfill from existing text `students.batch_id` to `batches.id`.
- Update student create/update/import to resolve or create batch explicitly.
- Keep text `batch_id` as display/legacy until all code migrates.

### Phase 4 — Timetable Batch FK Migration

- Add `batch_fk_id` to timetable template and weekly timetable models/API types.
- Update upsert/list APIs to accept and return actual batch.
- Change timetable uniqueness from `batch_pattern` to actual batch.
- UI must select a batch, not just delivery pattern.

### Phase 5 — Attendance Correction

- Update `ensure_session_students` to use batch FK.
- Update attendance summary reports to join students by batch FK.
- Add tests for:
  - two batches in same class
  - same period timetable for both batches
  - attendance for Batch A excludes Batch B students

### Phase 6 — Import Cleanup

- Update student CSV template:
  - replace ambiguous `batch_id` with `batch_code` / `batch_name`
  - optionally include `class_offering` fields
- Update SIP master import:
  - class plan remains class offering
  - batch count or batch names should create actual batches

## Acceptance Criteria

- Class XI JEE Weekdays can have two batches A and B.
- Each batch can have its own timetable for the same day/period.
- Students belong to exactly one active batch for the relevant academic period.
- Attendance for Batch A does not include Batch B students.
- Reports roll up Batch → Class Offering → School.
- UI no longer uses `Batch Pattern` where it means `Delivery Pattern`.
- No operational screen treats `Weekday` as a batch identity.

## Final Recommendation

Do not build more timetable or attendance features until this model is corrected.

The current implementation can work for one batch per class-pattern, but it will produce incorrect behavior as soon as a school has parallel batches under the same class offering. This is a foundational data-model issue and should be the next backend/frontend stabilization pass.
