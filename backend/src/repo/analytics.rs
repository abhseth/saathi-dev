use crate::models::{
    ComplianceMetrics, DeviationScore, FacultyCrossSchoolSchedule, FacultyOverload, SubjectGap,
    SubstitutionRecord, TimetableHealthStatus, UnderutilizedBatch, WeeklyTimetableSlot,
};
use rusqlite::{params, Connection};

use super::common::*;
use crate::repo::faculty::get_weekly_timetable_slots_by_ids;

pub fn list_faculty_cross_school_schedule(
    conn: &Connection,
    faculty_user_id: i64,
    week_start_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyCrossSchoolSchedule>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT wts.faculty_user_id, COALESCE(u.display_name, ''),
                wts.school_id, s.name,
                wts.day_of_week, wts.period,
                wts.start_time, wts.end_time,
                sub.name, wts.grade_level, wts.track, wts.batch_pattern,
                wts.room, wts.week_start_date
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         JOIN subjects sub ON sub.id = wts.subject_id
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         WHERE wts.faculty_user_id = ?1
           AND wts.week_start_date = ?2",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(faculty_user_id.into());
    p.push(week_start_date.to_string().into());
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND wts.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY wts.day_of_week, wts.period LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(FacultyCrossSchoolSchedule {
                faculty_user_id: row.get(0)?,
                faculty_name: row.get(1)?,
                school_id: row.get(2)?,
                school_name: row.get(3)?,
                day_of_week: row.get(4)?,
                period: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                subject_name: row.get(8)?,
                grade_level: row.get(9)?,
                track: row.get(10)?,
                batch_pattern: row.get(11)?,
                room: row.get(12)?,
                week_start_date: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_timetable_health_status(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<TimetableHealthStatus>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "WITH school_stats AS (
            SELECT
                s.id AS school_id,
                s.name AS school_name,
                COALESCE(r.name, '') AS region_name,
                COALESCE(s.aom_name, '') AS aom_name,
                (SELECT COUNT(*) FROM school_class_plans WHERE school_id = s.id) AS plan_count,
                (SELECT COUNT(*) FROM timetable_slots WHERE school_id = s.id AND deleted_at IS NULL) AS slot_count,
                (SELECT COUNT(*) FROM lecture_sessions WHERE school_id = s.id AND session_date >= date('now', 'weekday 1', '-7 days') AND session_date < date('now', 'weekday 1')) AS session_count,
                (SELECT COUNT(*) FROM timetable_slots WHERE school_id = s.id AND deleted_at IS NULL AND faculty_user_id IS NULL) AS unassigned_count
            FROM schools s
            LEFT JOIN regions r ON r.id = s.region_id
            WHERE s.is_dropped = 0
        )
        SELECT school_id, school_name, region_name, aom_name,
               CASE WHEN plan_count > 0 THEN 1 ELSE 0 END,
               CASE WHEN slot_count > 0 THEN 1 ELSE 0 END,
               CASE WHEN session_count > 0 THEN 1 ELSE 0 END,
               unassigned_count,
               CASE
                   WHEN plan_count > 0 AND slot_count > 0 AND session_count > 0 AND unassigned_count = 0 THEN 'Green'
                   WHEN plan_count = 0 AND slot_count = 0 THEN 'Red'
                   WHEN unassigned_count > 2 THEN 'Red'
                   ELSE 'Amber'
               END,
               datetime('now', 'localtime')
        FROM school_stats
        WHERE 1=1",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(TimetableHealthStatus {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                region_name: row.get(2)?,
                aom_name: row.get(3)?,
                class_plans_configured: row.get::<_, i64>(4)? == 1,
                master_timetable_complete: row.get::<_, i64>(5)? == 1,
                sessions_generated: row.get::<_, i64>(6)? == 1,
                gaps_count: row.get(7)?,
                status: row.get(8)?,
                last_updated: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_compliance_metrics(
    conn: &Connection,
    school_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<ComplianceMetrics>, String> {
    const MAX_ROWS: i64 = 1000;
    let week_start: String = conn
        .query_row("SELECT date('now', 'weekday 1', '-7 days')", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "WITH all_keys AS (
            SELECT DISTINCT school_id, grade_level, track, subject_name FROM (
                SELECT ts.school_id, ts.grade_level, ts.track, sub.name AS subject_name
                FROM timetable_slots ts
                JOIN subjects sub ON sub.id = ts.subject_id
                WHERE ts.deleted_at IS NULL
                UNION
                SELECT wts.school_id, wts.grade_level, wts.track, sub.name AS subject_name
                FROM timetable_weekly_slots wts
                JOIN subjects sub ON sub.id = wts.subject_id
                WHERE wts.week_start_date = ?1
            )
        ),
        template_counts AS (
            SELECT ts.school_id, ts.grade_level, ts.track, sub.name AS subject_name, COUNT(*) AS planned
            FROM timetable_slots ts
            JOIN subjects sub ON sub.id = ts.subject_id
            WHERE ts.deleted_at IS NULL
            GROUP BY ts.school_id, ts.grade_level, ts.track, sub.name
        ),
        weekly_counts AS (
            SELECT wts.school_id, wts.grade_level, wts.track, sub.name AS subject_name, COUNT(*) AS actual,
                   SUM(CASE WHEN wts.session_type = 'Lecture' THEN 1 ELSE 0 END) AS lecture_count
            FROM timetable_weekly_slots wts
            JOIN subjects sub ON sub.id = wts.subject_id
            WHERE wts.week_start_date = ?1
            GROUP BY wts.school_id, wts.grade_level, wts.track, sub.name
        )
        SELECT k.school_id, s.name, k.grade_level, k.track, k.subject_name,
               COALESCE(t.planned, 0), COALESCE(w.actual, 0),
               COALESCE(t.planned, 0) - COALESCE(w.actual, 0),
               CASE WHEN COALESCE(w.actual, 0) = 0 THEN 0.0
                    ELSE (COALESCE(w.lecture_count, 0) * 100.0 / w.actual)
               END
        FROM all_keys k
        LEFT JOIN template_counts t ON t.school_id = k.school_id AND t.grade_level = k.grade_level AND t.track = k.track AND t.subject_name = k.subject_name
        LEFT JOIN weekly_counts w ON w.school_id = k.school_id AND w.grade_level = k.grade_level AND w.track = k.track AND w.subject_name = k.subject_name
        JOIN schools s ON s.id = k.school_id
        WHERE 1=1",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(week_start.into());
    if let Some(id) = school_id {
        sql.push_str(" AND k.school_id = ?");
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND k.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(ComplianceMetrics {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                grade_level: row.get(2)?,
                track: row.get(3)?,
                subject_name: row.get(4)?,
                planned_periods: row.get(5)?,
                actual_periods: row.get(6)?,
                deviation: row.get(7)?,
                lecture_model_adherence_pct: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_deviation_score(conn: &Connection, school_id: i64) -> Result<DeviationScore, String> {
    let school = get_school(conn, school_id)?;
    let metrics = list_compliance_metrics(conn, Some(school_id), None)?;

    let mut subject_gaps: Vec<SubjectGap> = Vec::new();
    let mut total_deviation: f64 = 0.0;
    let mut count: i64 = 0;

    for m in &metrics {
        total_deviation += (m.deviation as f64).abs();
        count += 1;
        if m.actual_periods < m.planned_periods {
            subject_gaps.push(SubjectGap {
                subject_name: m.subject_name.clone(),
                grade_level: m.grade_level.clone(),
                track: m.track.clone(),
                planned: m.planned_periods,
                actual: m.actual_periods,
            });
        }
    }

    let overall_deviation_score = if count > 0 {
        total_deviation / count as f64
    } else {
        0.0
    };

    let week_start: String = conn
        .query_row("SELECT date('now', 'weekday 1', '-7 days')", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let mut faculty_overloads: Vec<FacultyOverload> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT u.display_name, s.name, COUNT(*)
                 FROM timetable_weekly_slots wts
                 JOIN schools s ON s.id = wts.school_id
                 LEFT JOIN users u ON u.id = wts.faculty_user_id
                 WHERE wts.week_start_date = ?1
                   AND wts.school_id = ?2
                   AND wts.faculty_user_id IS NOT NULL
                 GROUP BY wts.faculty_user_id, wts.school_id
                 HAVING COUNT(*) > 24
                 LIMIT 100",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![&week_start, school_id], |row| {
                Ok(FacultyOverload {
                    faculty_name: row.get(0)?,
                    school_name: row.get(1)?,
                    period_count: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            faculty_overloads.push(r.map_err(|e| e.to_string())?);
        }
    }

    let mut underutilized_batches: Vec<UnderutilizedBatch> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "WITH planned AS (
                    SELECT school_id, grade_level, track, batch_pattern, COUNT(*) AS cnt
                    FROM timetable_slots
                    WHERE deleted_at IS NULL AND school_id = ?1
                    GROUP BY school_id, grade_level, track, batch_pattern
                ),
                actual AS (
                    SELECT school_id, grade_level, track, batch_pattern, COUNT(*) AS cnt
                    FROM timetable_weekly_slots
                    WHERE week_start_date = ?2 AND school_id = ?1
                    GROUP BY school_id, grade_level, track, batch_pattern
                )
                SELECT p.school_id, s.name, p.grade_level, p.track, p.batch_pattern,
                       COALESCE(a.cnt, 0) * 100 / p.cnt
                FROM planned p
                LEFT JOIN actual a ON a.school_id = p.school_id AND a.grade_level = p.grade_level AND a.track = p.track AND a.batch_pattern = p.batch_pattern
                JOIN schools s ON s.id = p.school_id
                WHERE COALESCE(a.cnt, 0) * 100 / p.cnt < 80
                LIMIT 100",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![school_id, &week_start], |row| {
                Ok(UnderutilizedBatch {
                    school_name: row.get(1)?,
                    grade_level: row.get(2)?,
                    track: row.get(3)?,
                    batch_pattern: row.get(4)?,
                    utilization_pct: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            underutilized_batches.push(r.map_err(|e| e.to_string())?);
        }
    }

    Ok(DeviationScore {
        school_id,
        school_name: school.name,
        overall_deviation_score,
        subject_gaps,
        faculty_overloads,
        underutilized_batches,
    })
}

pub fn list_substitution_records(
    conn: &Connection,
    school_id: Option<i64>,
    faculty_user_id: Option<i64>,
    week_start_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<SubstitutionRecord>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT ls.id, ls.session_date,
                ts.faculty_user_id, COALESCE(orig_fac.display_name, ''),
                ls.actual_faculty_user_id, COALESCE(sub_fac.display_name, ''),
                sub.name, ts.grade_level, ts.track, ts.batch_pattern, ls.status
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN subjects sub ON sub.id = ts.subject_id
         LEFT JOIN users orig_fac ON orig_fac.id = ts.faculty_user_id
         LEFT JOIN users sub_fac ON sub_fac.id = ls.actual_faculty_user_id
         WHERE ls.actual_faculty_user_id IS NOT NULL
           AND ts.faculty_user_id IS NOT NULL
           AND ls.actual_faculty_user_id != ts.faculty_user_id
           AND ls.session_date >= ?1
           AND ls.session_date < date(?1, '+7 days')",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(week_start_date.to_string().into());
    if let Some(id) = school_id {
        sql.push_str(" AND ls.school_id = ?");
        p.push(id.into());
    }
    if let Some(id) = faculty_user_id {
        sql.push_str(" AND (ts.faculty_user_id = ? OR ls.actual_faculty_user_id = ?)");
        p.push(id.into());
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ls.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY ls.session_date DESC, ls.id DESC LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(SubstitutionRecord {
                session_id: row.get(0)?,
                session_date: row.get(1)?,
                original_faculty_user_id: row.get(2)?,
                original_faculty_name: row.get(3)?,
                substitute_faculty_user_id: row.get(4).ok(),
                substitute_faculty_name: row.get(5)?,
                subject_name: row.get(6)?,
                grade_level: row.get(7)?,
                track: row.get(8)?,
                batch_pattern: row.get(9)?,
                status: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_pending_substitution_records(
    conn: &Connection,
    school_id: Option<i64>,
    week_start_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<SubstitutionRecord>, String> {
    const MAX_ROWS: i64 = 1000;
    // Empty scope → no results for scoped roles
    if let Some(ids) = scope_school_ids {
        if ids.is_empty() || ids == [-1] {
            return Ok(Vec::new());
        }
    }

    let mut sql = String::from(
        "SELECT ls.id, ls.session_date,
                ts.faculty_user_id, COALESCE(orig_fac.display_name, ''),
                ls.actual_faculty_user_id, COALESCE(sub_fac.display_name, ''),
                sub.name, ts.grade_level, ts.track, ts.batch_pattern, ls.status
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN subjects sub ON sub.id = ts.subject_id
         LEFT JOIN users orig_fac ON orig_fac.id = ts.faculty_user_id
         LEFT JOIN users sub_fac ON sub_fac.id = ls.actual_faculty_user_id
         WHERE ls.status = 'Needs Substitution'
           AND ls.session_date >= ?1
           AND ls.session_date < date(?1, '+7 days')",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(week_start_date.to_string().into());
    if let Some(id) = school_id {
        sql.push_str(" AND ls.school_id = ?");
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ls.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY ls.session_date DESC, ls.id DESC LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(SubstitutionRecord {
                session_id: row.get(0)?,
                session_date: row.get(1)?,
                original_faculty_user_id: row.get(2)?,
                original_faculty_name: row.get(3)?,
                substitute_faculty_user_id: row.get(4).ok(),
                substitute_faculty_name: row.get(5)?,
                subject_name: row.get(6)?,
                grade_level: row.get(7)?,
                track: row.get(8)?,
                batch_pattern: row.get(9)?,
                status: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_room_conflicts(
    conn: &Connection,
    school_id: i64,
    week_start_date: &str,
) -> Result<Vec<(WeeklyTimetableSlot, WeeklyTimetableSlot)>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut stmt = conn.prepare(
        "WITH conflicts AS (
            SELECT a.id AS a_id, b.id AS b_id
            FROM timetable_weekly_slots a
            JOIN timetable_weekly_slots b ON
                a.school_id = b.school_id
                AND a.week_start_date = b.week_start_date
                AND a.day_of_week = b.day_of_week
                AND a.room = b.room
                AND a.room != ''
                AND a.id < b.id
                AND (
                    (a.start_time != '' AND b.start_time != '' AND a.start_time < b.end_time AND a.end_time > b.start_time)
                    OR (a.start_time = '' AND b.start_time = '' AND a.period = b.period)
                )
            WHERE a.school_id = ?1 AND a.week_start_date = ?2
            LIMIT ?3
        )
        SELECT a_id, b_id FROM conflicts"
    ).map_err(|e| e.to_string())?;

    let pairs = stmt
        .query_map(params![school_id, week_start_date, MAX_ROWS], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if pairs.is_empty() {
        return Ok(vec![]);
    }

    let mut ids = std::collections::HashSet::new();
    for (a_id, b_id) in &pairs {
        ids.insert(*a_id);
        ids.insert(*b_id);
    }
    let id_vec: Vec<i64> = ids.into_iter().collect();
    let slot_map = get_weekly_timetable_slots_by_ids(conn, &id_vec)?;

    let mut result = Vec::new();
    for (a_id, b_id) in pairs {
        if let (Some(a), Some(b)) = (slot_map.get(&a_id), slot_map.get(&b_id)) {
            result.push((a.clone(), b.clone()));
        }
    }
    Ok(result)
}

// ── Phase 6: Substitution & Leave Engine ─────────────────────────────────────
