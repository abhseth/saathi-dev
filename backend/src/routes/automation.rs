use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::{
    alerts,
    auth::{
        enforce_school_scope, require_admin, require_admin_or_aom, require_ticket_writer,
        scope_filter,
    },
    bulk_ops, digests,
    error::AppError,
    escalation,
    models::*,
    policies, repositories,
};
use rusqlite::params;

fn alert_hash(alert: &Alert) -> String {
    let mut hasher = DefaultHasher::new();
    alert.category.hash(&mut hasher);
    alert.message.hash(&mut hasher);
    alert.school_id.hash(&mut hasher);
    alert.grade_level.hash(&mut hasher);
    alert.subject_name.hash(&mut hasher);
    alert.faculty_user_id.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// ── Policies ─────────────────────────────────────────────────────────────────

pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<CentralPolicy>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(policies::list_policies(&*conn)?))
}

pub async fn update_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
    Json(input): Json<UpsertPolicyInput>,
) -> Result<Json<CentralPolicy>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(policies::upsert_policy(
        &*conn,
        &key,
        &input.value,
        input.region_id,
    )?))
}

// ── Escalation Rules ─────────────────────────────────────────────────────────

pub async fn list_escalation_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<EscalationRule>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(escalation::list_escalation_rules(&*conn)?))
}

pub async fn create_escalation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateEscalationRuleInput>,
) -> Result<Json<EscalationRule>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(escalation::create_escalation_rule(&*conn, &input)?))
}

pub async fn update_escalation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateEscalationRuleInput>,
) -> Result<Json<EscalationRule>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut updated = input;
    updated.id = id;
    Ok(Json(escalation::update_escalation_rule(&*conn, &updated)?))
}

// ── Digests ──────────────────────────────────────────────────────────────────

pub async fn intervention_digest(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<InterventionDigest>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let digest = digests::generate_intervention_digest(&*conn)?;
    // TODO: stubbed email sending — wire to email service here
    Ok(Json(digest))
}

pub async fn sip_brief(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SipBrief>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let brief = digests::generate_sip_brief(&*conn)?;
    // TODO: stubbed email sending — wire to email service here
    Ok(Json(brief))
}

// ── Alert Inbox ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AlertInboxQuery {
    pub user_id: Option<i64>,
}

pub async fn alert_inbox(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AlertInboxQuery>,
) -> Result<Json<Vec<Alert>>, AppError> {
    let user_id = q.user_id.unwrap_or_else(|| claims.sub.parse().unwrap_or(0));
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut alerts =
        alerts::get_all_alerts(&*conn, scope_filter(&claims)).map_err(|e| AppError::internal(e))?;

    // Filter out dismissed / snoozed
    let dismissed: Vec<String> = conn
        .prepare("SELECT alert_hash FROM alert_states WHERE user_id = ?1 AND dismissed_at != ''")
        .map_err(|e| AppError::internal(e.to_string()))?
        .query_map(params![user_id], |row| row.get::<_, String>(0))
        .map_err(|e| AppError::internal(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::internal(e.to_string()))?;

    let snoozed: Vec<String> = conn.prepare(
        "SELECT alert_hash FROM alert_states WHERE user_id = ?1 AND snoozed_until > datetime('now', 'localtime')"
    ).map_err(|e| AppError::internal(e.to_string()))?
    .query_map(params![user_id], |row| row.get::<_, String>(0))
    .map_err(|e| AppError::internal(e.to_string()))?
    .collect::<Result<Vec<_>, _>>().map_err(|e| AppError::internal(e.to_string()))?;

    let dismissed_set: std::collections::HashSet<String> = dismissed.into_iter().collect();
    let snoozed_set: std::collections::HashSet<String> = snoozed.into_iter().collect();

    for alert in &mut alerts {
        let hash = alert_hash(alert);
        alert.id = hash.clone();
        if dismissed_set.contains(&hash) || snoozed_set.contains(&hash) {
            alert.severity = "dismissed".to_string();
        }
    }

    alerts.retain(|a| a.severity != "dismissed");
    Ok(Json(alerts))
}

pub async fn dismiss_alert(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(hash): Path<String>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    conn.execute(
        "INSERT INTO alert_states (alert_hash, user_id, dismissed_at, snoozed_until, converted_ticket_id, created_at, updated_at)
         VALUES (?1, ?2, datetime('now', 'localtime'), '', NULL, datetime('now', 'localtime'), datetime('now', 'localtime'))
         ON CONFLICT(alert_hash, user_id) DO UPDATE SET dismissed_at = excluded.dismissed_at, snoozed_until = '', updated_at = excluded.updated_at",
        params![hash, user_id],
    ).map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(()))
}

