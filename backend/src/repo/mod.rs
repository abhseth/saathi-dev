pub mod analytics;
pub mod audit;
pub mod common;
pub mod faculty;
pub mod ops;
pub mod schools;
pub mod tickets;

// ── Re-exports for backward compatibility ──────────────────────────────────
// Tickets
pub use tickets::{
    add_comment, create_ticket, delete_ticket, get_comment, get_escalation_policy,
    get_student_timeline, get_ticket, list_all_comments, list_assignment_rules, list_comments,
    list_communication_templates, list_history, list_sla_policies, list_tickets,
    refresh_escalations, resolve_ticket_school, update_assignment_rule, update_comment_status,
    update_communication_template, update_escalation_policy, update_sla_policy, update_ticket,
};

// Schools
pub use schools::{
    archive_batch, create_batch, create_lecture_model, create_school, create_student,
    delete_region, delete_school, delete_student, drop_school, get_batch,
    get_school_delete_impact, get_school_program_dashboard, get_student, list_audit_log,
    list_batches, list_dropped_schools, list_lecture_models, list_regions,
    list_school_class_plans, list_school_region_history, list_schools, list_students_paginated,
    restore_school, update_batch, update_student, upsert_region,
    upsert_school_class_plan,
};

// Faculty
pub use faculty::{
    admin_reset_password, authenticate_user, change_password, clone_week_to_week,
    create_faculty_assignment, create_faculty_login, create_faculty_member,
    create_faculty_school_membership, create_subject, create_user, create_vp_center,
    create_vp_center_building, delete_faculty_assignment, delete_faculty_member,
    delete_faculty_school_membership, delete_subject, delete_timetable_slot, delete_user,
    delete_vp_center, delete_vp_center_building, delete_weekly_timetable_slot,
    get_active_user_by_id, get_faculty_assignment_school_id, get_faculty_member,
    get_faculty_profile, get_timetable_slot_school_id, get_weekly_timetable_slot_school_id,
    is_faculty_at_school, is_faculty_in_scope, link_faculty_user, list_effective_subjects,
    list_faculty_assignments, list_faculty_members, list_faculty_profiles,
    list_faculty_school_memberships, list_subjects, list_timetable_slots, list_users,
    list_vp_center_buildings, list_vp_centers, list_weekly_timetable_slots, update_faculty_member,
    update_subject, update_user, update_vp_center, update_vp_center_building,
    upsert_faculty_profile, upsert_timetable_slot, upsert_weekly_timetable_slot,
};

// Ops
pub use ops::{
    accept_substitution, accept_swap_request, approve_leave_request, assign_substitute,
    attendance_summary, bulk_mark_faculty_absent, cancel_session, chronic_absentees, das_report,
    bulk_create_holidays, create_holiday, create_leave_audit_log, create_leave_request, create_makeup_session,
    create_notification, create_swap_request, decline_substitution, delete_holiday,
    ensure_session_students, get_holiday_school_id, get_leave_impact_preview, get_leave_request,
    get_lecture_session, get_lecture_session_school_id, get_lecture_session_status,
    get_session_attendance, get_substitution_balance, get_substitution_detail,
    get_substitution_reports, get_swap_request, get_today_substitutions,
    list_all_today_makeup_sessions, list_all_today_sessions, list_bulk_absence_target_schools,
    list_faculty_today_makeup_sessions, list_faculty_today_sessions, list_holidays,
    list_leave_requests, list_lecture_sessions, list_school_approver_user_ids, list_swap_requests,
    mark_attendance, mark_attendance_quick, reject_leave_request, restore_session,
    set_school_optional_subject, subject_attendance, substitute_session,
};

// Analytics
pub use analytics::{
    get_deviation_score, list_compliance_metrics, list_faculty_cross_school_schedule,
    list_pending_substitution_records, list_room_conflicts, list_substitution_records,
    list_timetable_health_status,
};

// Audit
pub use audit::insert_audit_log;

// Common helpers (re-exported for backward compat where routes use them directly)
