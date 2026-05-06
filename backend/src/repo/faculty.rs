use crate::models::{
    AppUser, ChangePasswordInput, CreateFacultyAssignmentInput, CreateFacultyMemberInput,
    CreateFacultySchoolMembershipInput, CreateSubjectInput, CreateUserInput,
    CreateVpCenterBuildingInput, CreateVpCenterInput, EffectiveSubject, FacultyAssignment,
    FacultyMember, FacultyProfile, FacultySchoolMembership, SessionUser, Subject, TimetableSlot,
    UpdateFacultyMemberInput, UpdateSubjectInput, UpdateUserInput, UpdateVpCenterBuildingInput,
    UpdateVpCenterInput, UpsertFacultyProfileInput, UpsertTimetableSlotInput,
    UpsertWeeklyTimetableSlotInput, VpCenter, VpCenterBuilding, WeeklyTimetableSlot,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;

use super::audit::*;
use super::common::*;
use super::schools::get_batch;

pub fn list_users(conn: &Connection) -> Result<Vec<AppUser>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut stmt = conn
        .prepare(
            "
            SELECT id, username, display_name, role, is_active, created_at, last_login_at
            FROM users
            ORDER BY display_name
            LIMIT ?1
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![MAX_ROWS], |row| {
            let is_active: i64 = row.get(4)?;
            Ok(AppUser {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                role: row.get(3)?,
                is_active: is_active != 0,
                created_at: row.get(5)?,
                last_login_at: row.get(6)?,
                school_ids: Vec::new(),
            })
        })
        .map_err(|error| error.to_string())?;

    let mut users = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    for u in &mut users {
        u.school_ids = list_user_schools(conn, u.id)?;
    }
    Ok(users)
}

// ── user_schools (M:M scope between users and schools) ───────────────────────

pub fn list_user_schools(conn: &Connection, user_id: i64) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare("SELECT school_id FROM user_schools WHERE user_id = ?1 ORDER BY school_id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn is_faculty_at_school(
    conn: &Connection,
    faculty_user_id: i64,
    school_id: i64,
) -> Result<bool, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM faculty_assignments WHERE faculty_user_id = ?1 AND school_id = ?2",
        params![faculty_user_id, school_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn set_user_schools(conn: &Connection, user_id: i64, school_ids: &[i64]) -> Result<(), String> {
    conn.execute(
        "DELETE FROM user_schools WHERE user_id = ?1",
        params![user_id],
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)")
        .map_err(|e| e.to_string())?;
    for &sid in school_ids {
        stmt.execute(params![user_id, sid])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn get_active_user_by_id(conn: &Connection, user_id: i64) -> Result<SessionUser, String> {
    let result = conn.query_row(
        "SELECT id, username, display_name, role, is_active FROM users WHERE id = ?1",
        params![user_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    );

    match result {
        Err(_) => Err("User not found".to_string()),
        Ok((id, username, display_name, role, is_active)) => {
            if is_active == 0 {
                return Err("Account is disabled".to_string());
            }
            let school_ids = list_user_schools(conn, id)?;
            Ok(SessionUser {
                id,
                username,
                display_name,
                role,
                school_ids,
            })
        }
    }
}

pub fn authenticate_user(
    conn: &Connection,
    username: &str,
    password: &str,
) -> Result<SessionUser, String> {
    let result = conn.query_row(
        "
        SELECT id, username, display_name, role, password_hash, is_active
        FROM users
        WHERE username = ?1
        ",
        params![username.trim()],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        },
    );

    match result {
        Err(_) => Err("Invalid username or password".to_string()),
        Ok((id, uname, display_name, role, hash, is_active)) => {
            if is_active == 0 {
                return Err("Account is disabled. Contact an administrator.".to_string());
            }
            let valid = bcrypt::verify(password, &hash).map_err(|error| error.to_string())?;
            if !valid {
                return Err("Invalid username or password".to_string());
            }
            conn.execute(
                "UPDATE users SET last_login_at = datetime('now', 'localtime') WHERE id = ?1",
                params![id],
            )
            .map_err(|error| error.to_string())?;
            let school_ids = list_user_schools(conn, id)?;
            Ok(SessionUser {
                id,
                username: uname,
                display_name,
                role,
                school_ids,
            })
        }
    }
}

pub fn create_user(
    conn: &Connection,
    input: &CreateUserInput,
    actor: &str,
) -> Result<AppUser, String> {
    validate_nonempty("Username", &input.username)?;
    validate_nonempty("Display name", &input.display_name)?;
    validate_nonempty("Password", &input.password)?;
    validate_user_role(&input.role)?;
    if input.password.len() < 6 {
        return Err("Password must be at least 6 characters".to_string());
    }

    let hash =
        bcrypt::hash(&input.password, bcrypt::DEFAULT_COST).map_err(|error| error.to_string())?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let result = (|| -> Result<AppUser, String> {
        conn.execute(
            "
            INSERT INTO users (username, display_name, role, password_hash)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                input.username.trim().to_lowercase(),
                input.display_name.trim(),
                input.role.trim(),
                hash
            ],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                "Username already exists".to_string()
            } else {
                error.to_string()
            }
        })?;

        let id = conn.last_insert_rowid();
        set_user_schools(conn, id, &input.school_ids)?;
        let user = get_user(conn, id)?;
        record_audit(
            conn,
            "user",
            id,
            "created",
            actor,
            &format!(
                "Created user '{}' with role '{}'",
                user.display_name, user.role
            ),
        )?;
        Ok(user)
    })();
    match result {
        Ok(user) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(user)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn update_user(
    conn: &Connection,
    input: &UpdateUserInput,
    actor: &str,
) -> Result<AppUser, String> {
    validate_nonempty("Username", &input.username)?;
    validate_nonempty("Display name", &input.display_name)?;
    validate_user_role(&input.role)?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let result = (|| -> Result<AppUser, String> {
        conn.execute(
            "
            UPDATE users
            SET username = ?1, display_name = ?2, role = ?3, is_active = ?4
            WHERE id = ?5
            ",
            params![
                input.username.trim().to_lowercase(),
                input.display_name.trim(),
                input.role.trim(),
                if input.is_active { 1 } else { 0 },
                input.id
            ],
        )
        .map_err(|error| error.to_string())?;

        set_user_schools(conn, input.id, &input.school_ids)?;
        let user = get_user(conn, input.id)?;
        let status = if user.is_active { "active" } else { "disabled" };
        record_audit(
            conn,
            "user",
            input.id,
            "updated",
            actor,
            &format!(
                "Updated user '{}': role='{}' status='{}'",
                user.display_name, user.role, status
            ),
        )?;
        Ok(user)
    })();
    match result {
        Ok(user) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(user)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn delete_user(
    conn: &Connection,
    id: i64,
    current_user_id: i64,
    actor: &str,
) -> Result<(), String> {
    if id == current_user_id {
        return Err("Cannot delete your own account".to_string());
    }
    let display_name: String = conn
        .query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| format!("id={id}"));
    let deleted = conn
        .execute("DELETE FROM users WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    if deleted == 0 {
        return Err(format!("User {id} was not found"));
    }
    record_audit(
        conn,
        "user",
        id,
        "deleted",
        actor,
        &format!("Deleted user '{display_name}'"),
    )?;
    Ok(())
}

pub fn change_password(
    conn: &Connection,
    user_id: i64,
    input: &ChangePasswordInput,
    actor: &str,
) -> Result<(), String> {
    if input.new_password.len() < 6 {
        return Err("New password must be at least 6 characters".to_string());
    }

    let current_hash: String = conn
        .query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|_| "User not found".to_string())?;

    let valid = bcrypt::verify(&input.current_password, &current_hash)
        .map_err(|error| error.to_string())?;
    if !valid {
        return Err("Current password is incorrect".to_string());
    }

    let new_hash = bcrypt::hash(&input.new_password, bcrypt::DEFAULT_COST)
        .map_err(|error| error.to_string())?;

    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        params![new_hash, user_id],
    )
    .map_err(|error| error.to_string())?;

    record_audit(
        conn,
        "user",
        user_id,
        "password_changed",
        actor,
        "Password changed",
    )?;
    Ok(())
}

pub fn admin_reset_password(
    conn: &Connection,
    user_id: i64,
    new_password: &str,
    actor: &str,
) -> Result<(), String> {
    if new_password.len() < 6 {
        return Err("New password must be at least 6 characters".to_string());
    }

    let display_name: String = conn
        .query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|_| "User not found".to_string())?;

    let new_hash =
        bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|error| error.to_string())?;

    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        params![new_hash, user_id],
    )
    .map_err(|error| error.to_string())?;

    record_audit(
        conn,
        "user",
        user_id,
        "password_reset",
        actor,
        &format!("Admin reset password for '{display_name}'"),
    )?;
    Ok(())
}

