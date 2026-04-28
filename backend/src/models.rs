use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// ── App state ────────────────────────────────────────────────────────────────

pub struct AppState {
    pub db: Mutex<Connection>,
    pub jwt_secret: String,
}

// ── JWT ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id as string
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub exp: usize,        // unix timestamp
}

// ── Auth ──────────────────────────────────────────────────────────────────────

// Alias kept for compatibility with repositories.rs (desktop used SessionUser)
pub type SessionUser = CurrentUser;

#[derive(Debug, Serialize, Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: CurrentUser,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

// ── User management ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AppUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: String,
    pub school_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub password: String,
    #[serde(default)]
    pub school_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub is_active: bool,
    #[serde(default)]
    pub school_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

// ── Tickets ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTicketInput {
    pub title: String,
    pub description: String,
    pub requester: String,
    pub priority: String,
    pub school_id: Option<i64>,
    pub school_name: String,
    pub student_name: String,
    pub grade_level: String,
    pub program_track: String,
    pub issue_category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketInput {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub requester: String,
    pub status: String,
    pub priority: String,
    pub assignee: String,
    pub queue: String,
    pub school_id: Option<i64>,
    pub school_name: String,
    pub student_name: String,
    pub grade_level: String,
    pub program_track: String,
    pub issue_category: String,
}

