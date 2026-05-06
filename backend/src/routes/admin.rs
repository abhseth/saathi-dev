use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{require_admin_or_aom, scope_filter},
    db,
    error::AppError,
    models::*,
    repositories,
};

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct DasReportQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>,
    pub school_id: Option<i64>,
}

fn require_admin(claims: &Claims) -> Result<(), AppError> {
    if claims.role != "admin" {
        Err(AppError::forbidden("Admin role required"))
    } else {
        Ok(())
    }
}

// ── Users ─────────────────────────────────────────────────────────────────────

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AppUser>>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_users(&*conn)?))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<AppUser>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::create_user(
        &*conn,
        &input,
        &claims.display_name,
    )?))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(mut input): Json<UpdateUserInput>,
) -> Result<Json<AppUser>, AppError> {
    require_admin(&claims)?;
    input.id = id;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_user(
        &*conn,
        &input,
        &claims.display_name,
    )?))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let current_user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::delete_user(&*conn, id, current_user_id, &claims.display_name)?;
    Ok(Json(()))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<ChangePasswordInput>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::change_password(&*conn, user_id, &input, &claims.display_name)?;
    Ok(Json(()))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<AdminResetPasswordInput>,
) -> Result<Json<()>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    repositories::admin_reset_password(&*conn, id, &input.new_password, &claims.display_name)?;
    Ok(Json(()))
}

// ── Audit log ─────────────────────────────────────────────────────────────────

pub async fn list_audit_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Paginated<AuditLogEntry>>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let page_size = q.page_size.unwrap_or(50).clamp(1, 500);
    let offset = ((q.page.unwrap_or(1) - 1).max(0)) * page_size;
    Ok(Json(repositories::list_audit_log(
        &*conn, page_size, offset,
    )?))
}

// ── Policies ──────────────────────────────────────────────────────────────────

pub async fn list_sla_policies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SlaPolicy>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_sla_policies(&*conn)?))
}

pub async fn update_sla_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateSlaPolicyInput>,
) -> Result<Json<SlaPolicy>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_sla_policy(&*conn, &input)?))
}

pub async fn list_assignment_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AssignmentRule>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_assignment_rules(&*conn)?))
}

pub async fn update_assignment_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateAssignmentRuleInput>,
) -> Result<Json<AssignmentRule>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_assignment_rule(&*conn, &input)?))
}

pub async fn get_escalation_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<EscalationPolicy>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_escalation_policy(&*conn)?))
}

pub async fn update_escalation_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateEscalationPolicyInput>,
) -> Result<Json<EscalationPolicy>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_escalation_policy(
        &*conn, &input,
    )?))
}

pub async fn list_communication_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<CommunicationTemplate>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::list_communication_templates(&*conn)?))
}

pub async fn update_communication_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<UpdateCommunicationTemplateInput>,
) -> Result<Json<CommunicationTemplate>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::update_communication_template(
        &*conn, &input,
    )?))
}

// ── DB snapshot / restore (admin only) ───────────────────────────────────────
//
// Used to backup the live SQLite database before infrastructure changes (e.g.
// attaching a Railway volume) and to restore it after. Both endpoints require
// the admin role. The snapshot is created via SQLite's VACUUM INTO so it's a
// consistent point-in-time copy even while connections are active.

fn db_path() -> String {
    std::env::var("DATABASE_PATH").unwrap_or_else(|_| "tickets.sqlite3".to_string())
}

pub async fn db_snapshot(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, AppError> {
    require_admin(&claims)?;

    let result = tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, String), AppError> {
        let live_path = db_path();
        let temp_path = format!("{live_path}.snapshot.{}", std::process::id());

        {
            let conn = state
                .db
                .get()
                .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
            let escaped = temp_path.replace('\'', "''");
            conn.execute(&format!("VACUUM INTO '{escaped}'"), [])
                .map_err(|e| AppError::internal(format!("VACUUM INTO failed: {e}")))?;
        }

        let bytes = std::fs::read(&temp_path)
            .map_err(|e| AppError::internal(format!("read snapshot: {e}")))?;
        let _ = std::fs::remove_file(&temp_path);

        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let filename = format!("saathi-snapshot-{stamp}.sqlite3");
        Ok((bytes, filename))
    })
    .await
    .map_err(|e| AppError::internal(format!("Blocking task failed: {e}")))?;

    let (bytes, filename) = result?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(bytes),
    )
        .into_response())
}

