use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{enforce_school_scope, require_admin, require_admin_or_aom, scope_filter},
    error::AppError,
    models::*,
    repositories,
};

#[derive(Deserialize)]
pub struct SchoolIdQuery {
    pub school_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct StudentListQuery {
    pub school_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

pub async fn list_schools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<School>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_schools(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn list_dropped_schools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<School>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_dropped_schools(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn create_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateSchoolInput>,
) -> Result<Json<School>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_school(
        &*conn,
        &input,
        &claims.display_name,
    )?))
}

pub async fn drop_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<School>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, id)?;
    let reason = body["reason"].as_str().unwrap_or("").to_string();
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::drop_school(
        &*conn,
        id,
        &reason,
        &claims.display_name,
    )?))
}

pub async fn restore_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<School>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::restore_school(
        &*conn,
        id,
        &claims.display_name,
    )?))
}

pub async fn school_delete_impact(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<SchoolDeleteImpact>, AppError> {
    require_admin(&claims)?;
    enforce_school_scope(&claims, id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_school_delete_impact(&*conn, id)?))
}

pub async fn delete_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    enforce_school_scope(&claims, id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_school(&*conn, id, &claims.display_name)?;
    Ok(Json(()))
}

pub async fn list_regions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Region>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_regions(&*conn)?))
}

pub async fn upsert_region(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertRegionInput>,
) -> Result<Json<Region>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_region(
        &*conn,
        &input,
        &claims.display_name,
    )?))
}

pub async fn delete_region(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_region(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_students(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<StudentListQuery>,
) -> Result<Json<Paginated<Student>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(100).clamp(1, 250);
    Ok(Json(repositories::list_students_paginated(
        &*conn,
        q.school_id,
        scope_filter(&claims),
        q.search.as_deref(),
        page_size,
        (page - 1) * page_size,
    )?))
}

pub async fn get_student_timeline(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<StudentTimeline>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let student = repositories::get_student(&*conn, id)?;
    enforce_school_scope(&claims, student.school_id)?;
    Ok(Json(repositories::get_student_timeline(&*conn, id)?))
}

pub async fn create_student(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateStudentInput>,
) -> Result<Json<Student>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if input.batch_ref_id > 0 {
        let batch = repositories::get_batch(&*conn, input.batch_ref_id)?;
        enforce_school_scope(&claims, batch.school_id)?;
    } else {
        enforce_school_scope(&claims, input.school_id)?;
    }
    Ok(Json(repositories::create_student(&*conn, &input)?))
}

pub async fn update_student(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateStudentInput>,
) -> Result<Json<Student>, AppError> {
    if input.id != id {
        return Err(AppError::bad_request("Body id does not match path"));
    }
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let existing = repositories::get_student(&*conn, id)?;
    enforce_school_scope(&claims, existing.school_id)?;
    if input.batch_ref_id > 0 {
        let batch = repositories::get_batch(&*conn, input.batch_ref_id)?;
        enforce_school_scope(&claims, batch.school_id)?;
    } else {
        enforce_school_scope(&claims, input.school_id)?;
    }
    Ok(Json(repositories::update_student(&*conn, &input)?))
}

pub async fn delete_student(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let student = repositories::get_student(&*conn, id)?;
    enforce_school_scope(&claims, student.school_id)?;
    repositories::delete_student(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SchoolIdQuery>,
) -> Result<Json<Vec<Batch>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_batches(
        &*conn,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateBatchInput>,
) -> Result<Json<Batch>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_batch(&*conn, &input)?))
}

pub async fn update_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(mut input): Json<UpdateBatchInput>,
) -> Result<Json<Batch>, AppError> {
    require_admin_or_aom(&claims)?;
    input.id = id;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let existing = repositories::get_batch(&*conn, id)?;
    enforce_school_scope(&claims, existing.school_id)?;
    enforce_school_scope(&claims, input.school_id)?;
    Ok(Json(repositories::update_batch(&*conn, &input)?))
}

pub async fn archive_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let existing = repositories::get_batch(&*conn, id)?;
    enforce_school_scope(&claims, existing.school_id)?;
    repositories::archive_batch(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_lecture_models(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<LectureModel>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_lecture_models(&*conn)?))
}

pub async fn create_lecture_model(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateLectureModelInput>,
) -> Result<Json<LectureModel>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_lecture_model(&*conn, &input)?))
}

pub async fn list_class_plans(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SchoolIdQuery>,
) -> Result<Json<Vec<SchoolClassPlan>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_school_class_plans(
        &*conn,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn upsert_class_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertSchoolClassPlanInput>,
) -> Result<Json<SchoolClassPlan>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_school_class_plan(
        &*conn, &input,
    )?))
}

pub async fn program_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SchoolProgramDashboard>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_school_program_dashboard(&*conn)?))
}

pub async fn school_region_history(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SchoolRegionHistory>>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_school_region_history(&*conn)?))
}