fn get_user(conn: &Connection, id: i64) -> Result<AppUser, String> {
    let mut user = conn
        .query_row(
            "
        SELECT id, username, display_name, role, is_active, created_at, last_login_at
        FROM users WHERE id = ?1
        ",
            params![id],
            |row| {
                let is_active: i64 = row.get(4)?;
                Ok(AppUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    role: row.get(3)?,
                    is_active: is_active != 0,
                    created_at: row.get(5)?,
                    last_login_at: row.get(6)?,
                    school_ids: Vec::new(),
                })
            },
        )
        .map_err(|_| format!("User {id} was not found"))?;
    user.school_ids = list_user_schools(conn, id)?;
    Ok(user)
}

fn validate_user_role(role: &str) -> Result<(), String> {
    match role.trim() {
        "admin" | "agent" | "viewer" | "aom" | "faculty" => Ok(()),
        _ => Err(format!(
            "Invalid role '{role}'. Must be admin, agent, viewer, aom, or faculty"
        )),
    }
}

// ── Subjects ─────────────────────────────────────────────────────────────────

fn subject_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Subject> {
    Ok(Subject {
        id: row.get(0)?,
        name: row.get(1)?,
        track: row.get(2)?,
        is_default: row.get::<_, i64>(3)? == 1,
        sort_order: row.get(4)?,
    })
}

