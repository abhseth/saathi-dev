use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{error::AppError, models::*};

#[derive(Deserialize)]
pub struct CreateNotificationInput {
    pub user_id: i64,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    #[serde(default)]
    pub payload_json: String,
}

pub async fn list_notifications(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Notification>>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, type, title, message, payload_json, read_at, created_at FROM notification_log WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 200"
    ).map_err(|e| AppError::internal(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(Notification {
                id: row.get(0)?,
                user_id: row.get(1)?,
                notification_type: row.get(2)?,
                title: row.get(3)?,
                message: row.get(4)?,
                payload_json: row.get(5)?,
                read_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::internal(e.to_string()))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| AppError::internal(e.to_string()))?);
    }
    Ok(Json(items))
}

pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notification_log WHERE user_id = ?1 AND read_at = ''",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "count": count })))
}

pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    conn.execute(
        "UPDATE notification_log SET read_at = datetime('now', 'localtime') WHERE id = ?1 AND user_id = ?2",
        rusqlite::params![id, user_id],
    ).map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(()))
}

pub async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    conn.execute(
        "UPDATE notification_log SET read_at = datetime('now', 'localtime') WHERE user_id = ?1 AND read_at = ''",
        rusqlite::params![user_id],
    ).map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(()))
}

pub async fn create_notification(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateNotificationInput>,
) -> Result<Json<Notification>, AppError> {
    if claims.role != "admin" && claims.role != "aom" {
        return Err(AppError::forbidden("Only admins can create notifications"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    conn.execute(
        "INSERT INTO notification_log (user_id, type, title, message, payload_json) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![input.user_id, input.notification_type, input.title, input.message, input.payload_json],
    ).map_err(|e| AppError::internal(e.to_string()))?;
    let id = conn.last_insert_rowid();
    let notification = conn.query_row(
        "SELECT id, user_id, type, title, message, payload_json, read_at, created_at FROM notification_log WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Notification {
                id: row.get(0)?,
                user_id: row.get(1)?,
                notification_type: row.get(2)?,
                title: row.get(3)?,
                message: row.get(4)?,
                payload_json: row.get(5)?,
                read_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    ).map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(notification))
}