pub async fn db_restore(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&claims)?;

    let mut bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            let b = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("Failed to read upload: {e}")))?;
            bytes = Some(b.to_vec());
            break;
        }
    }
    let bytes = bytes.ok_or_else(|| AppError::bad_request("No file field in upload"))?;

    // SQLite files start with "SQLite format 3\0"
    if bytes.len() < 16 || !bytes.starts_with(b"SQLite format 3\0") {
        return Err(AppError::bad_request(
            "Uploaded file is not a valid SQLite database",
        ));
    }

    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value, AppError> {
        let live_path = db_path();
        let staging_path = format!("{live_path}.restore.{}", std::process::id());

        std::fs::write(&staging_path, &bytes)
            .map_err(|e| AppError::internal(format!("write staging: {e}")))?;

        {
            let staged = rusqlite::Connection::open(&staging_path)
                .map_err(|e| AppError::bad_request(format!("Cannot open uploaded DB: {e}")))?;
            db::initialize_db(&staged).map_err(|e| {
                AppError::bad_request(format!("Migrations on uploaded DB failed: {e}"))
            })?;
        }

        let mut dst_conn = state
            .db
            .get()
            .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
        let src_conn = rusqlite::Connection::open(&staging_path).map_err(|e| {
            AppError::bad_request(format!("Cannot open staged DB for restore: {e}"))
        })?;

        let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
            .map_err(|e| AppError::internal(format!("Backup init failed: {e}")))?;
        backup
            .step(-1)
            .map_err(|e| AppError::internal(format!("Backup step failed: {e}")))?;
        drop(backup);
        drop(src_conn);
        let _ = std::fs::remove_file(&staging_path);

        Ok(serde_json::json!({
            "ok": true,
            "size_bytes": bytes.len(),
            "path": live_path,
        }))
    })
    .await
    .map_err(|e| AppError::internal(format!("Blocking task failed: {e}")))?;

    Ok(Json(result?))
}

// ── Reporting (Phase 4) ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DateQuery {
    pub date: Option<String>,
}

pub async fn attendance_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<DateQuery>,
) -> Result<Json<Vec<AttendanceSummaryRow>>, AppError> {
    require_admin_or_aom(&claims)?;
    let date = q
        .date
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::attendance_summary(
        &*conn,
        &date,
        scope_filter(&claims),
    )?))
}

pub async fn das_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<DasReportQuery>,
) -> Result<Json<Vec<DasReportRow>>, AppError> {
    require_admin_or_aom(&claims)?;
    if let Some(id) = q.school_id {
        crate::auth::enforce_school_scope(&claims, id)?;
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start_date = q.start_date.unwrap_or_else(|| today.clone());
    let end_date = q.end_date.unwrap_or(today);
    let group_by = q.group_by.unwrap_or_else(|| "school".to_string());
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::das_report(
        &*conn,
        &start_date,
        &end_date,
        &group_by,
        q.school_id,
        scope_filter(&claims),
    )?))
}

pub async fn chronic_absentees(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<ChronicAbsentee>>, AppError> {
    require_admin_or_aom(&claims)?;
    let since = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(30))
        .unwrap_or_else(|| chrono::Local::now())
        .format("%Y-%m-%d")
        .to_string();
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::chronic_absentees(
        &*conn,
        &since,
        scope_filter(&claims),
    )?))
}

pub async fn subject_attendance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<DateQuery>,
) -> Result<Json<Vec<SubjectAttendanceRow>>, AppError> {
    require_admin_or_aom(&claims)?;
    let date = q
        .date
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::subject_attendance(
        &*conn,
        &date,
        scope_filter(&claims),
    )?))
}
