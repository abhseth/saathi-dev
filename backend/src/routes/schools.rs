use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::{enforce_school_scope, require_admin, require_admin_or_aom, scope_filter}, error::AppError, models::*, repositories};

#[derive(Deserialize)]
pub struct SchoolIdQuery {
    pub school_id: Option<i64>,
}

pub async fn list_schools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<School>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_schools(&*conn, scope_filter(&claims))?))
}

pub async fn list_dropped_schools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<School>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_dropped_schools(&*conn, scope_filter(&claims))?))
}

pub async fn create_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateSchoolInput>,
) -> Result<Json<School>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_school(&*conn, &input, &claims.display_name)?))
}

pub async fn drop_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<School>, AppError> {
    enforce_school_scope(&claims, id)?;
    let reason = body["reason"].as_str().unwrap_or("").to_string();
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::drop_school(&*conn, id, &reason, &claims.display_name)?))
}

pub async fn restore_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<School>, AppError> {
    enforce_school_scope(&claims, id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::restore_school(&*conn, id, &claims.display_name)?))
}

pub async fn delete_school(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    enforce_school_scope(&claims, id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_school(&*conn, id, &claims.display_name)?;
    Ok(Json(()))
}

pub async fn list_regions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Region>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_regions(&*conn)?))
}

pub async fn upsert_region(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertRegionInput>,
) -> Result<Json<Region>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_region(&*conn, &input, &claims.display_name)?))
}

pub async fn delete_region(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::forbidden("Only admins can delete regions"));
    }
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_region(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_students(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SchoolIdQuery>,
) -> Result<Json<Vec<Student>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_students(&*conn, q.school_id, scope_filter(&claims))?))
}

pub async fn create_student(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateStudentInput>,
) -> Result<Json<Student>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_student(&*conn, &input)?))
}

pub async fn list_lecture_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LectureModel>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_lecture_models(&*conn)?))
}

pub async fn create_lecture_model(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateLectureModelInput>,
) -> Result<Json<LectureModel>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_lecture_model(&*conn, &input)?))
}

pub async fn list_class_plans(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SchoolIdQuery>,
) -> Result<Json<Vec<SchoolClassPlan>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_school_class_plans(&*conn, q.school_id, scope_filter(&claims))?))
}

pub async fn upsert_class_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpsertSchoolClassPlanInput>,
) -> Result<Json<SchoolClassPlan>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::upsert_school_class_plan(&*conn, &input)?))
}

pub async fn program_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SchoolProgramDashboard>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_school_program_dashboard(&*conn)?))
}

pub async fn school_region_history(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SchoolRegionHistory>>, AppError> {
    require_admin(&claims)?;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_school_region_history(&*conn)?))
}
