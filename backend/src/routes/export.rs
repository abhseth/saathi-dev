use axum::{
    extract::{Extension, State},
    http::header,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{
    auth::{require_admin_or_aom, scope_filter},
    error::AppError,
    models::{AppState, Claims},
    repositories,
};

fn field(s: &str) -> String {
    // Defuse formula injection first, before CSV escaping
    let needs_defuse = s.starts_with(|c| c == '=' || c == '+' || c == '-' || c == '@' || c == '\t');
    let mut result = if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    };
    // If the original started with a formula trigger, prefix with single quote
    // (this covers both plain and quoted fields)
    if needs_defuse {
        result.insert(0, '\'');
    }
    result
}

pub async fn tickets_csv(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let tickets = repositories::list_tickets(&*conn, scope_filter(&claims), 10_000, 0)?.items;

    let mut csv = String::from(
        "ID,Title,Requester,Assignee,Status,Priority,Queue,School,Student,Grade Level,Program Track,Issue Category,SLA Due,Escalation Status,Created At,Updated At\n"
    );
    for t in &tickets {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            t.id,
            field(&t.title),
            field(&t.requester),
            field(&t.assignee),
            field(&t.status),
            field(&t.priority),
            field(&t.queue),
            field(&t.school_name),
            field(&t.student_name),
            field(&t.grade_level),
            field(&t.program_track),
            field(&t.issue_category),
            field(&t.sla_due_at),
            field(&t.escalation_status),
            field(&t.created_at),
            field(&t.updated_at),
        ));
    }

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"tickets.csv\"",
            ),
        ],
        csv,
    )
        .into_response())
}

pub async fn communications_csv(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let comments = repositories::list_all_comments(&*conn, scope_filter(&claims), 10_000, 0)?.items;

    let mut csv = String::from(
        "ID,Ticket ID,Author,Body,Channel,Audience,Recipient Name,Recipient Contact,Delivery Status,Last Contacted At,Next Follow Up Due,Created At\n"
    );
    for c in &comments {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            c.id,
            c.ticket_id,
            field(&c.author),
            field(&c.body),
            field(&c.channel),
            field(&c.audience),
            field(&c.recipient_name),
            field(&c.recipient_contact),
            field(&c.delivery_status),
            field(&c.last_contacted_at),
            field(&c.next_follow_up_due),
            field(&c.created_at),
        ));
    }

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"communications.csv\"",
            ),
        ],
        csv,
    )
        .into_response())
}

pub async fn sip_master_csv(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, AppError> {
    require_admin_or_aom(&claims)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let schools = repositories::list_schools(&*conn, scope_filter(&claims))?;

    let mut csv = String::from(
        "School Name,Region,Program Model,Distance Classification,\
SIP Academic Owner Role,SIP Academic Owner Name,SIP Academic Owner Mobile,SIP Academic Owner Email,\
Center Head Name,Center Head Mobile,Center Head Email,\
Principal Name,Principal Mobile,Principal Email,\
School SPOC Name,School SPOC Mobile,School SPOC Email,\
Central Academic SPOC Name,Central Academic SPOC Mobile,Central Academic SPOC Email,\
Central Business SPOC Name,Central Business SPOC Mobile,Central Business SPOC Email,\
BH Name,BH Mobile,BH Email,AOM Name,AOM Mobile,AOM Email\n",
    );
    for s in &schools {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            field(&s.name), field(&s.region_name), field(&s.program_model),
            field(&s.distance_classification),
            field(&s.sip_academic_owner_role), field(&s.sip_academic_owner_name),
            field(&s.sip_academic_owner_mobile), field(&s.sip_academic_owner_email),
            field(&s.center_head_name), field(&s.center_head_mobile), field(&s.center_head_email),
            field(&s.principal_name), field(&s.principal_mobile), field(&s.principal_email),
            field(&s.school_spoc_name), field(&s.school_spoc_mobile), field(&s.school_spoc_email),
            field(&s.central_academic_spoc_name), field(&s.central_academic_spoc_mobile),
            field(&s.central_academic_spoc_email), field(&s.central_business_spoc_name),
            field(&s.central_business_spoc_mobile), field(&s.central_business_spoc_email),
            field(&s.bh_name), field(&s.bh_mobile), field(&s.bh_email),
            field(&s.aom_name), field(&s.aom_mobile), field(&s.aom_email),
        ));
    }

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"sip-master.csv\"",
            ),
        ],
        csv,
    )
        .into_response())
}
