use crate::{
    alerts,
    auth::{
        enforce_school_scope, require_admin, require_admin_or_aom, require_faculty_or_admin,
        require_faculty_or_admin_or_aom, require_head_or_admin_or_aom, scope_filter,
    },
    error::AppError,
    models::*,
    repositories,
};
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use chrono::Datelike;
use rusqlite::params;
use serde::Deserialize;
use std::sync::Arc;

fn alert_id(alert: &Alert) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    alert.category.hash(&mut hasher);
    alert.message.hash(&mut hasher);
    alert.school_id.hash(&mut hasher);
    alert.grade_level.hash(&mut hasher);
    alert.subject_name.hash(&mut hasher);
    alert.faculty_user_id.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[allow(unused_imports)]
use crate::models::{TimetableSlot, UpsertTimetableSlotInput};

// ── Subjects ─────────────────────────────────────────────────────────────────

pub async fn list_subjects(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<Subject>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_subjects(&*conn)?))
}

pub async fn create_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateSubjectInput>,
) -> Result<Json<Subject>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_subject(&*conn, &input)?))
}

pub async fn update_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateSubjectInput>,
) -> Result<Json<Subject>, AppError> {
    require_admin(&claims)?;
    if input.id != id {
        return Err(AppError::bad_request("Path ID does not match body ID"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_subject(&*conn, &input)?))
}

pub async fn delete_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_subject(&*conn, id)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct EffectiveSubjectsQuery {
    pub track: String,
}

pub async fn list_effective_subjects(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(school_id): Path<i64>,
    Query(q): Query<EffectiveSubjectsQuery>,
) -> Result<Json<Vec<EffectiveSubject>>, AppError> {
    enforce_school_scope(&claims, school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_effective_subjects(
        &*conn, school_id, &q.track,
    )?))
}

#[derive(Deserialize)]
pub struct ToggleOptionalSubjectInput {
    pub subject_id: i64,
    pub enabled: bool,
}

pub async fn set_school_optional_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(school_id): Path<i64>,
    Json(input): Json<ToggleOptionalSubjectInput>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::set_school_optional_subject(&*conn, school_id, input.subject_id, input.enabled)?;
    Ok(Json(()))
}

// ── Faculty assignments ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FacultyAssignmentQuery {
    pub school_id: Option<i64>,
    pub faculty_id: Option<i64>,
}

pub async fn list_faculty_assignments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<FacultyAssignmentQuery>,
) -> Result<Json<Vec<FacultyAssignment>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_faculty_assignments(
        &*conn,
        q.school_id,
        q.faculty_id,
        scope_filter(&claims),
    )?))
}

pub async fn create_faculty_assignment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateFacultyAssignmentInput>,
) -> Result<Json<FacultyAssignment>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let batch = repositories::get_batch(&*conn, input.batch_id)?;
    enforce_school_scope(&claims, batch.school_id)?;
    Ok(Json(repositories::create_faculty_assignment(
        &*conn, &input,
    )?))
}

pub async fn delete_faculty_assignment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_faculty_assignment_school_id(&*conn, id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::delete_faculty_assignment(&*conn, id)?;
    Ok(Json(()))
}

// ── Timetable slots ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TimetableSlotQuery {
    pub school_id: Option<i64>,
    pub grade_level: Option<String>,
    pub track: Option<String>,
    pub batch_pattern: Option<String>,
}

pub async fn list_timetable_slots(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<TimetableSlotQuery>,
) -> Result<Json<Vec<TimetableSlot>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_timetable_slots(
        &*conn,
        q.school_id,
        q.grade_level.as_deref(),
        q.track.as_deref(),
        q.batch_pattern.as_deref(),
        scope_filter(&claims),
    )?))
}

pub async fn upsert_timetable_slot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertTimetableSlotInput>,
) -> Result<Json<TimetableSlot>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let batch = repositories::get_batch(&*conn, input.batch_id)?;
    enforce_school_scope(&claims, batch.school_id)?;
    Ok(Json(repositories::upsert_timetable_slot(&*conn, &input)?))
}

pub async fn delete_timetable_slot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_timetable_slot_school_id(&*conn, id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::delete_timetable_slot(&*conn, id)?;
    Ok(Json(()))
}

