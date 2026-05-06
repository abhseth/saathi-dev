use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{
        enforce_school_scope, require_admin_or_aom, require_faculty_or_admin_or_aom,
        require_head_or_admin_or_aom, scope_filter,
    },
    error::AppError,
    models::*,
    repositories,
};

// ── Suggest substitutes ──────────────────────────────────────────────────────

pub async fn suggest_substitutes(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<SuggestSubstitutesInput>,
) -> Result<Json<Vec<SubstituteCandidate>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, input.session_id)?;
    enforce_school_scope(&claims, school_id)?;
    let candidates =
        crate::substitution_engine::rank_substitute_candidates(&*conn, input.session_id)
            .map_err(|e| AppError::internal(e))?;
    Ok(Json(candidates))
}

// ── Leave requests ───────────────────────────────────────────────────────────

pub async fn create_leave_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(mut input): Json<CreateLeaveRequestInput>,
) -> Result<Json<LeaveRequest>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    enforce_school_scope(&claims, input.school_id)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    // Derive faculty identity from JWT for faculty callers; ignore whatever the body says
    let caller_id: i64 = claims.sub.parse().unwrap_or(0);
    if claims.role == "faculty" {
        input.faculty_user_id = caller_id;
    }

    // Validate the target faculty is actually assigned to the requested school
    if !repositories::is_faculty_at_school(&*conn, input.faculty_user_id, input.school_id)? {
        return Err(AppError::bad_request(
            "Faculty is not assigned to the selected school",
        ));
    }

    let lr = repositories::create_leave_request(&*conn, &input)?;

    // Audit log: submit
    let _ = repositories::create_leave_audit_log(
        &*conn,
        lr.id,
        caller_id,
        "submit",
        "",
        "Pending",
        "",
        lr.school_id,
    );

    // Notify approvers at this school
    if let Ok(approver_ids) = repositories::list_school_approver_user_ids(&*conn, lr.school_id) {
        for approver_id in approver_ids {
            if approver_id != caller_id {
                let _ = repositories::create_notification(
                    &*conn,
                    approver_id,
                    "leave",
                    "New leave request",
                    &format!(
                        "{} requested leave from {} to {}",
                        lr.faculty_name, lr.start_date, lr.end_date
                    ),
                );
            }
        }
    }

    Ok(Json(lr))
}

pub async fn list_leave_requests(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<LeaveRequest>>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // Faculty callers can only see their own leave requests
    let faculty_filter = if claims.role == "faculty" {
        Some(claims.sub.parse().unwrap_or(0))
    } else {
        None
    };
    Ok(Json(repositories::list_leave_requests(
        &*conn,
        scope_filter(&claims),
        faculty_filter,
    )?))
}

pub async fn approve_leave_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<LeaveRequest>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let lr = repositories::get_leave_request(&*conn, id)?;
    enforce_school_scope(&claims, lr.school_id)?;
    let approved_by_user_id: i64 = claims.sub.parse().unwrap_or(0);
    let old_status = lr.status.clone();
    let lr = repositories::approve_leave_request(&*conn, id, approved_by_user_id)?;

    // Audit log
    let _ = repositories::create_leave_audit_log(
        &*conn,
        id,
        approved_by_user_id,
        "approve",
        &old_status,
        "Approved",
        "",
        lr.school_id,
    );

    // Notify faculty
    let _ = repositories::create_notification(
        &*conn,
        lr.faculty_user_id,
        "leave",
        "Leave request approved",
        &format!(
            "Your leave request from {} to {} has been approved.",
            lr.start_date, lr.end_date
        ),
    );

    Ok(Json(lr))
}