pub async fn bulk_alert_action(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkAlertActionInput>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    for hash in &input.ids {
        match input.action.as_str() {
            "dismiss" => {
                conn.execute(
                    "INSERT INTO alert_states (alert_hash, user_id, dismissed_at, snoozed_until, converted_ticket_id, created_at, updated_at)
                     VALUES (?1, ?2, datetime('now', 'localtime'), '', NULL, datetime('now', 'localtime'), datetime('now', 'localtime'))
                     ON CONFLICT(alert_hash, user_id) DO UPDATE SET dismissed_at = excluded.dismissed_at, snoozed_until = '', updated_at = excluded.updated_at",
                    params![hash, user_id],
                ).map_err(|e| AppError::internal(e.to_string()))?;
            }
            "snooze" => {
                let hours = input.snooze_hours.unwrap_or(24).clamp(0, 720);
                let sql = "INSERT INTO alert_states (alert_hash, user_id, dismissed_at, snoozed_until, converted_ticket_id, created_at, updated_at)
                     VALUES (?1, ?2, '', datetime('now', '+' || ?3 || ' hours', 'localtime'), NULL, datetime('now', 'localtime'), datetime('now', 'localtime'))
                     ON CONFLICT(alert_hash, user_id) DO UPDATE SET dismissed_at = '', snoozed_until = excluded.snoozed_until, updated_at = excluded.updated_at";
                conn.execute(&sql, params![hash, user_id, hours])
                    .map_err(|e| AppError::internal(e.to_string()))?;
            }
            "ticket" => {
                require_ticket_writer(&claims)?;
                // Create a placeholder ticket for the alert
                let ticket = repositories::create_ticket(
                    &*conn,
                    &CreateTicketInput {
                        title: format!("Alert: {}", hash),
                        description: format!("Auto-created from alert hash {}", hash),
                        requester: claims.username.clone(),
                        priority: "Medium".to_string(),
                        school_id: None,
                        school_name: "".to_string(),
                        student_name: "".to_string(),
                        grade_level: "".to_string(),
                        program_track: "".to_string(),
                        issue_category: "Academic Support".to_string(),
                    },
                    &claims.username,
                )?;
                conn.execute(
                    "INSERT INTO alert_states (alert_hash, user_id, dismissed_at, snoozed_until, converted_ticket_id, created_at, updated_at)
                     VALUES (?1, ?2, datetime('now', 'localtime'), '', ?3, datetime('now', 'localtime'), datetime('now', 'localtime'))
                     ON CONFLICT(alert_hash, user_id) DO UPDATE SET dismissed_at = excluded.dismissed_at, snoozed_until = '', converted_ticket_id = excluded.converted_ticket_id, updated_at = excluded.updated_at",
                    params![hash, user_id, ticket.id],
                ).map_err(|e| AppError::internal(e.to_string()))?;
            }
            _ => return Err(AppError::bad_request("Unknown action")),
        }
    }
    Ok(Json(()))
}

// ── Ticket from Gap ──────────────────────────────────────────────────────────