#[derive(Debug, Serialize)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub requester: String,
    pub assignee: String,
    pub status: String,
    pub priority: String,
    pub queue: String,
    pub school_id: Option<i64>,
    pub school_name: String,
    pub student_name: String,
    pub grade_level: String,
    pub program_track: String,
    pub issue_category: String,
    pub sla_due_at: String,
    pub escalation_status: String,
    pub escalated_at: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── Comments ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddCommentInput {
    pub ticket_id: i64,
    pub author: String,
    pub body: String,
    pub is_internal: bool,
    pub channel: String,
    pub audience: String,
    pub recipient_name: String,
    pub recipient_contact: String,
    pub next_follow_up_due: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentStatusInput {
    pub id: i64,
    pub delivery_status: String,
    pub next_follow_up_due: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TicketComment {
    pub id: i64,
    pub ticket_id: i64,
    pub author: String,
    pub body: String,
    pub is_internal: bool,
    pub channel: String,
    pub audience: String,
    pub recipient_name: String,
    pub recipient_contact: String,
    pub delivery_status: String,
    pub last_contacted_at: String,
    pub next_follow_up_due: String,
    pub created_at: String,
}

// ── History / attachments ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TicketAttachment {
    pub id: i64,
    pub ticket_id: i64,
    pub original_filename: String,
    pub stored_path: String,
    pub size_bytes: i64,
    pub uploaded_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TicketHistory {
    pub id: i64,
    pub ticket_id: i64,
    pub actor: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub action: String,
    pub actor: String,
    pub summary: String,
    pub created_at: String,
}

// ── Policies ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SlaPolicy {
    pub issue_category: String,
    pub hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSlaPolicyInput {
    pub issue_category: String,
    pub hours: i64,
}

#[derive(Debug, Serialize)]
pub struct AssignmentRule {
    pub queue: String,
    pub assignee: String,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentRuleInput {
    pub queue: String,
    pub assignee: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct EscalationPolicy {
    pub at_risk_hours: i64,
    pub escalation_assignee: String,
    pub auto_assign_on_breach: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEscalationPolicyInput {
    pub at_risk_hours: i64,
    pub escalation_assignee: String,
    pub auto_assign_on_breach: bool,
}

#[derive(Debug, Serialize)]
pub struct CommunicationTemplate {
    pub id: i64,
    pub name: String,
    pub audience: String,
    pub body: String,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommunicationTemplateInput {
    pub id: Option<i64>,
    pub name: String,
    pub audience: String,
    pub body: String,
    pub is_active: bool,
}

// ── Schools / regions / SIP data ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct School {
    pub id: i64,
    pub name: String,
    pub region_id: Option<i64>,
    pub region_name: String,
    pub program_model: String,
    pub distance_classification: String,
    pub sip_academic_owner_role: String,
    pub sip_academic_owner_name: String,
    pub sip_academic_owner_mobile: String,
    pub sip_academic_owner_email: String,
    pub center_head_name: String,
    pub center_head_mobile: String,
    pub center_head_email: String,
    pub principal_name: String,
    pub principal_mobile: String,
    pub principal_email: String,
    pub school_spoc_name: String,
    pub school_spoc_mobile: String,
    pub school_spoc_email: String,
    pub central_academic_spoc_name: String,
    pub central_academic_spoc_mobile: String,
    pub central_academic_spoc_email: String,
    pub central_business_spoc_name: String,
    pub central_business_spoc_mobile: String,
    pub central_business_spoc_email: String,
    pub bh_name: String,
    pub bh_mobile: String,
    pub bh_email: String,
    pub aom_name: String,
    pub aom_mobile: String,
    pub aom_email: String,
    pub mapped_vp_center: String,
    pub is_dropped: bool,
    pub dropped_at: String,
    pub dropped_reason: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSchoolInput {
    pub name: String,
    pub region_id: Option<i64>,
    pub program_model: String,
    pub distance_classification: String,
    pub sip_academic_owner_role: String,
    pub sip_academic_owner_name: String,
    pub sip_academic_owner_mobile: String,
    pub sip_academic_owner_email: String,
    pub center_head_name: String,
    pub center_head_mobile: String,
    pub center_head_email: String,
    pub principal_name: String,
    pub principal_mobile: String,
    pub principal_email: String,
    pub school_spoc_name: String,
    pub school_spoc_mobile: String,
    pub school_spoc_email: String,
    pub central_academic_spoc_name: String,
    pub central_academic_spoc_mobile: String,
    pub central_academic_spoc_email: String,
    pub central_business_spoc_name: String,
    pub central_business_spoc_mobile: String,
    pub central_business_spoc_email: String,
    pub bh_name: String,
    pub bh_mobile: String,
    pub bh_email: String,
    pub aom_name: String,
    pub aom_mobile: String,
    pub aom_email: String,
    #[serde(default)]
    pub mapped_vp_center: String,
}

#[derive(Debug, Serialize)]
pub struct Region {
    pub id: i64,
    pub name: String,
    pub regional_academic_head_name: String,
    pub regional_academic_head_mobile: String,
    pub regional_academic_head_email: String,
    pub regional_business_head_name: String,
    pub regional_business_head_mobile: String,
    pub regional_business_head_email: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertRegionInput {
    pub id: Option<i64>,
    pub name: String,
    pub regional_academic_head_name: String,
    pub regional_academic_head_mobile: String,
    pub regional_academic_head_email: String,
    pub regional_business_head_name: String,
    pub regional_business_head_mobile: String,
    pub regional_business_head_email: String,
}

#[derive(Debug, Serialize)]
pub struct Student {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub name: String,
    pub grade_level: String,
    pub program_track: String,
    pub track: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudentInput {
    pub school_id: i64,
    pub name: String,
    pub grade_level: String,
    pub program_track: String,
    #[serde(default)]
    pub track: String,
}

#[derive(Debug, Serialize)]
pub struct LectureModel {
    pub id: i64,
    pub name: String,
    pub days_per_week: i64,
    pub lectures_per_day: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLectureModelInput {
    pub name: String,
    pub days_per_week: i64,
    pub lectures_per_day: i64,
}

#[derive(Debug, Serialize)]
pub struct SchoolClassPlan {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub lecture_model_id: i64,
    pub lecture_model_name: String,
    pub days_per_week: i64,
    pub lectures_per_day: i64,
    pub batch_pattern: String,
    pub aop_admissions: i64,
    pub registrations: i64,
    pub actual_admissions: i64,
    pub admission_gap: i64,
    pub admission_attainment_percent: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSchoolClassPlanInput {
    pub school_id: i64,
    pub grade_level: String,
    #[serde(default)]
    pub track: String,
    pub lecture_model_id: i64,
    pub batch_pattern: String,
    pub aop_admissions: i64,
    #[serde(default)]
    pub registrations: i64,
    pub actual_admissions: i64,
}

#[derive(Debug, Serialize)]
pub struct SchoolRegionHistory {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub old_region_id: Option<i64>,
    pub old_region_name: String,
    pub new_region_id: Option<i64>,
    pub new_region_name: String,
    pub changed_at: String,
}

#[derive(Debug, Serialize)]
pub struct SchoolProgramDashboard {
    pub total_schools: i64,
    pub schools_with_class_plans: i64,
    pub total_classes: i64,
    pub total_aop_admissions: i64,
    pub total_actual_admissions: i64,
    pub admission_gap: i64,
    pub admission_attainment_percent: i64,
    pub remote_school_count: i64,
    pub near_proximity_school_count: i64,
    pub aspire_school_count: i64,
    pub minimum_guarantee_school_count: i64,
    pub class_plans: Vec<SchoolClassPlan>,
}

#[derive(Debug, Serialize)]
pub struct StudentTimeline {
    pub school_name: String,
    pub student_name: String,
    pub tickets: Vec<Ticket>,
    pub comments: Vec<TicketComment>,
    pub history: Vec<TicketHistory>,
    pub attachments: Vec<TicketAttachment>,
}

// ── Faculty / timetable / subjects ─────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct Subject {
    pub id: i64,
    pub name: String,
    pub track: String,
    pub is_default: bool,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubjectInput {
    pub name: String,
    pub track: String,
    #[serde(default = "default_true")]
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i64,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct UpdateSubjectInput {
    pub id: i64,
    pub name: String,
    pub track: String,
    pub is_default: bool,
    pub sort_order: i64,
}

#[derive(Debug, Serialize)]
pub struct EffectiveSubject {
    pub id: i64,
    pub name: String,
    pub track: String,
    pub is_default: bool,
    pub is_offered: bool,  // true if this subject applies to the school+track
}

#[derive(Debug, Serialize)]
pub struct FacultyAssignment {
    pub id: i64,
    pub faculty_user_id: i64,
    pub faculty_display_name: String,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub subject_id: i64,
    pub subject_name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultyAssignmentInput {
    pub faculty_user_id: i64,
    pub school_id: i64,
    pub grade_level: String,
    #[serde(default)]
    pub track: String,
    pub subject_id: i64,
}

#[derive(Debug, Serialize)]
pub struct TimetableSlot {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub day_of_week: i64,
    pub period: i64,
    pub subject_id: i64,
    pub subject_name: String,
    pub faculty_user_id: Option<i64>,
    pub faculty_display_name: String,
    pub start_time: String,
    pub end_time: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertTimetableSlotInput {
    pub school_id: i64,
    pub grade_level: String,
    #[serde(default)]
    pub track: String,
    pub batch_pattern: String,
    pub day_of_week: i64,
    pub period: i64,
    pub subject_id: i64,
    pub faculty_user_id: Option<i64>,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub end_time: String,
}