// ── Faculty app endpoints (Phase 2) ──────────────────────────────────────────

pub async fn today_sessions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FacultyTodaySession>>, AppError> {
    require_faculty_or_admin(&claims)?;
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let day_of_week = chrono::Local::now().weekday().num_days_from_monday() as i64;

    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut regular = repositories::list_faculty_today_sessions(
        &*conn,
        user_id,
        &today,
        day_of_week,
        scope_filter(&claims),
    )?;
    let mut makeup = repositories::list_faculty_today_makeup_sessions(
        &*conn,
        user_id,
        &today,
        scope_filter(&claims),
    )?;
    regular.append(&mut makeup);
    regular.sort_by_key(|s| s.period);
    Ok(Json(regular))
}

#[derive(Deserialize)]
pub struct LectureSessionQuery {
    pub school_id: i64,
    pub grade_level: Option<String>,
    pub from: String,
    pub to: String,
}

pub async fn list_lecture_sessions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<LectureSessionQuery>,
) -> Result<Json<Vec<LectureSession>>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, q.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_lecture_sessions(
        &*conn,
        q.school_id,
        q.grade_level.as_deref(),
        &q.from,
        &q.to,
        scope_filter(&claims),
    )?))
}

/// Verify that the caller is either admin/aom or the assigned/substituted faculty
/// for the given lecture session.
fn enforce_session_ownership(
    conn: &rusqlite::Connection,
    claims: &crate::models::Claims,
    session_id: i64,
) -> Result<(), crate::error::AppError> {
    if claims.role == "admin" || claims.role == "aom" {
        return Ok(());
    }
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let (actual_faculty, slot_faculty): (Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT ls.actual_faculty_user_id, ts.faculty_user_id
             FROM lecture_sessions ls
             LEFT JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             WHERE ls.id = ?1",
            rusqlite::params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| crate::error::AppError::internal(e.to_string()))?;
    if actual_faculty == Some(user_id) || slot_faculty == Some(user_id) {
        return Ok(());
    }
    Err(crate::error::AppError::forbidden(
        "You can only access sessions assigned to you",
    ))
}

pub async fn session_attendance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<AttendanceRecord>>, AppError> {
    require_faculty_or_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    enforce_session_ownership(&*conn, &claims, session_id)?;
    let status = repositories::get_lecture_session_status(&*conn, session_id)
        .map_err(|e| AppError::internal(e))?;
    if status == "Cancelled" {
        return Ok(Json(vec![]));
    }
    repositories::ensure_session_students(&*conn, session_id).map_err(|e| AppError::internal(e))?;
    Ok(Json(repositories::get_session_attendance(
        &*conn, session_id,
    )?))
}

pub async fn mark_attendance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
    Json(input): Json<MarkAttendanceInput>,
) -> Result<Json<()>, AppError> {
    require_faculty_or_admin(&claims)?;
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    enforce_session_ownership(&*conn, &claims, session_id)?;

    let status = repositories::get_lecture_session_status(&*conn, session_id)
        .map_err(|e| AppError::internal(e))?;
    if status == "Cancelled" {
        return Err(AppError::forbidden(
            "Cannot mark attendance for a cancelled session",
        ));
    }

    let records: Vec<(i64, String)> = input
        .records
        .into_iter()
        .map(|r| (r.student_id, r.status))
        .collect();

    let allow_override = claims.role == "admin" || claims.role == "aom";

    repositories::mark_attendance(
        &*conn,
        session_id,
        &records,
        user_id,
        &claims.username,
        allow_override,
    )
    .map_err(|e| AppError::forbidden(e))?;
    Ok(Json(()))
}

pub async fn substitute_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
    Json(input): Json<SubstituteSessionInput>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::substitute_session(&*conn, session_id, input.substitute_faculty_user_id)
        .map_err(|e| AppError::not_found(e))?;
    let summary = format!(
        "Assigned faculty user {} as substitute for session {}",
        input.substitute_faculty_user_id, session_id
    );
    repositories::insert_audit_log(
        &*conn,
        "lecture_session",
        Some(session_id),
        "substitute",
        &claims.username,
        &summary,
    )
    .map_err(|e| AppError::internal(e))?;
    Ok(Json(()))
}

