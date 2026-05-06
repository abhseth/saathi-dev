use crate::models::*;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn monday_of(date: &str) -> Result<String, String> {
    use chrono::{Datelike, NaiveDate};
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let days_back = d.weekday().num_days_from_monday();
    let mon = d - chrono::Duration::days(days_back as i64);
    Ok(mon.format("%Y-%m-%d").to_string())
}

fn weeks_back(n: i64) -> Vec<String> {
    use chrono::{Datelike, Local};
    let today = Local::now().date_naive();
    let days_back = today.weekday().num_days_from_monday();
    let mon = today - chrono::Duration::days(days_back as i64);
    let mut out = Vec::new();
    for i in 0..n {
        let w = mon - chrono::Duration::weeks(i);
        out.push(w.format("%Y-%m-%d").to_string());
    }
    out.reverse();
    out
}

// ── 1. Actionable Compliance Scorecard ──────────────────────────────────────

pub fn compliance_scorecard(
    conn: &Connection,
    school_id: Option<i64>,
    scope: Option<&[i64]>,
) -> Result<Vec<ActionableComplianceItem>, String> {
    let mut sql = String::from(
        "SELECT s.id, s.name, sc.grade_level, sc.track, sub.name AS subject_name,
                lm.days_per_week * lm.lectures_per_day AS planned_periods,
                COUNT(DISTINCT ts.id) AS actual_periods
         FROM school_class_plans sc
         JOIN schools s ON s.id = sc.school_id
         JOIN lecture_models lm ON lm.id = sc.lecture_model_id
         JOIN subjects sub ON (
             (sub.track = sc.track OR (sc.track = '' AND sub.track = 'Foundation'))
         )
         LEFT JOIN timetable_slots ts ON ts.school_id = sc.school_id
             AND ts.grade_level = sc.grade_level
             AND (ts.track = sc.track OR (sc.track = '' AND ts.track = ''))
             AND ts.subject_id = sub.id
             AND ts.deleted_at IS NULL
         WHERE 1=1",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    let school_id_ref = school_id.as_ref();
    if let Some(id) = school_id_ref {
        sql.push_str(" AND sc.school_id = ?");
        params_vec.push(id);
    }
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND sc.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(
        " GROUP BY s.id, s.name, sc.grade_level, sc.track, sub.name, lm.days_per_week, lm.lectures_per_day
          HAVING planned_periods > actual_periods
          ORDER BY (planned_periods - actual_periods) DESC LIMIT 100",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let school_id: i64 = row.get(0)?;
            let school_name: String = row.get(1)?;
            let grade_level: String = row.get(2)?;
            let track: String = row.get(3)?;
            let subject_name: String = row.get(4)?;
            let planned: i64 = row.get(5)?;
            let actual: i64 = row.get(6)?;
            let deviation = planned - actual;
            let severity = if deviation >= 5 {
                "critical"
            } else if deviation >= 2 {
                "warning"
            } else {
                "info"
            };
            let message = format!(
                "{grade_level} missing {deviation}/{planned} {subject_name} slots — Schedule now"
            );
            Ok(ActionableComplianceItem {
                severity: severity.to_string(),
                message,
                school_id,
                school_name,
                grade_level,
                track,
                subject_name,
                planned_periods: planned,
                actual_periods: actual,
                deviation,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 2. Control Tower Dashboard ───────────────────────────────────────────────

pub fn control_tower(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<ControlTowerCard>, String> {
    let mut sql = String::from(
        "SELECT s.id, s.name, COALESCE(r.name, '') AS region_name,
                COUNT(DISTINCT tws.id) AS filled_periods,
                COALESCE(tp.total_periods, 0) AS total_periods,
                COALESCE(alc.alert_count, 0) AS alert_count,
                COALESCE(att.attendance_pct, 0) AS attendance_pct,
                COALESCE(sub.active_substitutions, 0) AS active_substitutions
         FROM schools s
         LEFT JOIN regions r ON r.id = s.region_id
         LEFT JOIN timetable_weekly_slots tws ON tws.school_id = s.id
             AND tws.week_start_date = (SELECT MAX(week_start_date) FROM timetable_weekly_slots WHERE school_id = s.id)
         LEFT JOIN (
             SELECT scp.school_id, COALESCE(SUM(lm.days_per_week * lm.lectures_per_day), 0) AS total_periods
             FROM school_class_plans scp
             JOIN lecture_models lm ON lm.id = scp.lecture_model_id
             GROUP BY scp.school_id
         ) tp ON tp.school_id = s.id
         LEFT JOIN (
             SELECT school_id, COUNT(*) AS alert_count
             FROM tickets
             WHERE status != 'Closed'
             GROUP BY school_id
         ) alc ON alc.school_id = s.id
         LEFT JOIN (
             SELECT ls.school_id,
                    COUNT(CASE WHEN ar.status = 'Present' THEN 1 END) * 100.0 / NULLIF(COUNT(*), 0) AS attendance_pct
             FROM attendance_records ar
             JOIN lecture_sessions ls ON ls.id = ar.lecture_session_id
             GROUP BY ls.school_id
         ) att ON att.school_id = s.id
         LEFT JOIN (
             SELECT school_id, COUNT(*) AS active_substitutions
             FROM lecture_sessions
             WHERE actual_faculty_user_id IS NOT NULL AND status = 'Substituted'
             GROUP BY school_id
         ) sub ON sub.school_id = s.id
         WHERE s.is_dropped = 0",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY s.id, s.name, region_name ORDER BY s.name");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(ControlTowerCard {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                region_name: row.get(2)?,
                filled_periods: row.get(3)?,
                total_periods: row.get(4)?,
                alert_count: row.get(5)?,
                attendance_percent: row.get::<_, f64>(6)?.round() as i64,
                active_substitutions: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 3. Faculty Utilization Trend Lines ───────────────────────────────────────

pub fn faculty_utilization_trend(
    conn: &Connection,
    faculty_id: Option<i64>,
    weeks: i64,
    scope: Option<&[i64]>,
) -> Result<Vec<FacultyUtilizationTrend>, String> {
    let week_starts = weeks_back(weeks);

    // Single set-based query instead of O(F×W) round-trips.
    let week_placeholders = week_starts
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let mut sql = format!(
        "SELECT tws.faculty_user_id, u.display_name, tws.week_start_date, COUNT(*) AS period_count
         FROM timetable_weekly_slots tws
         JOIN users u ON u.id = tws.faculty_user_id
         WHERE tws.week_start_date IN ({week_placeholders})
           AND tws.faculty_user_id IS NOT NULL",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = week_starts
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    if let Some(fid) = &faculty_id {
        sql.push_str(" AND tws.faculty_user_id = ?");
        params_vec.push(fid);
    }

    sql.push_str(
        " GROUP BY tws.faculty_user_id, u.display_name, tws.week_start_date
                   ORDER BY u.display_name, tws.week_start_date",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut by_faculty: HashMap<i64, (String, Vec<FacultyUtilizationWeek>)> = HashMap::new();
    for r in rows {
        let (fid, fname, wk, count) = r.map_err(|e| e.to_string())?;
        by_faculty
            .entry(fid)
            .or_insert_with(|| (fname, Vec::new()))
            .1
            .push(FacultyUtilizationWeek {
                week_start_date: wk,
                period_count: count,
            });
    }

    // Ensure every requested week appears (fill missing with 0).
    let mut trends: Vec<FacultyUtilizationTrend> = Vec::new();
    for (fid, (fname, mut weeks)) in by_faculty {
        let mut week_map: HashMap<String, i64> = weeks
            .into_iter()
            .map(|w| (w.week_start_date, w.period_count))
            .collect();
        let mut full_weeks = Vec::new();
        for wk in &week_starts {
            full_weeks.push(FacultyUtilizationWeek {
                week_start_date: wk.clone(),
                period_count: week_map.remove(wk).unwrap_or(0),
            });
        }
        trends.push(FacultyUtilizationTrend {
            faculty_user_id: fid,
            faculty_name: fname,
            weeks: full_weeks,
        });
    }
    trends.sort_by(|a, b| a.faculty_name.cmp(&b.faculty_name));

    // Scope filter: one query to find faculty with slots in scoped schools.
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let scope_sql = format!(
                "SELECT DISTINCT faculty_user_id FROM timetable_weekly_slots
                 WHERE faculty_user_id IS NOT NULL AND school_id IN ({placeholders})"
            );
            let mut stmt = conn.prepare(&scope_sql).map_err(|e| e.to_string())?;
            let scoped_faculty: HashSet<i64> = stmt
                .query_map(rusqlite::params_from_iter(ids.iter()), |row| {
                    row.get::<_, i64>(0)
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<HashSet<_>, _>>()
                .map_err(|e| e.to_string())?;
            trends.retain(|t| scoped_faculty.contains(&t.faculty_user_id));
        }
    }

    Ok(trends)
}

// ── 4. Central Deviation Scoreboard ──────────────────────────────────────────

pub fn deviation_scoreboard(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<DeviationScoreboardRow>, String> {
    let mut sql = String::from(
        "SELECT s.id, s.name, COALESCE(r.name, '') AS region_name,
                sc.grade_level, sc.track, sub.name AS subject_name,
                lm.days_per_week * lm.lectures_per_day AS planned,
                COUNT(DISTINCT ts.id) AS actual
         FROM school_class_plans sc
         JOIN schools s ON s.id = sc.school_id
         LEFT JOIN regions r ON r.id = s.region_id
         JOIN lecture_models lm ON lm.id = sc.lecture_model_id
         JOIN subjects sub ON (sub.track = sc.track OR (sc.track = '' AND sub.track = 'Foundation'))
         LEFT JOIN timetable_slots ts ON ts.school_id = sc.school_id
             AND ts.grade_level = sc.grade_level
             AND (ts.track = sc.track OR (sc.track = '' AND ts.track = ''))
             AND ts.subject_id = sub.id
             AND ts.deleted_at IS NULL
         WHERE s.is_dropped = 0",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(
        " GROUP BY s.id, s.name, region_name, sc.grade_level, sc.track, sub.name, planned
          ORDER BY s.id, (planned - actual) DESC",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut map: HashMap<i64, (String, String, Vec<SubjectGap>, i64, i64)> = HashMap::new();

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let sid: i64 = row.get(0)?;
            let sname: String = row.get(1)?;
            let rname: String = row.get(2)?;
            let grade: String = row.get(3)?;
            let track: String = row.get(4)?;
            let subject: String = row.get(5)?;
            let planned: i64 = row.get(6)?;
            let actual: i64 = row.get(7)?;
            let deviation = (planned - actual).max(0);
            Ok((
                sid, sname, rname, grade, track, subject, planned, actual, deviation,
            ))
        })
        .map_err(|e| e.to_string())?;

    for r in rows {
        let (sid, sname, rname, grade, track, subject, planned, actual, deviation) =
            r.map_err(|e| e.to_string())?;
        let entry = map
            .entry(sid)
            .or_insert_with(|| (sname, rname, Vec::new(), 0, 0));
        entry.2.push(SubjectGap {
            subject_name: subject,
            grade_level: grade,
            track,
            planned,
            actual,
        });
        entry.3 += deviation;
        entry.4 += planned;
    }

    let mut out: Vec<DeviationScoreboardRow> = map
        .into_iter()
        .map(|(sid, (sname, rname, gaps, total_dev, total_plan))| {
            let score = if total_plan > 0 {
                (total_dev as f64 / total_plan as f64) * 100.0
            } else {
                0.0
            };
            let top_gaps = gaps.into_iter().take(3).collect();
            DeviationScoreboardRow {
                school_id: sid,
                school_name: sname,
                region_name: rname,
                overall_deviation_score: score,
                top_gaps,
            }
        })
        .collect();

    out.sort_by(|a, b| {
        b.overall_deviation_score
            .partial_cmp(&a.overall_deviation_score)
            .unwrap()
    });
    Ok(out)
}

// ── 5. Session-Type Adherence Breakdown ──────────────────────────────────────

pub fn session_type_breakdown(
    conn: &Connection,
    school_id: Option<i64>,
    scope: Option<&[i64]>,
) -> Result<Vec<SessionTypeBreakdown>, String> {
    let mut sql = String::from(
        "SELECT ts.session_type,
                COUNT(DISTINCT ts.id) AS planned,
                COUNT(DISTINCT ls.id) AS actual
         FROM timetable_slots ts
         LEFT JOIN lecture_sessions ls ON ls.timetable_slot_id = ts.id
             AND ls.status NOT IN ('Cancelled', 'Absent')
         WHERE ts.deleted_at IS NULL",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    let school_id_ref = school_id.as_ref();
    if let Some(id) = school_id_ref {
        sql.push_str(" AND ts.school_id = ?");
        params_vec.push(id);
    }
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ts.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY ts.session_type ORDER BY planned DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let session_type: String = row.get(0)?;
            let planned: i64 = row.get(1)?;
            let actual: i64 = row.get(2)?;
            let pct = if planned > 0 {
                (actual as f64 / planned as f64) * 100.0
            } else {
                100.0
            };
            Ok(SessionTypeBreakdown {
                session_type,
                planned_periods: planned,
                actual_periods: actual,
                adherence_pct: pct,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 6. Faculty Stability Score ───────────────────────────────────────────────

pub fn faculty_stability(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<FacultyStabilityRow>, String> {
    let mut sql = String::from(
        "SELECT u.id, u.display_name, s.name AS school_name,
                (SELECT COUNT(*) FROM lecture_sessions ls_sub
                 JOIN timetable_slots ts ON ts.id = ls_sub.timetable_slot_id
                 WHERE ts.faculty_user_id = u.id AND ts.deleted_at IS NULL
                   AND ls_sub.status = 'Substituted') AS sub_count,
                (SELECT COUNT(*) FROM lecture_sessions ls_can
                 JOIN timetable_slots ts ON ts.id = ls_can.timetable_slot_id
                 WHERE ts.faculty_user_id = u.id AND ts.deleted_at IS NULL
                   AND ls_can.status = 'Cancelled') AS cancel_count,
                (SELECT COUNT(*) FROM timetable_slots ts
                 WHERE ts.faculty_user_id = u.id AND ts.deleted_at IS NULL) AS planned_count,
                (SELECT COUNT(*) FROM lecture_sessions ls_act
                 JOIN timetable_slots ts ON ts.id = ls_act.timetable_slot_id
                 WHERE ts.faculty_user_id = u.id AND ts.deleted_at IS NULL
                   AND ls_act.status NOT IN ('Cancelled', 'Absent')) AS actual_count
         FROM users u
         JOIN faculty_assignments fa ON fa.faculty_user_id = u.id
         JOIN schools s ON s.id = fa.school_id
         WHERE u.role = 'faculty'",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND fa.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY u.id, u.display_name, s.name ORDER BY u.display_name LIMIT 200");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let planned: i64 = row.get(5)?;
            let actual: i64 = row.get(6)?;
            let sub: i64 = row.get(3)?;
            let can: i64 = row.get(4)?;
            let sub_rate = if planned > 0 {
                (sub as f64 / planned as f64) * 100.0
            } else {
                0.0
            };
            let cancel_rate = if planned > 0 {
                (can as f64 / planned as f64) * 100.0
            } else {
                0.0
            };
            Ok(FacultyStabilityRow {
                faculty_user_id: row.get(0)?,
                faculty_name: row.get(1)?,
                school_name: row.get(2)?,
                substitution_rate_pct: sub_rate,
                cancellation_rate_pct: cancel_rate,
                planned_vs_actual_variance: planned - actual,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 7. Subject Coverage Heatmap by Region ────────────────────────────────────

pub fn subject_coverage_heatmap(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<SubjectCoverageCell>, String> {
    let mut sql = String::from(
        "SELECT COALESCE(r.name, 'Unassigned') AS region_name, sub.name AS subject_name,
                COUNT(DISTINCT ts.id) AS actual,
                COUNT(DISTINCT scp.id) * lm.days_per_week * lm.lectures_per_day AS planned
         FROM school_class_plans scp
         JOIN schools s ON s.id = scp.school_id
         LEFT JOIN regions r ON r.id = s.region_id
         JOIN lecture_models lm ON lm.id = scp.lecture_model_id
         JOIN subjects sub ON (sub.track = scp.track OR (scp.track = '' AND sub.track = 'Foundation'))
         LEFT JOIN timetable_slots ts ON ts.school_id = scp.school_id
             AND ts.grade_level = scp.grade_level
             AND (ts.track = scp.track OR (scp.track = '' AND ts.track = ''))
             AND ts.subject_id = sub.id
             AND ts.deleted_at IS NULL
         WHERE s.is_dropped = 0",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY region_name, sub.name ORDER BY region_name, sub.name");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let planned: i64 = row.get(3)?;
            let actual: i64 = row.get(2)?;
            let pct = if planned > 0 {
                (actual as f64 / planned as f64) * 100.0
            } else {
                100.0
            };
            Ok(SubjectCoverageCell {
                region_name: row.get(0)?,
                subject_name: row.get(1)?,
                adherence_pct: pct,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 8. Week-over-Week Health & Compliance Trends ─────────────────────────────

pub fn health_trends(
    conn: &Connection,
    _weeks: i64,
    scope: Option<&[i64]>,
) -> Result<Vec<HealthTrendWeek>, String> {
    let week_starts = weeks_back(_weeks.max(2).min(52));

    // Build a dynamic weeks CTE so every requested week appears in output.
    let mut weeks_parts = vec!["SELECT ?".to_string()];
    for _ in 1..week_starts.len() {
        weeks_parts.push("UNION ALL SELECT ?".to_string());
    }
    let weeks_cte = weeks_parts.join(" ");

    let mut scope_filter = String::new();
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for wk in &week_starts {
        params_vec.push(wk as &dyn rusqlite::ToSql);
    }

    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            scope_filter = format!(" AND s.id IN ({placeholders})");
            for id in ids {
                params_vec.push(id);
            }
        }
    }

    let sql = format!(
        "WITH weeks(wk) AS ( {weeks_cte} ),
         school_planned AS (
             SELECT scp.school_id,
                    COALESCE(SUM(lm.days_per_week * lm.lectures_per_day), 0) AS planned
             FROM school_class_plans scp
             JOIN lecture_models lm ON lm.id = scp.lecture_model_id
             GROUP BY scp.school_id
         ),
         weekly_filled AS (
             SELECT school_id, week_start_date, COUNT(*) AS filled
             FROM timetable_weekly_slots
             WHERE week_start_date IN (SELECT wk FROM weeks)
             GROUP BY school_id, week_start_date
         )
         SELECT
             w.wk AS week_start_date,
             SUM(CASE WHEN sp.planned = 0 THEN 1 ELSE 0 END) AS red_no_plan,
             SUM(CASE WHEN wf.filled >= sp.planned AND sp.planned > 0 THEN 1 ELSE 0 END) AS green_count,
             SUM(CASE WHEN wf.filled >= sp.planned * 3 / 4 AND wf.filled < sp.planned AND sp.planned > 0 THEN 1 ELSE 0 END) AS amber_count,
             SUM(CASE WHEN wf.filled < sp.planned * 3 / 4 AND sp.planned > 0 THEN 1 ELSE 0 END) AS red_count,
             COALESCE(SUM(sp.planned), 0) AS total_planned,
             COALESCE(SUM(wf.filled), 0) AS total_filled
         FROM weeks w
         CROSS JOIN schools s
         LEFT JOIN school_planned sp ON sp.school_id = s.id
         LEFT JOIN weekly_filled wf ON wf.school_id = s.id AND wf.week_start_date = w.wk
         WHERE s.is_dropped = 0 {scope_filter}
         GROUP BY w.wk
         ORDER BY w.wk"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let total_planned: i64 = row.get(5)?;
            let total_filled: i64 = row.get(6)?;
            let net_pct = if total_planned > 0 {
                (total_filled as f64 / total_planned as f64) * 100.0
            } else {
                0.0
            };
            Ok(HealthTrendWeek {
                week_start_date: row.get(0)?,
                green_count: row.get(1)?,
                amber_count: row.get(2)?,
                red_count: row.get(3)?,
                network_adherence_pct: net_pct,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 9. Substitutions Trend Report ────────────────────────────────────────────

pub fn substitution_trends(
    conn: &Connection,
    _weeks: i64,
    scope: Option<&[i64]>,
) -> Result<Vec<SubstitutionTrendWeek>, String> {
    let week_starts = weeks_back(_weeks.max(2).min(52));
    let first_week = week_starts.first().map(|s| s.as_str()).unwrap_or("");
    let last_week = week_starts.last().map(|s| s.as_str()).unwrap_or("");

    // Scope filter reused across both queries.
    let mut scope_filter = String::new();
    let mut scope_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            scope_filter = format!(" AND s.id IN ({placeholders})");
            for id in ids {
                scope_params.push(id);
            }
        }
    }

    // Query 1: absences + short-staffed, grouped by week.
    let main_sql = format!(
        "SELECT
            date(ls.session_date, 'weekday 1', '-7 days') AS week_start,
            COUNT(DISTINCT CASE WHEN ls.status = 'Substituted' THEN ls.id END) AS absences,
            COUNT(DISTINCT CASE WHEN ls.actual_faculty_user_id IS NULL
                                 AND ls.status NOT IN ('Cancelled', 'Absent')
                                THEN ls.timetable_slot_id || '-' || ls.session_date END) AS short_staffed
         FROM lecture_sessions ls
         JOIN schools s ON s.id = ls.school_id
         WHERE ls.session_date >= ?1
           AND ls.session_date < date(?2, '+7 days')
           AND s.is_dropped = 0
           {scope_filter}
         GROUP BY week_start
         ORDER BY week_start"
    );
    let mut main_params: Vec<&dyn rusqlite::ToSql> = vec![&first_week, &last_week];
    main_params.extend(scope_params.iter().cloned());

    let mut stmt = conn.prepare(&main_sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(main_params.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut by_week: HashMap<String, (i64, i64)> = HashMap::new();
    for r in rows {
        let (wk, abs, short) = r.map_err(|e| e.to_string())?;
        by_week.insert(wk, (abs, short));
    }

    // Query 2: over-utilized substitutes per week.
    let over_sql = format!(
        "SELECT date(session_date, 'weekday 1', '-7 days') AS week_start, COUNT(*)
         FROM (
             SELECT ls.session_date
             FROM lecture_sessions ls
             JOIN schools s ON s.id = ls.school_id
             WHERE ls.status = 'Substituted'
               AND ls.session_date >= ?1
               AND ls.session_date < date(?2, '+7 days')
               AND s.is_dropped = 0
               {scope_filter}
             GROUP BY date(ls.session_date, 'weekday 1', '-7 days'), ls.actual_faculty_user_id
             HAVING COUNT(*) > 5
         )
         GROUP BY week_start
         ORDER BY week_start"
    );
    let mut over_params: Vec<&dyn rusqlite::ToSql> = vec![&first_week, &last_week];
    over_params.extend(scope_params.iter().cloned());

    let mut stmt2 = conn.prepare(&over_sql).map_err(|e| e.to_string())?;
    let over_rows = stmt2
        .query_map(rusqlite::params_from_iter(over_params.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut over_by_week: HashMap<String, i64> = HashMap::new();
    for r in over_rows {
        let (wk, cnt) = r.map_err(|e| e.to_string())?;
        over_by_week.insert(wk, cnt);
    }

    // Merge with week_starts to ensure every week appears.
    let mut out = Vec::new();
    for wk in &week_starts {
        let (absences, short_staffed) = by_week.get(wk).copied().unwrap_or((0, 0));
        let over_util = over_by_week.get(wk).copied().unwrap_or(0);
        out.push(SubstitutionTrendWeek {
            week_start_date: wk.clone(),
            faculty_absences: absences,
            short_staffed_periods: short_staffed,
            over_utilized_substitutes: over_util,
        });
    }
    Ok(out)
}

// ── 10. Region Heat Map ──────────────────────────────────────────────────────

pub fn region_heatmap(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<RegionHeatmapCell>, String> {
    let mut sql = String::from(
        "SELECT s.id, s.name, ts.day_of_week,
                COUNT(DISTINCT ts.id) - COUNT(DISTINCT ls.id) AS issue_count
         FROM schools s
         JOIN timetable_slots ts ON ts.school_id = s.id AND ts.deleted_at IS NULL
         LEFT JOIN lecture_sessions ls ON ls.timetable_slot_id = ts.id
             AND ls.status NOT IN ('Cancelled', 'Absent')
         WHERE s.is_dropped = 0",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY s.id, s.name, ts.day_of_week ORDER BY s.name, ts.day_of_week");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(RegionHeatmapCell {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                day_of_week: row.get(2)?,
                issue_count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 11. Room Conflict Radar ──────────────────────────────────────────────────

pub fn room_conflicts_radar(
    conn: &Connection,
    school_id: Option<i64>,
    week_start: &str,
    scope: Option<&[i64]>,
) -> Result<Vec<RoomConflictRadarCell>, String> {
    let mut sql = String::from(
        "SELECT tws.room, tws.day_of_week, tws.period,
                COUNT(*) AS conflict_count,
                GROUP_CONCAT(DISTINCT s.name) AS schools
         FROM timetable_weekly_slots tws
         JOIN schools s ON s.id = tws.school_id
         WHERE tws.room != '' AND tws.week_start_date = ?",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&week_start];
    let school_id_ref = school_id.as_ref();
    if let Some(id) = school_id_ref {
        sql.push_str(" AND tws.school_id = ?");
        params_vec.push(id);
    }
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND tws.school_id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(
        " GROUP BY tws.room, tws.day_of_week, tws.period
          HAVING conflict_count > 1
          ORDER BY tws.room, tws.day_of_week, tws.period",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let schools_str: String = row.get(4)?;
            Ok(RoomConflictRadarCell {
                room: row.get(0)?,
                day_of_week: row.get(1)?,
                period: row.get(2)?,
                conflict_count: row.get(3)?,
                schools: schools_str.split(',').map(|s| s.to_string()).collect(),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 12. Lecture-Model Adherence Comparison Chart ─────────────────────────────

pub fn adherence_comparison(
    conn: &Connection,
    scope: Option<&[i64]>,
) -> Result<Vec<AdherenceComparisonRow>, String> {
    let mut sql = String::from(
        "SELECT s.id, s.name,
                COUNT(DISTINCT ts.id) AS actual,
                (SELECT COALESCE(SUM(lm.days_per_week * lm.lectures_per_day), 0)
                 FROM school_class_plans scp
                 JOIN lecture_models lm ON lm.id = scp.lecture_model_id
                 WHERE scp.school_id = s.id) AS planned
         FROM schools s
         LEFT JOIN timetable_slots ts ON ts.school_id = s.id AND ts.deleted_at IS NULL
         WHERE s.is_dropped = 0",
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(" GROUP BY s.id, s.name ORDER BY s.name");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let planned: i64 = row.get(3)?;
            let actual: i64 = row.get(2)?;
            let pct = if planned > 0 {
                (actual as f64 / planned as f64) * 100.0
            } else {
                0.0
            };
            Ok(AdherenceComparisonRow {
                school_id: row.get(0)?,
                school_name: row.get(1)?,
                adherence_pct: pct,
                deviation: planned - actual,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── 13. Week-over-Week Diff Highlight ────────────────────────────────────────

pub fn week_diff(
    conn: &Connection,
    school_id: i64,
    week_a: &str,
    week_b: &str,
) -> Result<Vec<WeekDiffSlot>, String> {
    let sql = "SELECT tws.id, tws.school_id, tws.grade_level, tws.track, tws.batch_pattern,
                tws.day_of_week, tws.period, sub.name, u.display_name, tws.room, tws.session_type,
                tws.week_start_date
         FROM timetable_weekly_slots tws
         LEFT JOIN subjects sub ON sub.id = tws.subject_id
         LEFT JOIN users u ON u.id = tws.faculty_user_id
         WHERE tws.school_id = ? AND tws.week_start_date IN (?, ?)";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![school_id, week_a, week_b], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let slots: Vec<_> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut a_map: HashMap<String, (i64, Option<String>, String, String)> = HashMap::new();
    let mut b_map: HashMap<String, (i64, Option<String>, String, String)> = HashMap::new();

    for (id, _, grade, track, batch, day, period, subject, faculty, room, stype, week) in slots {
        let key = format!(
            "{}|{}|{}|{}|{}|{}",
            grade, track, batch, day, period, subject
        );
        if week == week_a {
            a_map.insert(key, (id, faculty, room, stype));
        } else {
            b_map.insert(key, (id, faculty, room, stype));
        }
    }

    let mut out = Vec::new();
    for (key, (id, faculty, room, stype)) in &a_map {
        if let Some((_, b_faculty, b_room, b_stype)) = b_map.get(key) {
            if b_faculty != faculty || b_room != room || b_stype != stype {
                let parts: Vec<&str> = key.split('|').collect();
                out.push(WeekDiffSlot {
                    id: *id,
                    school_id,
                    grade_level: parts[0].to_string(),
                    track: parts[1].to_string(),
                    batch_pattern: parts[2].to_string(),
                    day_of_week: parts[3].parse().unwrap_or(0),
                    period: parts[4].parse().unwrap_or(0),
                    subject_name: parts[5].to_string(),
                    faculty_display_name: faculty.clone(),
                    room: room.clone(),
                    session_type: stype.clone(),
                    change_type: "modified".to_string(),
                });
            }
        } else {
            let parts: Vec<&str> = key.split('|').collect();
            out.push(WeekDiffSlot {
                id: *id,
                school_id,
                grade_level: parts[0].to_string(),
                track: parts[1].to_string(),
                batch_pattern: parts[2].to_string(),
                day_of_week: parts[3].parse().unwrap_or(0),
                period: parts[4].parse().unwrap_or(0),
                subject_name: parts[5].to_string(),
                faculty_display_name: faculty.clone(),
                room: room.clone(),
                session_type: stype.clone(),
                change_type: "removed".to_string(),
            });
        }
    }
    for (key, (id, faculty, room, stype)) in &b_map {
        if !a_map.contains_key(key) {
            let parts: Vec<&str> = key.split('|').collect();
            out.push(WeekDiffSlot {
                id: *id,
                school_id,
                grade_level: parts[0].to_string(),
                track: parts[1].to_string(),
                batch_pattern: parts[2].to_string(),
                day_of_week: parts[3].parse().unwrap_or(0),
                period: parts[4].parse().unwrap_or(0),
                subject_name: parts[5].to_string(),
                faculty_display_name: faculty.clone(),
                room: room.clone(),
                session_type: stype.clone(),
                change_type: "added".to_string(),
            });
        }
    }

    out.sort_by_key(|s| (s.day_of_week, s.period, s.grade_level.clone()));
    Ok(out)
}

// ── 14. Compliance Analytics Pivot Toggle ────────────────────────────────────

pub fn compliance_pivot(
    conn: &Connection,
    pivot: &str, // "subject" | "school" | "region"
    scope: Option<&[i64]>,
) -> Result<Vec<CompliancePivotRow>, String> {
    let dim = match pivot {
        "school" => "s.name",
        "region" => "COALESCE(r.name, 'Unassigned')",
        _ => "sub.name",
    };

    let mut sql = format!(
        "SELECT {},
                SUM(lm.days_per_week * lm.lectures_per_day) AS planned,
                COUNT(DISTINCT ts.id) AS actual
         FROM school_class_plans sc
         JOIN schools s ON s.id = sc.school_id
         LEFT JOIN regions r ON r.id = s.region_id
         JOIN lecture_models lm ON lm.id = sc.lecture_model_id
         JOIN subjects sub ON (sub.track = sc.track OR (sc.track = '' AND sub.track = 'Foundation'))
         LEFT JOIN timetable_slots ts ON ts.school_id = sc.school_id
             AND ts.grade_level = sc.grade_level
             AND (ts.track = sc.track OR (sc.track = '' AND ts.track = ''))
             AND ts.subject_id = sub.id
             AND ts.deleted_at IS NULL
         WHERE s.is_dropped = 0",
        dim
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND s.id IN ({placeholders})"));
            for id in ids {
                params_vec.push(id);
            }
        }
    }
    sql.push_str(&format!(
        " GROUP BY {} ORDER BY planned DESC LIMIT 500",
        dim
    ));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let planned: i64 = row.get(1)?;
            let actual: i64 = row.get(2)?;
            let deviation = planned - actual;
            let pct = if planned > 0 {
                (actual as f64 / planned as f64) * 100.0
            } else {
                100.0
            };
            Ok(CompliancePivotRow {
                dimension_value: row.get(0)?,
                planned_periods: planned,
                actual_periods: actual,
                deviation,
                adherence_pct: pct,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
