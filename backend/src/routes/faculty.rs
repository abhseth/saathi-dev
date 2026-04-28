use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::{enforce_school_scope, require_admin, require_admin_or_aom, scope_filter}, error::AppError, models::*, repositories};

#[allow(unused_imports)]
use crate::models::{TimetableSlot, UpsertTimetableSlotInput};

// ── Subjects ─────────────────────────────────────────────────────────────────

pub async fn list_subjects(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<Subject>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_subjects(&*conn)?))
}

pub async fn create_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateSubjectInput>,
) -> Result<Json<Subject>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_subject(&*conn, &input)?))
}

pub async fn update_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(_id): Path<i64>,
    Json(input): Json<UpdateSubjectInput>,
) -> Result<Json<Subject>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_subject(&*conn, &input)?))
}

pub async fn delete_subject(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
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
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
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
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::set_school_optional_subject(&*conn, school_id, input.subject_id, input.enabled)?;
    Ok(Json(()))
}

// ── Faculty assignments ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FacultyAssignmentQuery {
    pub school_id: Option<i64>,
    pub faculty_user_id: Option<i64>,
}

pub async fn list_faculty_assignments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<FacultyAssignmentQuery>,
) -> Result<Json<Vec<FacultyAssignment>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_faculty_assignments(
        &*conn,
        q.school_id,
        q.faculty_user_id,
        scope_filter(&claims),
    )?))
}

pub async fn create_faculty_assignment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateFacultyAssignmentInput>,
) -> Result<Json<FacultyAssignment>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_faculty_assignment(&*conn, &input)?))
}

pub async fn delete_faculty_assignment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
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
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
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
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_timetable_slot(&*conn, &input)?))
}

pub async fn delete_timetable_slot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_timetable_slot(&*conn, id)?;
    Ok(Json(()))
}
