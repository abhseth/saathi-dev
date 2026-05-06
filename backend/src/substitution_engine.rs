use crate::models::SubstituteCandidate;
use rusqlite::{params, Connection};

/// Rank available substitute faculty for a given lecture session.
/// Scoring: subject_match (40) + free_period (30) + same_school (20) - workload_penalty (max 20)
pub fn rank_substitute_candidates(
    conn: &Connection,
    session_id: i64,
) -> Result<Vec<SubstituteCandidate>, String> {
    // Get session details
    let (school_id, subject_id, session_date, day_of_week, period): (i64, i64, String, i64, i64) = conn.query_row(
        "SELECT COALESCE(ls.school_id, ts.school_id), ts.subject_id, ls.session_date, ts.day_of_week, ts.period
         FROM lecture_sessions ls
         LEFT JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
         WHERE ls.id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).map_err(|e| e.to_string())?;

    // Get subject name for cross-track matching
    let subject_name: String = conn
        .query_row(
            "SELECT name FROM subjects WHERE id = ?1",
            params![subject_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    // Find all faculty who teach the same subject (exact id or same name cross-track)
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT fa.faculty_user_id, u.display_name, fa.school_id, fa.subject_id
         FROM faculty_assignments fa
         JOIN users u ON u.id = fa.faculty_user_id
         JOIN subjects sub ON sub.id = fa.subject_id
         WHERE (fa.subject_id = ?1 OR sub.name = ?2) AND u.role = 'faculty'",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![subject_id, subject_name], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut candidates: Vec<SubstituteCandidate> = Vec::new();
    for r in rows {
        let (faculty_user_id, faculty_name, faculty_school_id, matched_subject_id) =
            r.map_err(|e| e.to_string())?;

        // Skip if this faculty is already assigned to this session
        let is_already_assigned: bool = conn
            .query_row(
                "SELECT 1 FROM lecture_sessions WHERE id = ?1 AND actual_faculty_user_id = ?2",
                params![session_id, faculty_user_id],
                |_row| Ok(true),
            )
            .unwrap_or(false);
        if is_already_assigned {
            continue;
        }

        // Check if faculty is free during this period on this day
        let has_conflict: bool = conn
            .query_row(
                "SELECT 1 FROM lecture_sessions ls
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE ls.session_date = ?1
               AND ts.day_of_week = ?2
               AND ts.period = ?3
               AND ls.status != 'Cancelled'
               AND ls.actual_faculty_user_id = ?4
               AND ls.id != ?5
             LIMIT 1",
                params![
                    &session_date,
                    day_of_week,
                    period,
                    faculty_user_id,
                    session_id
                ],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        // Workload: count how many substitutions this faculty has done in the last 30 days
        let sub_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lecture_sessions ls
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE ls.actual_faculty_user_id = ?1
               AND ls.actual_faculty_user_id != ts.faculty_user_id
               AND ls.session_date >= date('now', '-30 days')",
                params![faculty_user_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let exact_match = matched_subject_id == subject_id;
        let subject_match = true; // matched by id or by cross-track name
        let free_period = !has_conflict;
        let same_school = faculty_school_id == school_id;
        let workload_score = sub_count;

        let mut overall_score = 0i64;
        if exact_match {
            overall_score += 40;
        } else {
            overall_score += 32;
        } // cross-track: 80% affinity
        if free_period {
            overall_score += 30;
        }
        if same_school {
            overall_score += 20;
        }
        overall_score -= workload_score * 5;
        if overall_score < 0 {
            overall_score = 0;
        }

        candidates.push(SubstituteCandidate {
            faculty_user_id,
            faculty_name,
            subject_match,
            free_period,
            same_school,
            workload_score,
            overall_score,
        });
    }

    // Also consider faculty who don't teach this subject but are free (lower score)
    let mut other_stmt = conn
        .prepare(
            "SELECT u.id, u.display_name, COALESCE(us.school_id, 0)
         FROM users u
         LEFT JOIN user_schools us ON us.user_id = u.id
         WHERE u.role = 'faculty'
           AND u.id NOT IN (
               SELECT fa.faculty_user_id
               FROM faculty_assignments fa
               JOIN subjects sub ON sub.id = fa.subject_id
               WHERE fa.subject_id = ?1 OR sub.name = ?2
           )",
        )
        .map_err(|e| e.to_string())?;

    let other_rows = other_stmt
        .query_map(params![subject_id, subject_name], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for r in other_rows {
        let (faculty_user_id, faculty_name, faculty_school_id) = r.map_err(|e| e.to_string())?;

        let is_already_assigned: bool = conn
            .query_row(
                "SELECT 1 FROM lecture_sessions WHERE id = ?1 AND actual_faculty_user_id = ?2",
                params![session_id, faculty_user_id],
                |_row| Ok(true),
            )
            .unwrap_or(false);
        if is_already_assigned {
            continue;
        }

        let has_conflict: bool = conn
            .query_row(
                "SELECT 1 FROM lecture_sessions ls
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE ls.session_date = ?1
               AND ts.day_of_week = ?2
               AND ts.period = ?3
               AND ls.status != 'Cancelled'
               AND ls.actual_faculty_user_id = ?4
               AND ls.id != ?5
             LIMIT 1",
                params![
                    &session_date,
                    day_of_week,
                    period,
                    faculty_user_id,
                    session_id
                ],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        let sub_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lecture_sessions ls
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE ls.actual_faculty_user_id = ?1
               AND ls.actual_faculty_user_id != ts.faculty_user_id
               AND ls.session_date >= date('now', '-30 days')",
                params![faculty_user_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if has_conflict {
            continue; // skip if busy
        }

        let same_school = faculty_school_id == school_id;
        let mut overall_score = 10i64;
        if same_school {
            overall_score += 10;
        }
        overall_score -= sub_count * 5;
        if overall_score < 0 {
            overall_score = 0;
        }

        candidates.push(SubstituteCandidate {
            faculty_user_id,
            faculty_name,
            subject_match: false,
            free_period: true,
            same_school,
            workload_score: sub_count,
            overall_score,
        });
    }

    candidates.sort_by(|a, b| b.overall_score.cmp(&a.overall_score));
    Ok(candidates)
}

/// Validate that a swap between two slots has no conflicts.
/// If `expected_a_faculty` or `expected_b_faculty` are provided, also verifies
/// that the slots currently belong to those faculty members.
pub fn validate_swap(
    conn: &Connection,
    slot_a_id: i64,
    slot_b_id: i64,
    expected_a_faculty: Option<i64>,
    expected_b_faculty: Option<i64>,
) -> Result<(), String> {
    let (a_faculty, a_day, a_period, a_school): (Option<i64>, i64, i64, i64) = conn.query_row(
        "SELECT faculty_user_id, day_of_week, period, school_id FROM timetable_slots WHERE id = ?1",
        params![slot_a_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;

    let (b_faculty, b_day, b_period, b_school): (Option<i64>, i64, i64, i64) = conn.query_row(
        "SELECT faculty_user_id, day_of_week, period, school_id FROM timetable_slots WHERE id = ?1",
        params![slot_b_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).map_err(|e| e.to_string())?;

    if a_faculty.is_none() || b_faculty.is_none() {
        return Err("Both slots must have assigned faculty".to_string());
    }

    if let Some(expected) = expected_a_faculty {
        if a_faculty != Some(expected) {
            return Err("Slot A is no longer assigned to the expected faculty".to_string());
        }
    }

    if let Some(expected) = expected_b_faculty {
        if b_faculty != Some(expected) {
            return Err("Slot B is no longer assigned to the expected faculty".to_string());
        }
    }

    if a_school != b_school {
        return Err("Swap must be within the same school".to_string());
    }

    // Check if faculty A is free during slot B's time
    if a_day != b_day || a_period != b_period {
        let conflict_a: bool = conn
            .query_row(
                "SELECT 1 FROM timetable_slots
             WHERE school_id = ?1 AND day_of_week = ?2 AND period = ?3
               AND faculty_user_id = ?4 AND id != ?5
             LIMIT 1",
                params![a_school, b_day, b_period, a_faculty.unwrap(), slot_a_id],
                |_row| Ok(true),
            )
            .unwrap_or(false);
        if conflict_a {
            return Err("Requester faculty has a conflict with recipient's slot time".to_string());
        }
    }

    // Check if faculty B is free during slot A's time
    if a_day != b_day || a_period != b_period {
        let conflict_b: bool = conn
            .query_row(
                "SELECT 1 FROM timetable_slots
             WHERE school_id = ?1 AND day_of_week = ?2 AND period = ?3
               AND faculty_user_id = ?4 AND id != ?5
             LIMIT 1",
                params![a_school, a_day, a_period, b_faculty.unwrap(), slot_b_id],
                |_row| Ok(true),
            )
            .unwrap_or(false);
        if conflict_b {
            return Err("Recipient faculty has a conflict with requester's slot time".to_string());
        }
    }

    Ok(())
}
