use rusqlite::{params, Connection};

use crate::models::*;

pub fn bulk_assign_users(
    conn: &Connection,
    input: &BulkAssignUsersInput,
) -> Result<BulkOperationLog, String> {
    let mut assigned = 0;
    for user_id in &input.user_ids {
        for school_id in &input.school_ids {
            conn.execute(
                "INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)",
                params![user_id, school_id],
            )
            .map_err(|e| e.to_string())
            .map_err(|e| e.to_string())?;
            assigned += 1;
        }
        // Update role if needed
        conn.execute(
            "UPDATE users SET role = ?1 WHERE id = ?2 AND role != 'admin'",
            params![input.role, user_id],
        )
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;
    }
    Ok(log_bulk_op(
        conn,
        "assign-users",
        "completed",
        &format!("{{\"assigned\":{assigned}}}"),
    )?)
}

pub fn bulk_import_subjects(
    conn: &Connection,
    input: &BulkImportSubjectsInput,
) -> Result<BulkOperationLog, String> {
    let mut imported = 0;
    for line in input.csv_data.lines().skip(1) {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            let name = cols[0].trim();
            let track = cols[1].trim();
            if !name.is_empty() {
                conn.execute(
                    "INSERT OR IGNORE INTO subjects (name, track) VALUES (?1, ?2)",
                    params![name, track],
                )
                .map_err(|e| e.to_string())
                .map_err(|e| e.to_string())?;
                imported += 1;
            }
        }
    }
    Ok(log_bulk_op(
        conn,
        "import-subjects",
        "completed",
        &format!("{{\"imported\":{imported}}}"),
    )?)
}

pub fn bulk_publish_timetables(
    conn: &Connection,
    input: &BulkPublishTimetablesInput,
) -> Result<BulkOperationLog, String> {
    let mut published = 0;
    let schools: Vec<i64> = if input.school_ids.is_empty() {
        if let Some(rid) = input.region_id {
            let mut stmt = conn.prepare("SELECT id FROM schools WHERE region_id = ?1 AND (dropped_at = '' OR dropped_at IS NULL)").map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![rid], |row| row.get::<_, i64>(0))
                .map_err(|e| e.to_string())
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        } else {
            vec![]
        }
    } else {
        input.school_ids.clone()
    };

    for school_id in schools {
        let mut stmt = conn.prepare(
            "SELECT grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, room, session_type
             FROM timetable_slots
             WHERE school_id = ?1 AND deleted_at IS NULL"
        ).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![school_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                ))
            })
            .map_err(|e| e.to_string())
            .map_err(|e| e.to_string())?;

        for r in rows {
            let (gl, tr, bp, dow, period, sub_id, fac_id, st, et, room, stype) =
                r.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT OR REPLACE INTO timetable_weekly_slots
                 (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, week_start_date, room, session_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![school_id, gl, tr, bp, dow, period, sub_id, fac_id, st, et, input.week_start_date, room, stype],
            ).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
            published += 1;
        }
    }
    Ok(log_bulk_op(
        conn,
        "publish-timetables",
        "completed",
        &format!("{{\"published\":{published}}}"),
    )?)
}

pub fn reassign_faculty(
    conn: &Connection,
    input: &ReassignFacultyInput,
) -> Result<ReassignFacultyResult, String> {
    let mut cloned = 0i64;
    let mut conflicts = Vec::new();

    // Clone template slots from source to target for the faculty
    let mut stmt = conn.prepare(
        "SELECT grade_level, track, batch_pattern, day_of_week, period, subject_id, start_time, end_time, room, session_type
         FROM timetable_slots
         WHERE school_id = ?1 AND faculty_user_id = ?2 AND deleted_at IS NULL"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params![input.source_school_id, input.faculty_user_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            },
        )
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;

    for r in rows {
        let (gl, tr, bp, dow, period, sub_id, st, et, room, stype) =
            r.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;

        // Check for conflicts at target school
        let exists: bool = conn.query_row(
            "SELECT 1 FROM timetable_slots WHERE school_id = ?1 AND grade_level = ?2 AND track = ?3 AND batch_pattern = ?4 AND day_of_week = ?5 AND period = ?6 AND deleted_at IS NULL LIMIT 1",
            params![input.target_school_id, &gl, &tr, &bp, dow, period],
            |_| Ok(true),
        ).unwrap_or(false);

        if exists {
            conflicts.push(format!(
                "Conflict: {} {} day={} period={}",
                gl, tr, dow, period
            ));
            continue;
        }

        conn.execute(
            "INSERT INTO timetable_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, room, session_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![input.target_school_id, &gl, &tr, &bp, dow, period, sub_id, input.faculty_user_id, &st, &et, &room, &stype],
        ).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        cloned += 1;
    }

    // Also clone weekly slots for the effective week
    let mut stmt = conn.prepare(
        "SELECT grade_level, track, batch_pattern, day_of_week, period, subject_id, start_time, end_time, room, session_type
         FROM timetable_weekly_slots
         WHERE school_id = ?1 AND faculty_user_id = ?2 AND week_start_date = ?3"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params![
                input.source_school_id,
                input.faculty_user_id,
                &input.effective_week_start
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            },
        )
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;

    for r in rows {
        let (gl, tr, bp, dow, period, sub_id, st, et, room, stype) =
            r.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        let exists: bool = conn.query_row(
            "SELECT 1 FROM timetable_weekly_slots WHERE school_id = ?1 AND grade_level = ?2 AND track = ?3 AND batch_pattern = ?4 AND day_of_week = ?5 AND period = ?6 AND week_start_date = ?7 LIMIT 1",
            params![input.target_school_id, &gl, &tr, &bp, dow, period, &input.effective_week_start],
            |_| Ok(true),
        ).unwrap_or(false);
        if exists {
            conflicts.push(format!(
                "Weekly conflict: {} {} day={} period={}",
                gl, tr, dow, period
            ));
            continue;
        }
        conn.execute(
            "INSERT INTO timetable_weekly_slots (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, week_start_date, room, session_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![input.target_school_id, &gl, &tr, &bp, dow, period, sub_id, input.faculty_user_id, &st, &et, &input.effective_week_start, &room, &stype],
        ).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        cloned += 1;
    }

    Ok(ReassignFacultyResult {
        cloned_slots: cloned,
        conflicts,
    })
}

