use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::*,
    repositories,
};

pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Ticket>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::refresh_escalations(&*conn).map_err(|e| AppError::internal(&e))?;
    Ok(Json(repositories::list_tickets(&*conn)?))
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateTicketInput>,
) -> Result<Json<Ticket>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_ticket(&*conn, &input, &claims.display_name)?))
}

pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(mut input): Json<UpdateTicketInput>,
) -> Result<Json<Ticket>, AppError> {
    input.id = id;
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_ticket(&*conn, &input, &claims.display_name)?))
}

pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::forbidden("Only admins can delete tickets"));
    }
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_ticket(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(ticket_id): Path<i64>,
) -> Result<Json<Vec<TicketComment>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_comments(&*conn, ticket_id)?))
}

pub async fn list_all_comments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<TicketComment>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_all_comments(&*conn)?))
}

pub async fn add_comment(
    State(state): State<Arc<AppState>>,
    Path(_ticket_id): Path<i64>,
    Extension(_claims): Extension<Claims>,
    Json(input): Json<AddCommentInput>,
) -> Result<Json<TicketComment>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::add_comment(&*conn, &input)?))
}

pub async fn update_comment_status(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<i64>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateCommentStatusInput>,
) -> Result<Json<TicketComment>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_comment_status(&*conn, &input, &claims.display_name)?))
}

pub async fn list_history(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(ticket_id): Path<i64>,
) -> Result<Json<Vec<TicketHistory>>, AppError> {
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_history(&*conn, ticket_id)?))
}
