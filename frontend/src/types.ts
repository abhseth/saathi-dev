export type Paginated<T> = {
  items: T[];
  total_count: number;
  page: number;
  page_size: number;
};

export type UserRole = "admin" | "agent" | "viewer" | "aom" | "head" | "faculty";

export type CurrentUser = {
  id: number;
  username: string;
  display_name: string;
  role: UserRole;
  school_ids: number[];
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
  faculty_id: number;
  faculty_user_id: number | null;
  faculty_display_name: string;
  school_id: number;
  school_name: string;
  batch_id: number;
  batch_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  subject_id: number;
  subject_name: string;
  created_at: string;
};

export type CreateFacultyAssignmentDraft = {
  faculty_id: number;
  batch_id: number;
  subject_id: number;
};

export type FacultyMember = {
  id: number;
  name: string;
  email: string;
  mobile: string;
  pwid: string;
  qualification: string;
  experience_years: number;
  designation: string;
  specialization: string;
  employment_type: string;
  is_active: boolean;
  user_id: number | null;
  user_username: string | null;
  user_display_name: string | null;
  created_at: string;
  updated_at: string;
};

export type CreateFacultyMemberInput = {
  name: string;
  email?: string;
  mobile?: string;
  pwid?: string;
  qualification?: string;
  experience_years?: number;
  designation?: string;
  specialization?: string;
  employment_type?: string;
  is_active?: boolean;
  user_id?: number | null;
  initial_school_id?: number | null;
};

export type UpdateFacultyMemberInput = {
  id: number;
  name: string;
  email?: string;
  mobile?: string;
  pwid?: string;
  qualification?: string;
  experience_years?: number;
  designation?: string;
  specialization?: string;
  employment_type?: string;
  is_active: boolean;
  user_id?: number | null;
};

export type FacultySchoolMembership = {
  id: number;
  faculty_id: number;
  school_id: number;
  school_name: string;
  role_at_school: string;
  is_primary: boolean;
  created_at: string;
};

export type CreateFacultySchoolMembershipInput = {
  faculty_id: number;
  school_id: number;
  role_at_school?: string;
  is_primary?: boolean;
};

export type CreateFacultyLoginInput = {
  username: string;
  display_name: string;
  password: string;
};