pub async fn reject_leave_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(input): Json<RejectLeaveRequestInput>,
) -> Result<Json<LeaveRequest>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    if input.reason.trim().is_empty() {
        return Err(AppError::bad_request("Rejection reason is required"));
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let lr = repositories::get_leave_request(&*conn, id)?;
    enforce_school_scope(&claims, lr.school_id)?;
    let rejected_by_user_id: i64 = claims.sub.parse().unwrap_or(0);
    let old_status = lr.status.clone();
    let lr = repositories::reject_leave_request(&*conn, id, rejected_by_user_id, &input.reason)?;

    // Audit log
    let _ = repositories::create_leave_audit_log(
        &*conn,
        id,
        rejected_by_user_id,
        "reject",
        &old_status,
        "Rejected",
        &input.reason,
        lr.school_id,
    );

    // Notify faculty
    let _ = repositories::create_notification(
        &*conn,
        lr.faculty_user_id,
        "leave",
        "Leave request rejected",
        &format!(
            "Your leave request from {} to {} was rejected. Reason: {}",
            lr.start_date, lr.end_date, input.reason
        ),
    );

    Ok(Json(lr))
}

pub async fn get_leave_impact(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<LeaveImpactPreview>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let lr = repositories::get_leave_request(&*conn, id)?;
    enforce_school_scope(&claims, lr.school_id)?;
    Ok(Json(repositories::get_leave_impact_preview(&*conn, id)?))
}

pub async fn list_leave_audit_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<LeaveRequestAuditLog>>, AppError> {
    require_head_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let lr = repositories::get_leave_request(&*conn, id)?;
    enforce_school_scope(&claims, lr.school_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, leave_request_id, actor_user_id, action, old_status, new_status, reason, school_id, created_at
         FROM leave_request_audit_log WHERE leave_request_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| AppError::internal(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(LeaveRequestAuditLog {
                id: row.get(0)?,
                leave_request_id: row.get(1)?,
                actor_user_id: row.get(2)?,
                action: row.get(3)?,
                old_status: row.get(4)?,
                new_status: row.get(5)?,
                reason: row.get(6)?,
                school_id: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| AppError::internal(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Json(rows))
}

// ── Swap requests ────────────────────────────────────────────────────────────

pub async fn create_swap_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<CreateSwapRequestInput>,
) -> Result<Json<SwapRequest>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    // Faculty can create swap requests for themselves; admin/aom can create for anyone
    if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        if user_id != input.requester_faculty_id {
            return Err(AppError::forbidden(
                "Can only create swap requests for yourself",
            ));
        }
    }
    // Enforce school scope on both slots
    let school_a = repositories::get_timetable_slot_school_id(&*conn, input.slot_a_id)?;
    let school_b = repositories::get_timetable_slot_school_id(&*conn, input.slot_b_id)?;
    if school_a != school_b {
        return Err(AppError::bad_request("Slots must be in the same school"));
    }
    enforce_school_scope(&claims, school_a)?;
    Ok(Json(repositories::create_swap_request(&*conn, &input)?))
}

#[derive(Deserialize)]
pub struct ListSwapQuery {
    pub faculty_user_id: Option<i64>,
}

pub async fn list_swap_requests(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<ListSwapQuery>,
) -> Result<Json<Vec<SwapRequest>>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let faculty_filter = if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        Some(user_id)
    } else {
        q.faculty_user_id
    };
    Ok(Json(repositories::list_swap_requests(
        &*conn,
        scope_filter(&claims),
        faculty_filter,
    )?))
}

pub async fn accept_swap_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<SwapRequest>, AppError> {
    require_faculty_or_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let sr = repositories::get_swap_request(&*conn, id)?;
    if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        if user_id != sr.recipient_faculty_id {
            return Err(AppError::forbidden(
                "Only the recipient can accept this swap",
            ));
        }
    }
    // Enforce school scope
    let school_id = repositories::get_timetable_slot_school_id(&*conn, sr.slot_a_id)?;
    enforce_school_scope(&claims, school_id)?;
    Ok(Json(repositories::accept_swap_request(&*conn, id)?))
}

// ── Today substitutions command center ───────────────────────────────────────

pub async fn today_substitutions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<TodaySubstitutions>, AppError> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_today_substitutions(
        &*conn,
        &today,
        scope_filter(&claims),
    )?))
}