pub fn list_subjects(conn: &Connection) -> Result<Vec<Subject>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut stmt = conn
        .prepare("SELECT id, name, track, is_default, sort_order FROM subjects ORDER BY track, sort_order, name LIMIT ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![MAX_ROWS], subject_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_subject(conn: &Connection, input: &CreateSubjectInput) -> Result<Subject, String> {
    validate_nonempty("Subject name", &input.name)?;
    validate_track(&input.track)?;
    conn.execute(
        "INSERT INTO subjects (name, track, is_default, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![
            input.name.trim(),
            input.track.trim(),
            if input.is_default { 1 } else { 0 },
            input.sort_order
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    get_subject(conn, id)
}

pub fn update_subject(conn: &Connection, input: &UpdateSubjectInput) -> Result<Subject, String> {
    validate_nonempty("Subject name", &input.name)?;
    validate_track(&input.track)?;
    conn.execute(
        "UPDATE subjects SET name = ?1, track = ?2, is_default = ?3, sort_order = ?4 WHERE id = ?5",
        params![
            input.name.trim(),
            input.track.trim(),
            if input.is_default { 1 } else { 0 },
            input.sort_order,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    get_subject(conn, input.id)
}

pub fn delete_subject(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM subjects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_subject(conn: &Connection, id: i64) -> Result<Subject, String> {
    conn.query_row(
        "SELECT id, name, track, is_default, sort_order FROM subjects WHERE id = ?1",
        params![id],
        subject_from_row,
    )
    .map_err(|e| e.to_string())
}

fn validate_track(track: &str) -> Result<(), String> {
    match track.trim() {
        "JEE" | "NEET" | "Foundation" => Ok(()),
        _ => Err(format!(
            "Invalid track '{track}'. Must be JEE, NEET, or Foundation"
        )),
    }
}

// Effective subjects available at a school for a track:
// - Track defaults always included
// - Foundation optional subjects only if school has opted in via school_optional_subjects
pub fn list_effective_subjects(
    conn: &Connection,
    school_id: i64,
    track: &str,
) -> Result<Vec<EffectiveSubject>, String> {
    validate_track(track)?;
    let mut stmt = conn
        .prepare(
            "
            SELECT s.id, s.name, s.track, s.is_default,
                   CASE
                       WHEN s.is_default = 1 THEN 1
                       WHEN sos.subject_id IS NOT NULL THEN 1
                       ELSE 0
                   END AS is_offered
            FROM subjects s
            LEFT JOIN school_optional_subjects sos
              ON sos.subject_id = s.id AND sos.school_id = ?1
            WHERE s.track = ?2
            ORDER BY s.sort_order, s.name
            ",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![school_id, track], |row| {
            Ok(EffectiveSubject {
                id: row.get(0)?,
                name: row.get(1)?,
                track: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
                is_offered: row.get::<_, i64>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── Faculty assignments ──────────────────────────────────────────────────────

pub fn list_faculty_assignments(
    conn: &Connection,
    school_id: Option<i64>,
    faculty_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyAssignment>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT fa.id, fa.faculty_id, fa.faculty_user_id, fm.name,
                fa.school_id, s.name,
                fa.batch_id, b.batch_id, fa.grade_level, fa.track, b.batch_pattern,
                fa.subject_id, sub.name,
                fa.created_at
         FROM faculty_assignments fa
         JOIN faculty_members fm ON fm.id = fa.faculty_id
         JOIN schools s ON s.id = fa.school_id
         JOIN batches b ON b.id = fa.batch_id
         JOIN subjects sub ON sub.id = fa.subject_id
         WHERE 1=1",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(v) = school_id {
        sql.push_str(" AND fa.school_id = ?");
        p.push(v.into());
    }
    if let Some(v) = faculty_id {
        sql.push_str(" AND fa.faculty_id = ?");
        p.push(v.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND fa.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(
        " ORDER BY fm.name, s.name, fa.grade_level, fa.track, b.batch_id, sub.name LIMIT ?",
    );
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<FacultyAssignment> {
        Ok(FacultyAssignment {
            id: row.get(0)?,
            faculty_id: row.get(1)?,
            faculty_user_id: row.get(2)?,
            faculty_display_name: row.get(3)?,
            school_id: row.get(4)?,
            school_name: row.get(5)?,
            batch_id: row.get(6)?,
            batch_name: row.get(7)?,
            grade_level: row.get(8)?,
            track: row.get(9)?,
            batch_pattern: row.get(10)?,
            subject_id: row.get(11)?,
            subject_name: row.get(12)?,
            created_at: row.get(13)?,
        })
    };

    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), map_row)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_faculty_assignment(
    conn: &Connection,
    input: &CreateFacultyAssignmentInput,
) -> Result<FacultyAssignment, String> {
    let batch = get_batch(conn, input.batch_id)?;
    // Subject's track must match the assignment's track. For Foundation
    // batches the batch's track is empty but the subject's track is
    // "Foundation" — accept that as a match.
    let subject = get_subject(conn, input.subject_id)?;
    let assignment_track = if batch.track.is_empty() {
        "Foundation"
    } else {
        batch.track.as_str()
    };
    if subject.track != assignment_track {
        return Err(format!(
            "Subject '{}' belongs to track '{}', does not match assignment track '{}'",
            subject.name, subject.track, assignment_track
        ));
    }
    // Faculty member must exist.
    let faculty = get_faculty_member(conn, input.faculty_id)?;

    conn.execute(
        "
        INSERT INTO faculty_assignments
            (faculty_id, faculty_user_id, school_id, batch_id, grade_level, track, subject_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        params![
            input.faculty_id,
            faculty.user_id,
            batch.school_id,
            batch.id,
            batch.grade_level,
            batch.track,
            input.subject_id,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "This faculty already has this exact assignment".to_string()
        } else {
            e.to_string()
        }
    })?;

    let id = conn.last_insert_rowid();
    list_faculty_assignments(conn, None, None, None)?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| "Failed to read back the new assignment".to_string())
}

// ── Timetable slots ──────────────────────────────────────────────────────────

fn timetable_slot_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimetableSlot> {
    Ok(TimetableSlot {
        id: row.get(0)?,
        school_id: row.get(1)?,
        school_name: row.get(2)?,
        batch_id: row.get(3)?,
        batch_name: row.get(4)?,
        grade_level: row.get(5)?,
        track: row.get(6)?,
        batch_pattern: row.get(7)?,
        day_of_week: row.get(8)?,
        period: row.get(9)?,
        subject_id: row.get(10)?,
        subject_name: row.get(11)?,
        faculty_user_id: row.get(12)?,
        faculty_display_name: row.get::<_, Option<String>>(13)?.unwrap_or_default(),
        start_time: row.get(14)?,
        end_time: row.get(15)?,
        room: row.get(16)?,
        session_type: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

pub fn list_timetable_slots(
    conn: &Connection,
    school_id: Option<i64>,
    grade_level: Option<&str>,
    track: Option<&str>,
    batch_pattern: Option<&str>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<TimetableSlot>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT ts.id, ts.school_id, s.name,
                COALESCE(NULLIF(ts.batch_ref_id, 0), b.id, 0) AS concrete_batch_id,
                COALESCE(b.batch_id, ts.batch_id) AS concrete_batch_name,
                ts.grade_level, ts.track, ts.batch_pattern,
                ts.day_of_week, ts.period,
                ts.subject_id, sub.name,
                ts.faculty_user_id, u.display_name,
                ts.start_time, ts.end_time, ts.room, ts.session_type, ts.updated_at
         FROM timetable_slots ts
         JOIN schools  s   ON s.id   = ts.school_id
         JOIN subjects sub ON sub.id = ts.subject_id
         LEFT JOIN batches b ON b.id = ts.batch_ref_id
         LEFT JOIN users u ON u.id   = ts.faculty_user_id
         WHERE ts.deleted_at IS NULL",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(v) = school_id {
        sql.push_str(" AND ts.school_id = ?");
        p.push(v.into());
    }
    if let Some(v) = grade_level {
        sql.push_str(" AND ts.grade_level = ?");
        p.push(v.to_string().into());
    }
    if let Some(v) = track {
        sql.push_str(" AND ts.track = ?");
        p.push(v.to_string().into());
    }
    if let Some(v) = batch_pattern {
        sql.push_str(" AND ts.batch_pattern = ?");
        p.push(v.to_string().into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND ts.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY ts.day_of_week, ts.period LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(p.iter()),
            timetable_slot_from_row,
        )
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn upsert_timetable_slot(
    conn: &Connection,
    input: &UpsertTimetableSlotInput,
) -> Result<TimetableSlot, String> {
    let batch = get_batch(conn, input.batch_id)?;
    if input.day_of_week < 0 || input.day_of_week > 6 {
        return Err("day_of_week must be in 0..6".to_string());
    }
    if input.period < 1 {
        return Err("period must be >= 1".to_string());
    }
    let subject = get_subject(conn, input.subject_id)?;
    let assignment_track = if batch.track.is_empty() {
        "Foundation"
    } else {
        batch.track.as_str()
    };
    if subject.track != assignment_track {
        return Err(format!(
            "Subject '{}' belongs to track '{}', does not match slot track '{}'",
            subject.name, subject.track, assignment_track
        ));
    }
    if let Some(fid) = input.faculty_user_id {
        let _ = get_user(conn, fid)?;

        // ── Conflict check 1: Faculty double-booking ───────────────────────
        // A faculty cannot be in two different classes at the same day+period.
        let conflict: Option<(String, String, String, i64)> = conn.query_row(
            "SELECT s.name, ts.grade_level, ts.track, ts.period
             FROM timetable_slots ts
             JOIN schools s ON s.id = ts.school_id
             WHERE ts.faculty_user_id = ?1
               AND ts.day_of_week = ?2
               AND ts.period = ?3
               AND NOT (ts.batch_ref_id = ?4)
             LIMIT 1",
            params![
                fid, input.day_of_week, input.period,
                batch.id
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).ok();
        if let Some((school_name, grade, track, period)) = conflict {
            return Err(format!(
                "Faculty is already scheduled at {} {} period {} in {}. Cannot double-book.",
                grade,
                if track.is_empty() {
                    "Foundation"
                } else {
                    &track
                },
                period,
                school_name
            ));
        }

        // ── Conflict check 2: Faculty-subject eligibility ──────────────────
        // The faculty must have an assignment for this subject at this school/grade/track.
        let eligible_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM faculty_assignments
             WHERE faculty_user_id = ?1 AND batch_id = ?2 AND subject_id = ?3",
            params![fid, batch.id, input.subject_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if eligible_count == 0 {
            let subject = get_subject(conn, input.subject_id)?;
            let user = get_user(conn, fid)?;
            return Err(format!(
                "{} is not assigned to teach {} at this school/grade/track. Add a faculty assignment first.",
                user.display_name, subject.name
            ));
        }
    }

    conn.execute(
        "
        INSERT INTO timetable_slots
            (school_id, grade_level, track, batch_pattern, batch_id, batch_ref_id,
             day_of_week, period, subject_id, faculty_user_id,
             start_time, end_time, room, session_type)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ON CONFLICT(school_id, grade_level, track, batch_pattern, day_of_week, period)
        DO UPDATE SET
            batch_id        = excluded.batch_id,
            batch_ref_id    = excluded.batch_ref_id,
            subject_id      = excluded.subject_id,
            faculty_user_id = excluded.faculty_user_id,
            start_time      = excluded.start_time,
            end_time        = excluded.end_time,
            room            = excluded.room,
            session_type    = excluded.session_type,
            updated_at      = datetime('now', 'localtime')
        ",
        params![
            batch.school_id,
            batch.grade_level.trim(),
            batch.track.trim(),
            batch.batch_pattern.trim(),
            batch.batch_id.trim(),
            batch.id,
            input.day_of_week,
            input.period,
            input.subject_id,
            input.faculty_user_id,
            input.start_time.trim(),
            input.end_time.trim(),
            input.room.trim(),
            input.session_type.trim(),
        ],
    )
    .map_err(|e| e.to_string())?;

    let slots = list_timetable_slots(
        conn,
        Some(batch.school_id),
        Some(batch.grade_level.trim()),
        Some(batch.track.trim()),
        Some(batch.batch_pattern.trim()),
        None,
    )?;
    slots
        .into_iter()
        .find(|s| s.day_of_week == input.day_of_week && s.period == input.period)
        .ok_or_else(|| "Failed to read back the timetable slot".to_string())
}

pub fn get_timetable_slot_school_id(conn: &Connection, id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT school_id FROM timetable_slots WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Timetable slot {id} was not found"))
}

pub fn delete_timetable_slot(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute(
            "UPDATE timetable_slots SET deleted_at = datetime('now', 'localtime') WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!(
            "Timetable slot {id} was not found or already deleted"
        ));
    }
    Ok(())
}

// ── Weekly timetable slots (date-based, override template) ───────────────────

pub fn list_weekly_timetable_slots(
    conn: &Connection,
    school_id: i64,
    week_start_date: &str,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<WeeklyTimetableSlot>, String> {
    if let Some(ids) = scope_school_ids {
        if !ids.contains(&school_id) {
            return Ok(vec![]);
        }
    }

    const MAX_ROWS: i64 = 1000;

    // Query weekly slots for this school+week
    let mut weekly_stmt = conn.prepare(
        "SELECT
            wts.id, wts.school_id, s.name, wts.grade_level, wts.track, wts.batch_pattern,
            wts.day_of_week, wts.period, wts.subject_id, sub.name,
            wts.faculty_user_id, u.display_name,
            wts.start_time, wts.end_time, wts.room, wts.session_type, wts.week_start_date, wts.updated_at
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         JOIN subjects sub ON sub.id = wts.subject_id
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         WHERE wts.school_id = ?1 AND wts.week_start_date = ?2
         ORDER BY wts.day_of_week, wts.period, wts.batch_pattern
         LIMIT ?3"
    ).map_err(|e| e.to_string())?;

    let weekly_rows = weekly_stmt
        .query_map(params![school_id, week_start_date, MAX_ROWS], |row| {
            Ok(WeeklyTimetableSlot {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                grade_level: row.get(3)?,
                track: row.get(4)?,
                batch_pattern: row.get(5)?,
                day_of_week: row.get(6)?,
                period: row.get(7)?,
                subject_id: row.get(8)?,
                subject_name: row.get(9)?,
                faculty_user_id: row.get(10)?,
                faculty_display_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                room: row.get(14)?,
                session_type: row.get(15)?,
                week_start_date: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut weekly: Vec<WeeklyTimetableSlot> = weekly_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // If no weekly slots exist for this week, fall back to the template
    if weekly.is_empty() {
        let mut template_stmt = conn.prepare(
            "SELECT
                ts.id, ts.school_id, s.name, ts.grade_level, ts.track, ts.batch_pattern,
                ts.day_of_week, ts.period, ts.subject_id, sub.name,
                ts.faculty_user_id, u.display_name,
                ts.start_time, ts.end_time, ts.room, ts.session_type, ?2 AS week_start_date, ts.updated_at
             FROM timetable_slots ts
             JOIN schools s ON s.id = ts.school_id
             JOIN subjects sub ON sub.id = ts.subject_id
             LEFT JOIN users u ON u.id = ts.faculty_user_id
             WHERE ts.school_id = ?1
               AND ts.deleted_at IS NULL
             ORDER BY ts.day_of_week, ts.period, ts.batch_pattern
             LIMIT ?3"
        ).map_err(|e| e.to_string())?;

        let template_rows = template_stmt
            .query_map(params![school_id, week_start_date, MAX_ROWS], |row| {
                Ok(WeeklyTimetableSlot {
                    id: row.get(0)?,
                    school_id: row.get(1)?,
                    school_name: row.get(2)?,
                    grade_level: row.get(3)?,
                    track: row.get(4)?,
                    batch_pattern: row.get(5)?,
                    day_of_week: row.get(6)?,
                    period: row.get(7)?,
                    subject_id: row.get(8)?,
                    subject_name: row.get(9)?,
                    faculty_user_id: row.get(10)?,
                    faculty_display_name: row.get(11)?,
                    start_time: row.get(12)?,
                    end_time: row.get(13)?,
                    room: row.get(14)?,
                    session_type: row.get(15)?,
                    week_start_date: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })
            .map_err(|e| e.to_string())?;

        weekly = template_rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
    }

    Ok(weekly)
}

pub fn upsert_weekly_timetable_slot(
    conn: &Connection,
    input: &UpsertWeeklyTimetableSlotInput,
) -> Result<WeeklyTimetableSlot, String> {
    validate_nonempty("Grade level", &input.grade_level)?;
    validate_nonempty("Batch pattern", &input.batch_pattern)?;
    validate_nonempty("Week start date", &input.week_start_date)?;
    if !input.track.is_empty() {
        validate_track(&input.track)?;
    }
    if input.day_of_week < 0 || input.day_of_week > 6 {
        return Err("day_of_week must be in 0..6".to_string());
    }
    if input.period < 1 {
        return Err("period must be >= 1".to_string());
    }
    let _ = get_school(conn, input.school_id)?;
    let subject = get_subject(conn, input.subject_id)?;
    let assignment_track = if input.track.is_empty() {
        "Foundation"
    } else {
        input.track.as_str()
    };
    if subject.track != assignment_track {
        return Err(format!(
            "Subject '{}' belongs to track '{}', does not match slot track '{}'",
            subject.name, subject.track, assignment_track
        ));
    }
    if let Some(fid) = input.faculty_user_id {
        let _ = get_user(conn, fid)?;

        // Faculty-subject eligibility check
        let eligible_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM faculty_assignments
             WHERE faculty_user_id = ?1 AND school_id = ?2 AND grade_level = ?3 AND track = ?4 AND subject_id = ?5",
            params![fid, input.school_id, input.grade_level.trim(), input.track.trim(), input.subject_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if eligible_count == 0 {
            let user = get_user(conn, fid)?;
            return Err(format!(
                "{} is not assigned to teach {} at this school/grade/track. Add a faculty assignment first.",
                user.display_name, subject.name
            ));
        }
    }

    conn.execute(
        "INSERT INTO timetable_weekly_slots
            (school_id, grade_level, track, batch_pattern,
             day_of_week, period, subject_id, faculty_user_id,
             start_time, end_time, room, session_type, week_start_date)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(school_id, grade_level, track, batch_pattern, week_start_date, day_of_week, period)
        DO UPDATE SET
            subject_id      = excluded.subject_id,
            faculty_user_id = excluded.faculty_user_id,
            start_time      = excluded.start_time,
            end_time        = excluded.end_time,
            room            = excluded.room,
            session_type    = excluded.session_type,
            updated_at      = datetime('now', 'localtime')",
        params![
            input.school_id,
            input.grade_level.trim(),
            input.track.trim(),
            input.batch_pattern.trim(),
            input.day_of_week,
            input.period,
            input.subject_id,
            input.faculty_user_id,
            input.start_time.trim(),
            input.end_time.trim(),
            input.room.trim(),
            input.session_type.trim(),
            input.week_start_date,
        ],
    ).map_err(|e| e.to_string())?;

    let slots = list_weekly_timetable_slots(conn, input.school_id, &input.week_start_date, None)?;
    slots
        .into_iter()
        .find(|s| {
            s.day_of_week == input.day_of_week
                && s.period == input.period
                && s.grade_level == input.grade_level.trim()
                && s.track == input.track.trim()
                && s.batch_pattern == input.batch_pattern.trim()
        })
        .ok_or_else(|| "Failed to read back the weekly timetable slot".to_string())
}

pub fn get_weekly_timetable_slot_school_id(conn: &Connection, id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT school_id FROM timetable_weekly_slots WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Weekly timetable slot {id} was not found"))
}

pub fn delete_weekly_timetable_slot(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute(
            "DELETE FROM timetable_weekly_slots WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!("Weekly timetable slot {id} was not found"));
    }
    Ok(())
}

pub fn clone_week_to_week(
    conn: &Connection,
    from_week: &str,
    to_week: &str,
    school_id: i64,
) -> Result<usize, String> {
    let mut stmt = conn
        .prepare(
            "SELECT school_id, grade_level, track, batch_pattern,
                day_of_week, period, subject_id, faculty_user_id,
                start_time, end_time, room, session_type
         FROM timetable_weekly_slots
         WHERE school_id = ?1 AND week_start_date = ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![school_id, from_week], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;
    for row in rows {
        let (sid, gl, tr, bp, dow, per, subj, fid, st, et, room, session_type) =
            row.map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO timetable_weekly_slots
                (school_id, grade_level, track, batch_pattern,
                 day_of_week, period, subject_id, faculty_user_id,
                 start_time, end_time, room, session_type, week_start_date)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(school_id, grade_level, track, batch_pattern, week_start_date, day_of_week, period)
            DO UPDATE SET
                subject_id      = excluded.subject_id,
                faculty_user_id = excluded.faculty_user_id,
                start_time      = excluded.start_time,
                end_time        = excluded.end_time,
                room            = excluded.room,
                session_type    = excluded.session_type,
                updated_at      = datetime('now', 'localtime')",
            params![sid, gl, tr, bp, dow, per, subj, fid, st, et, room, session_type, to_week],
        ).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

pub fn get_faculty_assignment_school_id(conn: &Connection, id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT school_id FROM faculty_assignments WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Faculty assignment {id} was not found"))
}

pub fn delete_faculty_assignment(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = conn
        .execute("DELETE FROM faculty_assignments WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        return Err(format!("Assignment {id} was not found"));
    }
    Ok(())
}

pub fn get_weekly_timetable_slots_by_ids(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, WeeklyTimetableSlot>, String> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let mut stmt = conn.prepare(&format!(
        "SELECT
            wts.id, wts.school_id, s.name, wts.grade_level, wts.track, wts.batch_pattern,
            wts.day_of_week, wts.period, wts.subject_id, sub.name,
            wts.faculty_user_id, u.display_name,
            wts.start_time, wts.end_time, wts.room, wts.session_type, wts.week_start_date, wts.updated_at
         FROM timetable_weekly_slots wts
         JOIN schools s ON s.id = wts.school_id
         JOIN subjects sub ON sub.id = wts.subject_id
         LEFT JOIN users u ON u.id = wts.faculty_user_id
         WHERE wts.id IN ({placeholders})"
    )).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), |row| {
            let slot = WeeklyTimetableSlot {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                grade_level: row.get(3)?,
                track: row.get(4)?,
                batch_pattern: row.get(5)?,
                day_of_week: row.get(6)?,
                period: row.get(7)?,
                subject_id: row.get(8)?,
                subject_name: row.get(9)?,
                faculty_user_id: row.get(10)?,
                faculty_display_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                room: row.get(14)?,
                session_type: row.get(15)?,
                week_start_date: row.get(16)?,
                updated_at: row.get(17)?,
            };
            Ok((slot.id, slot))
        })
        .map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for r in rows {
        let (id, slot) = r.map_err(|e| e.to_string())?;
        map.insert(id, slot);
    }
    Ok(map)
}

fn vp_center_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VpCenter> {
    Ok(VpCenter {
        id: row.get(0)?,
        name: row.get(1)?,
        location: row.get(2)?,
        contact_name: row.get(3)?,
        contact_mobile: row.get(4)?,
        contact_email: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn list_vp_centers(conn: &Connection) -> Result<Vec<VpCenter>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, location, contact_name, contact_mobile, contact_email, created_at
             FROM vp_centers
             ORDER BY name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], vp_center_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_vp_center(conn: &Connection, id: i64) -> Result<VpCenter, String> {
    conn.query_row(
        "SELECT id, name, location, contact_name, contact_mobile, contact_email, created_at
         FROM vp_centers
         WHERE id = ?1",
        params![id],
        vp_center_from_row,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("VP Center {id} was not found"))
}

pub fn create_vp_center(
    conn: &Connection,
    input: &CreateVpCenterInput,
) -> Result<VpCenter, String> {
    validate_nonempty("Name", &input.name)?;
    conn.execute(
        "INSERT INTO vp_centers (name, location, contact_name, contact_mobile, contact_email)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.name.trim(),
            input.location.trim(),
            input.contact_name.trim(),
            input.contact_mobile.trim(),
            input.contact_email.trim(),
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    get_vp_center(conn, id)
}

pub fn update_vp_center(
    conn: &Connection,
    input: &UpdateVpCenterInput,
) -> Result<VpCenter, String> {
    validate_nonempty("Name", &input.name)?;
    conn.execute(
        "UPDATE vp_centers
         SET name = ?1,
             location = ?2,
             contact_name = ?3,
             contact_mobile = ?4,
             contact_email = ?5
         WHERE id = ?6",
        params![
            input.name.trim(),
            input.location.trim(),
            input.contact_name.trim(),
            input.contact_mobile.trim(),
            input.contact_email.trim(),
            input.id,
        ],
    )
    .map_err(|e| e.to_string())?;
    get_vp_center(conn, input.id)
}

pub fn delete_vp_center(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = conn
        .execute("DELETE FROM vp_centers WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        Err(format!("VP Center {id} was not found"))
    } else {
        Ok(())
    }
}

fn vp_center_building_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VpCenterBuilding> {
    Ok(VpCenterBuilding {
        id: row.get(0)?,
        vp_center_id: row.get(1)?,
        building_name: row.get(2)?,
        address: row.get(3)?,
        center_head_name: row.get(4)?,
        center_head_mobile: row.get(5)?,
        center_head_email: row.get(6)?,
        associate_center_head_name: row.get(7)?,
        associate_center_head_mobile: row.get(8)?,
        associate_center_head_email: row.get(9)?,
        created_at: row.get(10)?,
    })
}

pub fn list_vp_center_buildings(
    conn: &Connection,
    vp_center_id: i64,
) -> Result<Vec<VpCenterBuilding>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, vp_center_id, building_name, address, center_head_name, center_head_mobile,
                    center_head_email, associate_center_head_name, associate_center_head_mobile,
                    associate_center_head_email, created_at
             FROM vp_center_buildings
             WHERE vp_center_id = ?1
             ORDER BY building_name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![vp_center_id], vp_center_building_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_vp_center_building(conn: &Connection, id: i64) -> Result<VpCenterBuilding, String> {
    conn.query_row(
        "SELECT id, vp_center_id, building_name, address, center_head_name, center_head_mobile,
                center_head_email, associate_center_head_name, associate_center_head_mobile,
                associate_center_head_email, created_at
         FROM vp_center_buildings
         WHERE id = ?1",
        params![id],
        vp_center_building_from_row,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("VP Center Building {id} was not found"))
}

pub fn create_vp_center_building(
    conn: &Connection,
    input: &CreateVpCenterBuildingInput,
) -> Result<VpCenterBuilding, String> {
    validate_nonempty("Building name", &input.building_name)?;
    conn.execute(
        "INSERT INTO vp_center_buildings (
            vp_center_id, building_name, address, center_head_name, center_head_mobile,
            center_head_email, associate_center_head_name, associate_center_head_mobile,
            associate_center_head_email
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.vp_center_id,
            input.building_name.trim(),
            input.address.trim(),
            input.center_head_name.trim(),
            input.center_head_mobile.trim(),
            input.center_head_email.trim(),
            input.associate_center_head_name.trim(),
            input.associate_center_head_mobile.trim(),
            input.associate_center_head_email.trim(),
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    get_vp_center_building(conn, id)
}

pub fn update_vp_center_building(
    conn: &Connection,
    input: &UpdateVpCenterBuildingInput,
) -> Result<VpCenterBuilding, String> {
    validate_nonempty("Building name", &input.building_name)?;
    conn.execute(
        "UPDATE vp_center_buildings
         SET vp_center_id = ?1,
             building_name = ?2,
             address = ?3,
             center_head_name = ?4,
             center_head_mobile = ?5,
             center_head_email = ?6,
             associate_center_head_name = ?7,
             associate_center_head_mobile = ?8,
             associate_center_head_email = ?9
         WHERE id = ?10",
        params![
            input.vp_center_id,
            input.building_name.trim(),
            input.address.trim(),
            input.center_head_name.trim(),
            input.center_head_mobile.trim(),
            input.center_head_email.trim(),
            input.associate_center_head_name.trim(),
            input.associate_center_head_mobile.trim(),
            input.associate_center_head_email.trim(),
            input.id,
        ],
    )
    .map_err(|e| e.to_string())?;
    get_vp_center_building(conn, input.id)
}

pub fn delete_vp_center_building(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = conn
        .execute("DELETE FROM vp_center_buildings WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        Err(format!("VP Center Building {id} was not found"))
    } else {
        Ok(())
    }
}

fn faculty_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FacultyProfile> {
    let documents_verified: i64 = row.get(21)?;
    let is_active: i64 = row.get(22)?;
    Ok(FacultyProfile {
        faculty_user_id: row.get(0)?,
        faculty_display_name: row.get(1)?,
        pwid: row.get(2)?,
        email: row.get(3)?,
        mobile: row.get(4)?,
        emergency_contact_name: row.get(5)?,
        emergency_contact_mobile: row.get(6)?,
        vp_center_id: row.get(7)?,
        vp_center_name: row.get(8)?,
        sip_school_id: row.get(9)?,
        sip_school_name: row.get(10)?,
        primary_subject_id: row.get(11)?,
        primary_subject_name: row.get(12)?,
        employment_type: row.get(13)?,
        qualification: row.get(14)?,
        experience_years: row.get(15)?,
        designation: row.get(16)?,
        specialization: row.get(17)?,
        max_periods_per_week: row.get(18)?,
        joining_date: row.get(19)?,
        exit_date: row.get(20)?,
        documents_verified: documents_verified == 1,
        is_active: is_active == 1,
        wings: vec![],
        batch_ids: vec![],
        created_at: row.get(23)?,
        updated_at: row.get(24)?,
    })
}

pub fn list_faculty_profiles(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyProfile>, String> {
    let mut sql = String::from(
        "SELECT fp.faculty_user_id, u.display_name, fp.pwid, fp.email, fp.mobile,
                fp.emergency_contact_name, fp.emergency_contact_mobile, fp.vp_center_id,
                COALESCE(vc.name, ''), fp.sip_school_id, COALESCE(s.name, ''),
                fp.primary_subject_id, COALESCE(sub.name, ''), fp.employment_type,
                fp.qualification, fp.experience_years, fp.designation, fp.specialization,
                fp.max_periods_per_week, fp.joining_date, fp.exit_date, fp.documents_verified,
                fp.is_active, fp.created_at, fp.updated_at
         FROM faculty_profiles fp
         JOIN users u ON u.id = fp.faculty_user_id
         LEFT JOIN vp_centers vc ON vc.id = fp.vp_center_id
         LEFT JOIN schools s ON s.id = fp.sip_school_id
         LEFT JOIN subjects sub ON sub.id = fp.primary_subject_id
         WHERE 1=1",
    );
    let mut p: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND fp.sip_school_id IN ({placeholders})"));
            for id in ids {
                p.push(id);
            }
        }
    }
    sql.push_str(" ORDER BY u.display_name");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(p.iter()),
            faculty_profile_from_row,
        )
        .map_err(|e| e.to_string())?;
    let mut profiles = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for profile in &mut profiles {
        profile.wings = list_faculty_wings(conn, profile.faculty_user_id)?;
        profile.batch_ids = list_faculty_batch_ids(conn, profile.faculty_user_id)?;
    }
    Ok(profiles)
}

pub fn get_faculty_profile(
    conn: &Connection,
    faculty_user_id: i64,
) -> Result<FacultyProfile, String> {
    let mut profile = conn
        .query_row(
            "SELECT fp.faculty_user_id, u.display_name, fp.pwid, fp.email, fp.mobile,
                    fp.emergency_contact_name, fp.emergency_contact_mobile, fp.vp_center_id,
                    COALESCE(vc.name, ''), fp.sip_school_id, COALESCE(s.name, ''),
                    fp.primary_subject_id, COALESCE(sub.name, ''), fp.employment_type,
                    fp.qualification, fp.experience_years, fp.designation, fp.specialization,
                    fp.max_periods_per_week, fp.joining_date, fp.exit_date, fp.documents_verified,
                    fp.is_active, fp.created_at, fp.updated_at
             FROM faculty_profiles fp
             JOIN users u ON u.id = fp.faculty_user_id
             LEFT JOIN vp_centers vc ON vc.id = fp.vp_center_id
             LEFT JOIN schools s ON s.id = fp.sip_school_id
             LEFT JOIN subjects sub ON sub.id = fp.primary_subject_id
             WHERE fp.faculty_user_id = ?1",
            params![faculty_user_id],
            faculty_profile_from_row,
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Faculty profile for user {faculty_user_id} was not found"))?;
    profile.wings = list_faculty_wings(conn, faculty_user_id)?;
    profile.batch_ids = list_faculty_batch_ids(conn, faculty_user_id)?;
    Ok(profile)
}

pub fn upsert_faculty_profile(
    conn: &Connection,
    input: &UpsertFacultyProfileInput,
    actor: &str,
) -> Result<FacultyProfile, String> {
    conn.execute(
        "INSERT OR REPLACE INTO faculty_profiles (
            faculty_user_id, pwid, email, mobile,
            emergency_contact_name, emergency_contact_mobile,
            vp_center_id, sip_school_id, primary_subject_id,
            employment_type, qualification, experience_years,
            designation, specialization, max_periods_per_week,
            joining_date, exit_date, documents_verified, is_active,
            created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
            COALESCE((SELECT created_at FROM faculty_profiles WHERE faculty_user_id = ?1), datetime('now', 'localtime')),
            datetime('now', 'localtime')
        )",
        params![
            input.faculty_user_id,
            input.pwid.trim(),
            input.email.trim(),
            input.mobile.trim(),
            input.emergency_contact_name.trim(),
            input.emergency_contact_mobile.trim(),
            input.vp_center_id,
            input.sip_school_id,
            input.primary_subject_id,
            input.employment_type.trim(),
            input.qualification.trim(),
            input.experience_years,
            input.designation.trim(),
            input.specialization.trim(),
            input.max_periods_per_week,
            input.joining_date.trim(),
            input.exit_date.trim(),
            if input.documents_verified { 1 } else { 0 },
            if input.is_active { 1 } else { 0 },
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM faculty_wings WHERE faculty_user_id = ?1",
        params![input.faculty_user_id],
    )
    .map_err(|e| e.to_string())?;
    for wing in &input.wings {
        conn.execute(
            "INSERT INTO faculty_wings (faculty_user_id, wing) VALUES (?1, ?2)",
            params![input.faculty_user_id, wing.trim()],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        "DELETE FROM faculty_batches WHERE faculty_user_id = ?1",
        params![input.faculty_user_id],
    )
    .map_err(|e| e.to_string())?;
    for batch_id in &input.batch_ids {
        conn.execute(
            "INSERT INTO faculty_batches (faculty_user_id, batch_id) VALUES (?1, ?2)",
            params![input.faculty_user_id, batch_id],
        )
        .map_err(|e| e.to_string())?;
    }

    insert_audit_log(
        conn,
        "faculty_profile",
        Some(input.faculty_user_id),
        "upsert",
        actor,
        "Faculty profile updated",
    )?;

    get_faculty_profile(conn, input.faculty_user_id)
}

pub fn list_faculty_wings(conn: &Connection, faculty_user_id: i64) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT wing FROM faculty_wings WHERE faculty_user_id = ?1 ORDER BY wing")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![faculty_user_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_faculty_batch_ids(conn: &Connection, faculty_user_id: i64) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT batch_id FROM faculty_batches WHERE faculty_user_id = ?1 ORDER BY batch_id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![faculty_user_id], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── Faculty Members (master data, optional login) ─────────────────────────────

fn faculty_member_query_row(row: &rusqlite::Row) -> Result<FacultyMember, rusqlite::Error> {
    Ok(FacultyMember {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
        mobile: row.get(3)?,
        pwid: row.get(4)?,
        qualification: row.get(5)?,
        experience_years: row.get(6)?,
        designation: row.get(7)?,
        specialization: row.get(8)?,
        employment_type: row.get(9)?,
        is_active: row.get::<_, i64>(10)? != 0,
        user_id: row.get(11)?,
        user_username: row.get(12)?,
        user_display_name: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

pub fn list_faculty_members(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<FacultyMember>, String> {
    let sql = if let Some(ids) = scope_school_ids {
        if ids.is_empty() || ids == [-1] {
            return Ok(vec![]);
        }
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        format!(
            "
            SELECT DISTINCT
                fm.id,
                fm.name,
                fm.email,
                fm.mobile,
                fm.pwid,
                fm.qualification,
                fm.experience_years,
                fm.designation,
                fm.specialization,
                fm.employment_type,
                fm.is_active,
                fm.user_id,
                u.username,
                u.display_name,
                fm.created_at,
                fm.updated_at
            FROM faculty_members fm
            LEFT JOIN users u ON fm.user_id = u.id
            JOIN faculty_school_memberships fsm ON fsm.faculty_id = fm.id
            WHERE fsm.school_id IN ({placeholders})
            ORDER BY fm.name
        "
        )
    } else {
        "
            SELECT
                fm.id,
                fm.name,
                fm.email,
                fm.mobile,
                fm.pwid,
                fm.qualification,
                fm.experience_years,
                fm.designation,
                fm.specialization,
                fm.employment_type,
                fm.is_active,
                fm.user_id,
                u.username,
                u.display_name,
                fm.created_at,
                fm.updated_at
            FROM faculty_members fm
            LEFT JOIN users u ON fm.user_id = u.id
            ORDER BY fm.name
        "
        .to_string()
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = if let Some(ids) = scope_school_ids {
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        stmt.query_map(
            rusqlite::params_from_iter(params.iter()),
            faculty_member_query_row,
        )
    } else {
        stmt.query_map([], faculty_member_query_row)
    }
    .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_faculty_member(conn: &Connection, id: i64) -> Result<FacultyMember, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT
                fm.id,
                fm.name,
                fm.email,
                fm.mobile,
                fm.pwid,
                fm.qualification,
                fm.experience_years,
                fm.designation,
                fm.specialization,
                fm.employment_type,
                fm.is_active,
                fm.user_id,
                u.username,
                u.display_name,
                fm.created_at,
                fm.updated_at
            FROM faculty_members fm
            LEFT JOIN users u ON fm.user_id = u.id
            WHERE fm.id = ?1
        ",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row(params![id], faculty_member_query_row)
        .map_err(|e| e.to_string())
}

fn validate_faculty_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Faculty name is required".to_string());
    }
    Ok(())
}

pub fn create_faculty_member(
    conn: &Connection,
    input: &CreateFacultyMemberInput,
) -> Result<FacultyMember, String> {
    validate_faculty_name(&input.name)?;
    conn.execute(
        "INSERT INTO faculty_members
         (name, email, mobile, pwid, qualification, experience_years, designation, specialization, employment_type, is_active, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            &input.name,
            &input.email,
            &input.mobile,
            &input.pwid,
            &input.qualification,
            input.experience_years,
            &input.designation,
            &input.specialization,
            &input.employment_type,
            if input.is_active { 1 } else { 0 },
            input.user_id,
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    if let Some(school_id) = input.initial_school_id {
        create_faculty_school_membership(
            conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: id,
                school_id,
                role_at_school: "Faculty".to_string(),
                is_primary: true,
            },
        )?;
    }
    get_faculty_member(conn, id)
}

pub fn update_faculty_member(
    conn: &Connection,
    input: &UpdateFacultyMemberInput,
) -> Result<FacultyMember, String> {
    validate_faculty_name(&input.name)?;
    // Check duplicate user_id (excluding self)
    if let Some(uid) = input.user_id {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM faculty_members WHERE user_id = ?1 AND id <> ?2 LIMIT 1",
                params![uid, input.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if existing.is_some() {
            return Err(
                "Another faculty member is already linked to this user account".to_string(),
            );
        }
    }
    conn.execute(
        "UPDATE faculty_members SET
         name = ?1, email = ?2, mobile = ?3, pwid = ?4, qualification = ?5,
         experience_years = ?6, designation = ?7, specialization = ?8,
         employment_type = ?9, is_active = ?10, user_id = ?11,
         updated_at = datetime('now', 'localtime')
         WHERE id = ?12",
        params![
            &input.name,
            &input.email,
            &input.mobile,
            &input.pwid,
            &input.qualification,
            input.experience_years,
            &input.designation,
            &input.specialization,
            &input.employment_type,
            if input.is_active { 1 } else { 0 },
            input.user_id,
            input.id,
        ],
    )
    .map_err(|e| e.to_string())?;
    get_faculty_member(conn, input.id)
}

/// Archive (soft-delete) a faculty member by setting is_active = 0.
pub fn delete_faculty_member(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE faculty_members SET is_active = 0, updated_at = datetime('now', 'localtime') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns true if the faculty member is linked to any of the given school_ids.
pub fn is_faculty_in_scope(
    conn: &Connection,
    faculty_id: i64,
    school_ids: &[i64],
) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM faculty_school_memberships WHERE faculty_id = ?1 AND school_id IN (
                SELECT value FROM json_each(?2)
            )",
            params![faculty_id, format!("[{}]", school_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","))],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

// ── Faculty School Memberships ────────────────────────────────────────────────

fn normalize_faculty_role_label(role: &str) -> String {
    let trimmed = role.trim();
    if trimmed.eq_ignore_ascii_case("teacher") || trimmed.is_empty() {
        "Faculty".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn list_faculty_school_memberships(
    conn: &Connection,
    faculty_id: i64,
) -> Result<Vec<FacultySchoolMembership>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT
                fsm.id,
                fsm.faculty_id,
                fsm.school_id,
                s.name,
                fsm.role_at_school,
                fsm.is_primary,
                fsm.created_at
            FROM faculty_school_memberships fsm
            JOIN schools s ON fsm.school_id = s.id
            WHERE fsm.faculty_id = ?1
            ORDER BY s.name
        ",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![faculty_id], |row| {
            Ok(FacultySchoolMembership {
                id: row.get(0)?,
                faculty_id: row.get(1)?,
                school_id: row.get(2)?,
                school_name: row.get(3)?,
                role_at_school: row.get(4)?,
                is_primary: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_faculty_school_membership(
    conn: &Connection,
    input: &CreateFacultySchoolMembershipInput,
) -> Result<FacultySchoolMembership, String> {
    let role_at_school = normalize_faculty_role_label(&input.role_at_school);
    conn.execute(
        "INSERT INTO faculty_school_memberships
         (faculty_id, school_id, role_at_school, is_primary)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(faculty_id, school_id) DO UPDATE SET
         role_at_school = excluded.role_at_school,
         is_primary = excluded.is_primary",
        params![
            input.faculty_id,
            input.school_id,
            &role_at_school,
            if input.is_primary { 1 } else { 0 },
        ],
    )
    .map_err(|e| e.to_string())?;
    // Read back by natural key, not last_insert_rowid (which is wrong on UPDATE)
    let mut stmt = conn
        .prepare(
            "
            SELECT
                fsm.id,
                fsm.faculty_id,
                fsm.school_id,
                s.name,
                fsm.role_at_school,
                fsm.is_primary,
                fsm.created_at
            FROM faculty_school_memberships fsm
            JOIN schools s ON fsm.school_id = s.id
            WHERE fsm.faculty_id = ?1 AND fsm.school_id = ?2
        ",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row(params![input.faculty_id, input.school_id], |row| {
        Ok(FacultySchoolMembership {
            id: row.get(0)?,
            faculty_id: row.get(1)?,
            school_id: row.get(2)?,
            school_name: row.get(3)?,
            role_at_school: row.get(4)?,
            is_primary: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
        })
    })
    .map_err(|e| e.to_string())
}

pub fn delete_faculty_school_membership(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM faculty_school_memberships WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Copy faculty school memberships into user_schools for backward-compat scope.
fn sync_user_schools_from_memberships(
    conn: &Connection,
    user_id: i64,
    faculty_id: i64,
) -> Result<(), String> {
    let memberships = list_faculty_school_memberships(conn, faculty_id)?;
    for m in memberships {
        conn.execute(
            "INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)",
            params![user_id, m.school_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn create_faculty_login(
    conn: &Connection,
    faculty_id: i64,
    username: &str,
    display_name: &str,
    password: &str,
) -> Result<FacultyMember, String> {
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO users (username, display_name, role, password_hash) VALUES (?1, ?2, 'faculty', ?3)",
        params![username, display_name, hash],
    )
    .map_err(|e| e.to_string())?;
    let user_id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE faculty_members SET user_id = ?1, updated_at = datetime('now', 'localtime') WHERE id = ?2",
        params![user_id, faculty_id],
    )
    .map_err(|e| e.to_string())?;

    sync_user_schools_from_memberships(conn, user_id, faculty_id)?;
    get_faculty_member(conn, faculty_id)
}

pub fn link_faculty_user(
    conn: &Connection,
    faculty_id: i64,
    user_id: i64,
) -> Result<FacultyMember, String> {
    // Verify user exists and is not already linked to another faculty
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM faculty_members WHERE user_id = ?1 AND id <> ?2 LIMIT 1",
            params![user_id, faculty_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if existing.is_some() {
        return Err("This user is already linked to another faculty member".to_string());
    }

    conn.execute(
        "UPDATE faculty_members SET user_id = ?1, updated_at = datetime('now', 'localtime') WHERE id = ?2",
        params![user_id, faculty_id],
    )
    .map_err(|e| e.to_string())?;

    sync_user_schools_from_memberships(conn, user_id, faculty_id)?;
    get_faculty_member(conn, faculty_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        db::initialize_db(&conn).expect("initialize schema");
        conn
    }

    #[test]
    fn faculty_members_crud_and_backfill() {
        let conn = test_db();

        // Initially empty (test users not seeded in test_db)
        let members = list_faculty_members(&conn, None).expect("list faculty members");
        assert_eq!(members.len(), 0);

        // Create a new faculty member without a user account
        let created = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "New Faculty".to_string(),
                email: "new@example.com".to_string(),
                mobile: "9876543210".to_string(),
                pwid: "PW123".to_string(),
                qualification: "M.Sc.".to_string(),
                experience_years: 5,
                designation: "Senior Teacher".to_string(),
                specialization: "Physics".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty member");
        assert_eq!(created.name, "New Faculty");
        assert_eq!(created.user_id, None);

        // Update
        let updated = update_faculty_member(
            &conn,
            &UpdateFacultyMemberInput {
                id: created.id,
                name: "Updated Faculty".to_string(),
                email: "updated@example.com".to_string(),
                mobile: "9876543210".to_string(),
                pwid: "PW123".to_string(),
                qualification: "M.Sc.".to_string(),
                experience_years: 6,
                designation: "Lead Teacher".to_string(),
                specialization: "Physics".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
            },
        )
        .expect("update faculty member");
        assert_eq!(updated.name, "Updated Faculty");
        assert_eq!(updated.experience_years, 6);

        // Get
        let fetched = get_faculty_member(&conn, created.id).expect("get faculty member");
        assert_eq!(fetched.id, created.id);

        // Delete
        delete_faculty_member(&conn, created.id).expect("delete faculty member");
        let members_after = list_faculty_members(&conn, None).expect("list after delete");
        assert!(!members_after
            .iter()
            .any(|m| m.id == created.id && m.is_active));
    }

    #[test]
    fn faculty_school_memberships_crud() {
        let conn = test_db();

        // Need a faculty member and a school
        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Member Faculty".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty");

        // Use first school from seed
        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .expect("get school");

        // Create membership
        let membership = create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Coordinator".to_string(),
                is_primary: true,
            },
        )
        .expect("create membership");
        assert_eq!(membership.role_at_school, "Coordinator");
        assert!(membership.is_primary);

        // List
        let memberships =
            list_faculty_school_memberships(&conn, faculty.id).expect("list memberships");
        assert_eq!(memberships.len(), 1);

        // Delete
        delete_faculty_school_membership(&conn, membership.id).expect("delete membership");
        let memberships_after =
            list_faculty_school_memberships(&conn, faculty.id).expect("list after delete");
        assert!(memberships_after.is_empty());
    }

    #[test]
    fn faculty_members_scope_filtering_works() {
        let conn = test_db();

        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .expect("get school");

        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Scoped Faculty".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty");

        create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Faculty".to_string(),
                is_primary: false,
            },
        )
        .expect("create membership");

        // No scope → all members
        let all = list_faculty_members(&conn, None).expect("list all");
        assert!(all.iter().any(|m| m.id == faculty.id));

        // Scoped to the school → includes faculty
        let scoped = list_faculty_members(&conn, Some(&[school_id])).expect("list scoped");
        assert!(scoped.iter().any(|m| m.id == faculty.id));

        // Scoped to a different school → excludes faculty
        let other = list_faculty_members(&conn, Some(&[-1])).expect("list other");
        assert!(!other.iter().any(|m| m.id == faculty.id));
    }

    #[test]
    fn blank_faculty_name_is_rejected() {
        let conn = test_db();
        let result = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "   ".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name is required"));
    }

    #[test]
    fn duplicate_user_id_is_rejected() {
        let conn = test_db();

        // Create a user to link to
        let hash = bcrypt::hash("test123", bcrypt::DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash) VALUES ('u1', 'User One', 'faculty', ?1)",
            [&hash],
        ).unwrap();
        let user_id: i64 = conn
            .query_row("SELECT id FROM users WHERE username = 'u1'", [], |row| {
                row.get(0)
            })
            .unwrap();

        let f1 = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Faculty One".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: Some(user_id),
                initial_school_id: None,
            },
        )
        .expect("create first faculty");

        // Try to link another faculty to the same user
        let f2_result = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Faculty Two".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: Some(user_id),
                initial_school_id: None,
            },
        );
        assert!(f2_result.is_err());
        assert!(f2_result.unwrap_err().contains("UNIQUE constraint failed"));

        // Try to update a different faculty to the same user_id
        let f3 = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Faculty Three".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create third faculty");

        let update_result = update_faculty_member(
            &conn,
            &UpdateFacultyMemberInput {
                id: f3.id,
                name: "Faculty Three".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: Some(user_id),
            },
        );
        assert!(update_result.is_err());
        assert!(update_result.unwrap_err().contains("already linked"));

        // Updating self with same user_id should succeed
        let update_self = update_faculty_member(
            &conn,
            &UpdateFacultyMemberInput {
                id: f1.id,
                name: "Updated".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: Some(user_id),
            },
        );
        assert!(update_self.is_ok());
    }

    #[test]
    fn faculty_member_archive_behavior() {
        let conn = test_db();
        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "To Archive".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty");

        delete_faculty_member(&conn, faculty.id).expect("archive faculty");

        let archived = get_faculty_member(&conn, faculty.id).expect("get archived");
        assert!(!archived.is_active);

        // Memberships should still exist (CASCADE not triggered because row still exists)
        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .expect("get school");
        create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Faculty".to_string(),
                is_primary: false,
            },
        )
        .expect("membership on archived faculty still works");
    }

    #[test]
    fn membership_upsert_returns_correct_row() {
        let conn = test_db();
        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Upsert Test".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty");

        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .expect("get school");

        let m1 = create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Faculty".to_string(),
                is_primary: false,
            },
        )
        .expect("create membership");
        assert_eq!(m1.role_at_school, "Faculty");
        assert_eq!(m1.faculty_id, faculty.id);
        assert_eq!(m1.school_id, school_id);

        // Upsert same (faculty_id, school_id) with different role
        let m2 = create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Coordinator".to_string(),
                is_primary: true,
            },
        )
        .expect("upsert membership");
        assert_eq!(m2.role_at_school, "Coordinator");
        assert!(m2.is_primary);
        assert_eq!(m2.faculty_id, faculty.id);
        assert_eq!(m2.school_id, school_id);
    }

    #[test]
    fn teacher_membership_role_is_normalized_to_faculty() {
        let conn = test_db();
        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Role Normalize".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create faculty");

        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .expect("get school");

        let membership = create_faculty_school_membership(
            &conn,
            &CreateFacultySchoolMembershipInput {
                faculty_id: faculty.id,
                school_id,
                role_at_school: "Teacher".to_string(),
                is_primary: false,
            },
        )
        .expect("create membership");

        assert_eq!(membership.role_at_school, "Faculty");
    }

    #[test]
    fn faculty_assignment_can_use_unlinked_faculty() {
        let conn = test_db();
        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .unwrap();

        let subject = create_subject(
            &conn,
            &CreateSubjectInput {
                name: "Math".to_string(),
                track: "Foundation".to_string(),
                is_default: true,
                sort_order: 1,
            },
        )
        .expect("create subject");

        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Unlinked Teacher".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: Some(school_id),
            },
        )
        .expect("create faculty");
        assert!(faculty.user_id.is_none());
        let batch = crate::repo::schools::create_batch(
            &conn,
            &crate::models::CreateBatchInput {
                school_id,
                batch_id: "G9-FDN-WD-A".to_string(),
                grade_level: "Grade 9".to_string(),
                track: "".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 30,
            },
        )
        .expect("create batch");

        let assignment = create_faculty_assignment(
            &conn,
            &CreateFacultyAssignmentInput {
                faculty_id: faculty.id,
                batch_id: batch.id,
                subject_id: subject.id,
            },
        )
        .expect("create assignment for unlinked faculty");
        assert_eq!(assignment.faculty_id, faculty.id);
        assert_eq!(assignment.faculty_user_id, None);
    }

    #[test]
    fn create_login_links_user_and_syncs_memberships() {
        let conn = test_db();
        let school_id: i64 = conn
            .query_row("SELECT id FROM schools LIMIT 1", [], |row| row.get(0))
            .unwrap();

        let faculty = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "New Hire".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: Some(school_id),
            },
        )
        .expect("create faculty");
        assert!(faculty.user_id.is_none());

        let updated = create_faculty_login(&conn, faculty.id, "newhire", "New Hire", "password123")
            .expect("create login");
        assert!(updated.user_id.is_some());

        // Verify user_schools synced
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM user_schools WHERE user_id = ?1",
                params![updated.user_id.unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn link_user_rejects_duplicate() {
        let conn = test_db();
        let hash = bcrypt::hash("test123", bcrypt::DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash) VALUES ('u1', 'User One', 'faculty', ?1)",
            [&hash],
        ).unwrap();
        let user_id: i64 = conn
            .query_row("SELECT id FROM users WHERE username = 'u1'", [], |row| {
                row.get(0)
            })
            .unwrap();

        let f1 = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Faculty One".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: Some(user_id),
                initial_school_id: None,
            },
        )
        .expect("create first");

        let f2 = create_faculty_member(
            &conn,
            &CreateFacultyMemberInput {
                name: "Faculty Two".to_string(),
                email: "".to_string(),
                mobile: "".to_string(),
                pwid: "".to_string(),
                qualification: "".to_string(),
                experience_years: 0,
                designation: "".to_string(),
                specialization: "".to_string(),
                employment_type: "VP Payroll".to_string(),
                is_active: true,
                user_id: None,
                initial_school_id: None,
            },
        )
        .expect("create second");

        let result = link_faculty_user(&conn, f2.id, user_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already linked"));

        // Linking self should succeed
        let self_link = link_faculty_user(&conn, f1.id, user_id);
        assert!(self_link.is_ok());
    }
}
