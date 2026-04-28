# Phase 5 Analysis: System Polish & Data Utility

Phase 5 focuses on operational edge cases and tools for bulk data management, ensuring the system is ready for long-term use across multiple academic years.

## 1. The Holiday Calendar

Static timetables fail when holidays occur.
- **Table:** `holidays` (date, name, school_id/global).
- **Logic:** The `lecture_sessions` generation logic (from Phase 2) must be holiday-aware. If a date is a holiday, no `lecture_sessions` should be created.
- **Regional Holidays:** Support for regional holidays (e.g., state-specific festivals) via a `region_id` link in the `holidays` table.

## 2. Makeup Classes (Dynamic Sessions)

If a class was cancelled (Phase 3), it may need to be rescheduled for a non-standard time (e.g., Saturday afternoon).
- **Implementation:** Allow Admins to create "Ad-hoc" `lecture_sessions` that are NOT linked to a `timetable_slot_id`.
- **UI:** A "Reschedule/Makeup" wizard that picks a date, time, subject, and faculty.

## 3. The Bulk Timetable Importer

Setting up a 6-day timetable for 10 grades across 50 schools is impossible via UI alone.

### CSV Engine Design:
- **Template:** A matrix-style CSV where rows are Grade/Track/Batch and columns are Mon-1, Mon-2, ... Sat-6.
- **Validation:** 
    1. Verify Faculty User IDs exist.
    2. Verify Subject IDs exist.
    3. Ensure no Faculty is assigned to two different schools at the same time.
- **Conflict Resolution:** "Overwrite" or "Skip" existing slots.

## 4. Final System Polish

### A. Academic Year Transition
- A tool to "clone" school configurations and student lists into a new academic year.
- Archiving old attendance data to keep the active tables slim.

### B. Notification Engine (Optional/Bonus)
- Push notifications for Faculty when a session is about to start.
- Alerts for AOMs when attendance is not marked for a school by 11:00 AM.

## 5. Implementation Priorities
- **Importer:** This is the most technically complex part of Phase 5 and should be tackled first.
- **Holiday Logic:** High-value for user experience to prevent "phantom" missing attendance reports.
- **Optimization:** Final DB vacuuming and index tuning before the system goes into full "Production" mode.
