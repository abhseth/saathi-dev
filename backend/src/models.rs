use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};

// ── App state ────────────────────────────────────────────────────────────────

pub struct AppState {
    pub db: Pool<SqliteConnectionManager>,
    pub jwt_secret: String,
}

#[derive(Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
}

// ── JWT ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user id as string
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub school_ids: Vec<i64>,
    pub exp: usize, // unix timestamp
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
    pub school_ids: Vec<i64>,
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

#[derive(Debug, Deserialize)]
pub struct AdminResetPasswordInput {
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
    pub linked_grade_level: String,
    pub linked_subject: String,
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
#[allow(dead_code)]
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
    pub vp_tagging: String,
    pub is_dropped: bool,
    pub dropped_at: String,
    pub dropped_reason: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SchoolDeleteImpactItem {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct SchoolDeleteImpact {
    pub school_id: i64,
    pub school_name: String,
    pub total_linked_records: i64,
    pub items: Vec<SchoolDeleteImpactItem>,
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
    #[serde(default)]
    pub vp_tagging: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSchoolInput {
    pub id: i64,
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
    #[serde(default)]
    pub vp_tagging: String,
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
    pub regional_deputy_academic_head_name: String,
    pub regional_deputy_academic_head_mobile: String,
    pub regional_deputy_academic_head_email: String,
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
    pub regional_deputy_academic_head_name: String,
    pub regional_deputy_academic_head_mobile: String,
    pub regional_deputy_academic_head_email: String,
}

#[derive(Debug, Serialize)]
pub struct Student {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub name: String,
    pub registration_number: String,
    pub grade_level: String,
    pub program_track: String,
    pub track: String,
    pub student_mobile: String,
    pub student_email: String,
    pub father_name: String,
    pub father_email: String,
    pub father_mobile: String,
    pub mother_name: String,
    pub mother_email: String,
    pub mother_mobile: String,
    pub batch_ref_id: i64,
    pub batch_name: String,
    pub batch_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudentInput {
    pub school_id: i64,
    pub name: String,
    #[serde(default)]
    pub registration_number: String,
    pub grade_level: String,
    pub program_track: String,
    #[serde(default)]
    pub track: String,
    #[serde(default)]
    pub student_mobile: String,
    #[serde(default)]
    pub student_email: String,
    #[serde(default)]
    pub father_name: String,
    #[serde(default)]
    pub father_email: String,
    #[serde(default)]
    pub father_mobile: String,
    #[serde(default)]
    pub mother_name: String,
    #[serde(default)]
    pub mother_email: String,
    #[serde(default)]
    pub mother_mobile: String,
    #[serde(default)]
    pub batch_ref_id: i64,
    #[serde(default)]
    pub batch_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudentInput {
    pub id: i64,
    pub school_id: i64,
    pub name: String,
    #[serde(default)]
    pub registration_number: String,
    pub grade_level: String,
    pub program_track: String,
    #[serde(default)]
    pub track: String,
    #[serde(default)]
    pub student_mobile: String,
    #[serde(default)]
    pub student_email: String,
    #[serde(default)]
    pub father_name: String,
    #[serde(default)]
    pub father_email: String,
    #[serde(default)]
    pub father_mobile: String,
    #[serde(default)]
    pub mother_name: String,
    #[serde(default)]
    pub mother_email: String,
    #[serde(default)]
    pub mother_mobile: String,
    #[serde(default)]
    pub batch_ref_id: i64,
    #[serde(default)]
    pub batch_id: String,
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
pub struct Batch {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub batch_id: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub capacity: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBatchInput {
    pub school_id: i64,
    pub batch_id: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub capacity: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBatchInput {
    pub id: i64,
    pub school_id: i64,
    pub batch_id: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub capacity: i64,
}

#[derive(Debug, Serialize)]
pub struct BatchDetail {
    #[serde(flatten)]
    pub batch: Batch,
    pub student_count: i64,
    pub faculty_count: i64,
    pub active_ticket_count: i64,
    pub upcoming_session_count: i64,
}

#[derive(Debug, Serialize)]
pub struct BatchAnalytics {
    pub batches: Vec<BatchDetail>,
    pub total_students: i64,
    pub total_capacity: i64,
    pub overall_utilization: f64,
}

#[derive(Debug, Serialize)]
pub struct VpCenter {
    pub id: i64,
    pub name: String,
    pub location: String,
    pub contact_name: String,
    pub contact_mobile: String,
    pub contact_email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateVpCenterInput {
    pub name: String,
    pub location: String,
    pub contact_name: String,
    pub contact_mobile: String,
    pub contact_email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVpCenterInput {
    pub id: i64,
    pub name: String,
    pub location: String,
    pub contact_name: String,
    pub contact_mobile: String,
    pub contact_email: String,
}

#[derive(Debug, Serialize)]
pub struct VpCenterBuilding {
    pub id: i64,
    pub vp_center_id: i64,
    pub building_name: String,
    pub address: String,
    pub center_head_name: String,
    pub center_head_mobile: String,
    pub center_head_email: String,
    pub associate_center_head_name: String,
    pub associate_center_head_mobile: String,
    pub associate_center_head_email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateVpCenterBuildingInput {
    pub vp_center_id: i64,
    pub building_name: String,
    pub address: String,
    pub center_head_name: String,
    pub center_head_mobile: String,
    pub center_head_email: String,
    pub associate_center_head_name: String,
    pub associate_center_head_mobile: String,
    pub associate_center_head_email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVpCenterBuildingInput {
    pub id: i64,
    pub vp_center_id: i64,
    pub building_name: String,
    pub address: String,
    pub center_head_name: String,
    pub center_head_mobile: String,
    pub center_head_email: String,
    pub associate_center_head_name: String,
    pub associate_center_head_mobile: String,
    pub associate_center_head_email: String,
}

#[derive(Debug, Serialize)]
pub struct FacultyProfile {
    pub faculty_user_id: i64,
    pub faculty_display_name: String,
    pub pwid: String,
    pub email: String,
    pub mobile: String,
    pub emergency_contact_name: String,
    pub emergency_contact_mobile: String,
    pub vp_center_id: Option<i64>,
    pub vp_center_name: String,
    pub sip_school_id: Option<i64>,
    pub sip_school_name: String,
    pub primary_subject_id: Option<i64>,
    pub primary_subject_name: String,
    pub employment_type: String,
    pub qualification: String,
    pub experience_years: i64,
    pub designation: String,
    pub specialization: String,
    pub max_periods_per_week: i64,
    pub joining_date: String,
    pub exit_date: String,
    pub documents_verified: bool,
    pub is_active: bool,
    pub wings: Vec<String>,
    pub batch_ids: Vec<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertFacultyProfileInput {
    pub faculty_user_id: i64,
    pub pwid: String,
    pub email: String,
    pub mobile: String,
    pub emergency_contact_name: String,
    pub emergency_contact_mobile: String,
    pub vp_center_id: Option<i64>,
    pub sip_school_id: Option<i64>,
    pub primary_subject_id: Option<i64>,
    pub employment_type: String,
    pub qualification: String,
    pub experience_years: i64,
    pub designation: String,
    pub specialization: String,
    pub max_periods_per_week: i64,
    pub joining_date: String,
    pub exit_date: String,
    pub documents_verified: bool,
    pub is_active: bool,
    pub wings: Vec<String>,
    pub batch_ids: Vec<i64>,
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
pub struct StudentAttendanceSummary {
    pub session_date: String,
    pub subject_name: String,
    pub status: String,
    pub marked_at: String,
}

#[derive(Debug, Serialize)]
pub struct StudentTimeline {
    pub student: Student,
    pub tickets: Vec<Ticket>,
    pub comments: Vec<TicketComment>,
    pub history: Vec<TicketHistory>,
    pub attachments: Vec<TicketAttachment>,
    pub attendance: Vec<StudentAttendanceSummary>,
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

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

fn default_lecture() -> String {
    "Lecture".to_string()
}

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
    pub is_offered: bool, // true if this subject applies to the school+track
}

#[derive(Debug, Serialize)]
pub struct FacultyAssignment {
    pub id: i64,
    pub faculty_id: i64,
    pub faculty_user_id: Option<i64>,
    pub faculty_display_name: String,
    pub school_id: i64,
    pub school_name: String,
    pub batch_id: i64,
    pub batch_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub subject_id: i64,
    pub subject_name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultyAssignmentInput {
    pub faculty_id: i64,
    pub batch_id: i64,
    pub subject_id: i64,
}

// ── Faculty Members (master data, optional login) ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct FacultyMember {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub mobile: String,
    pub pwid: String,
    pub qualification: String,
    pub experience_years: i64,
    pub designation: String,
    pub specialization: String,
    pub employment_type: String,
    pub is_active: bool,
    pub user_id: Option<i64>,
    pub user_username: Option<String>,
    pub user_display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultyMemberInput {
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub mobile: String,
    #[serde(default)]
    pub pwid: String,
    #[serde(default)]
    pub qualification: String,
    #[serde(default)]
    pub experience_years: i64,
    #[serde(default)]
    pub designation: String,
    #[serde(default)]
    pub specialization: String,
    #[serde(default)]
    pub employment_type: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
    pub user_id: Option<i64>,
    #[serde(default)]
    pub initial_school_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacultyMemberInput {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub mobile: String,
    #[serde(default)]
    pub pwid: String,
    #[serde(default)]
    pub qualification: String,
    #[serde(default)]
    pub experience_years: i64,
    #[serde(default)]
    pub designation: String,
    #[serde(default)]
    pub specialization: String,
    #[serde(default)]
    pub employment_type: String,
    pub is_active: bool,
    pub user_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct FacultySchoolMembership {
    pub id: i64,
    pub faculty_id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub role_at_school: String,
    pub is_primary: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultySchoolMembershipInput {
    pub faculty_id: i64,
    pub school_id: i64,
    #[serde(default)]
    pub role_at_school: String,
    #[serde(default = "default_false")]
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct TimetableSlot {
    pub id: i64,
    pub school_id: i64,
    pub school_name: String,
    pub batch_id: i64,
    pub batch_name: String,
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
    pub room: String,
    pub session_type: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertTimetableSlotInput {
    #[serde(default)]
    pub school_id: i64,
    pub batch_id: i64,
    #[serde(default)]
    pub grade_level: String,
    #[serde(default)]
    pub track: String,
    #[serde(default)]
    pub batch_pattern: String,
    pub day_of_week: i64,
    pub period: i64,
    pub subject_id: i64,
    pub faculty_user_id: Option<i64>,
    #[serde(default)]
    pub start_time: String,
    #[serde(default)]
    pub end_time: String,
    #[serde(default)]
    pub room: String,
    #[serde(default = "default_lecture")]
    pub session_type: String,
}

// ── Faculty / attendance (Phase 2) ───────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct LectureSession {
    pub id: i64,
    pub timetable_slot_id: Option<i64>,
    pub session_date: String,
    pub actual_faculty_user_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub grade_level: Option<String>,
    pub track: Option<String>,
    pub school_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AttendanceRecord {
    pub id: i64,
    pub lecture_session_id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub status: String,
    pub marked_by_user_id: Option<i64>,
    pub marked_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FacultyTodaySession {
    pub session_id: i64,
    pub timetable_slot_id: i64,
    pub session_date: String,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub batch_id: String,
    pub period: i64,
    pub subject_id: i64,
    pub subject_name: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub total_students: i64,
    pub present_count: i64,
    pub late_count: i64,
    pub absent_count: i64,
    pub faculty_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkAttendanceInput {
    pub records: Vec<SingleAttendance>,
}

#[derive(Debug, Deserialize)]
pub struct SingleAttendance {
    pub student_id: i64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SubstituteSessionInput {
    pub substitute_faculty_user_id: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct WeeklyTimetableSlot {
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
    pub faculty_display_name: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub room: String,
    pub session_type: String,
    pub week_start_date: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertWeeklyTimetableSlotInput {
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
    #[serde(default)]
    pub room: String,
    #[serde(default = "default_lecture")]
    pub session_type: String,
    pub week_start_date: String,
}

#[derive(Debug, Deserialize)]
pub struct CloneWeekInput {
    pub from_week: String,
    pub to_week: String,
    pub school_id: i64,
}

// ── Timetable analytics / health ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TimetableHealthStatus {
    pub school_id: i64,
    pub school_name: String,
    pub region_name: String,
    pub aom_name: String,
    pub class_plans_configured: bool,
    pub master_timetable_complete: bool,
    pub sessions_generated: bool,
    pub gaps_count: i64,
    pub status: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize)]
pub struct FacultyCrossSchoolSchedule {
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub school_id: i64,
    pub school_name: String,
    pub day_of_week: i64,
    pub period: i64,
    pub start_time: String,
    pub end_time: String,
    pub subject_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub room: String,
    pub week_start_date: String,
}

#[derive(Debug, Serialize)]
pub struct ComplianceMetrics {
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub subject_name: String,
    pub planned_periods: i64,
    pub actual_periods: i64,
    pub deviation: i64,
    pub lecture_model_adherence_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct SubjectGap {
    pub subject_name: String,
    pub grade_level: String,
    pub track: String,
    pub planned: i64,
    pub actual: i64,
}

#[derive(Debug, Serialize)]
pub struct FacultyOverload {
    pub faculty_name: String,
    pub school_name: String,
    pub period_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UnderutilizedBatch {
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub utilization_pct: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviationScore {
    pub school_id: i64,
    pub school_name: String,
    pub overall_deviation_score: f64,
    pub subject_gaps: Vec<SubjectGap>,
    pub faculty_overloads: Vec<FacultyOverload>,
    pub underutilized_batches: Vec<UnderutilizedBatch>,
}

#[derive(Debug, Serialize)]
pub struct SubstitutionRecord {
    pub session_id: i64,
    pub session_date: String,
    pub original_faculty_user_id: i64,
    pub original_faculty_name: String,
    pub substitute_faculty_user_id: Option<i64>,
    pub substitute_faculty_name: String,
    pub subject_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub status: String,
}

// ── Holidays ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct Holiday {
    pub id: i64,
    pub date: String,
    pub name: String,
    pub scope: String,
    pub region_id: Option<i64>,
    pub region_name: Option<String>,
    pub school_id: Option<i64>,
    pub school_name: Option<String>,
    pub grade_level: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateHolidayInput {
    pub date: String,
    pub name: String,
    pub scope: String,
    pub region_id: Option<i64>,
    pub school_id: Option<i64>,
    pub grade_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateHolidayInput {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub scope: String,
    pub region_id: Option<i64>,
    pub school_id: Option<i64>,
    pub grade_levels: Option<Vec<String>>,
}

// ── Alerts ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub id: String,
    pub severity: String, // "info" | "warning" | "critical"
    pub category: String,
    pub message: String,
    pub school_id: Option<i64>,
    pub school_name: Option<String>,
    pub grade_level: Option<String>,
    pub subject_name: Option<String>,
    pub faculty_user_id: Option<i64>,
    pub faculty_name: Option<String>,
    pub created_at: String,
}

// ── Reporting (Phase 4) ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AttendanceSummaryRow {
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_id: String,
    pub total_students: i64,
    pub present_count: i64,
    pub late_count: i64,
    pub absent_count: i64,
    pub excused_count: i64,
    pub attendance_percent: i64,
}

#[derive(Debug, Serialize)]
pub struct DasReportRow {
    pub group_by: String,
    pub label: String,
    pub school_id: Option<i64>,
    pub school_name: String,
    pub grade_level: String,
    pub cohort: String,
    pub batch_id: String,
    pub student_id: Option<i64>,
    pub student_name: String,
    pub scheduled_lectures: i64,
    pub present_lectures: i64,
    pub das_percent: i64,
}

#[derive(Debug, Serialize)]
pub struct ChronicAbsentee {
    pub student_id: i64,
    pub student_name: String,
    pub school_name: String,
    pub grade_level: String,
    pub total_sessions: i64,
    pub present_count: i64,
    pub attendance_percent: i64,
}

#[derive(Debug, Serialize)]
pub struct SubjectAttendanceRow {
    pub subject_name: String,
    pub total_sessions: i64,
    pub present_count: i64,
    pub late_count: i64,
    pub absent_count: i64,
    pub attendance_percent: i64,
}

// ── Analytics & Dashboards (Phase 6) ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ActionableComplianceItem {
    pub severity: String, // "critical" | "warning" | "info"
    pub message: String,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub subject_name: String,
    pub planned_periods: i64,
    pub actual_periods: i64,
    pub deviation: i64,
}

#[derive(Debug, Serialize)]
pub struct ControlTowerCard {
    pub school_id: i64,
    pub school_name: String,
    pub region_name: String,
    pub filled_periods: i64,
    pub total_periods: i64,
    pub alert_count: i64,
    pub attendance_percent: i64,
    pub active_substitutions: i64,
}

#[derive(Debug, Serialize)]
pub struct FacultyUtilizationWeek {
    pub week_start_date: String,
    pub period_count: i64,
}

#[derive(Debug, Serialize)]
pub struct FacultyUtilizationTrend {
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub weeks: Vec<FacultyUtilizationWeek>,
}

#[derive(Debug, Serialize)]
pub struct DeviationScoreboardRow {
    pub school_id: i64,
    pub school_name: String,
    pub region_name: String,
    pub overall_deviation_score: f64,
    pub top_gaps: Vec<SubjectGap>,
}

#[derive(Debug, Serialize)]
pub struct SessionTypeBreakdown {
    pub session_type: String,
    pub planned_periods: i64,
    pub actual_periods: i64,
    pub adherence_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct FacultyStabilityRow {
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub school_name: String,
    pub substitution_rate_pct: f64,
    pub cancellation_rate_pct: f64,
    pub planned_vs_actual_variance: i64,
}

#[derive(Debug, Serialize)]
pub struct SubjectCoverageCell {
    pub region_name: String,
    pub subject_name: String,
    pub adherence_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct HealthTrendWeek {
    pub week_start_date: String,
    pub green_count: i64,
    pub amber_count: i64,
    pub red_count: i64,
    pub network_adherence_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct SubstitutionTrendWeek {
    pub week_start_date: String,
    pub faculty_absences: i64,
    pub short_staffed_periods: i64,
    pub over_utilized_substitutes: i64,
}

#[derive(Debug, Serialize)]
pub struct RegionHeatmapCell {
    pub school_id: i64,
    pub school_name: String,
    pub day_of_week: i64,
    pub issue_count: i64,
}

#[derive(Debug, Serialize)]
pub struct RoomConflictRadarCell {
    pub room: String,
    pub day_of_week: i64,
    pub period: i64,
    pub conflict_count: i64,
    pub schools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AdherenceComparisonRow {
    pub school_id: i64,
    pub school_name: String,
    pub adherence_pct: f64,
    pub deviation: i64,
}

#[derive(Debug, Serialize)]
pub struct WeekDiffSlot {
    pub id: i64,
    pub school_id: i64,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub day_of_week: i64,
    pub period: i64,
    pub subject_name: String,
    pub faculty_display_name: Option<String>,
    pub room: String,
    pub session_type: String,
    pub change_type: String, // "added" | "removed" | "modified"
}

#[derive(Debug, Serialize)]
pub struct CompliancePivotRow {
    pub dimension_value: String, // subject_name, school_name, or region_name
    pub planned_periods: i64,
    pub actual_periods: i64,
    pub deviation: i64,
    pub adherence_pct: f64,
}

// ── Notifications (Phase 6 Mobile & Notifications Layer) ─────────────────────

#[derive(Debug, Serialize)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub payload_json: String,
    pub read_at: String,
    pub created_at: String,
}

// ── Phase 6: Substitution & Leave Engine ─────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LeaveRequest {
    pub id: i64,
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub school_id: i64,
    pub school_name: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: String,
    pub status: String,
    pub approved_by_user_id: Option<i64>,
    pub approved_at: String,
    pub rejected_by_user_id: Option<i64>,
    pub rejected_at: String,
    pub rejection_reason: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveRequestInput {
    pub faculty_user_id: i64,
    pub school_id: i64,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectLeaveRequestInput {
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct LeaveImpactPreview {
    pub leave_request_id: i64,
    pub affected_session_count: i64,
    pub date_range_start: String,
    pub date_range_end: String,
    pub school_name: String,
    pub faculty_name: String,
}

#[derive(Debug, Serialize)]
pub struct LeaveRequestAuditLog {
    pub id: i64,
    pub leave_request_id: i64,
    pub actor_user_id: i64,
    pub action: String,
    pub old_status: String,
    pub new_status: String,
    pub reason: String,
    pub school_id: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SwapRequest {
    pub id: i64,
    pub requester_faculty_id: i64,
    pub requester_name: String,
    pub recipient_faculty_id: i64,
    pub recipient_name: String,
    pub slot_a_id: i64,
    pub slot_b_id: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSwapRequestInput {
    pub requester_faculty_id: i64,
    pub recipient_faculty_id: i64,
    pub slot_a_id: i64,
    pub slot_b_id: i64,
}

#[derive(Debug, Serialize)]
pub struct SubstituteCandidate {
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub subject_match: bool,
    pub free_period: bool,
    pub same_school: bool,
    pub workload_score: i64,
    pub overall_score: i64,
}

#[derive(Debug, Deserialize)]
pub struct SuggestSubstitutesInput {
    pub session_id: i64,
}

#[derive(Debug, Serialize)]
pub struct TodaySubstitutionLane {
    pub session_id: i64,
    pub session_date: String,
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub period: i64,
    pub subject_name: String,
    pub original_faculty_name: String,
    pub substitute_faculty_name: Option<String>,
    pub status: String,
    pub room: String,
}

#[derive(Debug, Serialize)]
pub struct TodaySubstitutions {
    pub unfilled: Vec<TodaySubstitutionLane>,
    pub assigned: Vec<TodaySubstitutionLane>,
    pub completed: Vec<TodaySubstitutionLane>,
}

#[derive(Debug, Serialize)]
pub struct SubstitutionDetail {
    pub session_id: i64,
    pub session_date: String,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub batch_pattern: String,
    pub subject_name: String,
    pub room: String,
    pub roster_count: i64,
    pub present_count: i64,
    pub absent_count: i64,
    pub last_covered_topics: String,
}

#[derive(Debug, Serialize)]
pub struct SubstitutionBalance {
    pub faculty_user_id: i64,
    pub faculty_name: String,
    pub given_count: i64,
    pub received_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SubstitutionReportRow {
    pub school_id: i64,
    pub school_name: String,
    pub month: String,
    pub request_count: i64,
    pub filled_count: i64,
    pub acceptance_rate_pct: i64,
    pub avg_minutes_to_fill: i64,
    pub top_absentee_name: String,
    pub top_absentee_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct BulkAttendanceInput {
    pub faculty_user_ids: Vec<i64>,
    pub date: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkAttendanceQuickInput {
    pub session_id: i64,
    pub student_id: i64,
    pub status: String,
}

// ── Automation & Policy Engine (Phase 6) ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CentralPolicy {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub region_id: Option<i64>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertPolicyInput {
    pub key: String,
    pub value: String,
    pub region_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EscalationRule {
    pub id: i64,
    pub name: String,
    pub conditions_json: String,
    pub action: String,
    pub assignee_role: String,
    pub hours_threshold: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEscalationRuleInput {
    pub name: String,
    pub conditions_json: String,
    pub action: String,
    pub assignee_role: String,
    pub hours_threshold: i64,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEscalationRuleInput {
    pub id: i64,
    pub name: String,
    pub conditions_json: String,
    pub action: String,
    pub assignee_role: String,
    pub hours_threshold: i64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AlertState {
    pub id: i64,
    pub alert_hash: String,
    pub user_id: i64,
    pub dismissed_at: String,
    pub snoozed_until: String,
    pub converted_ticket_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkAlertActionInput {
    pub ids: Vec<String>,
    pub action: String, // "dismiss" | "snooze" | "ticket"
    pub snooze_hours: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Announcement {
    pub id: i64,
    pub school_id: Option<i64>,
    pub school_name: Option<String>,
    pub message: String,
    pub pinned_until: String,
    pub created_by: i64,
    pub created_by_name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncementInput {
    pub school_id: Option<i64>,
    pub message: String,
    pub pinned_until: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct BulkOperationLog {
    pub id: i64,
    pub op_type: String,
    pub status: String,
    pub payload_json: String,
    pub result_json: String,
    pub created_at: String,
    pub completed_at: String,
}

#[derive(Debug, Serialize)]
pub struct InterventionDigest {
    pub generated_at: String,
    pub top_schools_by_deviation: Vec<SchoolDeviationBrief>,
    pub sla_breaches: Vec<SlaBreachBrief>,
    pub low_attendance_regions: Vec<LowAttendanceRegion>,
}

#[derive(Debug, Serialize)]
pub struct SchoolDeviationBrief {
    pub school_id: i64,
    pub school_name: String,
    pub deviation_score: f64,
}

#[derive(Debug, Serialize)]
pub struct SlaBreachBrief {
    pub ticket_id: i64,
    pub title: String,
    pub school_name: String,
    pub hours_overdue: i64,
}

#[derive(Debug, Serialize)]
pub struct LowAttendanceRegion {
    pub region_name: String,
    pub avg_attendance_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct SipBrief {
    pub generated_at: String,
    pub status_flips: Vec<StatusFlip>,
    pub high_deviation_subjects: Vec<SubjectDeviation>,
    pub high_substitution_faculty: Vec<FacultySubstitutionCount>,
    pub stale_tickets: Vec<StaleTicket>,
}

#[derive(Debug, Serialize)]
pub struct StatusFlip {
    pub school_id: i64,
    pub school_name: String,
    pub previous_status: String,
    pub current_status: String,
}

#[derive(Debug, Serialize)]
pub struct SubjectDeviation {
    pub school_id: i64,
    pub school_name: String,
    pub subject_name: String,
    pub deviation_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct FacultySubstitutionCount {
    pub faculty_name: String,
    pub substitution_count: i64,
}

#[derive(Debug, Serialize)]
pub struct StaleTicket {
    pub ticket_id: i64,
    pub title: String,
    pub days_open: i64,
}

#[derive(Debug, Deserialize)]
pub struct TicketFromGapInput {
    pub school_id: i64,
    pub grade_level: String,
    pub track: String,
    pub subject_name: String,
    pub gap_description: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssignUsersInput {
    pub user_ids: Vec<i64>,
    pub school_ids: Vec<i64>,
    pub role: String, // "aom" | "faculty"
}

#[derive(Debug, Deserialize)]
pub struct BulkImportSubjectsInput {
    pub school_id: i64,
    pub csv_data: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkPublishTimetablesInput {
    pub region_id: Option<i64>,
    pub school_ids: Vec<i64>,
    pub week_start_date: String,
}

#[derive(Debug, Deserialize)]
pub struct ReassignFacultyInput {
    pub faculty_user_id: i64,
    pub source_school_id: i64,
    pub target_school_id: i64,
    pub effective_week_start: String,
}

#[derive(Debug, Serialize)]
pub struct ReassignFacultyResult {
    pub cloned_slots: i64,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CloneWeekResult {
    pub cloned_slots: i64,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CrossSchoolRoomConflict {
    pub room: String,
    pub day_of_week: i64,
    pub period: i64,
    pub slots: Vec<CrossSchoolRoomConflictSlot>,
}

#[derive(Debug, Serialize)]
pub struct CrossSchoolRoomConflictSlot {
    pub school_id: i64,
    pub school_name: String,
    pub grade_level: String,
    pub track: String,
    pub subject_name: String,
    pub faculty_name: String,
    pub week_start_date: String,
}