export type TimetableSlot = {
  id: number;
  school_id: number;
  school_name: string;
  batch_id: number;
  batch_name: string;
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

export type WeeklyTimetableSlot = {
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
  faculty_display_name: string | null;
  start_time: string;
  end_time: string;
  room: string;
  session_type: string;
  is_substitution?: boolean;
  week_start_date: string;
  updated_at: string;
};

export type UpsertWeeklyTimetableSlotInput = {
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
  week_start_date: string;
};

export type CloneWeekInput = {
  from_week: string;
  to_week: string;
  school_id: number;
};

export type UpsertTimetableSlotDraft = {
  batch_id: number;
  day_of_week: number;
  period: number;
  subject_id: number;
  faculty_user_id: number | null;
  start_time: string;
  end_time: string;
};

export type FacultyTodaySession = {
  session_id: number;
  timetable_slot_id: number;
  session_date: string;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  batch_id: string;
  period: number;
  subject_id: number;
  subject_name: string;
  start_time: string;
  end_time: string;
  status: string;
  total_students: number;
  present_count: number;
  late_count: number;
  absent_count: number;
  faculty_name: string;
};

export type AttendanceRecord = {
  id: number;
  lecture_session_id: number;
  student_id: number;
  student_name: string;
  status: "Present" | "Absent" | "Late" | "Excused" | "Leave" | "Out of Class";
  marked_by_user_id: number | null;
  marked_at: string;
};

export type SingleAttendance = {
  student_id: number;
  status: "Present" | "Absent";
};

export type MarkAttendanceDraft = {
  records: SingleAttendance[];
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
  linked_grade_level: string;
  linked_subject: string;
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

export type StudentAttendanceSummary = {
  session_date: string;
  subject_name: string;
  status: string;
  marked_at: string;
};

export type StudentTimeline = {
  student: Student;
  tickets: Ticket[];
  comments: TicketComment[];
  history: TicketHistory[];
  attachments: TicketAttachment[];
  attendance: StudentAttendanceSummary[];
};

export type BackupResult = {
  path: string;
};

export type CsvExportResult = {
  path: string;
  files: string[];
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
  vp_tagging: string;
  is_dropped: boolean;
  dropped_at: string;
  dropped_reason: string;
  created_at: string;
};

export type SchoolDeleteImpactItem = {
  label: string;
  count: number;
};

export type SchoolDeleteImpact = {
  school_id: number;
  school_name: string;
  total_linked_records: number;
  items: SchoolDeleteImpactItem[];
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
  registration_number: string;
  grade_level: string;
  program_track: string;
  track: string;
  student_mobile: string;
  student_email: string;
  father_name: string;
  father_email: string;
  father_mobile: string;
  mother_name: string;
  mother_email: string;
  mother_mobile: string;
  batch_ref_id: number;
  batch_name: string;
  batch_id: string;
  created_at: string;
};

export type Batch = {
  id: number;
  school_id: number;
  school_name: string;
  batch_id: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  capacity: number;
  created_at: string;
};

export type CreateBatchInput = {
  school_id: number;
  batch_id: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  capacity: number;
};

export type UpdateBatchInput = CreateBatchInput & {
  id: number;
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

export type Holiday = {
  id: number;
  date: string;
  name: string;
  scope: string;
  region_id: number | null;
  region_name: string | null;
  school_id: number | null;
  school_name: string | null;
  grade_level: string | null;
  created_at: string;
};

export type CreateHolidayInput = {
  date: string;
  name: string;
  scope: string;
  region_id: number | null;
  school_id: number | null;
  grade_level: string | null;
};

export type BulkCreateHolidayInput = {
  name: string;
  start_date: string;
  end_date: string;
  scope: string;
  region_id: number | null;
  school_id: number | null;
  grade_levels: string[] | null;
};

export type CreateMakeupSessionInput = {
  school_id: number;
  grade_level: string;
  track?: string;
  subject_id: number;
  faculty_user_id?: number;
  session_date: string;
  start_time: string;
  end_time: string;
};

export type AttendanceSummaryRow = {
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  batch_id: string;
  total_students: number;
  present_count: number;
  late_count: number;
  absent_count: number;
  excused_count: number;
  attendance_percent: number;
};

export type DasGroupBy = "overall" | "school" | "class" | "cohort" | "student";

export type DasReportRow = {
  group_by: DasGroupBy;
  label: string;
  school_id: number | null;
  school_name: string;
  grade_level: string;
  cohort: string;
  batch_id: string;
  student_id: number | null;
  student_name: string;
  scheduled_lectures: number;
  present_lectures: number;
  das_percent: number;
};

export type ChronicAbsentee = {
  student_id: number;
  student_name: string;
  school_name: string;
  grade_level: string;
  total_sessions: number;
  present_count: number;
  attendance_percent: number;
};

export type SubjectAttendanceRow = {
  subject_name: string;
  total_sessions: number;
  present_count: number;
  late_count: number;
  absent_count: number;
  attendance_percent: number;
};

export type LectureSession = {
  id: number;
  timetable_slot_id: number | null;
  session_date: string;
  actual_faculty_user_id: number | null;
  subject_id: number | null;
  grade_level: string | null;
  track: string | null;
  school_id: number | null;
  start_time: string | null;
  end_time: string | null;
  status: string;
  created_at: string;
};

export type Alert = {
  id: string;
  severity: "info" | "warning" | "critical";
  category: string;
  message: string;
  school_id: number | null;
  school_name: string | null;
  grade_level: string | null;
  subject_name: string | null;
  faculty_user_id: number | null;
  faculty_name: string | null;
  created_at: string;
};

export type FacultyWeeklySlot = {
  faculty_user_id?: number;
  faculty_name?: string;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  day_of_week: number;
  period: number;
  subject_name: string;
  room: string | null;
  start_time: string;
  end_time: string;
  week_start_date: string;
  // optional fields for backward compatibility
  id?: number;
  is_substitution?: boolean;
  original_faculty_name?: string | null;
  notes?: string | null;
};

export type SubstitutionRecord = {
  session_id: number;
  session_date: string;
  original_faculty_user_id: number;
  original_faculty_name: string;
  substitute_faculty_user_id?: number;
  substitute_faculty_name: string;
  subject_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  status: string;
};

export type TimetableHealthStatus = {
  school_id: number;
  school_name: string;
  region_name: string;
  aom_name: string;
  class_plans_configured: boolean;
  master_timetable_complete: boolean;
  sessions_generated: boolean;
  gaps_count: number;
  gap_details?: string[];
  last_updated: string;
  status: "Green" | "Amber" | "Red";
};

export type ComplianceMetrics = {
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  subject_name: string;
  planned_periods: number;
  actual_periods: number;
  deviation: number;
  lecture_model_adherence_pct: number;
};



export type RoomConflict = {
  room: string;
  day_of_week: number;
  period: number;
  slots: Array<{
    school_id: number;
    school_name: string;
    grade_level: string;
    subject_name: string;
    faculty_name: string;
  }>;
};

// ── Analytics & Dashboards (Phase 6) ──────────────────────────────────────────

export type ActionableComplianceItem = {
  severity: "critical" | "warning" | "info";
  message: string;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  subject_name: string;
  planned_periods: number;
  actual_periods: number;
  deviation: number;
};

export type ControlTowerCard = {
  school_id: number;
  school_name: string;
  region_name: string;
  filled_periods: number;
  total_periods: number;
  alert_count: number;
  attendance_percent: number;
  active_substitutions: number;
};

export type FacultyUtilizationWeek = {
  week_start_date: string;
  period_count: number;
};

export type FacultyUtilizationTrend = {
  faculty_user_id: number;
  faculty_name: string;
  weeks: FacultyUtilizationWeek[];
};

export type SubjectGap = {
  subject_name: string;
  grade_level: string;
  track: string;
  planned: number;
  actual: number;
};

export type DeviationScoreboardRow = {
  school_id: number;
  school_name: string;
  region_name: string;
  overall_deviation_score: number;
  top_gaps: SubjectGap[];
};

export type SessionTypeBreakdown = {
  session_type: string;
  planned_periods: number;
  actual_periods: number;
  adherence_pct: number;
};

export type FacultyStabilityRow = {
  faculty_user_id: number;
  faculty_name: string;
  school_name: string;
  substitution_rate_pct: number;
  cancellation_rate_pct: number;
  planned_vs_actual_variance: number;
};

export type SubjectCoverageCell = {
  region_name: string;
  subject_name: string;
  adherence_pct: number;
};

export type HealthTrendWeek = {
  week_start_date: string;
  green_count: number;
  amber_count: number;
  red_count: number;
  network_adherence_pct: number;
};

export type SubstitutionTrendWeek = {
  week_start_date: string;
  faculty_absences: number;
  short_staffed_periods: number;
  over_utilized_substitutes: number;
};

export type RegionHeatmapCell = {
  school_id: number;
  school_name: string;
  day_of_week: number;
  issue_count: number;
};

export type RoomConflictRadarCell = {
  room: string;
  day_of_week: number;
  period: number;
  conflict_count: number;
  schools: string[];
};

export type AdherenceComparisonRow = {
  school_id: number;
  school_name: string;
  adherence_pct: number;
  deviation: number;
};

export type WeekDiffSlot = {
  id: number;
  school_id: number;
  grade_level: string;
  track: string;
  batch_pattern: string;
  day_of_week: number;
  period: number;
  subject_name: string;
  faculty_display_name: string | null;
  room: string;
  session_type: string;
  change_type: "added" | "removed" | "modified";
};

export type CompliancePivotRow = {
  dimension_value: string;
  planned_periods: number;
  actual_periods: number;
  deviation: number;
  adherence_pct: number;
};

// ── Notifications (Phase 6 Mobile & Notifications Layer) ─────────────────────

export type Notification = {
  id: number;
  user_id: number;
  notification_type: string;
  title: string;
  message: string;
  payload_json: string;
  read_at: string;
  created_at: string;
};

export type MobileDigestData = {
  facultyAbsent: number;
  periodsNeedingSubstitutes: number;
  urgentTickets: number;
};

export type AomSchoolCard = {
  school_id: number;
  school_name: string;
  attendance_percent: number;
  unfilled_periods: number;
  open_substitutions: number;
  status: "green" | "amber" | "red";
};

export type SpocKpiCard = {
  label: string;
  value: number | string;
  trend?: "up" | "down" | "neutral";
  alert?: boolean;
};

export type SipBriefCard = {
  title: string;
  count: number;
  items: string[];
};

export type ClassCard = {
  period: number;
  subject_name: string;
  room: string;
  grade_level: string;
  track: string;
  start_time: string;
  end_time: string;
  school_name: string;
  is_substitution?: boolean;
  original_faculty_name?: string | null;
};

// ── Phase 6: Substitution & Leave Engine ─────────────────────────────────────

export type LeaveRequest = {
  id: number;
  faculty_user_id: number;
  faculty_name: string;
  school_id: number;
  school_name: string;
  start_date: string;
  end_date: string;
  reason: string;
  status: "Pending" | "Approved" | "Rejected";
  approved_by_user_id?: number;
  approved_at: string;
  rejected_by_user_id?: number;
  rejected_at: string;
  rejection_reason: string;
  created_at: string;
};

export type CreateLeaveRequestInput = {
  faculty_user_id: number;
  school_id: number;
  start_date: string;
  end_date: string;
  reason?: string;
};

export type LeaveImpactPreview = {
  leave_request_id: number;
  affected_session_count: number;
  date_range_start: string;
  date_range_end: string;
  school_name: string;
  faculty_name: string;
};

export type LeaveRequestAuditLog = {
  id: number;
  leave_request_id: number;
  actor_user_id: number;
  action: string;
  old_status: string;
  new_status: string;
  reason: string;
  school_id: number;
  created_at: string;
};

export type SwapRequest = {
  id: number;
  requester_faculty_id: number;
  requester_name: string;
  recipient_faculty_id: number;
  recipient_name: string;
  slot_a_id: number;
  slot_b_id: number;
  status: "Pending" | "Accepted" | "Declined";
  created_at: string;
};

export type CreateSwapRequestInput = {
  requester_faculty_id: number;
  recipient_faculty_id: number;
  slot_a_id: number;
  slot_b_id: number;
};

export type SubstituteCandidate = {
  faculty_user_id: number;
  faculty_name: string;
  subject_match: boolean;
  free_period: boolean;
  same_school: boolean;
  workload_score: number;
  overall_score: number;
};

export type TodaySubstitutionLane = {
  session_id: number;
  session_date: string;
  school_id: number;
  school_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  period: number;
  subject_name: string;
  original_faculty_name: string;
  substitute_faculty_name: string | null;
  status: string;
  room: string;
};

export type TodaySubstitutions = {
  unfilled: TodaySubstitutionLane[];
  assigned: TodaySubstitutionLane[];
  completed: TodaySubstitutionLane[];
};

export type SubstitutionDetail = {
  session_id: number;
  session_date: string;
  school_name: string;
  grade_level: string;
  track: string;
  batch_pattern: string;
  subject_name: string;
  room: string;
  roster_count: number;
  present_count: number;
  absent_count: number;
  last_covered_topics: string;
};

export type SubstitutionBalance = {
  faculty_user_id: number;
  faculty_name: string;
  given_count: number;
  received_count: number;
};

export type SubstitutionReportRow = {
  school_id: number;
  school_name: string;
  month: string;
  request_count: number;
  filled_count: number;
  acceptance_rate_pct: number;
  avg_minutes_to_fill: number;
  top_absentee_name: string;
  top_absentee_count: number;
};

export type BulkAttendanceInput = {
  faculty_user_ids: number[];
  date: string;
  reason?: string;
};

export type MarkAttendanceQuickInput = {
  session_id: number;
  student_id: number;
  status: "Present" | "Absent" | "Late" | "Excused";
};


// ── Automation & Policy Engine (Phase 6) ─────────────────────────────────────

export type CentralPolicy = {
  id: number;
  key: string;
  value: string;
  region_id: number | null;
  updated_at: string;
};

export type EscalationRule = {
  id: number;
  name: string;
  conditions_json: string;
  action: string;
  assignee_role: string;
  hours_threshold: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type AlertState = {
  id: number;
  alert_hash: string;
  user_id: number;
  dismissed_at: string;
  snoozed_until: string;
  converted_ticket_id: number | null;
};

export type Announcement = {
  id: number;
  school_id: number | null;
  school_name: string | null;
  message: string;
  pinned_until: string;
  created_by: number;
  created_by_name: string;
  created_at: string;
};

export type BulkOperationLog = {
  id: number;
  op_type: string;
  status: string;
  payload_json: string;
  result_json: string;
  created_at: string;
  completed_at: string;
};

export type InterventionDigest = {
  generated_at: string;
  top_schools_by_deviation: Array<{
    school_id: number;
    school_name: string;
    deviation_score: number;
  }>;
  sla_breaches: Array<{
    ticket_id: number;
    title: string;
    school_name: string;
    hours_overdue: number;
  }>;
  low_attendance_regions: Array<{
    region_name: string;
    avg_attendance_pct: number;
  }>;
};

export type SipBrief = {
  generated_at: string;
  status_flips: Array<{
    school_id: number;
    school_name: string;
    previous_status: string;
    current_status: string;
  }>;
  high_deviation_subjects: Array<{
    school_id: number;
    school_name: string;
    subject_name: string;
    deviation_pct: number;
  }>;
  high_substitution_faculty: Array<{
    faculty_name: string;
    substitution_count: number;
  }>;
  stale_tickets: Array<{
    ticket_id: number;
    title: string;
    days_open: number;
  }>;
};

export type ReassignFacultyResult = {
  cloned_slots: number;
  conflicts: string[];
};

export type CloneWeekResult = {
  cloned_slots: number;
  conflicts: string[];
};

export type CrossSchoolRoomConflict = {
  room: string;
  day_of_week: number;
  period: number;
  slots: Array<{
    school_id: number;
    school_name: string;
    grade_level: string;
    track: string;
    subject_name: string;
    faculty_name: string;
    week_start_date: string;
  }>;
};

// ── VP Centers & Faculty Profiles ─────────────────────────────────────────────

export type VpCenter = {
  id: number;
  name: string;
  location: string;
  contact_name: string;
  contact_mobile: string;
  contact_email: string;
  created_at: string;
};

export type CreateVpCenterInput = {
  name: string;
  location: string;
  contact_name: string;
  contact_mobile: string;
  contact_email: string;
};

export type UpdateVpCenterInput = {
  id: number;
  name: string;
  location: string;
  contact_name: string;
  contact_mobile: string;
  contact_email: string;
};

export type VpCenterBuilding = {
  id: number;
  vp_center_id: number;
  building_name: string;
  address: string;
  center_head_name: string;
  center_head_mobile: string;
  center_head_email: string;
  associate_center_head_name: string;
  associate_center_head_mobile: string;
  associate_center_head_email: string;
  created_at: string;
};

export type CreateVpCenterBuildingInput = {
  vp_center_id: number;
  building_name: string;
  address: string;
  center_head_name: string;
  center_head_mobile: string;
  center_head_email: string;
  associate_center_head_name: string;
  associate_center_head_mobile: string;
  associate_center_head_email: string;
};

export type UpdateVpCenterBuildingInput = {
  id: number;
  vp_center_id: number;
  building_name: string;
  address: string;
  center_head_name: string;
  center_head_mobile: string;
  center_head_email: string;
  associate_center_head_name: string;
  associate_center_head_mobile: string;
  associate_center_head_email: string;
};

export type FacultyProfile = {
  faculty_user_id: number;
  faculty_display_name: string;
  pwid: string;
  email: string;
  mobile: string;
  emergency_contact_name: string;
  emergency_contact_mobile: string;
  vp_center_id: number | null;
  vp_center_name: string;
  sip_school_id: number | null;
  sip_school_name: string;
  primary_subject_id: number | null;
  primary_subject_name: string;
  employment_type: string;
  qualification: string;
  experience_years: number;
  designation: string;
  specialization: string;
  max_periods_per_week: number;
  joining_date: string;
  exit_date: string;
  documents_verified: boolean;
  is_active: boolean;
  wings: string[];
  batch_ids: number[];
  created_at: string;
  updated_at: string;
};

export type UpsertFacultyProfileInput = {
  faculty_user_id: number;
  pwid: string;
  email: string;
  mobile: string;
  emergency_contact_name: string;
  emergency_contact_mobile: string;
  vp_center_id: number | null;
  sip_school_id: number | null;
  primary_subject_id: number | null;
  employment_type: string;
  qualification: string;
  experience_years: number;
  designation: string;
  specialization: string;
  max_periods_per_week: number;
  joining_date: string;
  exit_date: string;
  documents_verified: boolean;
  is_active: boolean;
  wings: string[];
  batch_ids: number[];
};
