use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{enforce_school_scope, require_ticket_writer, scope_filter},
    error::AppError,
    models::*,
    repositories,
};

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Paginated<Ticket>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let page_size = q.page_size.unwrap_or(50).clamp(1, 500);
    let offset = ((q.page.unwrap_or(1) - 1).max(0)) * page_size;
    Ok(Json(repositories::list_tickets(
        &*conn,
        scope_filter(&claims),
        page_size,
        offset,
    )?))
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateTicketInput>,
) -> Result<Json<Ticket>, AppError> {
    require_ticket_writer(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let (resolved_school_id, _) =
        repositories::resolve_ticket_school(&*conn, input.school_id, &input.school_name)
            .map_err(|e| AppError::bad_request(e))?;
    if let Some(sid) = resolved_school_id {
        enforce_school_scope(&claims, sid)?;
    }
    let ticket = repositories::create_ticket(&*conn, &input, &claims.display_name)?;
    Ok(Json(ticket))
}

pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(mut input): Json<UpdateTicketInput>,
) -> Result<Json<Ticket>, AppError> {
    require_ticket_writer(&claims)?;
    input.id = id;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let existing = match repositories::get_ticket(&*conn, id) {
        Ok(t) => t,
        Err(e) if e.contains("not found") => return Err(AppError::not_found(e)),
        Err(e) => return Err(AppError::internal(e)),
    };
    if let Some(sid) = existing.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    let (resolved_school_id, _) =
        repositories::resolve_ticket_school(&*conn, input.school_id, &input.school_name)
            .map_err(|e| AppError::bad_request(e))?;
    if let Some(sid) = resolved_school_id {
        enforce_school_scope(&claims, sid)?;
    }
    let ticket = repositories::update_ticket(&*conn, &input, &claims.display_name)?;
    Ok(Json(ticket))
}

pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::forbidden("Only admins can delete tickets"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_ticket(&*conn, id)?;
    Ok(Json(()))
}

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(ticket_id): Path<i64>,
) -> Result<Json<Vec<TicketComment>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let ticket = repositories::get_ticket(&*conn, ticket_id)?;
    if let Some(sid) = ticket.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::list_comments(&*conn, ticket_id)?))
}

pub async fn list_all_comments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Paginated<TicketComment>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let page_size = q.page_size.unwrap_or(50).clamp(1, 500);
    let offset = ((q.page.unwrap_or(1) - 1).max(0)) * page_size;
    Ok(Json(repositories::list_all_comments(
        &*conn,
        scope_filter(&claims),
        page_size,
        offset,
    )?))
}

pub async fn add_comment(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<i64>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<AddCommentInput>,
) -> Result<Json<TicketComment>, AppError> {
    require_ticket_writer(&claims)?;
    if input.ticket_id != ticket_id {
        return Err(AppError::bad_request("Body ticket_id does not match path"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let ticket = repositories::get_ticket(&*conn, ticket_id)?;
    if let Some(sid) = ticket.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::add_comment(&*conn, &input)?))
}

pub async fn update_comment_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateCommentStatusInput>,
) -> Result<Json<TicketComment>, AppError> {
    require_ticket_writer(&claims)?;
    if input.id != id {
        return Err(AppError::bad_request("Body id does not match path"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let comment = repositories::get_comment(&*conn, id)?;
    let ticket = repositories::get_ticket(&*conn, comment.ticket_id)?;
    if let Some(sid) = ticket.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::update_comment_status(
        &*conn,
        &input,
        &claims.display_name,
    )?))
}

pub async fn list_history(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(ticket_id): Path<i64>,
) -> Result<Json<Vec<TicketHistory>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let ticket = repositories::get_ticket(&*conn, ticket_id)?;
    if let Some(sid) = ticket.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    Ok(Json(repositories::list_history(&*conn, ticket_id)?))
}

pub async fn refresh_escalations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::forbidden("Only admins can refresh SLA status"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    match repositories::refresh_escalations(&*conn) {
        Ok(count) => Ok(Json(serde_json::json!({ "updated": count }))),
        Err(e) => Err(AppError::internal(e)),
    }
}
