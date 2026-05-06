import React from "react";
import { api, uploadFile } from "../api";
import type {
  AttendanceRecord,
  AttendanceSummaryRow,
  ChronicAbsentee,
  ComplianceMetrics,
  DasGroupBy,
  DasReportRow,
  CreateFacultyAssignmentDraft,
  CreateFacultyLoginInput,
  CreateFacultyMemberInput,
  CreateFacultySchoolMembershipInput,
  BulkCreateHolidayInput,
  CreateHolidayInput,
  CreateLeaveRequestInput,
  CreateMakeupSessionInput,
  EffectiveSubject,
  FacultyAssignment,
  FacultyMember,
  FacultyProfile,
  FacultySchoolMembership,
  FacultyTodaySession,
  FacultyWeeklySlot,
  FacultyUtilizationWeek,
  Holiday,
  LeaveRequest,
  LeaveRequestAuditLog,
  LectureSession,
  MarkAttendanceDraft,
  SubstitutionRecord,
  Subject,
  SubjectAttendanceRow,
  SwapRequest,
  TimetableHealthStatus,
  TimetableSlot,
  TodaySubstitutions,
  UpdateFacultyMemberInput,
  UpsertTimetableSlotDraft,
  UpsertWeeklyTimetableSlotInput,
  VpCenter,
  VpCenterBuilding,
  WeeklyTimetableSlot,
} from "../types";

export interface UseFacultyStateOptions {
  currentUser: { role: string } | null;
  onError: (msg: string) => void;
  onNotice: (msg: string) => void;
}

export interface UseFacultyStateReturn {
  // ── Core entities ──
  subjects: Subject[];
  effectiveSubjects: EffectiveSubject[];
  facultyAssignments: FacultyAssignment[];
  timetableSlots: TimetableSlot[];
  weeklyTimetableSlots: WeeklyTimetableSlot[];

  // ── Operational state ──
  lectureSessions: LectureSession[];
  adminSessions: FacultyTodaySession[];
  facultyWeeklySlots: FacultyWeeklySlot[];

  // ── Leave & substitutions ──
  leaveRequests: LeaveRequest[];
  swapRequests: SwapRequest[];
  substitutions: SubstitutionRecord[];
  pendingSubstitutionRequests: SubstitutionRecord[];
  todaySubstitutions: TodaySubstitutions | null;
  substitutionBalance: Record<string, number>;
  substitutionReports: { date: string; given: number; taken: number }[];

  // ── Compliance & health ──
  timetableHealthData: TimetableHealthStatus | null;
  complianceData: ComplianceMetrics | null;

  // ── Attendance ──
  attendanceSummary: AttendanceSummaryRow[];
  dasReport: DasReportRow[];
  chronicAbsentees: ChronicAbsentee[];
  subjectAttendance: SubjectAttendanceRow[];
  attendanceReportDate: string;

  // ── VP centers & profiles ──
  vpCenters: VpCenter[];
  vpCenterBuildings: VpCenterBuilding[];
  facultyProfiles: FacultyProfile[];

  // ── Faculty Members (master data) ──
  facultyMembers: FacultyMember[];
  facultyMemberMemberships: Record<number, FacultySchoolMembership[]>;

  // ── Holidays ──
  holidays: Holiday[];

  // ── Loaders ──
  loadSubjects: () => Promise<void>;
  loadEffectiveSubjects: (schoolId: number, track: string) => Promise<EffectiveSubject[]>;
  loadFacultyAssignments: () => Promise<void>;
  loadTimetableSlots: (params: {
    schoolId?: number;
    gradeLevel?: string;
    track?: string;
    batchPattern?: string;
  }) => Promise<void>;
  loadWeeklyTimetable: (schoolId: number, weekStart: string) => Promise<void>;
  loadLectureSessions: (schoolId: number, gradeLevel: string) => Promise<void>;
  loadAdminSessions: () => Promise<void>;
  loadFacultyWeeklySlots: (facultyUserId: number, weekStart: string) => Promise<void>;
  loadSubstitutions: () => Promise<void>;
  loadPendingSubstitutionRequests: () => Promise<void>;
  loadTodaySubstitutions: () => Promise<void>;
  loadSubstitutionBalance: (facultyUserId: number) => Promise<void>;
  loadSubstitutionReports: (facultyUserId: number, weeks: number) => Promise<void>;
  loadTimetableHealth: () => Promise<void>;
  loadComplianceData: () => Promise<void>;
  loadAttendanceSummary: (date: string) => Promise<void>;
  loadDasReport: (startDate: string, endDate: string, groupBy: DasGroupBy, schoolId?: number) => Promise<void>;
  loadChronicAbsentees: () => Promise<void>;
  loadSubjectAttendance: (date: string) => Promise<void>;
  loadVpCenters: () => Promise<void>;
  loadVpCenterBuildings: (centerId: number) => Promise<void>;
  loadFacultyProfiles: () => Promise<void>;
  loadFacultyMembers: () => Promise<void>;
  loadFacultySchoolMemberships: (facultyId: number) => Promise<void>;
  loadHolidays: () => Promise<void>;
  loadLeaveRequests: () => Promise<void>;
  loadSwapRequests: () => Promise<void>;