pub fn clone_week_with_check(
    conn: &Connection,
    input: &CloneWeekInput,
) -> Result<CloneWeekResult, String> {
    let mut cloned = 0i64;
    let mut conflicts = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, room, session_type
         FROM timetable_weekly_slots
         WHERE school_id = ?1 AND week_start_date = ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![input.school_id, &input.from_week], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;

    for r in rows {
        let (gl, tr, bp, dow, period, sub_id, fac_id, st, et, room, stype) =
            r.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;

        // Check faculty overlap
        if let Some(fid) = fac_id {
            let overlap: bool = conn.query_row(
                "SELECT 1 FROM timetable_weekly_slots WHERE faculty_user_id = ?1 AND week_start_date = ?2 AND day_of_week = ?3 AND period = ?4 AND school_id != ?5 LIMIT 1",
                params![fid, &input.to_week, dow, period, input.school_id],
                |_| Ok(true),
            ).unwrap_or(false);
            if overlap {
                conflicts.push(format!(
                    "Faculty overlap for faculty_id={} day={} period={}",
                    fid, dow, period
                ));
            }
        }

        // Check room double-booking
        if !room.is_empty() {
            let room_conflict: bool = conn.query_row(
                "SELECT 1 FROM timetable_weekly_slots WHERE room = ?1 AND week_start_date = ?2 AND day_of_week = ?3 AND period = ?4 AND school_id != ?5 LIMIT 1",
                params![&room, &input.to_week, dow, period, input.school_id],
                |_| Ok(true),
            ).unwrap_or(false);
            if room_conflict {
                conflicts.push(format!(
                    "Room conflict: {} day={} period={}",
                    room, dow, period
                ));
            }
        }

        conn.execute(
            "INSERT OR REPLACE INTO timetable_weekly_slots
             (school_id, grade_level, track, batch_pattern, day_of_week, period, subject_id, faculty_user_id, start_time, end_time, week_start_date, room, session_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![input.school_id, &gl, &tr, &bp, dow, period, sub_id, fac_id, &st, &et, &input.to_week, &room, &stype],
        ).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        cloned += 1;
    }

    Ok(CloneWeekResult {
        cloned_slots: cloned,
        conflicts,
    })
}

pub fn list_cross_school_room_conflicts(
    conn: &Connection,
    week_start: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<CrossSchoolRoomConflict>, String> {
    let mut sql = String::from(
        "SELECT wts.room, wts.day_of_week, wts.period, wts.school_id, s.name, wts.grade_level, wts.track, sub.name, COALESCE(u.display_name, '')
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         JOIN subjects sub ON sub.id = wts.subject_id
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         WHERE wts.room != '' AND wts.week_start_date = ?1"
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    p.push(week_start.to_string().into());

    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND wts.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY wts.room, wts.day_of_week, wts.period, wts.school_id");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())
        .map_err(|e| e.to_string())?;

    let mut map: std::collections::HashMap<(String, i64, i64), Vec<CrossSchoolRoomConflictSlot>> =
        std::collections::HashMap::new();
    for r in rows {
        let (room, dow, period, sid, sname, gl, tr, subname, facname) =
            r.map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        map.entry((room.clone(), dow, period))
            .or_default()
            .push(CrossSchoolRoomConflictSlot {
                school_id: sid,
                school_name: sname,
                grade_level: gl,
                track: tr,
                subject_name: subname,
                faculty_name: facname,
                week_start_date: week_start.to_string(),
            });
    }

    let mut result = Vec::new();
    for ((room, dow, period), slots) in map {
        if slots.len() > 1 {
            result.push(CrossSchoolRoomConflict {
                room,
                day_of_week: dow,
                period,
                slots,
            });
        }
    }
    result.sort_by(|a, b| {
        a.room
            .cmp(&b.room)
            .then(a.day_of_week.cmp(&b.day_of_week))
            .then(a.period.cmp(&b.period))
    });
    Ok(result)
}

fn log_bulk_op(
    conn: &Connection,
    op_type: &str,
    status: &str,
    result_json: &str,
) -> Result<BulkOperationLog, String> {
    conn.execute(
        "INSERT INTO bulk_operation_log (type, status, payload_json, result_json, created_at)
         VALUES (?1, ?2, '{}', ?3, datetime('now', 'localtime'))",
        params![op_type, status, result_json],
    )
    .map_err(|e| e.to_string())
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(BulkOperationLog {
        id,
        op_type: op_type.to_string(),
        status: status.to_string(),
        payload_json: "{}".to_string(),
        result_json: result_json.to_string(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        completed_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