pub async fn cancel_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::cancel_session(&*conn, session_id)?;
    let summary = format!("Cancelled session {}", session_id);
    repositories::insert_audit_log(
        &*conn,
        "lecture_session",
        Some(session_id),
        "cancel",
        &claims.username,
        &summary,
    )
    .map_err(|e| AppError::internal(e))?;
    Ok(Json(()))
}

pub async fn restore_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::restore_session(&*conn, session_id)?;
    let summary = format!("Restored session {}", session_id);
    repositories::insert_audit_log(
        &*conn,
        "lecture_session",
        Some(session_id),
        "restore",
        &claims.username,
        &summary,
    )
    .map_err(|e| AppError::internal(e))?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct CreateMakeupSessionInput {
    pub school_id: i64,
    pub grade_level: String,
    #[serde(default)]
    pub track: String,
    pub subject_id: i64,
    pub faculty_user_id: Option<i64>,
    pub session_date: String,
    pub start_time: String,
    pub end_time: String,
}

pub async fn create_makeup_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateMakeupSessionInput>,
) -> Result<Json<LectureSession>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let session = repositories::create_makeup_session(
        &*conn,
        input.school_id,
        &input.grade_level,
        &input.track,
        input.subject_id,
        input.faculty_user_id,
        &input.session_date,
        &input.start_time,
        &input.end_time,
    )?;
    let summary = format!(
        "Created makeup session for {} on {}",
        input.grade_level, input.session_date
    );
    repositories::insert_audit_log(
        &*conn,
        "lecture_session",
        Some(session.id),
        "create_makeup",
        &claims.username,
        &summary,
    )
    .map_err(|e| AppError::internal(e))?;
    Ok(Json(session))
}

pub async fn admin_today_sessions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FacultyTodaySession>>, AppError> {
    require_admin_or_aom(&claims)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut regular = repositories::list_all_today_sessions(&*conn, &today, scope_filter(&claims))?;
    let mut makeup =
        repositories::list_all_today_makeup_sessions(&*conn, &today, scope_filter(&claims))?;
    regular.append(&mut makeup);
    regular.sort_by_key(|s| s.period);
    Ok(Json(regular))
}

// ── Weekly Timetable ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WeeklyTimetableQuery {
    school_id: i64,
    week_start: String,
}

pub async fn list_weekly_timetable_slots(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<WeeklyTimetableQuery>,
) -> Result<Json<Vec<WeeklyTimetableSlot>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    enforce_school_scope(&claims, query.school_id)?;
    Ok(Json(repositories::list_weekly_timetable_slots(
        &*conn,
        query.school_id,
        &query.week_start,
        scope_filter(&claims),
    )?))
}

pub async fn upsert_weekly_timetable_slot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertWeeklyTimetableSlotInput>,
) -> Result<Json<WeeklyTimetableSlot>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_weekly_timetable_slot(
        &*conn, &input,
    )?))
}

pub async fn delete_weekly_timetable_slot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_weekly_timetable_slot_school_id(&*conn, id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::delete_weekly_timetable_slot(&*conn, id)?;
    Ok(Json(()))
}

pub async fn clone_week(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CloneWeekInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let count = repositories::clone_week_to_week(
        &*conn,
        &input.from_week,
        &input.to_week,
        input.school_id,
    )?;
    Ok(Json(serde_json::json!({ "copied": count })))
}

// ── Holidays ───────────────────────────────────────────────────────────────────

pub async fn list_holidays(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Holiday>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_holidays(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn create_holiday(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateHolidayInput>,
) -> Result<Json<Holiday>, AppError> {
    require_admin_or_aom(&claims)?;
    if input.scope == "school" {
        if let Some(sid) = input.school_id {
            enforce_school_scope(&claims, sid)?;
        }
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_holiday(&*conn, &input)?))
}

pub async fn bulk_create_holiday(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkCreateHolidayInput>,
) -> Result<Json<Vec<Holiday>>, AppError> {
    require_admin_or_aom(&claims)?;
    if input.scope == "school" {
        if let Some(sid) = input.school_id {
            enforce_school_scope(&claims, sid)?;
        }
    }
    let mut conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::bulk_create_holidays(&mut *conn, &input)?))
}

pub async fn delete_holiday(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if let Some(school_id) = repositories::get_holiday_school_id(&*conn, id)? {
        enforce_school_scope(&claims, school_id)?;
    }
    repositories::delete_holiday(&*conn, id)?;
    Ok(Json(()))
}