  // ── Mutations ──
  createSubject: (input: Omit<Subject, "id">) => Promise<void>;
  updateSubject: (input: Subject) => Promise<void>;
  deleteSubject: (id: number) => Promise<void>;
  toggleOptionalSubject: (schoolId: number, subjectId: number, enabled: boolean) => Promise<void>;
  createFacultyAssignment: (input: CreateFacultyAssignmentDraft) => Promise<void>;
  deleteFacultyAssignment: (id: number) => Promise<void>;
  createFacultyMember: (input: CreateFacultyMemberInput) => Promise<void>;
  updateFacultyMember: (input: UpdateFacultyMemberInput) => Promise<void>;
  deleteFacultyMember: (id: number) => Promise<void>;
  createFacultySchoolMembership: (input: CreateFacultySchoolMembershipInput) => Promise<void>;
  deleteFacultySchoolMembership: (id: number, facultyId: number) => Promise<void>;
  importFacultyMembersCsv: (file: File) => Promise<void>;
  createFacultyLogin: (facultyId: number, input: CreateFacultyLoginInput) => Promise<void>;
  linkFacultyUser: (facultyId: number, userId: number) => Promise<void>;
  upsertTimetableSlot: (input: UpsertTimetableSlotDraft) => Promise<void>;
  deleteTimetableSlot: (id: number) => Promise<void>;
  upsertWeeklyTimetable: (input: UpsertWeeklyTimetableSlotInput) => Promise<void>;
  deleteWeeklyTimetable: (id: number) => Promise<void>;
  cloneWeek: (sourceWeekStart: string, targetWeekStart: string, schoolId?: number) => Promise<void>;
  createMakeupSession: (input: CreateMakeupSessionInput) => Promise<void>;
  handleAcceptSubstitution: (requestId: number) => Promise<void>;
  handleDeclineSubstitution: (requestId: number) => Promise<void>;
  handleSubstituteSession: (sessionId: number, substituteFacultyUserId: number) => Promise<void>;
  markAttendance: (sessionId: number, input: MarkAttendanceDraft) => Promise<void>;
  createHoliday: (input: CreateHolidayInput) => Promise<void>;
  bulkCreateHoliday: (input: BulkCreateHolidayInput) => Promise<void>;
  deleteHoliday: (id: number) => Promise<void>;
  createLeaveRequest: (input: CreateLeaveRequestInput) => Promise<void>;
  approveLeaveRequest: (id: number) => Promise<void>;
  rejectLeaveRequest: (id: number) => Promise<void>;
}

