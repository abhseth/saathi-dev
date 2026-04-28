# Phase 4 Analysis: Reporting & Hierarchical Visibility

This phase is where the raw data collected in Phases 2 and 3 is converted into actionable intelligence for leadership.

## 1. The Reporting Engine

Since we are using SQLite, we must be careful with complex aggregations across thousands of rows.

### Recommended Pattern: Materialized Summary (Optional)
If performance lags, we should implement a `daily_attendance_summaries` table that caches:
- `(date, school_id, grade, track, attendance_percent)`
However, for Phase 4, we should start with optimized **SQL Views**.

### Key Metrics:
- **Daily Attendance %:** (Present + Late) / Total Students.
- **Subject-Wise Attendance:** Helps identify if students are skipping specific subjects (e.g., "Students are skipping Physics on Fridays").
- **Chronic Absentees:** List of students with < 75% attendance in the last 30 days.

## 2. Hierarchical Visibility (Role-Based Scoping)

We must implement the visibility matrix defined in the project goals:

| Role | Visibility Scope |
| :--- | :--- |
| **Central SPOC (Admin)** | All schools nationwide. |
| **Regional Head** | All schools within their assigned `region_id`. |
| **AOM** | Specific schools assigned in `user_schools`. |
| **Principal** | Data for their specific `school_id` only. |

### Technical Strategy:
- **Query Builder:** The `repositories` layer should use a standard `WithScope` helper that appends `WHERE region_id = ?` or `WHERE school_id IN (...)` based on the user's role and `Claims`.

## 3. Data Visualization Strategy

The SAATHI Dashboard will need new charts:
- **Trend Line:** Daily attendance over the last 30 days.
- **Heatmap:** Attendance by Period (e.g., Period 1 is 90%, Period 6 is 60%).
- **School Ranking:** Best and worst-performing schools by attendance in a region.

## 4. Implementation Priorities
- **Backend:** Create a set of dedicated Reporting endpoints (e.g., `/api/reports/attendance/summary`).
- **Frontend:** Integrate a charting library (e.g., `recharts` or `chart.js`) into the existing `components.tsx`.
- **Optimization:** Use SQLite Indexes on `attendance_records(session_date)` and `lecture_sessions(school_id, session_date)`.