// ── Timetable analytics / health (Phase 5) ───────────────────────────────────

#[derive(Deserialize)]
pub struct FacultyScheduleQuery {
    pub week_start: String,
}

pub async fn faculty_cross_school_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(faculty_user_id): Path<i64>,
    Query(q): Query<FacultyScheduleQuery>,
) -> Result<Json<Vec<FacultyCrossSchoolSchedule>>, AppError> {
    if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        if user_id != faculty_user_id {
            return Err(AppError::forbidden("Can only view your own schedule"));
        }
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_faculty_cross_school_schedule(
        &*conn,
        faculty_user_id,
        &q.week_start,
        scope_filter(&claims),
    )?))
}

pub async fn timetable_health(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<TimetableHealthStatus>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_timetable_health_status(
        &*conn,
        scope_filter(&_claims),
    )?))
}

#[derive(Deserialize)]
pub struct ComplianceMetricsQuery {
    pub school_id: Option<i64>,
}

pub async fn compliance_metrics(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<ComplianceMetricsQuery>,
) -> Result<Json<Vec<ComplianceMetrics>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_compliance_metrics(
        &*conn,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn deviation_score(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(school_id): Path<i64>,
) -> Result<Json<DeviationScore>, AppError> {
    enforce_school_scope(&claims, school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_deviation_score(&*conn, school_id)?))
}

#[derive(Deserialize)]
pub struct SubstitutionsQuery {
    pub school_id: Option<i64>,
    pub faculty_user_id: Option<i64>,
    pub week_start: String,
}

pub async fn list_substitutions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SubstitutionsQuery>,
) -> Result<Json<Vec<SubstitutionRecord>>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // Faculty callers can only list their own substitutions; admin/aom can query any
    let faculty_filter = if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        Some(user_id)
    } else {
        q.faculty_user_id
    };
    if let Some(sid) = q.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::list_substitution_records(
        &*conn,
        q.school_id,
        faculty_filter,
        &q.week_start,
        scope_filter(&claims),
    )?))
}

pub async fn pending_substitutions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SubstitutionsQuery>,
) -> Result<Json<Vec<SubstitutionRecord>>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if let Some(sid) = q.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::list_pending_substitution_records(
        &*conn,
        q.school_id,
        &q.week_start,
        scope_filter(&claims),
    )?))
}

#[derive(Deserialize)]
pub struct AcceptSubstitutionInput {
    pub faculty_user_id: Option<i64>,
}

pub async fn accept_substitution(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
    input: Option<Json<AcceptSubstitutionInput>>,
) -> Result<Json<()>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    // Derive authorized faculty id: faculty callers always use JWT-derived id (no body required);
    // admin/aom may optionally provide one in the body
    let authorized_faculty_user_id: i64 = if claims.role == "admin" || claims.role == "aom" {
        input
            .and_then(|j| j.faculty_user_id)
            .unwrap_or_else(|| claims.sub.parse().unwrap_or(0))
    } else {
        claims.sub.parse().unwrap_or(0)
    };

    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;

    // For faculty callers, enforce that the session is available for them to accept
    // (no other faculty already assigned as substitute)
    if claims.role != "admin" && claims.role != "aom" {
        let session = repositories::get_lecture_session(&*conn, session_id)
            .map_err(|e| AppError::not_found(e))?;
        if session.actual_faculty_user_id.is_some()
            && session.actual_faculty_user_id != Some(authorized_faculty_user_id)
        {
            return Err(AppError::forbidden(
                "This session is already assigned to another faculty",
            ));
        }
    }

    repositories::accept_substitution(&*conn, session_id, authorized_faculty_user_id)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct DeclineSubstitutionInput {
    pub reason: String,
}

pub async fn decline_substitution(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
    Json(input): Json<DeclineSubstitutionInput>,
) -> Result<Json<()>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    // Derive authorized faculty id from claims for non-admin/aom
    let authorized_faculty_user_id: i64 = if claims.role == "admin" || claims.role == "aom" {
        0
    } else {
        claims.sub.parse().unwrap_or(0)
    };

    if claims.role != "admin" && claims.role != "aom" {
        let session = repositories::get_lecture_session(&*conn, session_id)
            .map_err(|e| AppError::not_found(e))?;
        if session.actual_faculty_user_id != Some(authorized_faculty_user_id) {
            return Err(AppError::forbidden(
                "Can only decline substitutions assigned to you",
            ));
        }
    }
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::decline_substitution(&*conn, session_id, &input.reason)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct RoomConflictsQuery {
    pub school_id: i64,
    pub week_start: String,
}

