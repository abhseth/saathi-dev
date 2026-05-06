use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{analytics, auth::scope_filter, error::AppError, models::*};

// ── Query structs ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ComplianceScorecardQuery {
    pub school_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct FacultyUtilizationQuery {
    pub faculty_id: Option<i64>,
    #[serde(default = "default_weeks_4")]
    pub weeks: i64,
}

fn default_weeks_4() -> i64 {
    4
}

#[derive(Deserialize)]
pub struct SessionTypeBreakdownQuery {
    pub school_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct HealthTrendsQuery {
    #[serde(default = "default_weeks_8")]
    pub weeks: i64,
}

fn default_weeks_8() -> i64 {
    8
}

#[derive(Deserialize)]
pub struct SubstitutionTrendsQuery {
    #[serde(default = "default_weeks_4")]
    pub weeks: i64,
}

#[derive(Deserialize)]
pub struct RoomConflictsRadarQuery {
    pub school_id: Option<i64>,
    pub week_start: String,
}

#[derive(Deserialize)]
pub struct WeekDiffQuery {
    pub school_id: i64,
    pub week_a: String,
    pub week_b: String,
}

#[derive(Deserialize)]
pub struct CompliancePivotQuery {
    pub pivot: String, // "subject" | "school" | "region"
}

// ── Route handlers ───────────────────────────────────────────────────────────

pub async fn compliance_scorecard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<ComplianceScorecardQuery>,
) -> Result<Json<Vec<ActionableComplianceItem>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if let Some(sid) = q.school_id {
        crate::auth::enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(analytics::compliance_scorecard(
        &*conn,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn control_tower(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<ControlTowerCard>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::control_tower(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn faculty_utilization(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<FacultyUtilizationQuery>,
) -> Result<Json<Vec<FacultyUtilizationTrend>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::faculty_utilization_trend(
        &*conn,
        q.faculty_id,
        q.weeks,
        scope_filter(&claims),
    )?))
}

pub async fn deviation_scoreboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<DeviationScoreboardRow>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::deviation_scoreboard(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn session_type_breakdown(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SessionTypeBreakdownQuery>,
) -> Result<Json<Vec<SessionTypeBreakdown>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if let Some(sid) = q.school_id {
        crate::auth::enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(analytics::session_type_breakdown(
        &*conn,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn faculty_stability(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FacultyStabilityRow>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::faculty_stability(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn subject_coverage_heatmap(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SubjectCoverageCell>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::subject_coverage_heatmap(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn health_trends(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<HealthTrendsQuery>,
) -> Result<Json<Vec<HealthTrendWeek>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::health_trends(
        &*conn,
        q.weeks,
        scope_filter(&claims),
    )?))
}

pub async fn substitution_trends(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SubstitutionTrendsQuery>,
) -> Result<Json<Vec<SubstitutionTrendWeek>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::substitution_trends(
        &*conn,
        q.weeks,
        scope_filter(&claims),
    )?))
}

pub async fn region_heatmap(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<RegionHeatmapCell>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::region_heatmap(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn room_conflicts_radar(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<RoomConflictsRadarQuery>,
) -> Result<Json<Vec<RoomConflictRadarCell>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    if let Some(sid) = q.school_id {
        crate::auth::enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(analytics::room_conflicts_radar(
        &*conn,
        q.school_id,
        &q.week_start,
        scope_filter(&claims),
    )?))
}

pub async fn adherence_comparison(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AdherenceComparisonRow>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::adherence_comparison(
        &*conn,
        scope_filter(&claims),
    )?))
}

pub async fn week_diff(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<WeekDiffQuery>,
) -> Result<Json<Vec<WeekDiffSlot>>, AppError> {
    crate::auth::enforce_school_scope(&claims, q.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(analytics::week_diff(
        &*conn,
        q.school_id,
        &q.week_a,
        &q.week_b,
    )?))
}

pub async fn compliance_pivot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<CompliancePivotQuery>,
) -> Result<Json<Vec<CompliancePivotRow>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let pivot = match q.pivot.as_str() {
        "school" | "region" => q.pivot.as_str(),
        _ => "subject",
    };
    Ok(Json(analytics::compliance_pivot(
        &*conn,
        pivot,
        scope_filter(&claims),
    )?))
}
