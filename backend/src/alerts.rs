use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::models::Alert;
use crate::policies::{get_policy_value, get_policy_value_as_i64};

static ALERT_CACHE: LazyLock<Mutex<HashMap<String, (Vec<Alert>, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn make_alert(
    severity: &str,
    category: &str,
    message: &str,
    school_id: Option<i64>,
    school_name: Option<&str>,
) -> Alert {
    Alert {
        id: String::new(),
        severity: severity.to_string(),
        category: category.to_string(),
        message: message.to_string(),
        school_id,
        school_name: school_name.map(|s| s.to_string()),
        grade_level: None,
        subject_name: None,
        faculty_user_id: None,
        faculty_name: None,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

fn current_week_start(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT date('now', 'weekday 1', '-7 days')", [], |row| {
        row.get(0)
    })
    .map_err(|e| e.to_string())
}

pub fn check_unfilled_periods(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let week_start = current_week_start(conn)?;
    let mut alerts = Vec::new();
    let mut sql = String::from(
        "SELECT wts.school_id, s.name, wts.grade_level, wts.track, wts.batch_pattern, wts.day_of_week, wts.period
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         WHERE wts.week_start_date = ?1
           AND wts.faculty_user_id IS NULL"
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&week_start];
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND wts.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" ORDER BY wts.school_id, wts.day_of_week, wts.period LIMIT 100");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (sid, sname, grade, track, batch, dow, period) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "critical",
            "unfilled_period",
            &format!(
                "Unfilled period: {} {} {} Period {} (day {})",
                grade,
                if track.is_empty() {
                    "Foundation"
                } else {
                    &track
                },
                batch,
                period,
                dow + 1
            ),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn check_double_bookings(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let week_start = current_week_start(conn)?;
    let mut alerts = Vec::new();
    let mut sql = String::from(
        "SELECT u.display_name, wts.day_of_week, wts.period, COUNT(DISTINCT wts.id), wts.school_id, s.name
         FROM timetable_weekly_slots wts
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         JOIN schools s ON s.id = wts.school_id
         WHERE wts.week_start_date = ?1
           AND wts.faculty_user_id IS NOT NULL"
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&week_start];
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND wts.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY wts.faculty_user_id, wts.day_of_week, wts.period, wts.school_id HAVING COUNT(DISTINCT wts.id) > 1 LIMIT 100");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (name, dow, period, count, sid, sname) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "critical",
            "double_booking",
            &format!(
                "Faculty {} double-booked on day {} period {} ({} slots)",
                name,
                dow + 1,
                period,
                count
            ),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn check_faculty_overload(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let week_start = current_week_start(conn)?;
    let mut alerts = Vec::new();
    let mut sql = String::from(
        "SELECT u.display_name, s.name, wts.school_id, COUNT(*)
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         WHERE wts.week_start_date = ?1
           AND wts.faculty_user_id IS NOT NULL",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&week_start];
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND wts.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    let threshold = get_policy_value_as_i64(conn, "max_periods_per_faculty", 24)?;
    sql.push_str(&format!(
        " GROUP BY wts.faculty_user_id, wts.school_id HAVING COUNT(*) > {threshold} LIMIT 100"
    ));
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (name, sname, sid, count) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "warning",
            "faculty_overload",
            &format!(
                "Faculty {} overloaded with {} periods at {}",
                name, count, sname
            ),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn check_core_subject_gaps(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let mut alerts = Vec::new();
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    let school_subquery_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            for id in ids {
                params_vec.push(id);
            }
            format!(" AND school_id IN ({placeholders})")
        }
        _ => String::new(),
    };
    let sql = format!(
        "SELECT ts.school_id, s.name, ts.grade_level, ts.track, sub.name
         FROM (
             SELECT DISTINCT school_id, grade_level, track FROM timetable_slots WHERE deleted_at IS NULL {school_subquery_filter}
         ) ts_combo
         CROSS JOIN subjects sub
         LEFT JOIN timetable_slots ts ON ts.school_id = ts_combo.school_id
             AND ts.grade_level = ts_combo.grade_level
             AND ts.track = ts_combo.track
             AND ts.subject_id = sub.id
             AND ts.deleted_at IS NULL
         JOIN schools s ON s.id = ts_combo.school_id
         WHERE sub.is_default = 1
           AND ts.id IS NULL
         LIMIT 100"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (sid, sname, grade, track, subject) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "warning",
            "core_subject_gap",
            &format!(
                "Core subject gap: {} missing for {} {} at {}",
                subject,
                grade,
                if track.is_empty() {
                    "Foundation"
                } else {
                    &track
                },
                sname
            ),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn check_timetable_not_published(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let mut alerts = Vec::new();
    let next_week: String = conn
        .query_row("SELECT date('now', 'weekday 1')", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT s.id, s.name
         FROM schools s
         WHERE s.is_dropped = 0
           AND NOT EXISTS (
               SELECT 1 FROM timetable_weekly_slots wts
               WHERE wts.school_id = s.id AND wts.week_start_date = ?1
           )
           AND EXISTS (
               SELECT 1 FROM timetable_slots ts
               WHERE ts.school_id = s.id AND ts.deleted_at IS NULL
           )",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&next_week];
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" LIMIT 100");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (sid, sname) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "warning",
            "timetable_not_published",
            &format!(
                "Timetable not published for next week (starting {})",
                next_week
            ),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn check_attendance_not_marked(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let mut alerts = Vec::new();
    let today: String = conn
        .query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT ls.school_id, s.name, ls.id, ls.session_date
         FROM lecture_sessions ls
         JOIN schools s ON s.id = ls.school_id
         WHERE ls.session_date = ?1
           AND ls.status != 'Cancelled'
           AND time('now', 'localtime') > ?2
           AND NOT EXISTS (
               SELECT 1 FROM attendance_records ar
               WHERE ar.lecture_session_id = ls.id
           )",
    );
    let deadline = get_policy_value(conn, "attendance_marking_deadline", "11:00")?;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&today, &deadline];
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ls.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" LIMIT 100");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (sid, sname, _session_id, _date) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "info",
            "attendance_not_marked",
            "Attendance not marked for today's sessions",
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}

pub fn get_all_alerts(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Alert>, String> {
    let cache_key = match scope_school_ids {
        Some(ids) if !ids.is_empty() => ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(","),
        _ => "all".to_string(),
    };
    {
        let cache = ALERT_CACHE.lock().map_err(|e| e.to_string())?;
        if let Some((alerts, instant)) = cache.get(&cache_key) {
            if instant.elapsed() < Duration::from_secs(30) {
                return Ok(alerts.clone());
            }
        }
    }

    let mut alerts = Vec::new();
    if let Ok(mut a) = check_unfilled_periods(conn, scope_school_ids) {
        alerts.append(&mut a);
    }
    if let Ok(mut a) = check_double_bookings(conn, scope_school_ids) {
        alerts.append(&mut a);
    }
    if let Ok(mut a) = check_faculty_overload(conn, scope_school_ids) {
        alerts.append(&mut a);
    }
    if let Ok(mut a) = check_core_subject_gaps(conn, scope_school_ids) {
        alerts.append(&mut a);
    }
    if let Ok(mut a) = check_timetable_not_published(conn, scope_school_ids) {
        alerts.append(&mut a);
    }
    if let Ok(mut a) = check_attendance_not_marked(conn, scope_school_ids) {
        alerts.append(&mut a);
    }

    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            alerts.retain(|a| a.school_id.map(|sid| ids.contains(&sid)).unwrap_or(false));
        }
    }

    {
        let mut cache = ALERT_CACHE.lock().map_err(|e| e.to_string())?;
        cache.insert(cache_key, (alerts.clone(), Instant::now()));
    }

    Ok(alerts)
}

pub fn get_faculty_specific_alerts(
    conn: &Connection,
    faculty_user_id: i64,
) -> Result<Vec<Alert>, String> {
    let mut alerts = Vec::new();
    let week_start: String = conn
        .query_row("SELECT date('now', 'weekday 1', '-7 days')", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;
    let threshold = get_policy_value_as_i64(conn, "max_periods_per_faculty", 24)?;
    let mut stmt = conn
        .prepare(&format!(
            "SELECT s.id, s.name, COUNT(*)
             FROM timetable_weekly_slots wts
             JOIN schools s ON s.id = wts.school_id
             WHERE wts.faculty_user_id = ?1
               AND wts.week_start_date = ?2
             GROUP BY wts.school_id
             HAVING COUNT(*) > {threshold}",
        ))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![faculty_user_id, &week_start], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (sid, sname, count) = r.map_err(|e| e.to_string())?;
        alerts.push(make_alert(
            "warning",
            "faculty_overload",
            &format!("You have {} periods this week at {}", count, sname),
            Some(sid),
            Some(&sname),
        ));
    }
    Ok(alerts)
}
