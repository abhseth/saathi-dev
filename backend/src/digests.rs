use rusqlite::Connection;

use crate::models::*;

pub fn generate_intervention_digest(conn: &Connection) -> Result<InterventionDigest, String> {
    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Top 5 schools by deviation score (computed from template vs weekly slot counts)
    let mut top_schools = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name,
                COALESCE(ABS(template_counts.planned - weekly_counts.actual) * 100.0 / NULLIF(template_counts.planned, 0), 0.0) as score
         FROM schools s
         LEFT JOIN (
             SELECT school_id, COUNT(*) as planned FROM timetable_slots WHERE deleted_at IS NULL GROUP BY school_id
         ) template_counts ON template_counts.school_id = s.id
         LEFT JOIN (
             SELECT school_id, COUNT(*) as actual FROM timetable_weekly_slots WHERE week_start_date = date('now', 'weekday 1', '-7 days') GROUP BY school_id
         ) weekly_counts ON weekly_counts.school_id = s.id
         WHERE s.dropped_at = '' OR s.dropped_at IS NULL
         ORDER BY score DESC
         LIMIT 5"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SchoolDeviationBrief {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                deviation_score: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        top_schools.push(r.map_err(|e| e.to_string())?);
    }

    // SLA breaches
    let mut sla_breaches = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, title, school_name,
                CAST((julianday('now', 'localtime') - julianday(sla_due_at)) * 24 AS INTEGER) as hours_overdue
         FROM tickets
         WHERE status != 'Closed' AND status != 'Resolved'
           AND sla_due_at < datetime('now', 'localtime')
         ORDER BY hours_overdue DESC
         LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SlaBreachBrief {
                ticket_id: row.get(0)?,
                title: row.get(1)?,
                school_name: row.get(2)?,
                hours_overdue: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        sla_breaches.push(r.map_err(|e| e.to_string())?);
    }

    // Low attendance regions
    let mut low_attendance = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT r.name as region_name, AVG(ar.present_count * 100.0 / NULLIF(ar.total_students, 0)) as avg_pct
         FROM attendance_records ar
         JOIN schools s ON s.id = ar.school_id
         JOIN regions r ON r.id = s.region_id
         WHERE ar.session_date >= date('now', '-7 days')
         GROUP BY r.name
         HAVING avg_pct < 80.0
         ORDER BY avg_pct ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LowAttendanceRegion {
                region_name: row.get(0)?,
                avg_attendance_pct: row.get(1).unwrap_or(0.0),
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        low_attendance.push(r.map_err(|e| e.to_string())?);
    }

    Ok(InterventionDigest {
        generated_at,
        top_schools_by_deviation: top_schools,
        sla_breaches,
        low_attendance_regions: low_attendance,
    })
}

pub fn generate_sip_brief(conn: &Connection) -> Result<SipBrief, String> {
    let generated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Red/Amber flips from weekly_health_snapshots
    let mut status_flips = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT h1.school_id, s.name, h1.status as prev, h2.status as curr
         FROM weekly_health_snapshots h1
         JOIN weekly_health_snapshots h2 ON h1.school_id = h2.school_id
         JOIN schools s ON s.id = h1.school_id
         WHERE h1.week_start_date = date('now', '-14 days', 'weekday 1')
           AND h2.week_start_date = date('now', '-7 days', 'weekday 1')
           AND h1.status != h2.status
         ORDER BY h1.school_id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(StatusFlip {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                previous_status: row.get(2)?,
                current_status: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        status_flips.push(r.map_err(|e| e.to_string())?);
    }

    // >10% deviation subjects
    let mut high_deviation = Vec::new();
    let week_start: String = conn
        .query_row("SELECT date('now', 'weekday 1', '-7 days')", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT k.school_id, s.name, k.subject_name,
                ABS(COALESCE(t.planned, 0) - COALESCE(w.actual, 0)) * 100.0 / NULLIF(t.planned, 0) as dev_pct
         FROM (
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
         ) k
         LEFT JOIN (
             SELECT ts.school_id, ts.grade_level, ts.track, sub.name AS subject_name, COUNT(*) AS planned
             FROM timetable_slots ts
             JOIN subjects sub ON sub.id = ts.subject_id
             WHERE ts.deleted_at IS NULL
             GROUP BY ts.school_id, ts.grade_level, ts.track, sub.name
         ) t ON t.school_id = k.school_id AND t.grade_level = k.grade_level AND t.track = k.track AND t.subject_name = k.subject_name
         LEFT JOIN (
             SELECT wts.school_id, wts.grade_level, wts.track, sub.name AS subject_name, COUNT(*) AS actual
             FROM timetable_weekly_slots wts
             JOIN subjects sub ON sub.id = wts.subject_id
             WHERE wts.week_start_date = ?1
             GROUP BY wts.school_id, wts.grade_level, wts.track, sub.name
         ) w ON w.school_id = k.school_id AND w.grade_level = k.grade_level AND w.track = k.track AND w.subject_name = k.subject_name
         JOIN schools s ON s.id = k.school_id
         WHERE dev_pct > 10.0
         ORDER BY dev_pct DESC
         LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([&week_start], |row| {
            Ok(SubjectDeviation {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                subject_name: row.get(2)?,
                deviation_pct: row.get(3).unwrap_or(0.0),
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        high_deviation.push(r.map_err(|e| e.to_string())?);
    }

    // >2 substitution faculty
    let mut high_sub_faculty = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT u.display_name, COUNT(*) as cnt
         FROM lecture_sessions ls
         JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         JOIN users u ON u.id = ls.actual_faculty_user_id
         WHERE ls.actual_faculty_user_id IS NOT NULL
           AND ts.faculty_user_id IS NOT NULL
           AND ls.actual_faculty_user_id != ts.faculty_user_id
           AND ls.session_date >= date('now', '-7 days')
         GROUP BY u.display_name
         HAVING cnt > 2
         ORDER BY cnt DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(FacultySubstitutionCount {
                faculty_name: row.get(0)?,
                substitution_count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        high_sub_faculty.push(r.map_err(|e| e.to_string())?);
    }

    // Stale tickets (>14 days open)
    let mut stale = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, title, CAST(julianday('now', 'localtime') - julianday(created_at) AS INTEGER) as days
         FROM tickets
         WHERE status != 'Closed' AND status != 'Resolved'
           AND created_at < date('now', '-14 days')
         ORDER BY days DESC
         LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(StaleTicket {
                ticket_id: row.get(0)?,
                title: row.get(1)?,
                days_open: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        stale.push(r.map_err(|e| e.to_string())?);
    }

    Ok(SipBrief {
        generated_at,
        status_flips,
        high_deviation_subjects: high_deviation,
        high_substitution_faculty: high_sub_faculty,
        stale_tickets: stale,
    })
}
