export type UserRole = "admin" | "agent" | "viewer" | "aom" | "faculty";

export type CurrentUser = {
  id: number;
  username: string;
  display_name: string;
  role: UserRole;
};

export type AppUser = {
  id: number;
  username: string;
  display_name: string;
  role: string;
  is_active: boolean;
  created_at: string;
  last_login_at: string;
  school_ids: number[];
};

export type LoginDraft = {
  username: string;
  password: string;
};

export type CreateUserDraft = {
  username: string;
  display_name: string;
  role: string;
  password: string;
  school_ids: number[];
};

export type UpdateUserDraft = {
  id: number;
  username: string;
  display_name: string;
  role: string;
  is_active: boolean;
  school_ids: number[];
};

export type Subject = {
  id: number;
  name: string;
  track: string;
  is_default: boolean;
  sort_order: number;
};

export type EffectiveSubject = {
  id: number;
  name: string;
  track: string;
  is_default: boolean;
  is_offered: boolean;
};

export type FacultyAssignment = {
  id: number;
  faculty_user_id: number;
  faculty_display_name: string;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  subject_id: number;
  subject_name: string;
  created_at: string;
};

export type CreateFacultyAssignmentDraft = {
  faculty_user_id: number;
  school_id: number;
  grade_level: string;
  track: string;
  subject_id: number;
};

export type TimetableSlot = {
  id: number;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  day_of_week: number;
  period: number;
  subject_id: number;
  subject_name: string;
  faculty_user_id: number | null;
  faculty_display_name: string;
  start_time: string;
  end_time: string;
  updated_at: string;
};

export type UpsertTimetableSlotDraft = {
  school_id: number;
  grade_level: string;
  track: string;
  batch_pattern: string;
  day_of_week: number;
  period: number;
  subject_id: number;
  faculty_user_id: number | null;
  start_time: string;
  end_time: string;
};

export type Status = "Open" | "In Progress" | "Pending" | "Resolved" | "Closed";
export type Priority = "Low" | "Medium" | "High" | "Critical";
export type Filter =
  | "Inbox"
  | "My Tickets"
  | "Unassigned"
  | "Pending SLA"
  | "Escalated"
  | "Resolved";
export type Queue =
  | "Academic Support"
  | "Learning Platform"
  | "IT / Device"
  | "Operations"
  | "Parent Communication";

export type ProgramScopeFilters = {
  school_name: string;
  grade_level: string;
  program_track: string;
  issue_category: string;
  queue: string;
};

export type Ticket = {
  id: number;
  title: string;
  description: string;
  requester: string;
  assignee: string;
  status: Status;
  priority: Priority;
  queue: Queue;
  school_id: number | null;
  school_name: string;
  student_name: string;
  grade_level: string;
  program_track: string;
  issue_category: string;
  sla_due_at: string;
  escalation_status: "None" | "At Risk" | "Escalated";
  escalated_at: string;
  created_at: string;
  updated_at: string;
};

export type TicketComment = {
  id: number;
  ticket_id: number;
  author: string;
  body: string;
  is_internal: boolean;
  channel: string;
  audience: string;
  recipient_name: string;
  recipient_contact: string;
  delivery_status: string;
  last_contacted_at: string;
  next_follow_up_due: string;
  created_at: string;
};

export type TicketHistory = {
  id: number;
  ticket_id: number;
  actor: string;
  field: string;
  old_value: string;
  new_value: string;
  created_at: string;
};

export type AuditLogEntry = {
  id: number;
  entity_type: string;
  entity_id: number;
  action: string;
  actor: string;
  summary: string;
  created_at: string;
};

export type TicketAttachment = {
  id: number;
  ticket_id: number;
  original_filename: string;
  stored_path: string;
  size_bytes: number;
  uploaded_by: string;
  created_at: string;
};

export type StudentTimeline = {
  school_name: string;
  student_name: string;
  tickets: Ticket[];
  comments: TicketComment[];
  history: TicketHistory[];
  attachments: TicketAttachment[];
};

export type BackupResult = {
  path: string;
};

export type CsvExportResult = {
  path: string;
  files: string[];
};

export type SyncSnapshotInfo = {
  path: string;
  size_bytes: number;
  ticket_count: number;
  comment_count: number;
  history_count: number;
  attachment_count: number;
  latest_ticket_update: string;
};

export type ImportSyncSnapshotResult = {
  imported: SyncSnapshotInfo;
  local_backup_path: string;
};

export type SlaPolicy = {
  issue_category: string;
  hours: number;
};

export type AssignmentRule = {
  queue: Queue;
  assignee: string;
  is_active: boolean;
  updated_at: string;
};

