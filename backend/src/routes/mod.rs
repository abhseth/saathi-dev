use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::{auth::require_auth, models::AppState};

mod admin;
mod analytics;
mod auth;
mod automation;
mod export;
mod faculty;
mod imports;
mod notifications;
mod schools;
mod substitutions;
mod tickets;

pub fn router(state: Arc<AppState>) -> Router {
    // Public routes (no auth required)
    let public = Router::new().route("/auth/login", post(auth::login));

    // Protected routes (require valid JWT)
    let protected = Router::new()
        // Auth
        .route("/auth/me", get(auth::me))
        // Tickets
        .route("/tickets", get(tickets::list_tickets))
        .route("/tickets", post(tickets::create_ticket))
        .route("/tickets/:id", put(tickets::update_ticket))
        .route("/tickets/:id", delete(tickets::delete_ticket))
        .route("/tickets/:id/comments", get(tickets::list_comments))
        .route("/tickets/:id/comments", post(tickets::add_comment))
        .route("/tickets/:id/history", get(tickets::list_history))
        .route(
            "/tickets/refresh-escalations",
            post(tickets::refresh_escalations),
        )
        .route("/comments/:id/status", put(tickets::update_comment_status))
        .route("/comments", get(tickets::list_all_comments))
        // Schools
        .route("/schools", get(schools::list_schools))
        .route("/schools", post(schools::create_school))
        .route("/schools/:id", put(schools::update_school))
        .route("/schools/:id/drop", post(schools::drop_school))
        .route("/schools/:id/restore", post(schools::restore_school))
        .route("/schools/:id/delete-impact", get(schools::school_delete_impact))
        .route("/schools/:id", delete(schools::delete_school))
        .route("/schools/dropped", get(schools::list_dropped_schools))
        .route("/regions", get(schools::list_regions))
        .route("/regions", post(schools::upsert_region))
        .route("/regions/:id", delete(schools::delete_region))
        .route("/students", get(schools::list_students))
        .route("/students", post(schools::create_student))
        .route("/students/:id", get(schools::get_student_timeline))
        .route("/students/:id", put(schools::update_student))
        .route("/students/:id", delete(schools::delete_student))
        .route("/batches", get(schools::list_batches))
        .route("/batches", post(schools::create_batch))
        .route("/batches/:id", get(schools::get_batch))
        .route("/batches/:id", put(schools::update_batch))
        .route("/batches/:id", delete(schools::archive_batch))
        .route("/batches/:id/students", get(schools::get_batch_students))
        .route("/batch-analytics", get(schools::batch_analytics))
        .route("/lecture-models", get(schools::list_lecture_models))
        .route("/lecture-models", post(schools::create_lecture_model))
        .route("/class-plans", get(schools::list_class_plans))
        .route("/class-plans", post(schools::upsert_class_plan))
        .route("/program-dashboard", get(schools::program_dashboard))
        .route(
            "/school-region-history",
            get(schools::school_region_history),
        )
        // Admin
        .route("/users", get(admin::list_users))
        .route("/users", post(admin::create_user))
        .route("/users/:id", put(admin::update_user))
        .route("/users/:id", delete(admin::delete_user))
        .route("/users/password", put(admin::change_password))
        .route("/users/:id/reset-password", post(admin::reset_password))
        .route("/audit-log", get(admin::list_audit_log))
        .route("/sla-policies", get(admin::list_sla_policies))
        .route("/sla-policies", put(admin::update_sla_policy))
        .route("/assignment-rules", get(admin::list_assignment_rules))
        .route("/assignment-rules", put(admin::update_assignment_rule))
        .route("/escalation-policy", get(admin::get_escalation_policy))
        .route("/escalation-policy", put(admin::update_escalation_policy))
        .route(
            "/communication-templates",
            get(admin::list_communication_templates),
        )
        .route(
            "/communication-templates",
            put(admin::update_communication_template),
        )
        // DB backup / restore (admin only) — used for volume migration + ad-hoc snapshots
        .route("/admin/db-snapshot", get(admin::db_snapshot))
        .route("/admin/db-restore", post(admin::db_restore))
        // Reporting (Phase 4)
        .route(
            "/reports/attendance-summary",
            get(admin::attendance_summary),
        )
        .route("/reports/das", get(admin::das_report))
        .route("/reports/chronic-absentees", get(admin::chronic_absentees))
        .route(
            "/reports/subject-attendance",
            get(admin::subject_attendance),
        )
        // Exports
        .route("/export/tickets.csv", get(export::tickets_csv))
        .route(
            "/export/communications.csv",
            get(export::communications_csv),
        )
        .route("/export/sip-master.csv", get(export::sip_master_csv))
        // Imports
        .route("/imports/schools.csv", post(imports::import_schools_csv))
        .route("/imports/students.csv", post(imports::import_students_csv))
        .route(
            "/imports/timetable.csv",
            post(imports::import_timetable_csv),
        )
        .route(
            "/imports/faculty-members.csv",
            post(imports::import_faculty_members_csv),
        )
        .route(
            "/imports/sip-master/preview",
            post(imports::preview_sip_master_import),
        )
        .route("/imports/sip-master", post(imports::import_sip_master))
        // VP Centers & Faculty Profiles
        .route("/vp-centers", get(faculty::list_vp_centers))
        .route("/vp-centers", post(faculty::create_vp_center))
        .route("/vp-centers/:id", put(faculty::update_vp_center))
        .route("/vp-centers/:id", delete(faculty::delete_vp_center))
        .route(
            "/vp-centers/:id/buildings",
            get(faculty::list_vp_center_buildings),
        )
        .route(
            "/vp-center-buildings",
            post(faculty::create_vp_center_building),
        )
        .route(
            "/vp-center-buildings/:id",
            put(faculty::update_vp_center_building),
        )
        .route(
            "/vp-center-buildings/:id",
            delete(faculty::delete_vp_center_building),
        )
        .route("/faculty-profiles", get(faculty::list_faculty_profiles))
        .route(
            "/faculty-profiles/:faculty_user_id",
            get(faculty::get_faculty_profile),
        )
        .route("/faculty-profiles", post(faculty::upsert_faculty_profile))
        // Faculty Members (master data, optional login)
        .route("/faculty-members", get(faculty::list_faculty_members))
        .route("/faculty-members", post(faculty::create_faculty_member))
        .route("/faculty-members/:id", put(faculty::update_faculty_member))
        .route(
            "/faculty-members/:id",
            delete(faculty::delete_faculty_member),
        )
        .route(
            "/faculty-members/:id/memberships",
            get(faculty::list_faculty_school_memberships),
        )
        .route(
            "/faculty-school-memberships",
            post(faculty::create_faculty_school_membership),
        )
        .route(
            "/faculty-school-memberships/:id",
            delete(faculty::delete_faculty_school_membership),
        )
        .route(
            "/faculty-members/:id/create-login",
            post(faculty::create_faculty_login),
        )
        .route(
            "/faculty-members/:id/link-user",
            post(faculty::link_faculty_user),
        )
        // Faculty / timetable / subjects (Phase 1)
        .route("/subjects", get(faculty::list_subjects))
        .route("/subjects", post(faculty::create_subject))
        .route("/subjects/:id", put(faculty::update_subject))
        .route("/subjects/:id", delete(faculty::delete_subject))
        .route(
            "/schools/:id/effective-subjects",
            get(faculty::list_effective_subjects),
        )
        .route(
            "/schools/:id/optional-subjects",
            post(faculty::set_school_optional_subject),
        )
        .route(
            "/faculty-assignments",
            get(faculty::list_faculty_assignments),
        )
        .route(
            "/faculty-assignments",
            post(faculty::create_faculty_assignment),
        )
        .route(
            "/faculty-assignments/:id",
            delete(faculty::delete_faculty_assignment),
        )
        .route("/timetable-slots", get(faculty::list_timetable_slots))
        .route("/timetable-slots", post(faculty::upsert_timetable_slot))
        .route(
            "/timetable-slots/:id",
            delete(faculty::delete_timetable_slot),
        )
        // Weekly timetable (Phase 4)
        .route(
            "/timetable-weekly",
            get(faculty::list_weekly_timetable_slots),
        )
        .route(
            "/timetable-weekly",
            post(faculty::upsert_weekly_timetable_slot),
        )
        .route(
            "/timetable-weekly/:id",
            delete(faculty::delete_weekly_timetable_slot),
        )
        .route("/timetable-weekly/clone", post(faculty::clone_week))
        // Holidays
        .route("/holidays", get(faculty::list_holidays))
        .route("/holidays", post(faculty::create_holiday))
        .route("/holidays/bulk", post(faculty::bulk_create_holiday))
        .route("/holidays/:id", delete(faculty::delete_holiday))
        // Faculty app (Phase 2)
        .route("/faculty/today-sessions", get(faculty::today_sessions))
        .route("/lecture-sessions", get(faculty::list_lecture_sessions))
        .route(
            "/faculty/sessions/:id/attendance",
            get(faculty::session_attendance),
        )
        .route(
            "/faculty/sessions/:id/attendance",
            post(faculty::mark_attendance),
        )
        // Faculty app (Phase 3) — substitution, cancellation, admin session manager
        .route(
            "/faculty/admin-sessions",
            get(faculty::admin_today_sessions),
        )
        .route(
            "/faculty/sessions/:id/substitute",
            post(faculty::substitute_session),
        )
        .route(
            "/faculty/sessions/:id/cancel",
            post(faculty::cancel_session),
        )
        .route(
            "/faculty/sessions/:id/restore",
            post(faculty::restore_session),
        )
        .route(
            "/faculty/makeup-sessions",
            post(faculty::create_makeup_session),
        )
        // Timetable analytics / health (Phase 5)
        .route(
            "/faculty-schedule/:faculty_user_id",
            get(faculty::faculty_cross_school_schedule),
        )
        .route("/timetable-health", get(faculty::timetable_health))
        .route("/compliance-metrics", get(faculty::compliance_metrics))
        .route("/deviation-score/:school_id", get(faculty::deviation_score))
        .route("/substitutions", get(faculty::list_substitutions))
        .route(
            "/pending-substitutions",
            get(faculty::pending_substitutions),
        )
        .route(
            "/substitutions/:id/accept",
            post(faculty::accept_substitution),
        )
        .route(
            "/substitutions/:id/decline",
            post(faculty::decline_substitution),
        )
        .route("/room-conflicts", get(faculty::room_conflicts))
        .route("/alerts", get(faculty::get_alerts))
        .route("/faculty/alerts", get(faculty::get_faculty_alerts))
        // Notifications (Phase 6)
        .route("/notifications", get(notifications::list_notifications))
        .route("/notifications", post(notifications::create_notification))
        .route(
            "/notifications/unread-count",
            get(notifications::unread_count),
        )
        .route(
            "/notifications/mark-all-read",
            post(notifications::mark_all_read),
        )
        .route("/notifications/:id/read", post(notifications::mark_read))
        // Substitution & Leave Engine (Phase 6)
        .route(
            "/suggest-substitutes",
            post(substitutions::suggest_substitutes),
        )
        .route("/leave-requests", post(substitutions::create_leave_request))
        .route("/leave-requests", get(substitutions::list_leave_requests))
        .route(
            "/leave-requests/:id/approve",
            post(substitutions::approve_leave_request),
        )
        .route(
            "/leave-requests/:id/reject",
            post(substitutions::reject_leave_request),
        )
        .route(
            "/leave-requests/:id/impact",
            get(substitutions::get_leave_impact),
        )
        .route(
            "/leave-requests/:id/audit",
            get(substitutions::list_leave_audit_log),
        )
        .route("/swap-requests", post(substitutions::create_swap_request))
        .route("/swap-requests", get(substitutions::list_swap_requests))
        .route(
            "/swap-requests/:id/accept",
            post(substitutions::accept_swap_request),
        )
        .route(
            "/today-substitutions",
            get(substitutions::today_substitutions),
        )
        .route(
            "/substitution-detail/:session_id",
            get(substitutions::substitution_detail),
        )
        .route(
            "/substitution-balance/:faculty_user_id",
            get(substitutions::substitution_balance),
        )
        .route(
            "/substitution-reports",
            get(substitutions::substitution_reports),
        )
        .route("/bulk-attendance", post(substitutions::bulk_attendance))
        .route(
            "/mark-attendance-quick",
            post(substitutions::mark_attendance_quick),
        )
        .route(
            "/assign-substitute/:session_id",
            post(substitutions::assign_substitute),
        )
        // Automation & Policy Engine (Phase 6)
        .route("/policies", get(automation::list_policies))
        .route("/policies/:key", put(automation::update_policy))
        .route("/escalation-rules", get(automation::list_escalation_rules))
        .route(
            "/escalation-rules",
            post(automation::create_escalation_rule),
        )
        .route(
            "/escalation-rules/:id",
            put(automation::update_escalation_rule),
        )
        .route(
            "/digests/intervention",
            get(automation::intervention_digest),
        )
        .route("/digests/sip", get(automation::sip_brief))
        .route("/alert-inbox", get(automation::alert_inbox))
        .route("/alert-inbox/:id/dismiss", post(automation::dismiss_alert))
        .route(
            "/alert-inbox/bulk-action",
            post(automation::bulk_alert_action),
        )
        .route("/tickets/from-gap", post(automation::ticket_from_gap))
        .route("/bulk/assign-users", post(automation::bulk_assign_users))
        .route(
            "/bulk/import-subjects",
            post(automation::bulk_import_subjects),
        )
        .route(
            "/bulk/publish-timetables",
            post(automation::bulk_publish_timetables),
        )
        .route("/faculty/reassign", post(automation::reassign_faculty))
        .route(
            "/week/clone-with-check",
            post(automation::clone_week_with_check),
        )
        .route("/announcements", get(automation::list_announcements))
        .route("/announcements", post(automation::create_announcement))
        .route(
            "/room-conflicts/cross-school",
            get(automation::cross_school_room_conflicts),
        )
        // Analytics & Dashboards (Phase 6)
        .route(
            "/analytics/compliance-scorecard",
            get(analytics::compliance_scorecard),
        )
        .route("/analytics/control-tower", get(analytics::control_tower))
        .route(
            "/analytics/faculty-utilization",
            get(analytics::faculty_utilization),
        )
        .route(
            "/analytics/deviation-scoreboard",
            get(analytics::deviation_scoreboard),
        )
        .route(
            "/analytics/session-type-breakdown",
            get(analytics::session_type_breakdown),
        )
        .route(
            "/analytics/faculty-stability",
            get(analytics::faculty_stability),
        )
        .route(
            "/analytics/subject-coverage-heatmap",
            get(analytics::subject_coverage_heatmap),
        )
        .route("/analytics/health-trends", get(analytics::health_trends))
        .route(
            "/analytics/substitution-trends",
            get(analytics::substitution_trends),
        )
        .route("/analytics/region-heatmap", get(analytics::region_heatmap))
        .route(
            "/analytics/room-conflicts-radar",
            get(analytics::room_conflicts_radar),
        )
        .route(
            "/analytics/adherence-comparison",
            get(analytics::adherence_comparison),
        )
        .route("/analytics/week-diff", get(analytics::week_diff))
        .route(
            "/analytics/compliance-pivot",
            get(analytics::compliance_pivot),
        )
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    public.merge(protected).with_state(state)
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn ready(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.db.get() {
        Ok(conn) => match conn.execute("SELECT 1", []) {
            Ok(_) => Ok(Json(serde_json::json!({"status": "ready"}))),
            Err(_) => Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "reason": "database unavailable"
                })),
            )),
        },
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "reason": "database unavailable"
            })),
        )),
    }
}

pub fn health_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(state)
}
