# Phase 3 Analysis: Status Expansion & Dynamic Exceptions

Phase 3 transforms the attendance system from a rigid template into a flexible tool that reflects the reality of daily school operations.

## 1. Status Expansion

Moving beyond Present/Absent requires a schema change in the `attendance_records` status column (or an enum in code).

### New Statuses:
- **Late:** Student arrived after the session started (track `arrival_time` if needed).
- **Excused:** Known absence with permission (medical/family).
- **Leave:** Longer-term absence.
- **Out of Class:** Student was at school but not in that specific lecture (e.g., in the lab or principal's office).

## 2. Substitution Logic (The "Floater" Problem)

Schools often have teachers who cover for absent colleagues. 

### Implementation:
- The `lecture_sessions` table (created in Phase 2) becomes critical here.
- When an AOM or Admin marks a session as "Substituted," the `actual_faculty_user_id` is updated to the covering teacher.
- This ensures the session appears on the **covering teacher's** "Today's Classes" list in the Faculty App, rather than the original teacher's list.

## 3. Session Cancellations

Sometimes a lecture is cancelled due to a school event or emergency.
- **Action:** Mark `lecture_sessions.status = 'Cancelled'`.
- **Logic:** This should automatically disable attendance marking for that session and exclude it from the "Attendance Percentage" denominator for students.

## 4. Retroactive Edits & Locking

To maintain data integrity, we need a "Locking" policy.
- **Policy:** Attendance can only be edited by Faculty for up to 24-48 hours.
- **Admin Override:** After the lock period, only Admin/AOM roles can modify attendance records (requires an audit trail).

## 5. Implementation Priorities
- **Refactor:** `attendance_records` status validation.
- **UI:** An "Admin Override" panel in the SAATHI dashboard to manage session-level changes (substitutions/cancellations).
- **Audit:** Track who changed a status and when, especially for retroactive edits.