pub async fn room_conflicts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<RoomConflictsQuery>,
) -> Result<Json<Vec<(WeeklyTimetableSlot, WeeklyTimetableSlot)>>, AppError> {
    enforce_school_scope(&claims, q.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_room_conflicts(
        &*conn,
        q.school_id,
        &q.week_start,
    )?))
}

// ── Alerts ───────────────────────────────────────────────────────────────────

pub async fn get_alerts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Alert>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut alerts =
        alerts::get_all_alerts(&*conn, scope_filter(&claims)).map_err(|e| AppError::internal(e))?;
    for alert in &mut alerts {
        alert.id = alert_id(alert);
    }
    Ok(Json(alerts))
}

pub async fn get_faculty_alerts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Alert>>, AppError> {
    require_faculty_or_admin(&claims)?;
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut alerts =
        alerts::get_faculty_specific_alerts(&*conn, user_id).map_err(|e| AppError::internal(e))?;
    for alert in &mut alerts {
        alert.id = alert_id(alert);
    }
    Ok(Json(alerts))
}

// ── VP Centers ───────────────────────────────────────────────────────────────

pub async fn list_vp_centers(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<VpCenter>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_vp_centers(&*conn)?))
}

pub async fn create_vp_center(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateVpCenterInput>,
) -> Result<Json<VpCenter>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_vp_center(&*conn, &input)?))
}

pub async fn update_vp_center(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateVpCenterInput>,
) -> Result<Json<VpCenter>, AppError> {
    require_admin_or_aom(&claims)?;
    if input.id != id {
        return Err(AppError::bad_request("Path ID does not match body ID"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_vp_center(&*conn, &input)?))
}

pub async fn delete_vp_center(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_vp_center(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_vp_center_buildings(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(vp_center_id): Path<i64>,
) -> Result<Json<Vec<VpCenterBuilding>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_vp_center_buildings(
        &*conn,
        vp_center_id,
    )?))
}

pub async fn create_vp_center_building(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateVpCenterBuildingInput>,
) -> Result<Json<VpCenterBuilding>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_vp_center_building(
        &*conn, &input,
    )?))
}

pub async fn update_vp_center_building(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateVpCenterBuildingInput>,
) -> Result<Json<VpCenterBuilding>, AppError> {
    require_admin_or_aom(&claims)?;
    if input.id != id {
        return Err(AppError::bad_request("Path ID does not match body ID"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_vp_center_building(
        &*conn, &input,
    )?))
}

pub async fn delete_vp_center_building(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_vp_center_building(&*conn, id)?;
    Ok(Json(()))
}

// ── Faculty Profiles ─────────────────────────────────────────────────────────

pub async fn list_faculty_profiles(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FacultyProfile>>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_faculty_profiles(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn get_faculty_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(faculty_user_id): Path<i64>,
) -> Result<Json<FacultyProfile>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let profile = repositories::get_faculty_profile(&*conn, faculty_user_id)?;
    if let Some(school_id) = profile.sip_school_id {
        enforce_school_scope(&claims, school_id)?;
    }
    Ok(Json(profile))
}

pub async fn upsert_faculty_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertFacultyProfileInput>,
) -> Result<Json<FacultyProfile>, AppError> {
    require_admin_or_aom(&claims)?;
    if let Some(school_id) = input.sip_school_id {
        enforce_school_scope(&claims, school_id)?;
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_faculty_profile(
        &*conn,
        &input,
        &claims.username,
    )?))
}

// ── Faculty Members (master data, optional login) ─────────────────────────────

pub async fn list_faculty_members(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FacultyMember>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let scope = scope_filter(&claims);
    Ok(Json(repositories::list_faculty_members(&*conn, scope)?))
}