export function useFacultyState(options: UseFacultyStateOptions): UseFacultyStateReturn {
  const { onError, onNotice } = options;

  // ── Core entities ──
  const [subjects, setSubjects] = React.useState<Subject[]>([]);
  const [effectiveSubjects, setEffectiveSubjects] = React.useState<EffectiveSubject[]>([]);
  const [facultyAssignments, setFacultyAssignments] = React.useState<FacultyAssignment[]>([]);
  const [timetableSlots, setTimetableSlots] = React.useState<TimetableSlot[]>([]);
  const [weeklyTimetableSlots, setWeeklyTimetableSlots] = React.useState<WeeklyTimetableSlot[]>([]);

  // ── Operational state ──
  const [lectureSessions, setLectureSessions] = React.useState<LectureSession[]>([]);
  const [adminSessions, setAdminSessions] = React.useState<FacultyTodaySession[]>([]);
  const [facultyWeeklySlots, setFacultyWeeklySlots] = React.useState<FacultyWeeklySlot[]>([]);

  // ── Leave & substitutions ──
  const [leaveRequests, setLeaveRequests] = React.useState<LeaveRequest[]>([]);
  const [swapRequests, setSwapRequests] = React.useState<SwapRequest[]>([]);
  const [substitutions, setSubstitutions] = React.useState<SubstitutionRecord[]>([]);
  const [pendingSubstitutionRequests, setPendingSubstitutionRequests] = React.useState<SubstitutionRecord[]>([]);
  const [todaySubstitutions, setTodaySubstitutions] = React.useState<TodaySubstitutions | null>(null);
  const [substitutionBalance, setSubstitutionBalance] = React.useState<Record<string, number>>({});
  const [substitutionReports, setSubstitutionReports] = React.useState<{ date: string; given: number; taken: number }[]>([]);

  // ── Compliance & health ──
  const [timetableHealthData, setTimetableHealthData] = React.useState<TimetableHealthStatus | null>(null);
  const [complianceData, setComplianceData] = React.useState<ComplianceMetrics | null>(null);

  // ── Attendance ──
  const [attendanceSummary, setAttendanceSummary] = React.useState<AttendanceSummaryRow[]>([]);
  const [dasReport, setDasReport] = React.useState<DasReportRow[]>([]);
  const [chronicAbsentees, setChronicAbsentees] = React.useState<ChronicAbsentee[]>([]);
  const [subjectAttendance, setSubjectAttendance] = React.useState<SubjectAttendanceRow[]>([]);
  const [attendanceReportDate, setAttendanceReportDate] = React.useState<string>(new Date().toISOString().split("T")[0]);

  // ── VP centers & profiles ──
  const [vpCenters, setVpCenters] = React.useState<VpCenter[]>([]);
  const [vpCenterBuildings, setVpCenterBuildings] = React.useState<VpCenterBuilding[]>([]);
  const [facultyProfiles, setFacultyProfiles] = React.useState<FacultyProfile[]>([]);

  // ── Faculty Members (master data) ──
  const [facultyMembers, setFacultyMembers] = React.useState<FacultyMember[]>([]);
  const [facultyMemberMemberships, setFacultyMemberMemberships] = React.useState<Record<number, FacultySchoolMembership[]>>({});

  // ── Holidays ──
  const [holidays, setHolidays] = React.useState<Holiday[]>([]);

  // ── Core loaders ──
  const loadSubjects = React.useCallback(async () => {
    try {
      setSubjects(await api<Subject[]>("list_subjects"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadEffectiveSubjects = React.useCallback(async (schoolId: number, track: string) => {
    return api<EffectiveSubject[]>("list_effective_subjects", { schoolId, track });
  }, []);

  const loadFacultyAssignments = React.useCallback(async () => {
    try {
      setFacultyAssignments(await api<FacultyAssignment[]>("list_faculty_assignments"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadTimetableSlots = React.useCallback(async (params: {
    schoolId?: number;
    gradeLevel?: string;
    track?: string;
    batchPattern?: string;
  }) => {
    try {
      setTimetableSlots(await api<TimetableSlot[]>("list_timetable_slots", params));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  // ── Core mutations ──
  const toggleOptionalSubject = React.useCallback(async (schoolId: number, subjectId: number, enabled: boolean) => {
    try {
      await api("set_school_optional_subject", { schoolId, input: { subject_id: subjectId, enabled } });
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const createFacultyAssignment = React.useCallback(async (input: CreateFacultyAssignmentDraft) => {
    try {
      await api<FacultyAssignment>("create_faculty_assignment", { input });
      await loadFacultyAssignments();
      onNotice("Faculty assignment added.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyAssignments, onError, onNotice]);

  const deleteFacultyAssignment = React.useCallback(async (id: number) => {
    try {
      await api("delete_faculty_assignment", { id });
      await loadFacultyAssignments();
      onNotice("Faculty assignment removed.");
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyAssignments, onError, onNotice]);

  const upsertTimetableSlot = React.useCallback(async (input: UpsertTimetableSlotDraft) => {
    try {
      await api<TimetableSlot>("upsert_timetable_slot", { input });
      await loadTimetableSlots({});
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadTimetableSlots, onError]);

  const deleteTimetableSlot = React.useCallback(async (id: number) => {
    try {
      const slot = timetableSlots.find((s) => s.id === id);
      await api("delete_timetable_slot", { id });
      if (slot) {
        await loadTimetableSlots({
          schoolId: slot.school_id,
          gradeLevel: slot.grade_level,
          track: slot.track,
          batchPattern: slot.batch_pattern,
        });
      }
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [timetableSlots, loadTimetableSlots, onError]);

  // ── Advanced loaders (scaffolded for future use) ──
  const loadWeeklyTimetable = React.useCallback(async (schoolId: number, weekStart: string) => {
    try {
      setWeeklyTimetableSlots(await api<WeeklyTimetableSlot[]>("list_weekly_timetable", { schoolId, weekStart }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadLectureSessions = React.useCallback(async (schoolId: number, gradeLevel: string) => {
    try {
      setLectureSessions(await api<LectureSession[]>("lecture_sessions", { school_id: schoolId, grade_level: gradeLevel, from: "", to: "" }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAdminSessions = React.useCallback(async () => {
    try {
      setAdminSessions(await api<FacultyTodaySession[]>("admin_today_sessions"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadFacultyWeeklySlots = React.useCallback(async (facultyUserId: number, weekStart: string) => {
    try {
      setFacultyWeeklySlots(await api<FacultyWeeklySlot[]>("list_faculty_schedule", { facultyUserId, weekStart }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSubstitutions = React.useCallback(async () => {
    try {
      setSubstitutions(await api<SubstitutionRecord[]>("list_substitutions"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadPendingSubstitutionRequests = React.useCallback(async () => {
    try {
      setPendingSubstitutionRequests(await api<SubstitutionRecord[]>("pending_substitutions"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadTodaySubstitutions = React.useCallback(async () => {
    try {
      setTodaySubstitutions(await api<TodaySubstitutions>("today_substitutions"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSubstitutionBalance = React.useCallback(async (facultyUserId: number) => {
    try {
      setSubstitutionBalance(await api<Record<string, number>>("substitution_balance", { facultyUserId }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSubstitutionReports = React.useCallback(async (facultyUserId: number, weeks: number) => {
    try {
      setSubstitutionReports(await api<{ date: string; given: number; taken: number }[]>("substitution_reports", { facultyUserId, weeks }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadTimetableHealth = React.useCallback(async () => {
    try {
      setTimetableHealthData(await api<TimetableHealthStatus>("list_timetable_health"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadComplianceData = React.useCallback(async () => {
    try {
      setComplianceData(await api<ComplianceMetrics>("list_compliance_metrics"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAttendanceSummary = React.useCallback(async (date: string) => {
    try {
      setAttendanceSummary(await api<AttendanceSummaryRow[]>("attendance_summary", { date }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadDasReport = React.useCallback(async (
    startDate: string,
    endDate: string,
    groupBy: DasGroupBy,
    schoolId?: number,
  ) => {
    try {
      setDasReport(await api<DasReportRow[]>("das_report", { startDate, endDate, groupBy, schoolId }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadChronicAbsentees = React.useCallback(async () => {
    try {
      setChronicAbsentees(await api<ChronicAbsentee[]>("chronic_absentees"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSubjectAttendance = React.useCallback(async (date: string) => {
    try {
      setSubjectAttendance(await api<SubjectAttendanceRow[]>("subject_attendance", { date }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadVpCenters = React.useCallback(async () => {
    try {
      setVpCenters(await api<VpCenter[]>("list_vp_centers"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadVpCenterBuildings = React.useCallback(async (centerId: number) => {
    try {
      setVpCenterBuildings(await api<VpCenterBuilding[]>("list_vp_center_buildings", { centerId }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadFacultyProfiles = React.useCallback(async () => {
    try {
      setFacultyProfiles(await api<FacultyProfile[]>("list_faculty_profiles"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadFacultyMembers = React.useCallback(async () => {
    try {
      setFacultyMembers(await api<FacultyMember[]>("list_faculty_members"));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadFacultySchoolMemberships = React.useCallback(async (facultyId: number) => {
    try {
      const data = await api<FacultySchoolMembership[]>("list_faculty_school_memberships", { facultyId });
      setFacultyMemberMemberships((prev) => ({ ...prev, [facultyId]: data }));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadHolidays = React.useCallback(async () => {
    try {
      setHolidays(await api<Holiday[]>("list_holidays"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadLeaveRequests = React.useCallback(async () => {
    try {
      setLeaveRequests(await api<LeaveRequest[]>("list_leave_requests"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadSwapRequests = React.useCallback(async () => {
    try {
      setSwapRequests(await api<SwapRequest[]>("list_swap_requests"));
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  // ── Advanced mutations (scaffolded) ──
  const upsertWeeklyTimetable = React.useCallback(async (input: UpsertWeeklyTimetableSlotInput) => {
    try {
      await api("upsert_weekly_timetable", { input });
      onNotice("Weekly timetable updated.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const deleteWeeklyTimetable = React.useCallback(async (id: number) => {
    try {
      await api("delete_weekly_timetable", { id });
      onNotice("Weekly slot removed.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const cloneWeek = React.useCallback(async (sourceWeekStart: string, targetWeekStart: string, schoolId?: number) => {
    try {
      await api("clone_week", { input: { source_week_start: sourceWeekStart, target_week_start: targetWeekStart, school_id: schoolId } });
      onNotice("Week cloned.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const createMakeupSession = React.useCallback(async (input: CreateMakeupSessionInput) => {
    try {
      await api("create_makeup_session", { input });
      onNotice("Make-up session created.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const handleAcceptSubstitution = React.useCallback(async (requestId: number) => {
    try {
      await api("accept_substitution", { requestId });
      onNotice("Substitution accepted.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const handleDeclineSubstitution = React.useCallback(async (requestId: number) => {
    try {
      await api("decline_substitution", { requestId });
      onNotice("Substitution declined.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const handleSubstituteSession = React.useCallback(async (sessionId: number, substituteFacultyUserId: number) => {
    try {
      await api("substitute_session", { sessionId, input: { substitute_faculty_user_id: substituteFacultyUserId } });
      onNotice("Session substituted.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const markAttendance = React.useCallback(async (sessionId: number, input: MarkAttendanceDraft) => {
    try {
      await api("mark_attendance", { sessionId, input });
      onNotice("Attendance marked.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const createHoliday = React.useCallback(async (input: CreateHolidayInput) => {
    try {
      await api("create_holiday", { input });
      await loadHolidays();
      onNotice("Holiday created.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadHolidays, onError, onNotice]);

  const bulkCreateHoliday = React.useCallback(async (input: BulkCreateHolidayInput) => {
    try {
      await api("create_holiday_bulk", { input });
      await loadHolidays();
      onNotice("Holidays created.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadHolidays, onError, onNotice]);

  const deleteHoliday = React.useCallback(async (id: number) => {
    try {
      await api("delete_holiday", { id });
      await loadHolidays();
      onNotice("Holiday removed.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadHolidays, onError, onNotice]);

  const createLeaveRequest = React.useCallback(async (input: CreateLeaveRequestInput) => {
    try {
      await api("create_leave_request", { input });
      onNotice("Leave request submitted.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const approveLeaveRequest = React.useCallback(async (id: number) => {
    try {
      await api("approve_leave_request", { id });
      onNotice("Leave request approved.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const rejectLeaveRequest = React.useCallback(async (id: number) => {
    try {
      await api("reject_leave_request", { id });
      onNotice("Leave request rejected.");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice]);

  const createSubject = React.useCallback(async (input: Omit<Subject, "id">) => {
    try {
      await api("create_subject", { input });
      onNotice("Subject created.");
      await loadSubjects();
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice, loadSubjects]);

  const createFacultyMember = React.useCallback(async (input: CreateFacultyMemberInput) => {
    try {
      await api("create_faculty_member", { input });
      onNotice("Faculty member created.");
      await loadFacultyMembers();
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, onError, onNotice]);

  const updateFacultyMember = React.useCallback(async (input: UpdateFacultyMemberInput) => {
    try {
      await api("update_faculty_member", { id: input.id, input });
      onNotice("Faculty member updated.");
      await loadFacultyMembers();
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, onError, onNotice]);

  const deleteFacultyMember = React.useCallback(async (id: number) => {
    try {
      await api("delete_faculty_member", { id });
      onNotice("Faculty member deleted.");
      await loadFacultyMembers();
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, onError, onNotice]);

  const createFacultySchoolMembership = React.useCallback(async (input: CreateFacultySchoolMembershipInput) => {
    try {
      await api("create_faculty_school_membership", { input });
      onNotice("School membership added.");
      await loadFacultySchoolMemberships(input.faculty_id);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultySchoolMemberships, onError, onNotice]);

  const importFacultyMembersCsv = React.useCallback(async (file: File) => {
    try {
      await uploadFile("/imports/faculty-members.csv", file);
      onNotice("Faculty members imported.");
      await loadFacultyMembers();
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, onError, onNotice]);

  const createFacultyLogin = React.useCallback(async (facultyId: number, input: CreateFacultyLoginInput) => {
    try {
      await api("create_faculty_login", { facultyId, input });
      onNotice("Faculty login created.");
      await loadFacultyMembers();
      await loadFacultySchoolMemberships(facultyId);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, loadFacultySchoolMemberships, onError, onNotice]);

  const linkFacultyUser = React.useCallback(async (facultyId: number, userId: number) => {
    try {
      await api("link_faculty_user", { facultyId, input: { user_id: userId } });
      onNotice("Faculty user linked.");
      await loadFacultyMembers();
      await loadFacultySchoolMemberships(facultyId);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultyMembers, loadFacultySchoolMemberships, onError, onNotice]);

  const deleteFacultySchoolMembership = React.useCallback(async (id: number, facultyId: number) => {
    try {
      await api("delete_faculty_school_membership", { id });
      onNotice("School membership removed.");
      await loadFacultySchoolMemberships(facultyId);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadFacultySchoolMemberships, onError, onNotice]);

  const updateSubject = React.useCallback(async (input: Subject) => {
    try {
      await api("update_subject", { input });
      onNotice("Subject updated.");
      await loadSubjects();
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice, loadSubjects]);

  const deleteSubject = React.useCallback(async (id: number) => {
    try {
      await api("delete_subject", { id });
      onNotice("Subject deleted.");
      await loadSubjects();
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError, onNotice, loadSubjects]);

  return {
    subjects,
    effectiveSubjects,
    facultyAssignments,
    timetableSlots,
    weeklyTimetableSlots,
    lectureSessions,
    adminSessions,
    facultyWeeklySlots,
    leaveRequests,
    swapRequests,
    substitutions,
    pendingSubstitutionRequests,
    todaySubstitutions,
    substitutionBalance,
    substitutionReports,
    timetableHealthData,
    complianceData,
    attendanceSummary,
    dasReport,
    chronicAbsentees,
    subjectAttendance,
    attendanceReportDate,
    vpCenters,
    vpCenterBuildings,
    facultyProfiles,
    facultyMembers,
    facultyMemberMemberships,
    holidays,
    loadSubjects,
    loadEffectiveSubjects,
    loadFacultyAssignments,
    loadTimetableSlots,
    loadWeeklyTimetable,
    loadLectureSessions,
    loadAdminSessions,
    loadFacultyWeeklySlots,
    loadSubstitutions,
    loadPendingSubstitutionRequests,
    loadTodaySubstitutions,
    loadSubstitutionBalance,
    loadSubstitutionReports,
    loadTimetableHealth,
    loadComplianceData,
    loadAttendanceSummary,
    loadDasReport,
    loadChronicAbsentees,
    loadSubjectAttendance,
    loadVpCenters,
    loadVpCenterBuildings,
    loadFacultyProfiles,
    loadFacultyMembers,
    loadFacultySchoolMemberships,
    loadHolidays,
    loadLeaveRequests,
    loadSwapRequests,
    createSubject,
    updateSubject,
    deleteSubject,
    toggleOptionalSubject,
    createFacultyAssignment,
    deleteFacultyAssignment,
    upsertTimetableSlot,
    deleteTimetableSlot,
    upsertWeeklyTimetable,
    deleteWeeklyTimetable,
    cloneWeek,
    createMakeupSession,
    handleAcceptSubstitution,
    handleDeclineSubstitution,
    handleSubstituteSession,
    markAttendance,
    createHoliday,
    bulkCreateHoliday,
    deleteHoliday,
    createLeaveRequest,
    approveLeaveRequest,
    rejectLeaveRequest,
    createFacultyMember,
    updateFacultyMember,
    deleteFacultyMember,
    createFacultySchoolMembership,
    deleteFacultySchoolMembership,
    importFacultyMembersCsv,
    createFacultyLogin,
    linkFacultyUser,
  };
}
