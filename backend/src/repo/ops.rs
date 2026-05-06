use crate::models::{
    AttendanceRecord, AttendanceSummaryRow, ChronicAbsentee, CreateHolidayInput,
    DasReportRow, FacultyTodaySession, Holiday, LectureSession, SubjectAttendanceRow,
};
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    BulkAttendanceInput, CreateLeaveRequestInput, CreateSwapRequestInput, LeaveImpactPreview,
    LeaveRequest, MarkAttendanceQuickInput, SubstitutionBalance, SubstitutionDetail,
    SubstitutionReportRow, SwapRequest, TodaySubstitutionLane, TodaySubstitutions,
};

use super::audit::*;
use crate::repo::faculty::get_subject;

pub fn set_school_optional_subject(
    conn: &Connection,
    school_id: i64,
    subject_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let subj = get_subject(conn, subject_id)?;
    if subj.is_default {
        return Err(format!(
            "{} is a default subject for {}; cannot toggle per school",
            subj.name, subj.track
        ));
    }
    if enabled {
        conn.execute(
            "INSERT OR IGNORE INTO school_optional_subjects (school_id, subject_id) VALUES (?1, ?2)",
            params![school_id, subject_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "DELETE FROM school_optional_subjects WHERE school_id = ?1 AND subject_id = ?2",
            params![school_id, subject_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Faculty attendance (Phase 2) ─────────────────────────────────────────────

pub fn upsert_lecture_session(
    conn: &Connection,
    timetable_slot_id: i64,
    session_date: &str,
    actual_faculty_user_id: Option<i64>,
) -> Result<LectureSession, String> {
    conn.execute(
        "INSERT INTO lecture_sessions (timetable_slot_id, session_date, actual_faculty_user_id)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(timetable_slot_id, session_date) DO UPDATE SET
             actual_faculty_user_id = COALESCE(excluded.actual_faculty_user_id, actual_faculty_user_id)",
        params![timetable_slot_id, session_date, actual_faculty_user_id],
    ).map_err(|e| e.to_string())?;

    get_lecture_session_by_timetable_and_date(conn, timetable_slot_id, session_date)
}

fn get_lecture_session_by_timetable_and_date(
    conn: &Connection,
    timetable_slot_id: i64,
    session_date: &str,
) -> Result<LectureSession, String> {
    conn.query_row(
        "SELECT id, timetable_slot_id, session_date, actual_faculty_user_id,
                subject_id, grade_level, track, school_id, start_time, end_time, status, created_at
         FROM lecture_sessions WHERE timetable_slot_id = ?1 AND session_date = ?2",
        params![timetable_slot_id, session_date],
        lecture_session_from_row,
    )
    .map_err(|e| e.to_string())
}

fn lecture_session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LectureSession> {
    Ok(LectureSession {
        id: row.get(0)?,
        timetable_slot_id: row.get(1)?,
        session_date: row.get(2)?,
        actual_faculty_user_id: row.get(3)?,
        subject_id: row.get(4)?,
        grade_level: row.get(5)?,
        track: row.get(6)?,
        school_id: row.get(7)?,
        start_time: row.get(8)?,
        end_time: row.get(9)?,
        status: row.get(10)?,
        created_at: row.get(11)?,
    })
}

pub fn create_makeup_session(
    conn: &Connection,
    school_id: i64,
    grade_level: &str,
    track: &str,
    subject_id: i64,
    faculty_user_id: Option<i64>,
    session_date: &str,
    start_time: &str,
    end_time: &str,
) -> Result<LectureSession, String> {
    conn.execute(
        "INSERT INTO lecture_sessions (session_date, actual_faculty_user_id, subject_id, grade_level, track, school_id, start_time, end_time, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'Scheduled')",
        params![session_date, faculty_user_id, subject_id, grade_level, track, school_id, start_time, end_time],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_lecture_session(conn, id)
}

pub fn get_lecture_session(conn: &Connection, id: i64) -> Result<LectureSession, String> {
    conn.query_row(
        "SELECT id, timetable_slot_id, session_date, actual_faculty_user_id,
                subject_id, grade_level, track, school_id, start_time, end_time, status, created_at
         FROM lecture_sessions WHERE id = ?1",
        params![id],
        lecture_session_from_row,
    )
    .map_err(|e| e.to_string())
}

pub fn list_lecture_sessions(
    conn: &Connection,
    school_id: i64,
    grade_level: Option<&str>,
    from_date: &str,
    to_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<LectureSession>, String> {
    // Empty scope → no results for scoped roles
    if let Some(ids) = scope_school_ids {
        if ids.is_empty() || ids == [-1] {
            return Ok(Vec::new());
        }
    }

    let mut sql = String::from(
        "SELECT id, timetable_slot_id, session_date, actual_faculty_user_id,
                subject_id, grade_level, track, school_id, start_time, end_time, status, created_at
         FROM lecture_sessions
         WHERE school_id = ?1
           AND session_date >= ?2
           AND session_date <= ?3",
    );
    let mut params: Vec<rusqlite::types::Value> = vec![
        school_id.into(),
        from_date.to_string().into(),
        to_date.to_string().into(),
    ];
    if let Some(gl) = grade_level {
        sql.push_str(" AND grade_level = ?4");
        params.push(gl.to_string().into());
    }
    if let Some(ids) = scope_school_ids {
        let placeholders: Vec<String> = (0..ids.len())
            .map(|i| format!("?{}", i + params.len() + 1))
            .collect();
        sql.push_str(&format!(" AND school_id IN ({})", placeholders.join(", ")));
        for id in ids {
            params.push((*id).into());
        }
    }
    sql.push_str(" ORDER BY session_date, start_time LIMIT 1000");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter()),
            lecture_session_from_row,
        )
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_faculty_today_sessions(
    conn: &Connection,
    faculty_user_id: i64,
    session_date: &str,
    day_of_week: i64,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyTodaySession>, String> {
    // Compute the Monday of the current week
    let week_start: String = conn
        .query_row(
            "SELECT date(?1, 'weekday 1', '-7 days')",
            params![session_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let school_filter_sql = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND ts.school_id IN ({placeholders})")
        }
        _ => String::new(),
    };

    // Build merged view: template slots with weekly overrides applied
    let merged_sql = format!(
        "SELECT
            ts.id AS template_id,
            ts.school_id,
            ts.grade_level,
            ts.track,
            ts.batch_pattern,
            ts.batch_ref_id,
            COALESCE(b.batch_id, ts.batch_id) AS batch_id,
            ts.day_of_week,
            ts.period,
            COALESCE(wts.subject_id, ts.subject_id) AS subject_id,
            COALESCE(wts.faculty_user_id, ts.faculty_user_id) AS faculty_user_id,
            COALESCE(wts.start_time, ts.start_time) AS start_time,
            COALESCE(wts.end_time, ts.end_time) AS end_time
         FROM timetable_slots ts
         LEFT JOIN batches b ON b.id = ts.batch_ref_id
         LEFT JOIN timetable_weekly_slots wts ON
             wts.school_id = ts.school_id
             AND wts.grade_level = ts.grade_level
             AND wts.track = ts.track
             AND wts.batch_pattern = ts.batch_pattern
             AND wts.day_of_week = ts.day_of_week
             AND wts.period = ts.period
             AND wts.week_start_date = ?
         WHERE ts.day_of_week = ?
           AND ts.deleted_at IS NULL
           {school_filter_sql}
           AND NOT EXISTS (
               SELECT 1 FROM holidays h
               WHERE h.date = ?
                 AND (
                     h.scope = 'global'
                     OR (h.scope = 'school' AND h.school_id = ts.school_id)
                     OR (h.scope = 'region' AND h.region_id = (SELECT region_id FROM schools WHERE id = ts.school_id))
                 )
                 AND (h.grade_level IS NULL OR h.grade_level = ts.grade_level)
           )"
    );

    let mut merged_stmt = conn.prepare(&merged_sql).map_err(|e| e.to_string())?;

    let mut merged_params: Vec<&dyn rusqlite::ToSql> = vec![&week_start, &day_of_week];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            merged_params.push(id);
        }
    }
    merged_params.push(&session_date);

    let merged_rows = merged_stmt
        .query_map(rusqlite::params_from_iter(merged_params), |row| {
            Ok((
                row.get::<_, i64>(0)?,         // template_id
                row.get::<_, i64>(1)?,         // school_id
                row.get::<_, i64>(9)?,          // subject_id
                row.get::<_, Option<i64>>(10)?, // faculty_user_id
                row.get::<_, String>(11)?,      // start_time
                row.get::<_, String>(12)?,      // end_time
            ))
        })
        .map_err(|e| e.to_string())?;

    // Upsert lecture_sessions for slots where the merged faculty matches
    for row in merged_rows {
        let (template_id, _school_id, _subject_id, merged_faculty, _st, _et) =
            row.map_err(|e| e.to_string())?;
        if merged_faculty == Some(faculty_user_id) {
            let _ = upsert_lecture_session(conn, template_id, session_date, None);
        }
    }

    // Build the final enriched query using the merged CTE
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND s.id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "WITH merged AS (
            SELECT
                ts.id AS template_id,
                ts.school_id,
                ts.grade_level,
                ts.track,
                ts.batch_pattern,
                ts.batch_ref_id,
                COALESCE(b.batch_id, ts.batch_id) AS batch_id,
                ts.period,
                COALESCE(wts.subject_id, ts.subject_id) AS subject_id,
                COALESCE(wts.faculty_user_id, ts.faculty_user_id) AS faculty_user_id,
                COALESCE(wts.start_time, ts.start_time) AS start_time,
                COALESCE(wts.end_time, ts.end_time) AS end_time
            FROM timetable_slots ts
            LEFT JOIN batches b ON b.id = ts.batch_ref_id
            LEFT JOIN timetable_weekly_slots wts ON
                wts.school_id = ts.school_id
                AND wts.grade_level = ts.grade_level
                AND wts.track = ts.track
                AND wts.batch_pattern = ts.batch_pattern
                AND wts.day_of_week = ts.day_of_week
                AND wts.period = ts.period
                AND wts.week_start_date = ?
            WHERE ts.day_of_week = ?
              AND NOT EXISTS (
                  SELECT 1 FROM holidays h
                  WHERE h.date = ?
                    AND (
                        h.scope = 'global'
                        OR (h.scope = 'school' AND h.school_id = ts.school_id)
                        OR (h.scope = 'region' AND h.region_id = (SELECT region_id FROM schools WHERE id = ts.school_id))
                    )
                    AND (h.grade_level IS NULL OR h.grade_level = ts.grade_level)
              )
        )
        SELECT
            ls.id AS session_id,
            m.template_id AS timetable_slot_id,
            ls.session_date,
            s.id AS school_id,
            s.name AS school_name,
            m.grade_level,
            m.track,
            m.batch_pattern,
            m.batch_id,
            m.period,
            sub.id AS subject_id,
            sub.name AS subject_name,
            m.start_time,
            m.end_time,
            ls.status,
            COALESCE((SELECT COUNT(*) FROM students st
                      WHERE (
                          (m.batch_ref_id > 0 AND st.batch_ref_id = m.batch_ref_id)
                          OR (
                              m.batch_ref_id = 0
                              AND st.school_id = s.id
                              AND st.grade_level = m.grade_level
                              AND (m.track = '' OR st.track = m.track)
                          )
                      )), 0) AS total_students,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Present'), 0) AS present_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Late'), 0) AS late_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Absent'), 0) AS absent_count,
            u.display_name AS faculty_name
         FROM lecture_sessions ls
         JOIN merged m ON m.template_id = ls.timetable_slot_id
         JOIN schools s ON s.id = m.school_id
         JOIN subjects sub ON sub.id = m.subject_id
         LEFT JOIN users u ON u.id = m.faculty_user_id
         WHERE ls.session_date = ?
           AND ls.status != 'Cancelled'
           AND (
               (m.faculty_user_id = ? AND ls.actual_faculty_user_id IS NULL)
               OR ls.actual_faculty_user_id = ?
           )
           {school_filter}
         ORDER BY m.period",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![
        &week_start,
        &day_of_week,
        &session_date,
        &session_date,
        &faculty_user_id,
        &faculty_user_id,
    ];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(FacultyTodaySession {
                session_id: row.get(0)?,
                timetable_slot_id: row.get(1)?,
                session_date: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                grade_level: row.get(5)?,
                track: row.get(6)?,
                batch_pattern: row.get(7)?,
                batch_id: row.get(8)?,
                period: row.get(9)?,
                subject_id: row.get(10)?,
                subject_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                status: row.get(14)?,
                total_students: row.get(15)?,
                present_count: row.get(16)?,
                late_count: row.get(17)?,
                absent_count: row.get(18)?,
                faculty_name: row.get(19).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_session_attendance(
    conn: &Connection,
    lecture_session_id: i64,
) -> Result<Vec<AttendanceRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT ar.id, ar.lecture_session_id, ar.student_id, st.name,
                ar.status, ar.marked_by_user_id, ar.marked_at
         FROM attendance_records ar
         JOIN students st ON st.id = ar.student_id
         WHERE ar.lecture_session_id = ?1
         ORDER BY st.name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![lecture_session_id], |row| {
            Ok(AttendanceRecord {
                id: row.get(0)?,
                lecture_session_id: row.get(1)?,
                student_id: row.get(2)?,
                student_name: row.get(3)?,
                status: row.get(4)?,
                marked_by_user_id: row.get(5)?,
                marked_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn ensure_session_students(conn: &Connection, lecture_session_id: i64) -> Result<(), String> {
    let status: String = conn
        .query_row(
            "SELECT status FROM lecture_sessions WHERE id = ?1",
            params![lecture_session_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if status == "Cancelled" {
        return Ok(()); // Don't create attendance records for cancelled sessions
    }

    // Get the session details (from timetable slot if regular, from lecture_sessions if makeup).
    let (school_id, grade_level, track, batch_ref_id): (i64, String, String, i64) = conn.query_row(
        "SELECT
            COALESCE(ts.school_id, ls.school_id),
            COALESCE(ts.grade_level, ls.grade_level),
            COALESCE(ts.track, ls.track),
            COALESCE(ts.batch_ref_id, 0)
         FROM lecture_sessions ls
         LEFT JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         WHERE ls.id = ?1",
        params![lecture_session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;

    // Prefer concrete batch membership. Fall back to class/track only for old
    // slots or ad-hoc makeup sessions that do not yet carry batch_ref_id.
    let student_ids: Vec<i64> = if batch_ref_id > 0 {
        let mut stmt = conn
            .prepare("SELECT id FROM students WHERE batch_ref_id = ?1")
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(params![batch_ref_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        ids
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM students
                 WHERE school_id = ?1 AND grade_level = ?2 AND (?3 = '' OR track = ?3)",
            )
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(params![school_id, grade_level, track], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        ids
    };

    if batch_ref_id > 0 {
        conn.execute(
            "DELETE FROM attendance_records
             WHERE lecture_session_id = ?1
               AND marked_by_user_id IS NULL
               AND status = 'Absent'
               AND student_id NOT IN (SELECT id FROM students WHERE batch_ref_id = ?2)",
            params![lecture_session_id, batch_ref_id],
        )
        .map_err(|e| e.to_string())?;
    }

    for sid in student_ids {
        conn.execute(
            "INSERT OR IGNORE INTO attendance_records (lecture_session_id, student_id, status)
             VALUES (?1, ?2, 'Absent')",
            params![lecture_session_id, sid],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

const VALID_ATTENDANCE_STATUSES: &[&str] = &[
    "Present",
    "Absent",
    "Late",
    "Excused",
    "Leave",
    "Out of Class",
];

const LOCK_HOURS: i64 = 48;

pub fn is_attendance_locked(
    conn: &Connection,
    lecture_session_id: i64,
    student_id: i64,
) -> Result<bool, String> {
    let modifier = format!("+{} hours", LOCK_HOURS);
    let locked: bool = conn
        .query_row(
            "SELECT CASE
            WHEN marked_at IS NULL THEN 0
            WHEN datetime(marked_at, ?1) < datetime('now', 'localtime') THEN 1
            ELSE 0
         END
         FROM attendance_records
         WHERE lecture_session_id = ?2 AND student_id = ?3",
            params![modifier, lecture_session_id, student_id],
            |row| row.get(0),
        )
        .unwrap_or(false);
    Ok(locked)
}

pub fn get_attendance_record_for_audit(
    conn: &Connection,
    lecture_session_id: i64,
    student_id: i64,
) -> Result<Option<(String, String)>, String> {
    // Returns (old_status, student_name) if record exists
    let result = conn.query_row(
        "SELECT ar.status, st.name
         FROM attendance_records ar
         JOIN students st ON st.id = ar.student_id
         WHERE ar.lecture_session_id = ?1 AND ar.student_id = ?2",
        params![lecture_session_id, student_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn mark_attendance(
    conn: &Connection,
    lecture_session_id: i64,
    records: &[(i64, String)],
    marked_by_user_id: i64,
    actor_name: &str,
    allow_override: bool,
) -> Result<(), String> {
    let session_status: String = conn
        .query_row(
            "SELECT status FROM lecture_sessions WHERE id = ?1",
            params![lecture_session_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if session_status == "Cancelled" {
        return Err("Cannot mark attendance for a cancelled session".to_string());
    }

    for (student_id, status) in records {
        if !VALID_ATTENDANCE_STATUSES.contains(&status.as_str()) {
            return Err(format!("Invalid attendance status: {}", status));
        }

        let locked = is_attendance_locked(conn, lecture_session_id, *student_id)?;
        if locked && !allow_override {
            return Err(format!(
                "Attendance record for student {} is locked (older than {} hours). Admin override required.",
                student_id, LOCK_HOURS
            ));
        }

        let old = get_attendance_record_for_audit(conn, lecture_session_id, *student_id)?;

        conn.execute(
            "INSERT INTO attendance_records (lecture_session_id, student_id, status, marked_by_user_id)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(lecture_session_id, student_id) DO UPDATE SET
                 status = excluded.status,
                 marked_by_user_id = excluded.marked_by_user_id,
                 marked_at = datetime('now', 'localtime')",
            params![lecture_session_id, student_id, status, marked_by_user_id],
        ).map_err(|e| e.to_string())?;

        if let Some((old_status, student_name)) = old {
            if old_status != *status {
                let summary = format!(
                    "Student {}: {} → {} in session {}",
                    student_name, old_status, status, lecture_session_id
                );
                insert_audit_log(
                    conn,
                    "attendance_record",
                    Some(*student_id),
                    "update_status",
                    actor_name,
                    &summary,
                )?;
            }
        }
    }
    conn.execute(
        "UPDATE lecture_sessions SET status = 'Completed' WHERE id = ?1",
        params![lecture_session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn substitute_session(
    conn: &Connection,
    lecture_session_id: i64,
    substitute_faculty_user_id: i64,
) -> Result<(), String> {
    let rows = conn
        .execute(
            "UPDATE lecture_sessions
         SET actual_faculty_user_id = ?1, status = 'Substituted'
         WHERE id = ?2",
            params![substitute_faculty_user_id, lecture_session_id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err("Lecture session not found".to_string());
    }
    Ok(())
}

pub fn accept_substitution(
    conn: &Connection,
    lecture_session_id: i64,
    faculty_user_id: i64,
) -> Result<(), String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE lecture_sessions
         SET actual_faculty_user_id = ?1, status = 'Substituted'
         WHERE id = ?2 AND (actual_faculty_user_id IS NULL OR actual_faculty_user_id = ?1)",
            params![faculty_user_id, lecture_session_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Session already assigned to another faculty".to_string());
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn decline_substitution(
    conn: &Connection,
    lecture_session_id: i64,
    _reason: &str,
) -> Result<(), String> {
    let rows = conn
        .execute(
            "UPDATE lecture_sessions
         SET actual_faculty_user_id = NULL, status = 'Needs Substitution'
         WHERE id = ?1",
            params![lecture_session_id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err("Lecture session not found".to_string());
    }
    Ok(())
}

pub fn cancel_session(conn: &Connection, lecture_session_id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE lecture_sessions
         SET status = 'Cancelled'
         WHERE id = ?1",
        params![lecture_session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn restore_session(conn: &Connection, lecture_session_id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE lecture_sessions
         SET actual_faculty_user_id = NULL, status = 'Scheduled'
         WHERE id = ?1",
        params![lecture_session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_lecture_session_status(
    conn: &Connection,
    lecture_session_id: i64,
) -> Result<String, String> {
    conn.query_row(
        "SELECT status FROM lecture_sessions WHERE id = ?1",
        params![lecture_session_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn get_lecture_session_school_id(conn: &Connection, id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT COALESCE(ts.school_id, ls.school_id)
         FROM lecture_sessions ls
         LEFT JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         WHERE ls.id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Lecture session {id} was not found"))
}

pub fn list_all_today_sessions(
    conn: &Connection,
    session_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyTodaySession>, String> {
    // Compute week_start (Monday) and day_of_week for SQLite (0=Sun..6=Sat)
    let (week_start, raw_dow): (String, i64) = conn
        .query_row(
            "SELECT date(?1, 'weekday 1', '-7 days'), CAST(strftime('%w', ?1) AS INTEGER)",
            params![session_date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Build merged view of all template slots for this day with weekly overrides applied
    let mut merged_stmt = conn.prepare(
        "SELECT ts.id AS template_id
         FROM timetable_slots ts
         LEFT JOIN timetable_weekly_slots wts ON
             wts.school_id = ts.school_id
             AND wts.grade_level = ts.grade_level
             AND wts.track = ts.track
             AND wts.batch_pattern = ts.batch_pattern
             AND wts.day_of_week = ts.day_of_week
             AND wts.period = ts.period
             AND wts.week_start_date = ?1
         WHERE ts.day_of_week = ?2
           AND ts.deleted_at IS NULL
           AND NOT EXISTS (
               SELECT 1 FROM holidays h
               WHERE h.date = ?3
                 AND (
                     h.scope = 'global'
                     OR (h.scope = 'school' AND h.school_id = ts.school_id)
                     OR (h.scope = 'region' AND h.region_id = (SELECT region_id FROM schools WHERE id = ts.school_id))
                 )
                 AND (h.grade_level IS NULL OR h.grade_level = ts.grade_level)
           )"
    ).map_err(|e| e.to_string())?;

    let template_ids: Vec<i64> = merged_stmt
        .query_map(params![&week_start, raw_dow, session_date], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Upsert lecture_sessions for all merged slots
    for template_id in template_ids {
        let _ = upsert_lecture_session(conn, template_id, session_date, None);
    }

    // Build the final enriched query using the merged CTE
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND s.id IN ({})", placeholders)
        }
        _ => String::new(),
    };

    let sql = format!(
        "WITH merged AS (
            SELECT
                ts.id AS template_id,
                ts.school_id,
                ts.grade_level,
                ts.track,
                ts.batch_pattern,
                ts.batch_ref_id,
                COALESCE(b.batch_id, ts.batch_id) AS batch_id,
                ts.period,
                COALESCE(wts.subject_id, ts.subject_id) AS subject_id,
                COALESCE(wts.faculty_user_id, ts.faculty_user_id) AS faculty_user_id,
                COALESCE(wts.start_time, ts.start_time) AS start_time,
                COALESCE(wts.end_time, ts.end_time) AS end_time
            FROM timetable_slots ts
            LEFT JOIN batches b ON b.id = ts.batch_ref_id
            LEFT JOIN timetable_weekly_slots wts ON
                wts.school_id = ts.school_id
                AND wts.grade_level = ts.grade_level
                AND wts.track = ts.track
                AND wts.batch_pattern = ts.batch_pattern
                AND wts.day_of_week = ts.day_of_week
                AND wts.period = ts.period
                AND wts.week_start_date = ?1
            WHERE ts.day_of_week = ?2
              AND ts.deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM holidays h
                  WHERE h.date = ?3
                    AND (
                        h.scope = 'global'
                        OR (h.scope = 'school' AND h.school_id = ts.school_id)
                        OR (h.scope = 'region' AND h.region_id = (SELECT region_id FROM schools WHERE id = ts.school_id))
                    )
                    AND (h.grade_level IS NULL OR h.grade_level = ts.grade_level)
              )
        )
        SELECT
            ls.id AS session_id,
            m.template_id AS timetable_slot_id,
            ls.session_date,
            s.id AS school_id,
            s.name AS school_name,
            m.grade_level,
            m.track,
            m.batch_pattern,
            m.batch_id,
            m.period,
            sub.id AS subject_id,
            sub.name AS subject_name,
            m.start_time,
            m.end_time,
            ls.status,
            COALESCE((SELECT COUNT(*) FROM students st
                      WHERE (
                          (m.batch_ref_id > 0 AND st.batch_ref_id = m.batch_ref_id)
                          OR (
                              m.batch_ref_id = 0
                              AND st.school_id = s.id
                              AND st.grade_level = m.grade_level
                              AND (m.track = '' OR st.track = m.track)
                          )
                      )), 0) AS total_students,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Present'), 0) AS present_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Late'), 0) AS late_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Absent'), 0) AS absent_count,
            u.display_name AS faculty_name
         FROM lecture_sessions ls
         JOIN merged m ON m.template_id = ls.timetable_slot_id
         JOIN schools s ON s.id = m.school_id
         JOIN subjects sub ON sub.id = m.subject_id
         LEFT JOIN users u ON u.id = COALESCE(ls.actual_faculty_user_id, m.faculty_user_id)
         WHERE ls.session_date = ?3
           AND ls.status != 'Cancelled'
           {}
         ORDER BY s.name, m.period",
        school_filter
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&week_start, &raw_dow, &session_date];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.clone()), |row| {
            Ok(FacultyTodaySession {
                session_id: row.get(0)?,
                timetable_slot_id: row.get(1)?,
                session_date: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                grade_level: row.get(5)?,
                track: row.get(6)?,
                batch_pattern: row.get(7)?,
                batch_id: row.get(8)?,
                period: row.get(9)?,
                subject_id: row.get(10)?,
                subject_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                status: row.get(14)?,
                total_students: row.get(15)?,
                present_count: row.get(16)?,
                late_count: row.get(17)?,
                absent_count: row.get(18)?,
                faculty_name: row.get(19).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_faculty_today_makeup_sessions(
    conn: &Connection,
    faculty_user_id: i64,
    session_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyTodaySession>, String> {
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND ls.school_id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT
            ls.id AS session_id,
            0 AS timetable_slot_id,
            ls.session_date,
            s.id AS school_id,
            s.name AS school_name,
            ls.grade_level,
            ls.track,
            '' AS batch_pattern,
            '' AS batch_id,
            0 AS period,
            sub.id AS subject_id,
            sub.name AS subject_name,
            COALESCE(ls.start_time, '') AS start_time,
            COALESCE(ls.end_time, '') AS end_time,
            ls.status,
            COALESCE((SELECT COUNT(*) FROM students st
                      WHERE st.school_id = s.id
                        AND st.grade_level = ls.grade_level
                        AND (COALESCE(ls.track, '') = '' OR st.track = ls.track)), 0) AS total_students,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Present'), 0) AS present_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Late'), 0) AS late_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Absent'), 0) AS absent_count,
            u.display_name AS faculty_name
         FROM lecture_sessions ls
         JOIN schools s ON s.id = ls.school_id
         JOIN subjects sub ON sub.id = ls.subject_id
         LEFT JOIN users u ON u.id = ls.actual_faculty_user_id
         WHERE ls.timetable_slot_id IS NULL
           AND ls.session_date = ?1
           AND ls.status != 'Cancelled'
           AND (ls.actual_faculty_user_id = ?2)
           {school_filter}
         ORDER BY ls.start_time, sub.name"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&session_date, &faculty_user_id];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(FacultyTodaySession {
                session_id: row.get(0)?,
                timetable_slot_id: row.get(1)?,
                session_date: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                grade_level: row.get(5)?,
                track: row.get(6)?,
                batch_pattern: row.get(7)?,
                batch_id: row.get(8)?,
                period: row.get(9)?,
                subject_id: row.get(10)?,
                subject_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                status: row.get(14)?,
                total_students: row.get(15)?,
                present_count: row.get(16)?,
                late_count: row.get(17)?,
                absent_count: row.get(18)?,
                faculty_name: row.get(19).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_all_today_makeup_sessions(
    conn: &Connection,
    session_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyTodaySession>, String> {
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND ls.school_id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT
            ls.id AS session_id,
            0 AS timetable_slot_id,
            ls.session_date,
            s.id AS school_id,
            s.name AS school_name,
            ls.grade_level,
            ls.track,
            '' AS batch_pattern,
            '' AS batch_id,
            0 AS period,
            sub.id AS subject_id,
            sub.name AS subject_name,
            COALESCE(ls.start_time, '') AS start_time,
            COALESCE(ls.end_time, '') AS end_time,
            ls.status,
            COALESCE((SELECT COUNT(*) FROM students st
                      WHERE st.school_id = s.id
                        AND st.grade_level = ls.grade_level
                        AND (COALESCE(ls.track, '') = '' OR st.track = ls.track)), 0) AS total_students,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Present'), 0) AS present_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Late'), 0) AS late_count,
            COALESCE((SELECT COUNT(*) FROM attendance_records ar
                      WHERE ar.lecture_session_id = ls.id AND ar.status = 'Absent'), 0) AS absent_count,
            u.display_name AS faculty_name
         FROM lecture_sessions ls
         JOIN schools s ON s.id = ls.school_id
         JOIN subjects sub ON sub.id = ls.subject_id
         LEFT JOIN users u ON u.id = ls.actual_faculty_user_id
         WHERE ls.timetable_slot_id IS NULL
           AND ls.session_date = ?1
           AND ls.status != 'Cancelled'
           {school_filter}
         ORDER BY s.name, ls.start_time, sub.name"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&session_date];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(FacultyTodaySession {
                session_id: row.get(0)?,
                timetable_slot_id: row.get(1)?,
                session_date: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                grade_level: row.get(5)?,
                track: row.get(6)?,
                batch_pattern: row.get(7)?,
                batch_id: row.get(8)?,
                period: row.get(9)?,
                subject_id: row.get(10)?,
                subject_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                status: row.get(14)?,
                total_students: row.get(15)?,
                present_count: row.get(16)?,
                late_count: row.get(17)?,
                absent_count: row.get(18)?,
                faculty_name: row.get(19).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── Holidays ──────────────────────────────────────────────────────────────────

pub fn list_holidays(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Holiday>, String> {
    const MAX_ROWS: i64 = 1000;
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!(
                " AND (h.scope = 'global' OR h.school_id IN ({}) OR (h.scope = 'region' AND h.region_id IN (SELECT region_id FROM schools WHERE id IN ({}))))",
                placeholders, placeholders
            )
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT h.id, h.date, h.name, h.scope, h.region_id, r.name, h.school_id, s.name, h.grade_level, h.created_at
         FROM holidays h
         LEFT JOIN regions r ON r.id = h.region_id
         LEFT JOIN schools s ON s.id = h.school_id
         WHERE 1=1 {}
         ORDER BY h.date DESC, h.name
         LIMIT ?",
        school_filter
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
        for id in ids {
            params_vec.push(id);
        }
    }
    params_vec.push(&MAX_ROWS);

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(Holiday {
                id: row.get(0)?,
                date: row.get(1)?,
                name: row.get(2)?,
                scope: row.get(3)?,
                region_id: row.get(4)?,
                region_name: row.get(5)?,
                school_id: row.get(6)?,
                school_name: row.get(7)?,
                grade_level: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_holiday(conn: &Connection, input: &CreateHolidayInput) -> Result<Holiday, String> {
    let scope = input.scope.trim();
    if !["global", "region", "school"].contains(&scope) {
        return Err("Invalid holiday scope".to_string());
    }
    conn.execute(
        "INSERT INTO holidays (date, name, scope, region_id, school_id, grade_level) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.date.trim(),
            input.name.trim(),
            scope,
            input.region_id,
            input.school_id,
            input.grade_level.as_deref(),
        ],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT h.id, h.date, h.name, h.scope, h.region_id, r.name, h.school_id, s.name, h.grade_level, h.created_at
         FROM holidays h
         LEFT JOIN regions r ON r.id = h.region_id
         LEFT JOIN schools s ON s.id = h.school_id
         WHERE h.id = ?1",
        params![id],
        |row| Ok(Holiday {
            id: row.get(0)?,
            date: row.get(1)?,
            name: row.get(2)?,
            scope: row.get(3)?,
            region_id: row.get(4)?,
            region_name: row.get(5)?,
            school_id: row.get(6)?,
            school_name: row.get(7)?,
            grade_level: row.get(8)?,
            created_at: row.get(9)?,
        }),
    ).map_err(|e| e.to_string())
}

pub fn bulk_create_holidays(
    conn: &mut Connection,
    input: &crate::models::BulkCreateHolidayInput,
) -> Result<Vec<Holiday>, String> {
    let scope = input.scope.trim();
    if !["global", "region", "school"].contains(&scope) {
        return Err("Invalid holiday scope".to_string());
    }

    let start = chrono::NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
        .map_err(|_| "Invalid start_date format".to_string())?;
    let end = chrono::NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d")
        .map_err(|_| "Invalid end_date format".to_string())?;
    if end < start {
        return Err("end_date must be on or after start_date".to_string());
    }

    let grades: Vec<Option<String>> = match &input.grade_levels {
        Some(g) if !g.is_empty() => g.iter().map(|s| Some(s.clone())).collect(),
        _ => vec![None],
    };

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut created = Vec::new();
    let mut current = start;
    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        for grade in &grades {
            tx.execute(
                "INSERT INTO holidays (date, name, scope, region_id, school_id, grade_level) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &date_str,
                    input.name.trim(),
                    scope,
                    input.region_id,
                    input.school_id,
                    grade.as_deref(),
                ],
            ).map_err(|e| e.to_string())?;
            let id = tx.last_insert_rowid();
            let row = tx.query_row(
                "SELECT h.id, h.date, h.name, h.scope, h.region_id, r.name, h.school_id, s.name, h.grade_level, h.created_at
                 FROM holidays h
                 LEFT JOIN regions r ON r.id = h.region_id
                 LEFT JOIN schools s ON s.id = h.school_id
                 WHERE h.id = ?1",
                params![id],
                |row| Ok(Holiday {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    name: row.get(2)?,
                    scope: row.get(3)?,
                    region_id: row.get(4)?,
                    region_name: row.get(5)?,
                    school_id: row.get(6)?,
                    school_name: row.get(7)?,
                    grade_level: row.get(8)?,
                    created_at: row.get(9)?,
                }),
            ).map_err(|e| e.to_string())?;
            created.push(row);
        }
        current = current.succ_opt().unwrap_or(current);
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(created)
}

pub fn get_holiday_school_id(conn: &Connection, id: i64) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT school_id FROM holidays WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn delete_holiday(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM holidays WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn attendance_summary(
    conn: &Connection,
    session_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<AttendanceSummaryRow>, String> {
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND s.id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT
            s.id AS school_id,
            s.name AS school_name,
            ts.grade_level,
            ts.track,
            COALESCE(b.batch_id, ts.batch_id) AS batch_id,
            COUNT(DISTINCT st.id) AS total_students,
            COALESCE(SUM(CASE WHEN ar.status = 'Present' THEN 1 ELSE 0 END), 0) AS present_count,
            COALESCE(SUM(CASE WHEN ar.status = 'Late' THEN 1 ELSE 0 END), 0) AS late_count,
            COALESCE(SUM(CASE WHEN ar.status = 'Absent' THEN 1 ELSE 0 END), 0) AS absent_count,
            COALESCE(SUM(CASE WHEN ar.status = 'Excused' THEN 1 ELSE 0 END), 0) AS excused_count
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         LEFT JOIN batches b ON b.id = ts.batch_ref_id
         JOIN schools s ON s.id = ts.school_id
         LEFT JOIN students st ON st.school_id = s.id
             AND (
                 (ts.batch_ref_id > 0 AND st.batch_ref_id = ts.batch_ref_id)
                 OR (
                     ts.batch_ref_id = 0
                     AND st.grade_level = ts.grade_level
                     AND (ts.track = '' OR st.track = ts.track)
                 )
             )
         LEFT JOIN attendance_records ar ON ar.lecture_session_id = ls.id AND ar.student_id = st.id
         WHERE ls.session_date = ?1
           {school_filter}
         GROUP BY s.id, s.name, ts.grade_level, ts.track, COALESCE(b.batch_id, ts.batch_id), ls.id
         ORDER BY s.name, ts.grade_level, ts.track, COALESCE(b.batch_id, ts.batch_id)"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&session_date];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let total: i64 = row.get(5)?;
            let present: i64 = row.get(6)?;
            let late: i64 = row.get(7)?;
            let absent: i64 = row.get(8)?;
            let excused: i64 = row.get(9)?;
            let attended = present + late;
            let percent = if total > 0 {
                (attended * 100) / total
            } else {
                0
            };
            Ok(AttendanceSummaryRow {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                grade_level: row.get(2)?,
                track: row.get(3)?,
                batch_id: row.get(4)?,
                total_students: total,
                present_count: present,
                late_count: late,
                absent_count: absent,
                excused_count: excused,
                attendance_percent: percent,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn das_report(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
    group_by: &str,
    school_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<DasReportRow>, String> {
    let normalized_group = match group_by {
        "overall" | "school" | "class" | "cohort" | "student" => group_by,
        _ => return Err("Invalid DAS grouping".to_string()),
    };

    let (select_fields, group_fields, order_fields) = match normalized_group {
        "overall" => (
            "
            'overall' AS label,
            NULL AS school_id,
            '' AS school_name,
            '' AS grade_level,
            '' AS cohort,
            '' AS batch_id,
            NULL AS student_id,
            '' AS student_name
            ",
            "",
            "label",
        ),
        "school" => (
            "
            s.name AS label,
            s.id AS school_id,
            s.name AS school_name,
            '' AS grade_level,
            '' AS cohort,
            '' AS batch_id,
            NULL AS student_id,
            '' AS student_name
            ",
            "s.id, s.name",
            "s.name",
        ),
        "class" => (
            "
            s.name || ' / ' || st.grade_level || CASE WHEN st.track != '' THEN ' ' || st.track ELSE ' Foundation' END || ' / ' || COALESCE(NULLIF(b.batch_id, ''), NULLIF(ts.batch_id, ''), 'Unbatched') AS label,
            s.id AS school_id,
            s.name AS school_name,
            st.grade_level AS grade_level,
            CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END AS cohort,
            COALESCE(NULLIF(b.batch_id, ''), NULLIF(ts.batch_id, ''), 'Unbatched') AS batch_id,
            NULL AS student_id,
            '' AS student_name
            ",
            "s.id, s.name, st.grade_level, CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END, COALESCE(NULLIF(b.batch_id, ''), NULLIF(ts.batch_id, ''), 'Unbatched')",
            "s.name, st.grade_level, cohort, batch_id",
        ),
        "cohort" => (
            "
            CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END AS label,
            NULL AS school_id,
            '' AS school_name,
            '' AS grade_level,
            CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END AS cohort,
            '' AS batch_id,
            NULL AS student_id,
            '' AS student_name
            ",
            "CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END",
            "cohort",
        ),
        "student" => (
            "
            st.name || ' / ' || s.name AS label,
            s.id AS school_id,
            s.name AS school_name,
            st.grade_level AS grade_level,
            CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END AS cohort,
            COALESCE(NULLIF(b.batch_id, ''), NULLIF(ts.batch_id, ''), 'Unbatched') AS batch_id,
            st.id AS student_id,
            st.name AS student_name
            ",
            "st.id, st.name, s.id, s.name, st.grade_level, CASE WHEN st.track IN ('JEE', 'NEET') THEN st.track ELSE 'Foundation' END, COALESCE(NULLIF(b.batch_id, ''), NULLIF(ts.batch_id, ''), 'Unbatched')",
            "s.name, st.grade_level, cohort, batch_id, st.name",
        ),
        _ => unreachable!(),
    };

    let mut filters = String::new();
    if school_id.is_some() {
        filters.push_str(" AND s.id = ? ");
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            filters.push_str(&format!(" AND s.id IN ({placeholders}) "));
        }
    }
    let group_clause = if group_fields.is_empty() {
        String::new()
    } else {
        format!("GROUP BY {group_fields}")
    };

    let sql = format!(
        "
        SELECT
            ?3 AS group_by,
            {select_fields},
            COUNT(*) AS scheduled_lectures,
            COALESCE(SUM(CASE WHEN ar.status = 'Present' THEN 1 ELSE 0 END), 0) AS present_lectures
        FROM lecture_sessions ls
        JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
        JOIN schools s ON s.id = ts.school_id
        LEFT JOIN batches b ON b.id = ts.batch_ref_id
        JOIN students st ON st.school_id = s.id
            AND (
                (ts.batch_ref_id > 0 AND st.batch_ref_id = ts.batch_ref_id)
                OR (
                    ts.batch_ref_id = 0
                    AND st.grade_level = ts.grade_level
                    AND (ts.track = '' OR st.track = ts.track)
                )
            )
        LEFT JOIN attendance_records ar ON ar.lecture_session_id = ls.id AND ar.student_id = st.id
        WHERE ls.session_date >= ?1
          AND ls.session_date <= ?2
          AND ls.status != 'Cancelled'
          {filters}
        {group_clause}
        ORDER BY {order_fields}
        "
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<rusqlite::types::Value> = vec![
        start_date.to_string().into(),
        end_date.to_string().into(),
        normalized_group.to_string().into(),
    ];
    if let Some(id) = school_id {
        params_vec.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push((*id).into());
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let scheduled: i64 = row.get(9)?;
            let present: i64 = row.get(10)?;
            let das_percent = if scheduled > 0 {
                (present * 100) / scheduled
            } else {
                0
            };
            Ok(DasReportRow {
                group_by: row.get(0)?,
                label: row.get(1)?,
                school_id: row.get(2)?,
                school_name: row.get(3)?,
                grade_level: row.get(4)?,
                cohort: row.get(5)?,
                batch_id: row.get(6)?,
                student_id: row.get(7)?,
                student_name: row.get(8)?,
                scheduled_lectures: scheduled,
                present_lectures: present,
                das_percent,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn chronic_absentees(
    conn: &Connection,
    since_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<ChronicAbsentee>, String> {
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND s.id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT
            st.id AS student_id,
            st.name AS student_name,
            s.name AS school_name,
            st.grade_level,
            COUNT(ar.id) AS total_sessions,
            COALESCE(SUM(CASE WHEN ar.status IN ('Present', 'Late') THEN 1 ELSE 0 END), 0) AS present_count
         FROM students st
         JOIN schools s ON s.id = st.school_id
         JOIN attendance_records ar ON ar.student_id = st.id
         JOIN lecture_sessions ls ON ls.id = ar.lecture_session_id
         WHERE ls.session_date >= ?1
           {school_filter}
         GROUP BY st.id, st.name, s.name, st.grade_level
         HAVING total_sessions > 0 AND (present_count * 100 / total_sessions) < 75
         ORDER BY (present_count * 100 / total_sessions) ASC, st.name"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&since_date];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let total: i64 = row.get(4)?;
            let present: i64 = row.get(5)?;
            let percent = if total > 0 {
                (present * 100) / total
            } else {
                0
            };
            Ok(ChronicAbsentee {
                student_id: row.get(0)?,
                student_name: row.get(1)?,
                school_name: row.get(2)?,
                grade_level: row.get(3)?,
                total_sessions: total,
                present_count: present,
                attendance_percent: percent,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn subject_attendance(
    conn: &Connection,
    session_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<SubjectAttendanceRow>, String> {
    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("AND s.id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let sql = format!(
        "SELECT
            sub.name AS subject_name,
            COUNT(ar.id) AS total_sessions,
            COALESCE(SUM(CASE WHEN ar.status = 'Present' THEN 1 ELSE 0 END), 0) AS present_count,
            COALESCE(SUM(CASE WHEN ar.status = 'Late' THEN 1 ELSE 0 END), 0) AS late_count,
            COALESCE(SUM(CASE WHEN ar.status = 'Absent' THEN 1 ELSE 0 END), 0) AS absent_count
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN schools s ON s.id = ts.school_id
         JOIN subjects sub ON sub.id = ts.subject_id
         LEFT JOIN attendance_records ar ON ar.lecture_session_id = ls.id
         WHERE ls.session_date = ?1
           {school_filter}
         GROUP BY sub.name, ls.id
         ORDER BY sub.name"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&session_date];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let total: i64 = row.get(1)?;
            let present: i64 = row.get(2)?;
            let late: i64 = row.get(3)?;
            let absent: i64 = row.get(4)?;
            let attended = present + late;
            let percent = if total > 0 {
                (attended * 100) / total
            } else {
                0
            };
            Ok(SubjectAttendanceRow {
                subject_name: row.get(0)?,
                total_sessions: total,
                present_count: present,
                late_count: late,
                absent_count: absent,
                attendance_percent: percent,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_leave_request(
    conn: &Connection,
    input: &CreateLeaveRequestInput,
) -> Result<LeaveRequest, String> {
    conn.execute(
        "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'Pending')",
        params![input.faculty_user_id, input.school_id, &input.start_date, &input.end_date, &input.reason],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_leave_request(conn, id)
}

pub fn list_leave_requests(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
    faculty_user_id: Option<i64>,
) -> Result<Vec<LeaveRequest>, String> {
    let mut sql = String::from(
        "SELECT lr.id, lr.faculty_user_id, u.display_name, lr.school_id, s.name,
                lr.start_date, lr.end_date, lr.reason, lr.status,
                lr.approved_by_user_id, lr.approved_at,
                lr.rejected_by_user_id, lr.rejected_at, lr.rejection_reason,
                lr.created_at
         FROM leave_requests lr
         JOIN users u ON u.id = lr.faculty_user_id
         JOIN schools s ON s.id = lr.school_id",
    );
    let mut conditions: Vec<String> = Vec::new();
    let mut p: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            conditions.push(format!("lr.school_id IN ({placeholders})"));
            for id in ids {
                p.push(id);
            }
        }
    }
    let faculty_id_holder = faculty_user_id;
    if let Some(ref fid) = faculty_id_holder {
        conditions.push("lr.faculty_user_id = ?".to_string());
        p.push(fid);
    }
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY lr.created_at DESC LIMIT 1000");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(LeaveRequest {
                id: row.get(0)?,
                faculty_user_id: row.get(1)?,
                faculty_name: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                reason: row.get(7)?,
                status: row.get(8)?,
                approved_by_user_id: row.get(9).ok(),
                approved_at: row.get(10).unwrap_or_default(),
                rejected_by_user_id: row.get(11).ok(),
                rejected_at: row.get(12).unwrap_or_default(),
                rejection_reason: row.get(13).unwrap_or_default(),
                created_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_leave_request(conn: &Connection, id: i64) -> Result<LeaveRequest, String> {
    conn.query_row(
        "SELECT lr.id, lr.faculty_user_id, u.display_name, lr.school_id, s.name,
                lr.start_date, lr.end_date, lr.reason, lr.status,
                lr.approved_by_user_id, lr.approved_at,
                lr.rejected_by_user_id, lr.rejected_at, lr.rejection_reason,
                lr.created_at
         FROM leave_requests lr
         JOIN users u ON u.id = lr.faculty_user_id
         JOIN schools s ON s.id = lr.school_id
         WHERE lr.id = ?1",
        params![id],
        |row| {
            Ok(LeaveRequest {
                id: row.get(0)?,
                faculty_user_id: row.get(1)?,
                faculty_name: row.get(2)?,
                school_id: row.get(3)?,
                school_name: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                reason: row.get(7)?,
                status: row.get(8)?,
                approved_by_user_id: row.get(9).ok(),
                approved_at: row.get(10).unwrap_or_default(),
                rejected_by_user_id: row.get(11).ok(),
                rejected_at: row.get(12).unwrap_or_default(),
                rejection_reason: row.get(13).unwrap_or_default(),
                created_at: row.get(14)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn approve_leave_request(
    conn: &Connection,
    leave_id: i64,
    approved_by_user_id: i64,
) -> Result<LeaveRequest, String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let lr = get_leave_request(conn, leave_id)?;
    if lr.status == "Approved" {
        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
        return Ok(lr);
    }
    if lr.status == "Rejected" {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Cannot approve a rejected leave request".to_string());
    }

    let rows = conn.execute(
        "UPDATE leave_requests SET status = 'Approved', approved_by_user_id = ?2, approved_at = datetime('now', 'localtime') WHERE id = ?1 AND status = 'Pending'",
        params![leave_id, approved_by_user_id],
    ).map_err(|e| e.to_string())?;

    if rows == 0 {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Leave request already processed".to_string());
    }

    // Find all lecture sessions for this faculty in the date range and mark them as needing substitution.
    // Use COALESCE to match both substituted sessions (actual_faculty_user_id set) and regular sessions
    // (actual_faculty_user_id null → falls back to timetable_slots.faculty_user_id).
    conn.execute(
        "UPDATE lecture_sessions
         SET status = 'Needs Substitution', leave_request_id = ?1
         FROM timetable_slots
         WHERE timetable_slots.id = lecture_sessions.timetable_slot_id
           AND COALESCE(lecture_sessions.actual_faculty_user_id, timetable_slots.faculty_user_id) = ?2
           AND lecture_sessions.session_date >= ?3
           AND lecture_sessions.session_date <= ?4
           AND lecture_sessions.status != 'Cancelled'
           AND timetable_slots.school_id = ?5",
        params![leave_id, lr.faculty_user_id, &lr.start_date, &lr.end_date, lr.school_id],
    ).map_err(|e| e.to_string())?;

    // Update balance: increment given_count for absent faculty
    conn.execute(
        "INSERT INTO faculty_substitution_balance (faculty_user_id, given_count, received_count)
         VALUES (?1, 1, 0)
         ON CONFLICT(faculty_user_id) DO UPDATE SET
             given_count = given_count + 1,
             updated_at = datetime('now', 'localtime')",
        params![lr.faculty_user_id],
    )
    .map_err(|e| e.to_string())?;

    let lr = get_leave_request(conn, leave_id)?;
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(lr)
}

pub fn reject_leave_request(
    conn: &Connection,
    leave_id: i64,
    rejected_by_user_id: i64,
    reason: &str,
) -> Result<LeaveRequest, String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let lr = get_leave_request(conn, leave_id)?;
    if lr.status == "Approved" {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Cannot reject an approved leave request".to_string());
    }
    if lr.status == "Rejected" {
        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
        return Ok(lr);
    }

    let rows = conn.execute(
        "UPDATE leave_requests SET status = 'Rejected', rejected_by_user_id = ?2, rejected_at = datetime('now', 'localtime'), rejection_reason = ?3 WHERE id = ?1 AND status = 'Pending'",
        params![leave_id, rejected_by_user_id, reason],
    ).map_err(|e| e.to_string())?;

    if rows == 0 {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Leave request already processed".to_string());
    }

    let lr = get_leave_request(conn, leave_id)?;
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(lr)
}

pub fn get_leave_impact_preview(
    conn: &Connection,
    leave_id: i64,
) -> Result<LeaveImpactPreview, String> {
    let lr = get_leave_request(conn, leave_id)?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         WHERE COALESCE(ls.actual_faculty_user_id, ts.faculty_user_id) = ?1
           AND ls.session_date >= ?2
           AND ls.session_date <= ?3
           AND ls.status != 'Cancelled'",
            params![lr.faculty_user_id, &lr.start_date, &lr.end_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(LeaveImpactPreview {
        leave_request_id: leave_id,
        affected_session_count: count,
        date_range_start: lr.start_date,
        date_range_end: lr.end_date,
        school_name: lr.school_name,
        faculty_name: lr.faculty_name,
    })
}

pub fn create_leave_audit_log(
    conn: &Connection,
    leave_request_id: i64,
    actor_user_id: i64,
    action: &str,
    old_status: &str,
    new_status: &str,
    reason: &str,
    school_id: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO leave_request_audit_log (leave_request_id, actor_user_id, action, old_status, new_status, reason, school_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![leave_request_id, actor_user_id, action, old_status, new_status, reason, school_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_school_approver_user_ids(
    conn: &Connection,
    school_id: i64,
) -> Result<Vec<i64>, String> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT user_id FROM user_schools WHERE school_id = ?1 AND role IN ('head', 'admin', 'aom')"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![school_id], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_notification(
    conn: &Connection,
    user_id: i64,
    notification_type: &str,
    title: &str,
    message: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO notification_log (user_id, type, title, message) VALUES (?1, ?2, ?3, ?4)",
        params![user_id, notification_type, title, message],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_swap_request(
    conn: &Connection,
    input: &CreateSwapRequestInput,
) -> Result<SwapRequest, String> {
    crate::substitution_engine::validate_swap(
        conn,
        input.slot_a_id,
        input.slot_b_id,
        Some(input.requester_faculty_id),
        Some(input.recipient_faculty_id),
    )?;

    conn.execute(
        "INSERT INTO swap_requests (requester_faculty_id, recipient_faculty_id, slot_a_id, slot_b_id, status)
         VALUES (?1, ?2, ?3, ?4, 'Pending')",
        params![input.requester_faculty_id, input.recipient_faculty_id, input.slot_a_id, input.slot_b_id],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_swap_request(conn, id)
}

pub fn list_swap_requests(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
    faculty_user_id: Option<i64>,
) -> Result<Vec<SwapRequest>, String> {
    let mut sql = String::from(
        "SELECT sr.id, sr.requester_faculty_id, u1.display_name,
                sr.recipient_faculty_id, u2.display_name,
                sr.slot_a_id, sr.slot_b_id, sr.status, sr.created_at
         FROM swap_requests sr
         JOIN users u1 ON u1.id = sr.requester_faculty_id
         JOIN users u2 ON u2.id = sr.recipient_faculty_id
         JOIN timetable_slots ts ON ts.id = sr.slot_a_id",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    let mut has_where = false;

    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" WHERE ts.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
            has_where = true;
        }
    }

    if let Some(ref uid) = faculty_user_id {
        if has_where {
            sql.push_str(" AND (sr.requester_faculty_id = ? OR sr.recipient_faculty_id = ?)");
        } else {
            sql.push_str(" WHERE (sr.requester_faculty_id = ? OR sr.recipient_faculty_id = ?)");
        }
        params_vec.push(uid);
        params_vec.push(uid);
    }
    sql.push_str(" ORDER BY sr.created_at DESC LIMIT 1000");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(SwapRequest {
                id: row.get(0)?,
                requester_faculty_id: row.get(1)?,
                requester_name: row.get(2)?,
                recipient_faculty_id: row.get(3)?,
                recipient_name: row.get(4)?,
                slot_a_id: row.get(5)?,
                slot_b_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_swap_request(conn: &Connection, id: i64) -> Result<SwapRequest, String> {
    conn.query_row(
        "SELECT sr.id, sr.requester_faculty_id, u1.display_name,
                sr.recipient_faculty_id, u2.display_name,
                sr.slot_a_id, sr.slot_b_id, sr.status, sr.created_at
         FROM swap_requests sr
         JOIN users u1 ON u1.id = sr.requester_faculty_id
         JOIN users u2 ON u2.id = sr.recipient_faculty_id
         WHERE sr.id = ?1",
        params![id],
        |row| {
            Ok(SwapRequest {
                id: row.get(0)?,
                requester_faculty_id: row.get(1)?,
                requester_name: row.get(2)?,
                recipient_faculty_id: row.get(3)?,
                recipient_name: row.get(4)?,
                slot_a_id: row.get(5)?,
                slot_b_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn accept_swap_request(conn: &Connection, id: i64) -> Result<SwapRequest, String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let sr = get_swap_request(conn, id)?;
    if sr.status != "Pending" {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Swap request is not pending".to_string());
    }

    crate::substitution_engine::validate_swap(
        conn,
        sr.slot_a_id,
        sr.slot_b_id,
        Some(sr.requester_faculty_id),
        Some(sr.recipient_faculty_id),
    )?;

    // Swap faculty assignments on timetable_slots
    conn.execute(
        "UPDATE timetable_slots SET faculty_user_id = ?1 WHERE id = ?2",
        params![sr.recipient_faculty_id, sr.slot_a_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE timetable_slots SET faculty_user_id = ?1 WHERE id = ?2",
        params![sr.requester_faculty_id, sr.slot_b_id],
    )
    .map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE swap_requests SET status = 'Accepted' WHERE id = ?1 AND status = 'Pending'",
            params![id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Swap request already processed".to_string());
    }

    let sr = get_swap_request(conn, id)?;
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(sr)
}

pub fn get_today_substitutions(
    conn: &Connection,
    date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<TodaySubstitutions, String> {
    let mut sql = String::from(
        "SELECT ls.id, ls.session_date, ts.school_id, s.name, ts.grade_level, ts.track, ts.batch_pattern,
                ts.period, sub.name, COALESCE(orig.display_name, ''), COALESCE(subst.display_name, ''), ls.status, ts.room
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN schools s ON s.id = ts.school_id
         JOIN subjects sub ON sub.id = ts.subject_id
         LEFT JOIN users orig ON orig.id = ts.faculty_user_id
         LEFT JOIN users subst ON subst.id = ls.actual_faculty_user_id
         WHERE ls.session_date = ?1
           AND (ls.status = 'Needs Substitution' OR ls.actual_faculty_user_id IS NOT NULL
                OR ls.status = 'Completed')"
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(date.to_string().into());

    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ts.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY ts.period ASC LIMIT 1000");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(TodaySubstitutionLane {
                session_id: row.get(0)?,
                session_date: row.get(1)?,
                school_id: row.get(2)?,
                school_name: row.get(3)?,
                grade_level: row.get(4)?,
                track: row.get(5)?,
                batch_pattern: row.get(6)?,
                period: row.get(7)?,
                subject_name: row.get(8)?,
                original_faculty_name: row.get(9)?,
                substitute_faculty_name: row.get(10)?,
                status: row.get(11)?,
                room: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut unfilled = Vec::new();
    let mut assigned = Vec::new();
    let mut completed = Vec::new();

    for r in rows {
        let lane = r.map_err(|e| e.to_string())?;
        if lane.status == "Needs Substitution" {
            unfilled.push(lane);
        } else if lane.status == "Completed" {
            completed.push(lane);
        } else if lane.substitute_faculty_name.is_some()
            && lane.substitute_faculty_name.as_deref() != Some("")
        {
            assigned.push(lane);
        } else {
            unfilled.push(lane);
        }
    }

    Ok(TodaySubstitutions {
        unfilled,
        assigned,
        completed,
    })
}

pub fn get_substitution_detail(
    conn: &Connection,
    session_id: i64,
) -> Result<SubstitutionDetail, String> {
    let (school_name, grade_level, track, batch_pattern, subject_name, room, session_date): (String, String, String, String, String, String, String) = conn.query_row(
        "SELECT s.name, ts.grade_level, ts.track, ts.batch_pattern, sub.name, ts.room, ls.session_date
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN schools s ON s.id = ts.school_id
         JOIN subjects sub ON sub.id = ts.subject_id
         WHERE ls.id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    ).map_err(|e| e.to_string())?;

    let roster_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM attendance_records WHERE lecture_session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let present_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM attendance_records WHERE lecture_session_id = ?1 AND status = 'Present'",
        params![session_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let absent_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM attendance_records WHERE lecture_session_id = ?1 AND status = 'Absent'",
        params![session_id],
        |row| row.get(0),
    ).unwrap_or(0);

    // TODO: last_covered_topics should come from a lesson_plan table when available
    let last_covered_topics = String::from("—");

    Ok(SubstitutionDetail {
        session_id,
        session_date,
        school_name,
        grade_level,
        track,
        batch_pattern,
        subject_name,
        room,
        roster_count,
        present_count,
        absent_count,
        last_covered_topics,
    })
}

pub fn get_substitution_balance(
    conn: &Connection,
    faculty_user_id: i64,
) -> Result<SubstitutionBalance, String> {
    let (name, given, received): (String, i64, i64) = conn
        .query_row(
            "SELECT u.display_name,
                COALESCE(fsb.given_count, 0),
                COALESCE(fsb.received_count, 0)
         FROM users u
         LEFT JOIN faculty_substitution_balance fsb ON fsb.faculty_user_id = u.id
         WHERE u.id = ?1",
            params![faculty_user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(SubstitutionBalance {
        faculty_user_id,
        faculty_name: name,
        given_count: given,
        received_count: received,
    })
}

pub fn get_substitution_reports(
    conn: &Connection,
    month: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<SubstitutionReportRow>, String> {
    // month format: YYYY-MM
    // Single query with window function instead of 1 + S round-trips.
    let mut scope_filter = String::new();
    let mut scope_params: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            scope_filter = format!(" AND ts.school_id IN ({placeholders})");
            for id in ids {
                scope_params.push((*id).into());
            }
        }
    }

    let sql = format!(
        "WITH base AS (
            SELECT ts.school_id, s.name,
                   COUNT(*) AS request_count,
                   SUM(CASE WHEN ls.actual_faculty_user_id IS NOT NULL AND ls.actual_faculty_user_id != ts.faculty_user_id THEN 1 ELSE 0 END) AS filled_count
            FROM lecture_sessions ls
            JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
            JOIN schools s ON s.id = ts.school_id
            WHERE strftime('%Y-%m', ls.session_date) = ?1
              AND (ls.status = 'Needs Substitution' OR ls.actual_faculty_user_id != ts.faculty_user_id)
              {scope_filter}
            GROUP BY ts.school_id, s.name
         ),
         faculty_counts AS (
            SELECT ts.school_id, u.display_name, COUNT(*) AS cnt,
                   ROW_NUMBER() OVER (PARTITION BY ts.school_id ORDER BY COUNT(*) DESC) AS rn
            FROM lecture_sessions ls
            JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
            JOIN users u ON u.id = ts.faculty_user_id
            WHERE strftime('%Y-%m', ls.session_date) = ?1
              AND (ls.status = 'Needs Substitution' OR ls.actual_faculty_user_id != ts.faculty_user_id)
              {scope_filter}
            GROUP BY ts.school_id, u.display_name
         )
         SELECT b.school_id, b.name, b.request_count, b.filled_count,
                COALESCE(fc.display_name, '') AS top_absentee_name,
                COALESCE(fc.cnt, 0) AS top_absentee_count
         FROM base b
         LEFT JOIN faculty_counts fc ON fc.school_id = b.school_id AND fc.rn = 1
         LIMIT 1000"
    );

    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(month.to_string().into());
    p.extend(scope_params.iter().cloned());
    p.push(month.to_string().into());
    p.extend(scope_params.iter().cloned());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            let request_count: i64 = row.get(2)?;
            let filled_count: i64 = row.get(3)?;
            let acceptance_rate = if request_count > 0 {
                (filled_count * 100) / request_count
            } else {
                0
            };
            Ok(SubstitutionReportRow {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                month: month.to_string(),
                request_count,
                filled_count,
                acceptance_rate_pct: acceptance_rate,
                avg_minutes_to_fill: 0,
                top_absentee_name: row.get(4)?,
                top_absentee_count: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// Returns (faculty_id, school_id) pairs for every concrete session that
/// bulk_mark_faculty_absent would touch on the given date.
/// Used by the route for pre-authorization scope checks.
pub fn list_bulk_absence_target_schools(
    conn: &Connection,
    faculty_ids: &[i64],
    date: &str,
) -> Result<Vec<(i64, i64)>, String> {
    if faculty_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = (0..faculty_ids.len())
        .map(|i| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "SELECT DISTINCT COALESCE(ls.actual_faculty_user_id, ts.faculty_user_id) as faculty_id,
                ts.school_id
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         WHERE COALESCE(ls.actual_faculty_user_id, ts.faculty_user_id) IN ({})
           AND ls.session_date = ?{}
           AND ls.status != 'Cancelled'",
        placeholders.join(", "),
        faculty_ids.len() + 1
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for id in faculty_ids {
        params_vec.push(id);
    }
    params_vec.push(&date);

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn bulk_mark_faculty_absent(
    conn: &Connection,
    input: &BulkAttendanceInput,
) -> Result<Vec<i64>, String> {
    let mut affected_session_ids = Vec::new();
    for faculty_id in &input.faculty_user_ids {
        let mut stmt = conn
            .prepare(
                "SELECT ls.id FROM lecture_sessions ls
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE COALESCE(ls.actual_faculty_user_id, ts.faculty_user_id) = ?1
               AND ls.session_date = ?2
               AND ls.status != 'Cancelled'",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![faculty_id, &input.date], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        for r in rows {
            let session_id: i64 = r.map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE lecture_sessions SET status = 'Needs Substitution' WHERE id = ?1",
                params![session_id],
            )
            .map_err(|e| e.to_string())?;
            affected_session_ids.push(session_id);

            // Create a linked ticket for the absence
            let school_id: i64 = conn
                .query_row(
                    "SELECT ts.school_id FROM lecture_sessions ls
                 JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
                 WHERE ls.id = ?1",
                    params![session_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let faculty_name: String = conn
                .query_row(
                    "SELECT display_name FROM users WHERE id = ?1",
                    params![faculty_id],
                    |row| row.get(0),
                )
                .unwrap_or_default();

            let _ = conn.execute(
                "INSERT INTO tickets (title, description, requester, assignee, priority, queue, school_id, school_name, status, issue_category)
                 SELECT ?1, ?2, ?3, 'Unassigned', 'High', 'Operations', s.id, s.name, 'Open', 'Attendance'
                 FROM schools s WHERE s.id = ?4",
                params![
                    format!("Faculty absence: {} on {}", faculty_name, input.date),
                    format!("{} marked absent. Reason: {}. Auto-generated substitution ticket.", faculty_name, input.reason),
                    faculty_name,
                    school_id,
                ],
            );
        }

        // Update balance
        conn.execute(
            "INSERT INTO faculty_substitution_balance (faculty_user_id, given_count, received_count)
             VALUES (?1, 1, 0)
             ON CONFLICT(faculty_user_id) DO UPDATE SET
                 given_count = given_count + 1,
                 updated_at = datetime('now', 'localtime')",
            params![faculty_id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(affected_session_ids)
}

pub fn mark_attendance_quick(
    conn: &Connection,
    input: &MarkAttendanceQuickInput,
    marked_by_user_id: i64,
) -> Result<(), String> {
    if !VALID_ATTENDANCE_STATUSES.contains(&input.status.as_str()) {
        return Err(format!("Invalid attendance status: {}", input.status));
    }

    let session = get_lecture_session(conn, input.session_id)?;
    if session.status == "Cancelled" {
        return Err("Cannot mark attendance for a cancelled session".to_string());
    }

    let locked = is_attendance_locked(conn, input.session_id, input.student_id)?;
    if locked {
        return Err(format!(
            "Attendance record for student {} is locked (older than {} hours). Admin override required.",
            input.student_id, LOCK_HOURS
        ));
    }

    let old = get_attendance_record_for_audit(conn, input.session_id, input.student_id)?;

    conn.execute(
        "INSERT INTO attendance_records (lecture_session_id, student_id, status, marked_by_user_id)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(lecture_session_id, student_id) DO UPDATE SET
             status = excluded.status,
             marked_by_user_id = excluded.marked_by_user_id,
             marked_at = datetime('now', 'localtime')",
        params![
            input.session_id,
            input.student_id,
            &input.status,
            marked_by_user_id
        ],
    )
    .map_err(|e| e.to_string())?;

    if let Some((old_status, student_name)) = old {
        if old_status != input.status {
            let actor_name: String = conn
                .query_row(
                    "SELECT display_name FROM users WHERE id = ?1",
                    params![marked_by_user_id],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| format!("id={}", marked_by_user_id));
            let summary = format!(
                "Student {}: {} → {} in session {}",
                student_name, old_status, input.status, input.session_id
            );
            insert_audit_log(
                conn,
                "attendance_record",
                Some(input.student_id),
                "update_status",
                &actor_name,
                &summary,
            )?;
        }
    }

    conn.execute(
        "UPDATE lecture_sessions SET status = 'Completed' WHERE id = ?1",
        params![input.session_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn assign_substitute(
    conn: &Connection,
    session_id: i64,
    substitute_faculty_user_id: i64,
) -> Result<(), String> {
    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE lecture_sessions SET actual_faculty_user_id = ?1, status = 'Substituted'
         WHERE id = ?2 AND (actual_faculty_user_id IS NULL OR actual_faculty_user_id = ?1)",
            params![substitute_faculty_user_id, session_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        conn.execute("ROLLBACK", []).map_err(|e| e.to_string())?;
        return Err("Session already assigned to another faculty".to_string());
    }

    // Update balance: increment received_count for substitute
    conn.execute(
        "INSERT INTO faculty_substitution_balance (faculty_user_id, given_count, received_count)
         VALUES (?1, 0, 1)
         ON CONFLICT(faculty_user_id) DO UPDATE SET
             received_count = received_count + 1,
             updated_at = datetime('now', 'localtime')",
        params![substitute_faculty_user_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{CreateBatchInput, CreateStudentInput};
    use crate::repo::analytics::list_substitution_records;
    use crate::repo::schools::{create_batch, create_student};

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        db::initialize_db(&conn).expect("initialize schema");
        conn
    }

    #[test]
    fn ensure_session_students_uses_concrete_batch_roster() {
        let conn = test_db();
        let batch_a = create_batch(
            &conn,
            &CreateBatchInput {
                school_id: 1,
                batch_id: "XI-JEE-WD-A-P5".to_string(),
                grade_level: "Grade 11".to_string(),
                track: "JEE".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 40,
            },
        )
        .unwrap();
        let batch_b = create_batch(
            &conn,
            &CreateBatchInput {
                school_id: 1,
                batch_id: "XI-JEE-WD-B-P5".to_string(),
                grade_level: "Grade 11".to_string(),
                track: "JEE".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 40,
            },
        )
        .unwrap();

        let student_a = create_student(
            &conn,
            &CreateStudentInput {
                school_id: 1,
                name: "Phase Five Batch A Student".to_string(),
                registration_number: "P5-A".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE".to_string(),
                track: "JEE".to_string(),
                student_mobile: String::new(),
                student_email: String::new(),
                father_name: String::new(),
                father_email: String::new(),
                father_mobile: String::new(),
                mother_name: String::new(),
                mother_email: String::new(),
                mother_mobile: String::new(),
                batch_ref_id: batch_a.id,
                batch_id: String::new(),
            },
        )
        .unwrap();
        let student_b = create_student(
            &conn,
            &CreateStudentInput {
                school_id: 1,
                name: "Phase Five Batch B Student".to_string(),
                registration_number: "P5-B".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE".to_string(),
                track: "JEE".to_string(),
                student_mobile: String::new(),
                student_email: String::new(),
                father_name: String::new(),
                father_email: String::new(),
                father_mobile: String::new(),
                mother_name: String::new(),
                mother_email: String::new(),
                mother_mobile: String::new(),
                batch_ref_id: batch_b.id,
                batch_id: String::new(),
            },
        )
        .unwrap();

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO timetable_slots
                (school_id, grade_level, track, batch_pattern, batch_id, batch_ref_id, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', 'JEE', 'Weekday', ?1, ?2, 1, 1, ?3, NULL)",
            params![batch_a.batch_id, batch_a.id, subject_id],
        )
        .unwrap();
        let slot_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-05-04', ?1, 'Scheduled')",
            params![slot_id],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        ensure_session_students(&conn, session_id).unwrap();
        let attendance = get_session_attendance(&conn, session_id).unwrap();
        assert_eq!(attendance.len(), 1);
        assert_eq!(attendance[0].student_id, student_a.id);
        assert_ne!(attendance[0].student_id, student_b.id);

        let summary = attendance_summary(&conn, "2026-05-04", None).unwrap();
        let row = summary
            .iter()
            .find(|r| r.batch_id == batch_a.batch_id)
            .expect("batch A attendance summary row");
        assert_eq!(row.total_students, 1);
    }

    #[test]
    fn das_report_counts_scheduled_student_lecture_opportunities() {
        let conn = test_db();
        let batch = create_batch(
            &conn,
            &CreateBatchInput {
                school_id: 1,
                batch_id: "XI-JEE-DAS-A".to_string(),
                grade_level: "Grade 11".to_string(),
                track: "JEE".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 40,
            },
        )
        .unwrap();

        let present_student = create_student(
            &conn,
            &CreateStudentInput {
                school_id: 1,
                name: "DAS Present Student".to_string(),
                registration_number: "DAS-P".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE".to_string(),
                track: "JEE".to_string(),
                student_mobile: String::new(),
                student_email: String::new(),
                father_name: String::new(),
                father_email: String::new(),
                father_mobile: String::new(),
                mother_name: String::new(),
                mother_email: String::new(),
                mother_mobile: String::new(),
                batch_ref_id: batch.id,
                batch_id: String::new(),
            },
        )
        .unwrap();
        let absent_student = create_student(
            &conn,
            &CreateStudentInput {
                school_id: 1,
                name: "DAS Unmarked Student".to_string(),
                registration_number: "DAS-U".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE".to_string(),
                track: "JEE".to_string(),
                student_mobile: String::new(),
                student_email: String::new(),
                father_name: String::new(),
                father_email: String::new(),
                father_mobile: String::new(),
                mother_name: String::new(),
                mother_email: String::new(),
                mother_mobile: String::new(),
                batch_ref_id: batch.id,
                batch_id: String::new(),
            },
        )
        .unwrap();

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO timetable_slots
                (school_id, grade_level, track, batch_pattern, batch_id, batch_ref_id, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', 'JEE', 'Weekday', ?1, ?2, 1, 2, ?3, NULL)",
            params![batch.batch_id, batch.id, subject_id],
        )
        .unwrap();
        let slot_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-05-04', ?1, 'Scheduled')",
            params![slot_id],
        )
        .unwrap();
        let session_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO attendance_records (lecture_session_id, student_id, status)
             VALUES (?1, ?2, 'Present')",
            params![session_id, present_student.id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO attendance_records (lecture_session_id, student_id, status)
             VALUES (?1, ?2, 'Absent')",
            params![session_id, absent_student.id],
        )
        .unwrap();

        let rows = das_report(&conn, "2026-05-04", "2026-05-04", "class", None, None)
            .expect("das report");
        let row = rows
            .iter()
            .find(|r| r.batch_id == batch.batch_id)
            .expect("batch DAS row");
        assert_eq!(row.scheduled_lectures, 2);
        assert_eq!(row.present_lectures, 1);
        assert_eq!(row.das_percent, 50);
    }

    #[test]
    fn approve_leave_marks_regular_sessions_for_substitution() {
        let conn = test_db();

        // Insert a faculty user
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('test_faculty', 'Test Faculty', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        // Pick an existing subject
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' AND track = 'Foundation' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Create a timetable slot for this faculty
        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 1, ?1, ?2)",
            params![subject_id, faculty_id],
        ).unwrap();
        let slot_id: i64 = conn.last_insert_rowid();

        // Create a regular lecture session (not substituted)
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-04-30', ?1, 'Scheduled')",
            params![slot_id],
        )
        .unwrap();
        let session_id: i64 = conn.last_insert_rowid();

        // Create a leave request
        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        // Approve leave
        let lr = approve_leave_request(&conn, leave_id, 999).unwrap();
        assert_eq!(lr.status, "Approved");
        assert_eq!(lr.approved_by_user_id, Some(999));

        // Verify the regular session is marked for substitution
        let status: String = conn
            .query_row(
                "SELECT status FROM lecture_sessions WHERE id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "Needs Substitution");
    }
    #[test]
    fn reject_leave_request_works() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        let lr = reject_leave_request(&conn, leave_id, 888, "Insufficient coverage").unwrap();
        assert_eq!(lr.status, "Rejected");
        assert_eq!(lr.rejected_by_user_id, Some(888));
        assert_eq!(lr.rejection_reason, "Insufficient coverage");
    }
    #[test]
    fn approve_after_reject_fails() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        reject_leave_request(&conn, leave_id, 888, "No").unwrap();
        let result = approve_leave_request(&conn, leave_id, 999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rejected"));
    }
    #[test]
    fn double_approve_is_idempotent() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        // Pick an existing subject
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' AND track = 'Foundation' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 1, ?1, ?2)",
            params![subject_id, faculty_id],
        ).unwrap();
        let slot_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-04-30', ?1, 'Scheduled')",
            params![slot_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        let lr1 = approve_leave_request(&conn, leave_id, 999).unwrap();
        let lr2 = approve_leave_request(&conn, leave_id, 999).unwrap();
        assert_eq!(lr1.id, lr2.id);
        assert_eq!(lr1.status, "Approved");

        // Verify only one balance record exists (not doubled)
        let count: i64 = conn
            .query_row(
                "SELECT given_count FROM faculty_substitution_balance WHERE faculty_user_id = ?1",
                params![faculty_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
    #[test]
    fn reject_after_approve_fails() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        approve_leave_request(&conn, leave_id, 999).unwrap();
        let result = reject_leave_request(&conn, leave_id, 888, "Too late");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("approved"));
    }
    #[test]
    fn list_leave_requests_filters_by_faculty() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_a', 'Faculty A', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let fac_a: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_b', 'Faculty B', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let fac_b: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'A', 'Pending')",
            params![fac_a],
        ).unwrap();
        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'B', 'Pending')",
            params![fac_b],
        ).unwrap();

        let all = list_leave_requests(&conn, None, None).unwrap();
        assert_eq!(all.len(), 2);

        let filtered = list_leave_requests(&conn, None, Some(fac_a)).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].reason, "A");
    }
    #[test]
    fn leave_impact_preview_counts_sessions() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' AND track = 'Foundation' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 1, ?1, ?2)",
            params![subject_id, faculty_id],
        ).unwrap();
        let slot_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-04-30', ?1, 'Scheduled')",
            params![slot_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        let preview = get_leave_impact_preview(&conn, leave_id).unwrap();
        assert_eq!(preview.leave_request_id, leave_id);
        assert_eq!(preview.affected_session_count, 1);
        assert_eq!(preview.faculty_name, "Faculty Test");
    }
    #[test]
    fn leave_audit_log_is_written() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('faculty_test', 'Faculty Test', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO leave_requests (faculty_user_id, school_id, start_date, end_date, reason, status)
             VALUES (?1, 1, '2026-04-30', '2026-04-30', 'Test', 'Pending')",
            params![faculty_id],
        ).unwrap();
        let leave_id: i64 = conn.last_insert_rowid();

        create_leave_audit_log(
            &conn, leave_id, 999, "approve", "Pending", "Approved", "", 1,
        )
        .unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM leave_request_audit_log WHERE leave_request_id = ?1 AND action = 'approve'",
            params![leave_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }
    #[test]
    fn substitution_accept_and_decline_work_correctly() {
        let conn = test_db();

        // Insert two faculty users
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_a', 'Faculty A', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_a: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_b', 'Faculty B', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_b: i64 = conn.last_insert_rowid();

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' AND track = 'Foundation' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Create timetable slot assigned to faculty_a
        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 1, ?1, ?2)",
            params![subject_id, faculty_a],
        ).unwrap();
        let slot_id: i64 = conn.last_insert_rowid();

        // Create lecture session (regular, not substituted)
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, school_id, status)
             VALUES ('2026-04-30', ?1, 1, 'Needs Substitution')",
            params![slot_id],
        )
        .unwrap();
        let session_id: i64 = conn.last_insert_rowid();

        // Accept substitution for faculty_b
        accept_substitution(&conn, session_id, faculty_b).unwrap();
        let session = get_lecture_session(&conn, session_id).unwrap();
        assert_eq!(session.actual_faculty_user_id, Some(faculty_b));
        assert_eq!(session.status, "Substituted");

        // Decline substitution
        decline_substitution(&conn, session_id, "Unavailable").unwrap();
        let session = get_lecture_session(&conn, session_id).unwrap();
        assert_eq!(session.actual_faculty_user_id, None);
        assert_eq!(session.status, "Needs Substitution");

        // list_substitution_records should show the session when filtered by the substitute (faculty_b)
        accept_substitution(&conn, session_id, faculty_b).unwrap();
        let records =
            list_substitution_records(&conn, Some(1), Some(faculty_b), "2026-04-30", None).unwrap();
        assert!(records.iter().any(|r| r.session_id == session_id));

        // list_substitution_records should also show the session when filtered by the original faculty (faculty_a)
        // because the query matches both original and substitute faculty
        let records_orig =
            list_substitution_records(&conn, Some(1), Some(faculty_a), "2026-04-30", None).unwrap();
        assert!(records_orig.iter().any(|r| r.session_id == session_id));
    }
    #[test]
    fn bulk_absence_catches_regular_and_substituted_sessions() {
        let conn = test_db();

        // Insert two faculty users
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_a', 'Faculty A', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_a: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash)
             VALUES ('fac_b', 'Faculty B', 'faculty', 'hash')",
            [],
        )
        .unwrap();
        let faculty_b: i64 = conn.last_insert_rowid();

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE name = 'Physics' AND track = 'Foundation' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Create two timetable slots assigned to faculty_a
        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 1, ?1, ?2)",
            params![subject_id, faculty_a],
        ).unwrap();
        let slot_a: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id)
             VALUES (1, 'Grade 11', '', 'Weekday', 1, 2, ?1, ?2)",
            params![subject_id, faculty_a],
        ).unwrap();
        let slot_b: i64 = conn.last_insert_rowid();

        // Create a regular session on slot_a (actual_faculty_user_id = NULL)
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, status)
             VALUES ('2026-04-30', ?1, 'Scheduled')",
            params![slot_a],
        )
        .unwrap();
        let regular_session: i64 = conn.last_insert_rowid();

        // Create a substituted session on slot_b (faculty_b is covering)
        conn.execute(
            "INSERT INTO lecture_sessions (session_date, timetable_slot_id, actual_faculty_user_id, status)
             VALUES ('2026-04-30', ?1, ?2, 'Scheduled')",
            params![slot_b, faculty_b],
        ).unwrap();
        let substituted_session: i64 = conn.last_insert_rowid();

        // Bulk mark faculty_a absent — should catch the regular session
        let affected_a = bulk_mark_faculty_absent(
            &conn,
            &BulkAttendanceInput {
                faculty_user_ids: vec![faculty_a],
                date: "2026-04-30".to_string(),
                reason: "Test".to_string(),
            },
        )
        .unwrap();
        assert!(affected_a.contains(&regular_session));
        assert!(!affected_a.contains(&substituted_session));

        // Bulk mark faculty_b absent — should catch the substituted session
        let affected_b = bulk_mark_faculty_absent(
            &conn,
            &BulkAttendanceInput {
                faculty_user_ids: vec![faculty_b],
                date: "2026-04-30".to_string(),
                reason: "Test".to_string(),
            },
        )
        .unwrap();
        assert!(affected_b.contains(&substituted_session));
        assert!(!affected_b.contains(&regular_session));
    }
}