pub async fn create_faculty_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateFacultyMemberInput>,
) -> Result<Json<FacultyMember>, AppError> {
    require_admin_or_aom(&claims)?;
    // AOM must provide an initial scoped school
    if claims.role != "admin" {
        let school_id = input
            .initial_school_id
            .ok_or_else(|| AppError::bad_request("AOM must provide an initial school_id"))?;
        enforce_school_scope(&claims, school_id)?;
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_faculty_member(&*conn, &input)?))
}

pub async fn update_faculty_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateFacultyMemberInput>,
) -> Result<Json<FacultyMember>, AppError> {
    require_admin_or_aom(&claims)?;
    if input.id != id {
        return Err(AppError::bad_request("Path ID does not match body ID"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // AOM can only update faculty linked to their scoped schools
    if claims.role != "admin" {
        let scope = scope_filter(&claims).unwrap_or(&[]);
        let in_scope = repositories::is_faculty_in_scope(&*conn, id, scope)
            .map_err(|e| AppError::internal(e))?;
        if !in_scope {
            return Err(AppError::forbidden(
                "This faculty is not in your school scope",
            ));
        }
    }
    Ok(Json(repositories::update_faculty_member(&*conn, &input)?))
}

pub async fn delete_faculty_member(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // AOM can only archive faculty linked to their scoped schools
    if claims.role != "admin" {
        let scope = scope_filter(&claims).unwrap_or(&[]);
        let in_scope = repositories::is_faculty_in_scope(&*conn, id, scope)
            .map_err(|e| AppError::internal(e))?;
        if !in_scope {
            return Err(AppError::forbidden(
                "This faculty is not in your school scope",
            ));
        }
    }
    repositories::delete_faculty_member(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_faculty_school_memberships(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(faculty_id): Path<i64>,
) -> Result<Json<Vec<FacultySchoolMembership>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // AOM can only view memberships for faculty in their scope
    if claims.role != "admin" {
        let scope = scope_filter(&claims).unwrap_or(&[]);
        let in_scope = repositories::is_faculty_in_scope(&*conn, faculty_id, scope)
            .map_err(|e| AppError::internal(e))?;
        if !in_scope {
            return Err(AppError::forbidden(
                "This faculty is not in your school scope",
            ));
        }
    }
    Ok(Json(repositories::list_faculty_school_memberships(
        &*conn, faculty_id,
    )?))
}

pub async fn create_faculty_school_membership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateFacultySchoolMembershipInput>,
) -> Result<Json<FacultySchoolMembership>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_faculty_school_membership(
        &*conn, &input,
    )?))
}

pub async fn delete_faculty_school_membership(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // AOM can only delete memberships for schools in their scope
    if claims.role != "admin" {
        let school_id: i64 = conn
            .query_row(
                "SELECT school_id FROM faculty_school_memberships WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| AppError::internal(e.to_string()))?;
        enforce_school_scope(&claims, school_id)?;
    }
    repositories::delete_faculty_school_membership(&*conn, id)?;
    Ok(Json(()))
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultyLoginInput {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

pub async fn create_faculty_login(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<CreateFacultyLoginInput>,
) -> Result<Json<FacultyMember>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if claims.role != "admin" {
        let scope = scope_filter(&claims).unwrap_or(&[]);
        let in_scope = repositories::is_faculty_in_scope(&*conn, id, scope)
            .map_err(|e| AppError::internal(e))?;
        if !in_scope {
            return Err(AppError::forbidden(
                "This faculty is not in your school scope",
            ));
        }
    }
    Ok(Json(repositories::create_faculty_login(
        &*conn,
        id,
        &input.username,
        &input.display_name,
        &input.password,
    )?))
}

#[derive(Debug, Deserialize)]
pub struct LinkFacultyUserInput {
    pub user_id: i64,
}

pub async fn link_faculty_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<LinkFacultyUserInput>,
) -> Result<Json<FacultyMember>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if claims.role != "admin" {
        let scope = scope_filter(&claims).unwrap_or(&[]);
        let in_scope = repositories::is_faculty_in_scope(&*conn, id, scope)
            .map_err(|e| AppError::internal(e))?;
        if !in_scope {
            return Err(AppError::forbidden(
                "This faculty is not in your school scope",
            ));
        }
    }
    Ok(Json(repositories::link_faculty_user(
        &*conn,
        id,
        input.user_id,
    )?))
}