// ── Substitution detail ──────────────────────────────────────────────────────

pub async fn substitution_detail(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
) -> Result<Json<SubstitutionDetail>, AppError> {
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    Ok(Json(repositories::get_substitution_detail(
        &*conn, session_id,
    )?))
}

// ── Substitution balance ─────────────────────────────────────────────────────

pub async fn substitution_balance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(faculty_user_id): Path<i64>,
) -> Result<Json<SubstitutionBalance>, AppError> {
    if claims.role != "admin" && claims.role != "aom" {
        let user_id: i64 = claims.sub.parse().unwrap_or(0);
        if user_id != faculty_user_id {
            return Err(AppError::forbidden("Can only view your own balance"));
        }
    }
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_substitution_balance(
        &*conn,
        faculty_user_id,
    )?))
}

// ── Substitution reports ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SubstitutionReportQuery {
    pub month: String,
}

pub async fn substitution_reports(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<SubstitutionReportQuery>,
) -> Result<Json<Vec<SubstitutionReportRow>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    Ok(Json(repositories::get_substitution_reports(
        &*conn,
        &q.month,
        scope_filter(&claims),
    )?))
}

// ── Bulk attendance ──────────────────────────────────────────────────────────

pub async fn bulk_attendance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BulkAttendanceInput>,
) -> Result<Json<Vec<i64>>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    // Pre-authorize against the concrete sessions that will be mutated.
    // Admin bypasses scope checks; AOM must have access to every target school.
    if claims.role == "aom" {
        let targets = repositories::list_bulk_absence_target_schools(
            &*conn,
            &input.faculty_user_ids,
            &input.date,
        )
        .map_err(|e| AppError::internal(e))?;

        // Reject if any faculty ID has zero target sessions (bad input)
        for faculty_id in &input.faculty_user_ids {
            if !targets.iter().any(|(fid, _)| fid == faculty_id) {
                return Err(AppError::bad_request(format!(
                    "Faculty {} has no target sessions on {}",
                    faculty_id, input.date
                )));
            }
        }

        // Enforce school scope on every concrete target row
        for (_, school_id) in &targets {
            enforce_school_scope(&claims, *school_id)?;
        }
    }

    // Wrap mutation in a transaction for atomicity
    conn.execute("BEGIN", [])
        .map_err(|e| AppError::internal(format!("BEGIN failed: {e}")))?;

    let result =
        repositories::bulk_mark_faculty_absent(&*conn, &input).map_err(|e| AppError::internal(e));

    match result {
        Ok(ids) => {
            conn.execute("COMMIT", [])
                .map_err(|e| AppError::internal(format!("COMMIT failed: {e}")))?;
            Ok(Json(ids))
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

// ── Quick attendance ─────────────────────────────────────────────────────────

pub async fn mark_attendance_quick(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<MarkAttendanceQuickInput>,
) -> Result<Json<()>, AppError> {
    let user_id: i64 = claims.sub.parse().unwrap_or(0);
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, input.session_id)?;
    enforce_school_scope(&claims, school_id)?;

    let session = repositories::get_lecture_session(&*conn, input.session_id)
        .map_err(|e| AppError::not_found(e))?;
    if claims.role != "admin" && claims.role != "aom" {
        if session.actual_faculty_user_id != Some(user_id) {
            return Err(AppError::forbidden(
                "Can only mark attendance for your own sessions",
            ));
        }
    }

    repositories::mark_attendance_quick(&*conn, &input, user_id)?;
    Ok(Json(()))
}

// ── One-tap assign substitute ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AssignSubstituteInput {
    pub substitute_faculty_user_id: i64,
}

pub async fn assign_substitute(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
    Json(input): Json<AssignSubstituteInput>,
) -> Result<Json<()>, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let school_id = repositories::get_lecture_session_school_id(&*conn, session_id)?;
    enforce_school_scope(&claims, school_id)?;
    repositories::assign_substitute(&*conn, session_id, input.substitute_faculty_user_id)?;
    Ok(Json(()))
}