export type EscalationPolicy = {
  at_risk_hours: number;
  escalation_assignee: string;
  auto_assign_on_breach: boolean;
  updated_at: string;
};

export type CommunicationTemplate = {
  id: number;
  name: string;
  audience: string;
  body: string;
  is_active: boolean;
  updated_at: string;
};

export type School = {
  id: number;
  name: string;
  region_id: number | null;
  region_name: string;
  program_model: string;
  distance_classification: string;
  sip_academic_owner_role: string;
  sip_academic_owner_name: string;
  sip_academic_owner_mobile: string;
  sip_academic_owner_email: string;
  center_head_name: string;
  center_head_mobile: string;
  center_head_email: string;
  principal_name: string;
  principal_mobile: string;
  principal_email: string;
  school_spoc_name: string;
  school_spoc_mobile: string;
  school_spoc_email: string;
  central_academic_spoc_name: string;
  central_academic_spoc_mobile: string;
  central_academic_spoc_email: string;
  central_business_spoc_name: string;
  central_business_spoc_mobile: string;
  central_business_spoc_email: string;
  bh_name: string;
  bh_mobile: string;
  bh_email: string;
  aom_name: string;
  aom_mobile: string;
  aom_email: string;
  mapped_vp_center: string;
  is_dropped: boolean;
  dropped_at: string;
  dropped_reason: string;
  created_at: string;
};

export type SchoolProfileDraft = Omit<
  School,
  "id" | "created_at" | "is_dropped" | "dropped_at" | "dropped_reason"
>;

export type SchoolRegionHistory = {
  id: number;
  school_id: number;
  school_name: string;
  old_region_id: number | null;
  old_region_name: string;
  new_region_id: number | null;
  new_region_name: string;
  changed_at: string;
};

export type Region = {
  id: number;
  name: string;
  regional_academic_head_name: string;
  regional_academic_head_mobile: string;
  regional_academic_head_email: string;
  regional_business_head_name: string;
  regional_business_head_mobile: string;
  regional_business_head_email: string;
  updated_at: string;
};

export type SchoolImportResult = {
  imported_count: number;
  skipped_count: number;
};

export type SipMasterImportPreview = {
  total_rows: number;
  new_school_count: number;
  existing_school_count: number;
  skipped_row_count: number;
  existing_schools: string[];
};

export type SipMasterImportResult = {
  imported_count: number;
  updated_count: number;
  skipped_count: number;
  class_plan_count: number;
};

export type LectureModel = {
  id: number;
  name: string;
  days_per_week: number;
  lectures_per_day: number;
  created_at: string;
};

export type SchoolClassPlan = {
  id: number;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  lecture_model_id: number;
  lecture_model_name: string;
  days_per_week: number;
  lectures_per_day: number;
  batch_pattern: string;
  aop_admissions: number;
  registrations: number;
  actual_admissions: number;
  admission_gap: number;
  admission_attainment_percent: number;
  updated_at: string;
};

export type SchoolProgramDashboard = {
  total_schools: number;
  schools_with_class_plans: number;
  total_classes: number;
  total_aop_admissions: number;
  total_actual_admissions: number;
  admission_gap: number;
  admission_attainment_percent: number;
  remote_school_count: number;
  near_proximity_school_count: number;
  aspire_school_count: number;
  minimum_guarantee_school_count: number;
  class_plans: SchoolClassPlan[];
};

export type Student = {
  id: number;
  school_id: number;
  school_name: string;
  name: string;
  grade_level: string;
  program_track: string;
  created_at: string;
};

export type CreateTicketDraft = {
  title: string;
  description: string;
  requester: string;
  priority: Priority;
  school_id: number | null;
  school_name: string;
  student_name: string;
  grade_level: string;
  program_track: string;
  issue_category: string;
};

export type ReplyDraft = {
  author: string;
  body: string;
  is_internal: boolean;
  channel: string;
  audience: string;
  recipient_name: string;
  recipient_contact: string;
  next_follow_up_due: string;
};

export type TicketEditDraft = {
  title: string;
  description: string;
  requester: string;
  school_id: number | null;
  school_name: string;
  student_name: string;
  grade_level: string;
  program_track: string;
  issue_category: string;
};

export type AttachmentDraft = {
  source_path: string;
  uploaded_by: string;
};

export type TicketChanges = Partial<
  Pick<
    Ticket,
    | "title"
    | "description"
    | "requester"
    | "status"
    | "priority"
    | "assignee"
    | "queue"
    | "school_id"
    | "school_name"
    | "student_name"
    | "grade_level"
    | "program_track"
    | "issue_category"
  >
>;
