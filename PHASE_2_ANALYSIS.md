# Phase 2 Analysis: Minimal Faculty App & Attendance Core

This phase transitions the system from a management tool (Phase 1) to a field-operational tool for faculty.

## 1. Project Structure & Code Sharing

Since the Faculty App is a separate frontend but shares the same backend and business logic, we must avoid "copy-paste" drift.

### Recommended Approach: Shared Library
- **Directory:** Create `frontend-shared/` containing `api.ts`, `types.ts`, and `constants.ts`.
- **Linking:** Use `npm workspaces` or simple relative imports if the build tool allows.
- **Goal:** Any change to the `Ticket` or `Student` types in Phase 1 should automatically be reflected in both the Admin and Faculty apps.

## 2. Data Modeling: Attendance Foundation

We need to bridge the gap between a **Timetable Slot** (the schedule) and a **Lecture Session** (the actual event).

### A. `lecture_sessions` Table
Tracks a specific instance of a class.
- `id` (PK)
- `timetable_slot_id` (FK)
- `session_date` (Date)
- `actual_faculty_user_id` (FK - defaults to timetable faculty, but allows overrides)
- `status` (Scheduled, Completed, Cancelled)

### B. `attendance_records` Table
The granular data.
- `id` (PK)
- `lecture_session_id` (FK)
- `student_id` (FK)
- `status` (Present, Absent - Phase 2 default)
- `marked_at` (Timestamp)

## 3. The "Today's Classes" Logic

The Faculty App's home screen relies on a complex join:
1. Identify the current `day_of_week`.
2. Query `timetable_slots` where `faculty_user_id == CURRENT_USER`.
3. Join with `schools` and `subjects` to provide context.
4. **The "Session Check":** On first load for the day, the backend should "upsert" rows into `lecture_sessions` based on the timetable template for that date.

## 4. Implementation Priorities
- **Endpoint:** `GET /api/faculty/today-sessions` (returns enriched session objects).
- **Endpoint:** `POST /api/faculty/sessions/:id/attendance` (accepts a list of student IDs and their statuses).
- **UI:** A mobile-optimized list view with large tap targets for the attendance roster.