pub async fn ticket_from_gap(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<TicketFromGapInput>,
) -> Result<Json<Ticket>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_name: String = conn
        .query_row(
            "SELECT name FROM schools WHERE id = ?1",
            params![input.school_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::not_found(e.to_string()))?;

    let ticket = repositories::create_ticket(
        &*conn,
        &CreateTicketInput {
            title: format!("Gap: {} - {}", input.grade_level, input.subject_name),
            description: input.gap_description,
            requester: claims.username.clone(),
            priority: "High".to_string(),
            school_id: Some(input.school_id),
            school_name,
            student_name: "".to_string(),
            grade_level: input.grade_level,
            program_track: input.track,
            issue_category: "Academic Support".to_string(),
        },
        &claims.username,
    )?;
    Ok(Json(ticket))
}

// ── Bulk Operations ──────────────────────────────────────────────────────────

pub async fn bulk_assign_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkAssignUsersInput>,
) -> Result<Json<BulkOperationLog>, AppError> {
    require_admin(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::bulk_assign_users(&*conn, &input)?))
}

pub async fn bulk_import_subjects(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkImportSubjectsInput>,
) -> Result<Json<BulkOperationLog>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::bulk_import_subjects(&*conn, &input)?))
}

pub async fn bulk_publish_timetables(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkPublishTimetablesInput>,
) -> Result<Json<BulkOperationLog>, AppError> {
    require_admin_or_aom(&claims)?;
    for school_id in &input.school_ids {
        enforce_school_scope(&claims, *school_id)?;
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::bulk_publish_timetables(&*conn, &input)?))
}

// ── Faculty Reassign ─────────────────────────────────────────────────────────

pub async fn reassign_faculty(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<ReassignFacultyInput>,
) -> Result<Json<ReassignFacultyResult>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.source_school_id)?;
    enforce_school_scope(&claims, input.target_school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::reassign_faculty(&*conn, &input)?))
}

// ── Week Clone ───────────────────────────────────────────────────────────────

pub async fn clone_week_with_check(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CloneWeekInput>,
) -> Result<Json<CloneWeekResult>, AppError> {
    require_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::clone_week_with_check(&*conn, &input)?))
}

// ── Announcements ────────────────────────────────────────────────────────────

pub async fn list_announcements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let scope = scope_filter(&claims);
    let mut sql = String::from(
        "SELECT a.id, a.school_id, s.name, a.message, a.pinned_until, a.created_by, u.display_name, a.created_at
         FROM announcements a
         LEFT JOIN schools s ON s.id = a.school_id
         LEFT JOIN users u ON u.id = a.created_by
         WHERE (a.pinned_until = '' OR a.pinned_until >= date('now', 'localtime'))"
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(
                " AND (a.school_id IS NULL OR a.school_id IN ({placeholders}))"
            ));
            for id in ids {
                params_vec.push(id);
            }
        } else {
            sql.push_str(" AND a.school_id IS NULL");
        }
    }
    sql.push_str(" ORDER BY a.created_at DESC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::internal(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(Announcement {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                message: row.get(3)?,
                pinned_until: row.get(4)?,
                created_by: row.get(5)?,
                created_by_name: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::internal(e.to_string()))?;
    let result: Vec<Announcement> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(result))
}

pub async fn create_announcement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateAnnouncementInput>,
) -> Result<Json<Announcement>, AppError> {
    require_admin_or_aom(&claims)?;
    if let Some(sid) = input.school_id {
        enforce_school_scope(&claims, sid)?;
    }
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    conn.execute(
        "INSERT INTO announcements (school_id, message, pinned_until, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now', 'localtime'))",
        params![input.school_id, input.message, input.pinned_until, user_id],
    )
    .map_err(|e| AppError::internal(e.to_string()))?;
    let id = conn.last_insert_rowid();
    Ok(Json(Announcement {
        id,
        school_id: input.school_id,
        school_name: None,
        message: input.message,
        pinned_until: input.pinned_until,
        created_by: user_id,
        created_by_name: claims.display_name,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ── Cross-School Room Conflicts ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RoomConflictsQuery {
    pub week_start: String,
}

pub async fn cross_school_room_conflicts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<RoomConflictsQuery>,
) -> Result<Json<Vec<CrossSchoolRoomConflict>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(bulk_ops::list_cross_school_room_conflicts(
        &*conn,
        &q.week_start,
        scope_filter(&claims),
    )?))
}
