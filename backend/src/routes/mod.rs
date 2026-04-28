use axum::{middleware, routing::{delete, get, post, put}, Router};
use std::sync::Arc;

use crate::{auth::require_auth, models::AppState};

mod auth;
mod tickets;
mod schools;
mod admin;
mod export;
mod imports;
mod faculty;

pub fn router(state: Arc<AppState>) -> Router {
    // Public routes (no auth required)
    let public = Router::new()
        .route("/auth/login", post(auth::login));

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
        .route("/comments/:id/status", put(tickets::update_comment_status))
        .route("/comments", get(tickets::list_all_comments))
        // Schools
        .route("/schools", get(schools::list_schools))
        .route("/schools", post(schools::create_school))
        .route("/schools/:id/drop", post(schools::drop_school))
        .route("/schools/:id/restore", post(schools::restore_school))
        .route("/schools/:id", delete(schools::delete_school))
        .route("/schools/dropped", get(schools::list_dropped_schools))
        .route("/regions", get(schools::list_regions))
        .route("/regions", post(schools::upsert_region))
        .route("/regions/:id", delete(schools::delete_region))
        .route("/students", get(schools::list_students))
        .route("/students", post(schools::create_student))
        .route("/lecture-models", get(schools::list_lecture_models))
        .route("/lecture-models", post(schools::create_lecture_model))
        .route("/class-plans", get(schools::list_class_plans))
        .route("/class-plans", post(schools::upsert_class_plan))
        .route("/program-dashboard", get(schools::program_dashboard))
        .route("/school-region-history", get(schools::school_region_history))
        // Admin
        .route("/users", get(admin::list_users))
        .route("/users", post(admin::create_user))
        .route("/users/:id", put(admin::update_user))
        .route("/users/:id", delete(admin::delete_user))
        .route("/users/password", put(admin::change_password))
        .route("/audit-log", get(admin::list_audit_log))
        .route("/sla-policies", get(admin::list_sla_policies))
        .route("/sla-policies", put(admin::update_sla_policy))
        .route("/assignment-rules", get(admin::list_assignment_rules))
        .route("/assignment-rules", put(admin::update_assignment_rule))
        .route("/escalation-policy", get(admin::get_escalation_policy))
        .route("/escalation-policy", put(admin::update_escalation_policy))
        .route("/communication-templates", get(admin::list_communication_templates))
        .route("/communication-templates", put(admin::update_communication_template))
        // DB backup / restore (admin only) — used for volume migration + ad-hoc snapshots
        .route("/admin/db-snapshot", get(admin::db_snapshot))
        .route("/admin/db-restore", post(admin::db_restore))
        // Exports
        .route("/export/tickets.csv", get(export::tickets_csv))
        .route("/export/communications.csv", get(export::communications_csv))
        .route("/export/sip-master.csv", get(export::sip_master_csv))
        // Imports
        .route("/imports/schools.csv", post(imports::import_schools_csv))
        .route("/imports/sip-master/preview", post(imports::preview_sip_master_import))
        .route("/imports/sip-master", post(imports::import_sip_master))
        // Faculty / timetable / subjects (Phase 1)
        .route("/subjects", get(faculty::list_subjects))
        .route("/subjects", post(faculty::create_subject))
        .route("/subjects/:id", put(faculty::update_subject))
        .route("/subjects/:id", delete(faculty::delete_subject))
        .route("/schools/:id/effective-subjects", get(faculty::list_effective_subjects))
        .route("/schools/:id/optional-subjects", post(faculty::set_school_optional_subject))
        .route("/faculty-assignments", get(faculty::list_faculty_assignments))
        .route("/faculty-assignments", post(faculty::create_faculty_assignment))
        .route("/faculty-assignments/:id", delete(faculty::delete_faculty_assignment))
        .route("/timetable-slots", get(faculty::list_timetable_slots))
        .route("/timetable-slots", post(faculty::upsert_timetable_slot))
        .route("/timetable-slots/:id", delete(faculty::delete_timetable_slot))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    public.merge(protected).with_state(state)
}
