import React from "react";
import { api, download, uploadFile, login as apiLogin, logout as apiLogout } from "./api";
import {
  AssignmentRulePanel,
  AuditLogPanel,
  BottomNav,
  CommunicationOperationsPanel,
  CommunicationTemplatePanel,
  CreateTicketModal,
  DirectoryPanel,
  DroppedSchoolsPanel,
  FacultyAssignmentsPanel,
  SubjectsPanel,
  TimetablePanel,
  EscalationPolicyPanel,
  LoginScreen,
  Metrics,
  MasterDataPanel,
  MobileMoreMenu,
  ProgramFilters,
  ProgramDashboardPanel,
  RegionHistoryPanel,
  ReportsPanel,
  Sidebar,
  SlaBreachAlert,
  SlaPolicyPanel,
  StudentTimelinePanel,
  SyncPanel,
  TicketDetail,
  TicketList,
  Topbar,
  UserManagementPanel,
} from "./components";
import { getSlaState } from "./formatters";
import { filterTickets } from "./ticketFilters";
import type {
  AppUser,
  AssignmentRule,
  AuditLogEntry,
  AttachmentDraft,
  BackupResult,
  CommunicationTemplate,
  CreateTicketDraft,
  CreateUserDraft,
  CurrentUser,
  CsvExportResult,
  EscalationPolicy,
  Filter,
  ImportSyncSnapshotResult,
  CreateFacultyAssignmentDraft,
  EffectiveSubject,
  FacultyAssignment,
  LectureModel,
  Subject,
  TimetableSlot,
  UpsertTimetableSlotDraft,
  LoginDraft,
  Priority,
  ProgramScopeFilters,
  ReplyDraft,
  Region,
  School,
  SchoolClassPlan,
  SchoolImportResult,
  SchoolProgramDashboard,
  SchoolProfileDraft,
  SchoolRegionHistory,
  SipMasterImportPreview,
  SipMasterImportResult,
  SlaPolicy,
  Student,
  StudentTimeline,
  SyncSnapshotInfo,
  Ticket,
  TicketAttachment,
  TicketChanges,
  TicketComment,
  TicketEditDraft,
  TicketHistory,
  UpdateUserDraft,
} from "./types";

const emptyTicketDraft: CreateTicketDraft = {
  title: "",
  description: "",
  requester: "",
  priority: "Medium",
  school_id: null,
  school_name: "",
  student_name: "",
  grade_level: "Grade 10",
  program_track: "Integrated STEM",
  issue_category: "Academic Support",
};

const emptyTicketEditDraft: TicketEditDraft = {
  title: "",
  description: "",
  requester: "",
  school_id: null,
  school_name: "",
  student_name: "",
  grade_level: "",
  program_track: "",
  issue_category: "",
};

const emptyAttachmentDraft: AttachmentDraft = {
  source_path: "",
  uploaded_by: "Service Desk",
};

const emptyProgramScopeFilters: ProgramScopeFilters = {
  school_name: "",
  grade_level: "",
  program_track: "",
  issue_category: "",
  queue: "",
};

type AdminView =
  | "master-data"
  | "program-dashboard"
  | "reports"
  | "communications"
  | "directory"
  | "dropped-schools"
  | "region-log"
  | "audit-log"
  | "routing"
  | "escalation"
  | "sla"
  | "templates"
  | "sync"
  | "faculty-assignments"
  | "timetable"
  | "subjects"
  | null;

export function App() {
  const [currentUser, setCurrentUser] = React.useState<CurrentUser | null>(null);
  const [loginDraft, setLoginDraft] = React.useState<LoginDraft>({ username: "", password: "" });
  const [loginError, setLoginError] = React.useState("");
  const [users, setUsers] = React.useState<AppUser[]>([]);
  const [showUsers, setShowUsers] = React.useState(false);
  const [tickets, setTickets] = React.useState<Ticket[]>([]);
  const [comments, setComments] = React.useState<TicketComment[]>([]);
  const [allComments, setAllComments] = React.useState<TicketComment[]>([]);
  const [history, setHistory] = React.useState<TicketHistory[]>([]);
  const [attachments, setAttachments] = React.useState<TicketAttachment[]>([]);
  const [selectedId, setSelectedId] = React.useState<number | null>(null);
  const [activeFilter, setActiveFilter] = React.useState<Filter>(() => {
    try { return (localStorage.getItem("td:activeFilter") as Filter) || "Inbox"; } catch { return "Inbox"; }
  });
  const [programScopeFilters, setProgramScopeFilters] = React.useState<ProgramScopeFilters>(() => {
    try { return JSON.parse(localStorage.getItem("td:scopeFilters") || "null") ?? emptyProgramScopeFilters; } catch { return emptyProgramScopeFilters; }
  });
  const [search, setSearch] = React.useState(() => {
    try { return localStorage.getItem("td:search") || ""; } catch { return ""; }
  });
  const [dateFrom, setDateFrom] = React.useState(() => {
    try { return localStorage.getItem("td:dateFrom") || ""; } catch { return ""; }
  });
  const [dateTo, setDateTo] = React.useState(() => {
    try { return localStorage.getItem("td:dateTo") || ""; } catch { return ""; }
  });
  const [error, setError] = React.useState("");
  const [notice, setNotice] = React.useState("");
  const [newBreachCount, setNewBreachCount] = React.useState(0);
  const knownBreachedIds = React.useRef(new Set<number>());
  const [isCreating, setIsCreating] = React.useState(false);
  const [isEditing, setIsEditing] = React.useState(false);
  const [isConfirmingDelete, setIsConfirmingDelete] = React.useState(false);
  const [isMobile, setIsMobile] = React.useState(() => window.innerWidth <= 768);
  const [showMobileDetail, setShowMobileDetail] = React.useState(false);
  const [mobileMoreOpen, setMobileMoreOpen] = React.useState(false);
  const [adminView, setAdminView] = React.useState<AdminView>(null);
  const [studentTimeline, setStudentTimeline] = React.useState<StudentTimeline | null>(null);
  const [syncSnapshot, setSyncSnapshot] = React.useState<SyncSnapshotInfo | null>(null);
  const [pendingImportSnapshot, setPendingImportSnapshot] = React.useState<SyncSnapshotInfo | null>(null);
  const [pendingImportPath, setPendingImportPath] = React.useState("");
  const [assignmentRules, setAssignmentRules] = React.useState<AssignmentRule[]>([]);
  const [communicationTemplates, setCommunicationTemplates] = React.useState<
    CommunicationTemplate[]
  >([]);
  const [escalationPolicy, setEscalationPolicy] = React.useState<EscalationPolicy | null>(null);
  const [slaPolicies, setSlaPolicies] = React.useState<SlaPolicy[]>([]);
  const [schools, setSchools] = React.useState<School[]>([]);
  const [droppedSchools, setDroppedSchools] = React.useState<School[]>([]);
  const [schoolRegionHistory, setSchoolRegionHistory] = React.useState<SchoolRegionHistory[]>([]);
  const [auditLog, setAuditLog] = React.useState<AuditLogEntry[]>([]);
  const [sipImportReview, setSipImportReview] = React.useState<{
    sourcePath: string;
    preview: SipMasterImportPreview;
  } | null>(null);
  const [regions, setRegions] = React.useState<Region[]>([]);
  const [lectureModels, setLectureModels] = React.useState<LectureModel[]>([]);
  const [classPlans, setClassPlans] = React.useState<SchoolClassPlan[]>([]);
  const [subjects, setSubjects] = React.useState<Subject[]>([]);
  const [facultyAssignments, setFacultyAssignments] = React.useState<FacultyAssignment[]>([]);
  const [timetableSlots, setTimetableSlots] = React.useState<TimetableSlot[]>([]);
  const [effectiveSubjects, setEffectiveSubjects] = React.useState<EffectiveSubject[]>([]);
  const [programDashboard, setProgramDashboard] =
    React.useState<SchoolProgramDashboard | null>(null);
  const [students, setStudents] = React.useState<Student[]>([]);
  const [draft, setDraft] = React.useState<CreateTicketDraft>(emptyTicketDraft);
  const [reply, setReply] = React.useState<ReplyDraft>({
    author: "Service Desk",
    body: "",
    is_internal: false,
    channel: "Local",
    audience: "School",
    recipient_name: "",
    recipient_contact: "",
    next_follow_up_due: "",
  });
  const [assigneeDraft, setAssigneeDraft] = React.useState("");
  const [editDraft, setEditDraft] = React.useState<TicketEditDraft>(emptyTicketEditDraft);
  const [attachmentDraft, setAttachmentDraft] =
    React.useState<AttachmentDraft>(emptyAttachmentDraft);

  const selected = tickets.find((ticket) => ticket.id === selectedId) ?? tickets[0] ?? null;

  React.useEffect(() => {
    void (async () => {
      try {
        const user = await api<CurrentUser>("get_current_user");
        setCurrentUser(user);
      } catch {
        // no session yet — stay on login screen
      }
    })();
    void loadTickets();
    void loadAssignmentRules();
    void loadCommunicationTemplates();
    void loadEscalationPolicy();
    void loadSlaPolicies();
    void loadSchools();
    void loadDroppedSchools();
    void loadSchoolRegionHistory();
    void loadAuditLog();
    void loadAllComments();
    void loadRegions();
    void loadLectureModels();
    void loadClassPlans();
    void loadProgramDashboard();
    void loadStudents();
    void loadSubjects();
    void loadFacultyAssignments();
  }, []);

  React.useEffect(() => {
    function handleResize() { setIsMobile(window.innerWidth <= 768); }
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  React.useEffect(() => {
    setReply((current) => ({
      ...current,
      author: currentUser?.display_name ?? "Service Desk",
    }));
  }, [currentUser?.display_name]);

  React.useEffect(() => {
    const currentBreached = tickets
      .filter(
        (t) =>
          getSlaState(t) === "Breached" && !["Resolved", "Closed"].includes(t.status),
      )
      .map((t) => t.id);
    const newIds = currentBreached.filter((id) => !knownBreachedIds.current.has(id));
    if (newIds.length > 0) {
      setNewBreachCount((c) => c + newIds.length);
      newIds.forEach((id) => knownBreachedIds.current.add(id));
    }
  }, [tickets]);

  React.useEffect(() => {
    const open = tickets.filter((t) => t.status !== "Closed").length;
    const breached = tickets.filter(
      (t) => getSlaState(t) === "Breached" && !["Resolved", "Closed"].includes(t.status),
    ).length;
    document.title = open > 0
      ? `Ticketing Desktop (${open} open${breached > 0 ? `, ${breached} breached` : ""})`
      : "Ticketing Desktop";
  }, [tickets]);

  React.useEffect(() => {
    try {
      localStorage.setItem("td:activeFilter", activeFilter);
      localStorage.setItem("td:scopeFilters", JSON.stringify(programScopeFilters));
      localStorage.setItem("td:search", search);
      localStorage.setItem("td:dateFrom", dateFrom);
      localStorage.setItem("td:dateTo", dateTo);
    } catch {}
  }, [activeFilter, programScopeFilters, search, dateFrom, dateTo]);

  React.useEffect(() => {
    if (selectedId) {
      try { localStorage.setItem(`td:reply:${selectedId}`, reply.body); } catch {}
    }
  }, [selectedId, reply.body]);

  React.useEffect(() => {
    if (selected?.id) {
      void loadComments(selected.id);
      void loadHistory(selected.id);
      void loadAttachments(selected.id);
      setAssigneeDraft(selected.assignee);
      setEditDraft({
        title: selected.title,
        description: selected.description,
        requester: selected.requester,
        school_id: selected.school_id,
        school_name: selected.school_name,
        student_name: selected.student_name,
        grade_level: selected.grade_level,
        program_track: selected.program_track,
        issue_category: selected.issue_category,
      });
      const school = schools.find((item) => item.id === selected.school_id);
      const savedBody = (() => { try { return localStorage.getItem(`td:reply:${selected.id}`) ?? ""; } catch { return ""; } })();
      setReply((current) => ({
        ...current,
        body: savedBody,
        audience: "School",
        channel: current.channel === "Internal Note" ? "Local" : current.channel,
        recipient_name: school?.school_spoc_name || school?.principal_name || selected.requester,
        recipient_contact:
          school?.school_spoc_email ||
          school?.school_spoc_mobile ||
          school?.principal_email ||
          school?.principal_mobile ||
          "",
        next_follow_up_due: "",
      }));
    } else {
      setComments([]);
      setHistory([]);
      setAttachments([]);
      setAssigneeDraft("");
      setEditDraft(emptyTicketEditDraft);
      setReply((current) => ({
        ...current,
        body: "",
        audience: "School",
        channel: "Local",
        recipient_name: "",
        recipient_contact: "",
        next_follow_up_due: "",
      }));
    }
    setIsEditing(false);
    setIsConfirmingDelete(false);
  }, [
    selected?.id,
    selected?.assignee,
    selected?.description,
    selected?.requester,
    selected?.title,
    selected?.school_id,
    schools,
  ]);

  async function loadTickets(preferredId?: number) {
    try {
      const items = await api<Ticket[]>("refresh_escalations");
      setTickets(items);
      setSelectedId((currentId) => {
        if (preferredId && items.some((ticket) => ticket.id === preferredId)) {
          return preferredId;
        }

        if (currentId && items.some((ticket) => ticket.id === currentId)) {
          return currentId;
        }

        return items[0]?.id ?? null;
      });
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadAuditLog() {
    try {
      const items = await api<AuditLogEntry[]>("list_audit_log", { limit: 150 });
      setAuditLog(items);
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadComments(ticketId: number) {
    try {
      setComments(await api<TicketComment[]>("list_comments", { ticketId }));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadAllComments() {
    try {
      setAllComments(await api<TicketComment[]>("list_all_comments"));
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadHistory(ticketId: number) {
    try {
      setHistory(await api<TicketHistory[]>("list_history", { ticketId }));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  function loadAttachments(_ticketId: number) {
    setAttachments([]);
  }

  async function loadSlaPolicies() {
    try {
      setSlaPolicies(await api<SlaPolicy[]>("list_sla_policies"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadAssignmentRules() {
    try {
      setAssignmentRules(await api<AssignmentRule[]>("list_assignment_rules"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadCommunicationTemplates() {
    try {
      setCommunicationTemplates(await api<CommunicationTemplate[]>("list_communication_templates"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadEscalationPolicy() {
    try {
      setEscalationPolicy(await api<EscalationPolicy>("get_escalation_policy"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadSchools() {
    try {
      setSchools(await api<School[]>("list_schools"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadDroppedSchools() {
    try {
      setDroppedSchools(await api<School[]>("list_dropped_schools"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadSchoolRegionHistory() {
    try {
      setSchoolRegionHistory(await api<SchoolRegionHistory[]>("list_school_region_history"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadRegions() {
    try {
      setRegions(await api<Region[]>("list_regions"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadLectureModels() {
    try {
      setLectureModels(await api<LectureModel[]>("list_lecture_models"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadClassPlans(schoolId?: number) {
    try {
      setClassPlans(await api<SchoolClassPlan[]>("list_school_class_plans", { schoolId }));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadSubjects() {
    try {
      setSubjects(await api<Subject[]>("list_subjects"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadEffectiveSubjects(schoolId: number, track: string) {
    return api<EffectiveSubject[]>("list_effective_subjects", { schoolId, track });
  }

  async function toggleOptionalSubject(schoolId: number, subjectId: number, enabled: boolean) {
    try {
      await api("set_school_optional_subject", {
        schoolId,
        input: { subject_id: subjectId, enabled },
      });
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadFacultyAssignments() {
    try {
      setFacultyAssignments(await api<FacultyAssignment[]>("list_faculty_assignments"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function createFacultyAssignment(input: CreateFacultyAssignmentDraft) {
    try {
      await api<FacultyAssignment>("create_faculty_assignment", { input });
      await loadFacultyAssignments();
      setNotice("Faculty assignment added.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function deleteFacultyAssignment(id: number) {
    try {
      await api("delete_faculty_assignment", { id });
      await loadFacultyAssignments();
      setNotice("Faculty assignment removed.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadTimetableSlots(params: {
    schoolId: number;
    gradeLevel: string;
    track: string;
    batchPattern: string;
  }) {
    try {
      setTimetableSlots(
        await api<TimetableSlot[]>("list_timetable_slots", params),
      );
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function upsertTimetableSlot(input: UpsertTimetableSlotDraft) {
    try {
      await api<TimetableSlot>("upsert_timetable_slot", { input });
      await loadTimetableSlots({
        schoolId: input.school_id,
        gradeLevel: input.grade_level,
        track: input.track,
        batchPattern: input.batch_pattern,
      });
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function deleteTimetableSlot(id: number) {
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
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadProgramDashboard() {
    try {
      setProgramDashboard(await api<SchoolProgramDashboard>("get_school_program_dashboard"));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadStudents(schoolId?: number) {
    try {
      setStudents(await api<Student[]>("list_students", { schoolId }));
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function loadUsers() {
    try {
      setUsers(await api<AppUser[]>("list_users"));
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function handleLogin() {
    try {
      const user = await apiLogin(loginDraft.username, loginDraft.password);
      setCurrentUser(user as CurrentUser);
      setLoginDraft({ username: "", password: "" });
      setLoginError("");
      setError("");
      void loadTickets();
      void loadAssignmentRules();
      void loadCommunicationTemplates();
      void loadEscalationPolicy();
      void loadSlaPolicies();
      void loadSchools();
      void loadDroppedSchools();
      void loadSchoolRegionHistory();
      void loadAuditLog();
      void loadAllComments();
      void loadRegions();
      void loadLectureModels();
      void loadClassPlans();
      void loadProgramDashboard();
      void loadStudents();
      void loadSubjects();
      void loadFacultyAssignments();
    } catch (caught) {
      setLoginError(String(caught));
    }
  }

  function handleLogout() {
    apiLogout();
    setCurrentUser(null);
    setLoginDraft({ username: "", password: "" });
    setLoginError("");
    setShowUsers(false);
  }

  async function handleCreateUser(draft: CreateUserDraft) {
    try {
      await api("create_user", { input: draft });
      await loadUsers();
      setNotice("User created.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function handleUpdateUser(draft: UpdateUserDraft) {
    try {
      await api("update_user", { input: draft });
      await loadUsers();
      setNotice("User updated.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function handleDeleteUser(id: number) {
    try {
      await api("delete_user", { id });
      await loadUsers();
      setNotice("User deleted.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function handleChangePassword(currentPassword: string, newPassword: string) {
    try {
      await api("change_password", { input: { current_password: currentPassword, new_password: newPassword } });
      setNotice("Password changed successfully.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function quickUpdateTicket(id: number, changes: TicketChanges) {
    const ticket = tickets.find((t) => t.id === id);
    if (!ticket) return;
    const input = {
      id: ticket.id,
      title: ticket.title,
      description: ticket.description,
      requester: ticket.requester,
      status: changes.status ?? ticket.status,
      priority: ticket.priority,
      assignee: changes.assignee ?? ticket.assignee,
      queue: ticket.queue,
      school_id: ticket.school_id,
      school_name: ticket.school_name,
      student_name: ticket.student_name,
      grade_level: ticket.grade_level,
      program_track: ticket.program_track,
      issue_category: ticket.issue_category,
    };
    try {
      const updated = await api<Ticket>("update_ticket", { input });
      setTickets((current) => current.map((t) => (t.id === updated.id ? updated : t)));
      if (selected?.id === id) await loadHistory(id);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function createTicket(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    try {
      const ticket = await api<Ticket>("create_ticket", { input: draft });
      setDraft(emptyTicketDraft);
      setSearch("");
      setActiveFilter("Inbox");
      setIsCreating(false);
      await loadTickets(ticket.id);
      await loadAuditLog();
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function updateTicket(changes: TicketChanges) {
    if (!selected) {
      return;
    }

    const input = {
      id: selected.id,
      title: changes.title ?? selected.title,
      description: changes.description ?? selected.description,
      requester: changes.requester ?? selected.requester,
      status: changes.status ?? selected.status,
      priority: changes.priority ?? selected.priority,
      assignee: changes.assignee ?? selected.assignee,
      queue: changes.queue ?? selected.queue,
      school_id: changes.school_id ?? selected.school_id,
      school_name: changes.school_name ?? selected.school_name,
      student_name: changes.student_name ?? selected.student_name,
      grade_level: changes.grade_level ?? selected.grade_level,
      program_track: changes.program_track ?? selected.program_track,
      issue_category: changes.issue_category ?? selected.issue_category,
    };

    try {
      const updated = await api<Ticket>("update_ticket", { input });
      setTickets((currentTickets) =>
        currentTickets.map((ticket) => (ticket.id === updated.id ? updated : ticket)),
      );
      await loadHistory(updated.id);
      await loadAuditLog();
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function saveTicketEdits(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    await updateTicket({
      title: editDraft.title.trim(),
      description: editDraft.description.trim(),
      requester: editDraft.requester.trim(),
      school_id: editDraft.school_id,
      school_name: editDraft.school_name.trim(),
      student_name: editDraft.student_name.trim(),
      grade_level: editDraft.grade_level.trim(),
      program_track: editDraft.program_track.trim(),
      issue_category: editDraft.issue_category.trim(),
    });
    setIsEditing(false);
  }

  async function deleteSelectedTicket() {
    if (!selected) {
      return;
    }

    try {
      await api("delete_ticket", { id: selected.id });
      setIsConfirmingDelete(false);
      await loadTickets();
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function addComment(isInternal: boolean) {
    if (!selected || !reply.body.trim()) {
      return;
    }

    try {
      await api<TicketComment>("add_comment", {
        input: {
          ticket_id: selected.id,
          author: reply.author,
          body: reply.body,
          is_internal: isInternal,
          channel: isInternal ? "Internal Note" : reply.channel,
          audience: isInternal ? "Internal" : reply.audience,
          recipient_name: isInternal ? "" : reply.recipient_name,
          recipient_contact: isInternal ? "" : reply.recipient_contact,
          next_follow_up_due: isInternal ? null : reply.next_follow_up_due || null,
        },
      });
      try { localStorage.removeItem(`td:reply:${selected.id}`); } catch {}
      setReply((current) => ({
        ...current,
        body: "",
        is_internal: isInternal,
      }));
      await Promise.all([loadComments(selected.id), loadAllComments(), loadTickets(selected.id)]);
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function updateCommentStatus(id: number, deliveryStatus: string, nextFollowUpDue = "") {
    if (!selected) {
      return;
    }

    try {
      await api<TicketComment>("update_comment_status", {
        input: {
          id,
          delivery_status: deliveryStatus,
          next_follow_up_due: nextFollowUpDue || null,
        },
      });
      await Promise.all([loadComments(selected.id), loadAllComments(), loadTickets(selected.id)]);
      setNotice(`Communication marked ${deliveryStatus}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  function selectTicket(ticketId: number) {
    setSelectedId(ticketId);
    if (isMobile) setShowMobileDetail(true);
  }

  function openTicketFromCommunication(ticketId: number) {
    setSelectedId(ticketId);
    setAdminView(null);
    if (isMobile) setShowMobileDetail(true);
  }

  function addAttachment(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setNotice("File attachments are not available in the web version.");
  }

  function browseAttachment() {
    setNotice("File browsing is not available in the web version.");
  }

  function openAttachment(_path: string) {
    setNotice("File access is not available in the web version.");
  }

  function viewStudentTimeline() {
    setNotice("Student timeline is not yet available in the web version.");
  }

  function exportBackup() {
    setNotice("Database backup is not available in the web version.");
  }

  async function exportTicketCsvBundle() {
    try {
      await download("/export/tickets.csv", "tickets.csv");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function exportCommunicationCsv() {
    try {
      await download("/export/communications.csv", "communications.csv");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function exportSipMasterExcel() {
    try {
      await download("/export/sip-master.csv", "sip-master.csv");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  function exportSyncSnapshot() {
    setNotice("Sync export is not available in the web version.");
  }

  function importSyncSnapshot() {
    setNotice("Sync import is not available in the web version.");
  }

  function confirmSyncImport() {
    setNotice("Sync import is not available in the web version.");
  }

  function cancelSyncImport() {
    setPendingImportSnapshot(null);
    setPendingImportPath("");
  }

  async function saveSlaPolicy(issueCategory: string, hours: number) {
    try {
      const policy = await api<SlaPolicy>("update_sla_policy", {
        input: {
          issue_category: issueCategory,
          hours,
        },
      });
      setSlaPolicies((current) =>
        current.map((item) => (item.issue_category === policy.issue_category ? policy : item)),
      );
      setNotice(`SLA policy updated for ${policy.issue_category}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function saveAssignmentRule(queue: string, assignee: string, isActive: boolean) {
    try {
      const rule = await api<AssignmentRule>("update_assignment_rule", {
        input: {
          queue,
          assignee,
          is_active: isActive,
        },
      });
      setAssignmentRules((current) =>
        current.map((item) => (item.queue === rule.queue ? rule : item)),
      );
      setNotice(`Routing updated for ${rule.queue}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function saveCommunicationTemplate(input: {
    id?: number;
    name: string;
    audience: string;
    body: string;
    is_active: boolean;
  }) {
    try {
      await api<CommunicationTemplate>("update_communication_template", { input });
      await loadCommunicationTemplates();
      setNotice(`Template saved: ${input.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function saveEscalationPolicy(input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) {
    try {
      const policy = await api<EscalationPolicy>("update_escalation_policy", { input });
      setEscalationPolicy(policy);
      await loadTickets(selected?.id);
      if (selected?.id) {
        await loadHistory(selected.id);
      }
      setNotice("Escalation policy updated.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function createSchool(input: SchoolProfileDraft) {
    try {
      const school = await api<School>("create_school", { input });
      await loadSchools();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      await loadAuditLog();
      setNotice(`School profile saved: ${school.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  // File-picker helper: attaches the <input> to the DOM (some desktop browsers
  // require this for programmatic .click() to open the dialog), runs the
  // handler, then cleans up.
  function pickFile(handler: (file: File) => Promise<void>) {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".csv,text/csv";
    input.style.display = "none";
    document.body.appendChild(input);
    const cleanup = () => {
      if (input.parentNode) input.parentNode.removeChild(input);
    };
    input.onchange = async () => {
      try {
        const file = input.files?.[0];
        if (file) await handler(file);
      } finally {
        cleanup();
      }
    };
    // Desktop browsers fire neither "change" nor "cancel" if user dismisses
    // the dialog without picking a file → leak the element. Use the modern
    // "cancel" event when available; otherwise rely on next file pick to
    // overwrite (rare leak, no functional impact).
    input.addEventListener("cancel", cleanup);
    input.click();
  }

  function importSchoolsCsv() {
    pickFile(async (file) => {
      try {
        const result = await uploadFile<{
          imported_count: number;
          skipped_count: number;
          errors: string[];
        }>("/imports/schools.csv", file);
        await loadSchools();
        await loadAuditLog();
        const errSuffix =
          result.errors.length > 0 ? ` First error: ${result.errors[0]}` : "";
        setNotice(
          `Imported ${result.imported_count} schools (${result.skipped_count} skipped).${errSuffix}`,
        );
        setError("");
      } catch (caught) {
        setError(`School import failed: ${caught}`);
      }
    });
  }

  const pendingSipFileRef = React.useRef<File | null>(null);

  function importSipMaster() {
    pickFile(async (file) => {
      try {
        const preview = await uploadFile<SipMasterImportPreview>(
          "/imports/sip-master/preview",
          file,
        );
        pendingSipFileRef.current = file;
        setSipImportReview({ sourcePath: file.name, preview });
        setError("");
      } catch (caught) {
        setError(`SIP master preview failed: ${caught}`);
      }
    });
  }

  async function confirmSipMasterImport(
    conflictAction: "skip_existing" | "update_existing",
  ) {
    const file = pendingSipFileRef.current;
    if (!file) {
      setError("No SIP master file in memory. Pick the file again.");
      return;
    }
    try {
      const form = new FormData();
      form.append("file", file);
      form.append("conflict_action", conflictAction);
      const token = sessionStorage.getItem("td:token");
      const response = await fetch("/api/imports/sip-master", {
        method: "POST",
        headers: token ? { Authorization: `Bearer ${token}` } : {},
        body: form,
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text);
      }
      const result = (await response.json()) as SipMasterImportResult;
      pendingSipFileRef.current = null;
      setSipImportReview(null);
      await loadSchools();
      await loadRegions();
      await loadClassPlans();
      await loadAuditLog();
      setNotice(
        `SIP master imported: ${result.imported_count} new, ${result.updated_count} updated, ${result.skipped_count} skipped, ${result.class_plan_count} class plans saved.`,
      );
      setError("");
    } catch (caught) {
      setError(`SIP master import failed: ${caught}`);
    }
  }

  function cancelSipMasterImport() {
    pendingSipFileRef.current = null;
    setSipImportReview(null);
  }

  async function saveRegion(input: {
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
  }) {
    try {
      const region = await api<Region>("upsert_region", { input });
      await loadRegions();
      await loadAuditLog();
      setNotice(`Region saved: ${region.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function deleteRegion(id: number) {
    try {
      await api("delete_region", { id });
      await loadRegions();
      setNotice("Region deleted.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function remapAndDeleteRegion(
    regionId: number,
    mappings: Array<{
      school_id: number;
      target_region_id?: number;
      new_region_name?: string;
    }>,
  ) {
    try {
      const createdRegions = new Map<string, number>();

      for (const mapping of mappings) {
        const school = schools.find((item) => item.id === mapping.school_id);
        if (!school) {
          continue;
        }

        let targetRegionId = mapping.target_region_id;
        const newRegionName = mapping.new_region_name?.trim() ?? "";

        if (!targetRegionId) {
          if (!newRegionName) {
            throw new Error(`New region name is required for ${school.name}.`);
          }

          const cacheKey = newRegionName.toLocaleLowerCase();
          targetRegionId = createdRegions.get(cacheKey);

          if (!targetRegionId) {
            const region = await api<Region>("upsert_region", {
              input: {
                name: newRegionName,
                regional_academic_head_name: "",
                regional_academic_head_mobile: "",
                regional_academic_head_email: "",
                regional_business_head_name: "",
                regional_business_head_mobile: "",
                regional_business_head_email: "",
              },
            });
            targetRegionId = region.id;
            createdRegions.set(cacheKey, region.id);
          }
        }

        await api<School>("create_school", {
          input: {
            ...schoolToProfileDraft(school),
            region_id: targetRegionId,
          },
        });
      }

      await api("delete_region", { id: regionId });
      await loadRegions();
      await loadSchools();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      setNotice("Schools moved and region deleted.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function dropSchool(id: number, reason: string) {
    try {
      const school = await api<School>("drop_school", { id, body: { reason } });
      await loadSchools();
      await loadDroppedSchools();
      await loadStudents();
      await loadClassPlans();
      await loadProgramDashboard();
      await loadAuditLog();
      setNotice(`School dropped: ${school.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function deleteSchool(id: number) {
    try {
      await api("delete_school", { id });
      await loadSchools();
      await loadDroppedSchools();
      await loadStudents();
      await loadClassPlans();
      await loadSchoolRegionHistory();
      await loadProgramDashboard();
      await loadAuditLog();
      setNotice("School master record deleted.");
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function restoreSchool(id: number) {
    try {
      const school = await api<School>("restore_school", { id });
      await loadSchools();
      await loadDroppedSchools();
      await loadStudents();
      await loadClassPlans();
      await loadProgramDashboard();
      await loadAuditLog();
      setNotice(`School restored: ${school.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function createStudent(input: {
    school_id: number;
    name: string;
    grade_level: string;
    program_track: string;
  }) {
    try {
      const student = await api<Student>("create_student", { input });
      await loadStudents();
      setNotice(`Student added: ${student.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function createLectureModel(input: {
    name: string;
    days_per_week: number;
    lectures_per_day: number;
  }) {
    try {
      const model = await api<LectureModel>("create_lecture_model", { input });
      await loadLectureModels();
      setNotice(`Lecture model saved: ${model.name}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  async function saveSchoolClassPlan(input: {
    school_id: number;
    grade_level: string;
    track: string;
    lecture_model_id: number;
    batch_pattern: string;
    aop_admissions: number;
    registrations: number;
    actual_admissions: number;
  }) {
    try {
      const plan = await api<SchoolClassPlan>("upsert_school_class_plan", { input });
      await loadClassPlans();
      await loadProgramDashboard();
      const trackLabel = plan.track ? ` (${plan.track})` : "";
      setNotice(`Class plan saved: ${plan.school_name} ${plan.grade_level}${trackLabel}.`);
      setError("");
    } catch (caught) {
      setError(String(caught));
    }
  }

  const visibleTickets = filterTickets(tickets, activeFilter, search, programScopeFilters, currentUser?.display_name, dateFrom, dateTo);

  const assigneeWorkload = tickets
    .filter((t) => !["Resolved", "Closed"].includes(t.status) && t.assignee !== "Unassigned")
    .reduce<Record<string, number>>((acc, t) => {
      acc[t.assignee] = (acc[t.assignee] ?? 0) + 1;
      return acc;
    }, {});

  const latestUpdate = tickets.reduce((latest, t) => (t.updated_at > latest ? t.updated_at : latest), "");

  React.useEffect(() => {
    function handleKey(event: KeyboardEvent) {
      const tag = (event.target as HTMLElement).tagName.toLowerCase();
      const isTyping = tag === "input" || tag === "textarea" || tag === "select";
      if (event.key === "Escape") {
        if (isCreating) { setIsCreating(false); return; }
        if (adminView) { setAdminView(null); return; }
        if (studentTimeline) { setStudentTimeline(null); return; }
        if (showUsers) { setShowUsers(false); return; }
        if (isEditing) { setIsEditing(false); return; }
      }
      if (isTyping || event.metaKey || event.ctrlKey || event.altKey) return;
      if (event.key === "n" && !isCreating && currentUser?.role !== "viewer") {
        setIsCreating(true);
      } else if (event.key === "/") {
        event.preventDefault();
        (document.querySelector('input[aria-label="Search tickets"]') as HTMLInputElement | null)?.focus();
      } else if ((event.key === "j" || event.key === "k") && !adminView) {
        const idx = visibleTickets.findIndex((t) => t.id === selectedId);
        const next = event.key === "j" ? visibleTickets[idx + 1] : visibleTickets[idx - 1];
        if (next) setSelectedId(next.id);
      }
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [isCreating, adminView, studentTimeline, showUsers, isEditing, currentUser, visibleTickets, selectedId]);

  const filterCounts = {
    Inbox: tickets.filter((t) => t.status !== "Closed").length,
    "My Tickets": tickets.filter(
      (t) =>
        t.status !== "Closed" &&
        (currentUser ? t.assignee === currentUser.display_name : t.assignee !== "Unassigned"),
    ).length,
    Unassigned: tickets.filter((t) => t.assignee === "Unassigned" && t.status !== "Closed").length,
    "Pending SLA": tickets.filter(
      (t) =>
        ["Breached", "At Risk"].includes(getSlaState(t)) &&
        !["Resolved", "Closed"].includes(t.status),
    ).length,
    Escalated: tickets.filter(
      (t) => t.escalation_status === "Escalated" && !["Resolved", "Closed"].includes(t.status),
    ).length,
    Resolved: tickets.filter((t) => ["Resolved", "Closed"].includes(t.status)).length,
  } satisfies Record<import("./types").Filter, number>;

  const openCount = tickets.filter((ticket) => ticket.status !== "Closed").length;
  const unassignedCount = tickets.filter((ticket) => ticket.assignee === "Unassigned").length;
  const pendingSlaCount = tickets.filter(
    (ticket) =>
      ["Breached", "At Risk"].includes(getSlaState(ticket)) &&
      !["Resolved", "Closed"].includes(ticket.status),
  ).length;
  const activeSchoolCount = new Set(
    tickets
      .filter((ticket) => ticket.status !== "Closed")
      .map((ticket) => ticket.school_name)
      .filter(Boolean),
  ).size;
  const activeQueueCount = new Set(
    tickets
      .filter((ticket) => ticket.status !== "Closed")
      .map((ticket) => ticket.queue)
      .filter(Boolean),
  ).size;
  const escalatedCount = tickets.filter(
    (ticket) => ticket.escalation_status === "Escalated" && ticket.status !== "Closed",
  ).length;
  const schoolOptions = uniqueTicketValues(tickets, "school_name");
  const adminContent =
    adminView === "sla" ? (
      <SlaPolicyPanel
        policies={slaPolicies}
        onClose={() => setAdminView(null)}
        onSave={saveSlaPolicy}
      />
    ) : adminView === "routing" ? (
      <AssignmentRulePanel
        rules={assignmentRules}
        onClose={() => setAdminView(null)}
        onSave={saveAssignmentRule}
      />
    ) : adminView === "escalation" && escalationPolicy ? (
      <EscalationPolicyPanel
        policy={escalationPolicy}
        onClose={() => setAdminView(null)}
        onSave={saveEscalationPolicy}
      />
    ) : adminView === "master-data" ? (
      <MasterDataPanel
        classPlans={classPlans}
        lectureModels={lectureModels}
        regions={regions}
        schools={schools}
        sipImportPreview={sipImportReview}
        students={students}
        onCancelSipMasterImport={cancelSipMasterImport}
        onConfirmSipMasterImport={confirmSipMasterImport}
        onClose={() => setAdminView(null)}
        onCreateLectureModel={createLectureModel}
        onCreateSchool={createSchool}
        onDeleteSchool={deleteSchool}
        onDropSchool={dropSchool}
        onExportSipMaster={exportSipMasterExcel}
        onImportSipMaster={importSipMaster}
        onImportSchools={importSchoolsCsv}
        onSaveRegion={saveRegion}
        onDeleteRegion={deleteRegion}
        onRemapAndDeleteRegion={remapAndDeleteRegion}
        onSaveClassPlan={saveSchoolClassPlan}
        onCreateStudent={createStudent}
      />
    ) : adminView === "program-dashboard" && programDashboard ? (
      <ProgramDashboardPanel dashboard={programDashboard} onClose={() => setAdminView(null)} />
    ) : adminView === "reports" && programDashboard ? (
      <ReportsPanel
        comments={allComments}
        dashboard={programDashboard}
        droppedSchools={droppedSchools}
        schools={schools}
        tickets={tickets}
        onClose={() => setAdminView(null)}
        onExportCsv={exportTicketCsvBundle}
        onExportSipMaster={exportSipMasterExcel}
      />
    ) : adminView === "communications" ? (
      <CommunicationOperationsPanel
        comments={allComments}
        schools={schools}
        tickets={tickets}
        onClose={() => setAdminView(null)}
        onExport={exportCommunicationCsv}
        onOpenTicket={openTicketFromCommunication}
        onUpdateCommentStatus={updateCommentStatus}
      />
    ) : adminView === "directory" ? (
      <DirectoryPanel regions={regions} schools={schools} onClose={() => setAdminView(null)} />
    ) : adminView === "dropped-schools" ? (
      <DroppedSchoolsPanel
        schools={droppedSchools}
        onClose={() => setAdminView(null)}
        onRestore={restoreSchool}
      />
    ) : adminView === "region-log" ? (
      <RegionHistoryPanel history={schoolRegionHistory} onClose={() => setAdminView(null)} />
    ) : adminView === "audit-log" ? (
      <AuditLogPanel entries={auditLog} onClose={() => setAdminView(null)} />
    ) : adminView === "templates" ? (
      <CommunicationTemplatePanel
        templates={communicationTemplates}
        onClose={() => setAdminView(null)}
        onSave={saveCommunicationTemplate}
      />
    ) : adminView === "sync" ? (
      <SyncPanel
        snapshot={syncSnapshot}
        pendingSnapshot={pendingImportSnapshot}
        onClose={() => { cancelSyncImport(); setAdminView(null); }}
        onExport={exportSyncSnapshot}
        onImport={importSyncSnapshot}
        onConfirmImport={confirmSyncImport}
        onCancelImport={cancelSyncImport}
      />
    ) : adminView === "faculty-assignments" ? (
      <FacultyAssignmentsPanel
        schools={schools}
        users={users}
        subjects={subjects}
        assignments={facultyAssignments}
        onClose={() => setAdminView(null)}
        onCreate={createFacultyAssignment}
        onDelete={deleteFacultyAssignment}
      />
    ) : adminView === "timetable" ? (
      <TimetablePanel
        schools={schools}
        users={users}
        subjects={subjects}
        slots={timetableSlots}
        onClose={() => setAdminView(null)}
        onLoad={loadTimetableSlots}
        onUpsert={upsertTimetableSlot}
        onDelete={deleteTimetableSlot}
      />
    ) : adminView === "subjects" ? (
      <SubjectsPanel
        schools={schools}
        subjects={subjects}
        onClose={() => setAdminView(null)}
        onLoadEffective={loadEffectiveSubjects}
        onToggleOptional={toggleOptionalSubject}
      />
    ) : null;

  if (!currentUser) {
    return (
      <LoginScreen
        draft={loginDraft}
        error={loginError}
        onDraftChange={setLoginDraft}
        onSubmit={handleLogin}
      />
    );
  }

  return (
    <main className="app-shell">
      <Sidebar
        activeFilter={activeFilter}
        currentUserRole={currentUser.role}
        filterCounts={filterCounts}
        onBackupClick={exportBackup}
        onCommunicationOpsClick={() => setAdminView("communications")}
        onCsvExportClick={exportTicketCsvBundle}
        onDirectoryClick={() => setAdminView("directory")}
        onAuditLogClick={() => {
          void loadAuditLog();
          setAdminView("audit-log");
        }}
        onDroppedSchoolsClick={() => {
          void loadDroppedSchools();
          setAdminView("dropped-schools");
        }}
        onEscalationPolicyClick={() => setAdminView("escalation")}
        onFilterChange={(filter) => {
          setAdminView(null);
          setActiveFilter(filter);
        }}
        onMasterDataClick={() => setAdminView("master-data")}
        onProgramDashboardClick={() => {
          void loadProgramDashboard();
          setAdminView("program-dashboard");
        }}
        onReportsClick={() => {
          void loadProgramDashboard();
          setAdminView("reports");
        }}
        onRegionLogClick={() => {
          void loadSchoolRegionHistory();
          setAdminView("region-log");
        }}
        onRoutingRulesClick={() => setAdminView("routing")}
        onSlaSettingsClick={() => setAdminView("sla")}
        onTemplatesClick={() => setAdminView("templates")}
        onSyncClick={() => setAdminView("sync")}
        onUsersClick={() => {
          void loadUsers();
          setShowUsers(true);
        }}
        onFacultyAssignmentsClick={() => {
          void loadFacultyAssignments();
          void loadSubjects();
          setAdminView("faculty-assignments");
        }}
        onSubjectsClick={() => {
          void loadSubjects();
          setAdminView("subjects");
        }}
        onTimetableClick={() => {
          void loadSubjects();
          void loadUsers();
          setAdminView("timetable");
        }}
      />

      <section className="workspace">
        <Topbar
          search={search}
          currentUser={currentUser}
          latestUpdate={latestUpdate}
          onSearchChange={setSearch}
          onCreateClick={() => setIsCreating(true)}
          onLogout={handleLogout}
          mobileBackLabel={showMobileDetail ? "Tickets" : undefined}
          onMobileBack={isMobile && showMobileDetail ? () => setShowMobileDetail(false) : undefined}
        />

        {adminView ? null : (
          <ProgramFilters
            filters={programScopeFilters}
            schoolOptions={schoolOptions}
            dateFrom={dateFrom}
            dateTo={dateTo}
            onChange={setProgramScopeFilters}
            onDateFromChange={setDateFrom}
            onDateToChange={setDateTo}
            onReset={() => {
              setProgramScopeFilters(emptyProgramScopeFilters);
              setDateFrom("");
              setDateTo("");
            }}
          />
        )}

        {error ? <div className="error-banner">{error}</div> : null}
        {notice ? <div className="notice-banner">{notice}</div> : null}

        {adminView ? (
          <div className="admin-workspace">{adminContent}</div>
        ) : (
          <>
            <SlaBreachAlert
              newBreachCount={newBreachCount}
              onView={() => {
                setActiveFilter("Pending SLA");
                setAdminView(null);
                setNewBreachCount(0);
              }}
              onDismiss={() => setNewBreachCount(0)}
            />
            <Metrics
              openCount={openCount}
              activeSchoolCount={activeSchoolCount}
              activeQueueCount={activeQueueCount}
              escalatedCount={escalatedCount}
              pendingSlaCount={pendingSlaCount}
              unassignedCount={unassignedCount}
            />

            <div className={`content-grid${showMobileDetail ? " mobile-show-detail" : ""}`}>
              <TicketList
                activeFilter={activeFilter}
                currentUser={currentUser}
                selectedId={selected?.id ?? null}
                tickets={visibleTickets}
                onSelectTicket={selectTicket}
                onQuickResolve={(id) => void quickUpdateTicket(id, { status: "Resolved" })}
                onQuickAssign={(id) => void quickUpdateTicket(id, { assignee: currentUser?.display_name ?? "Service Desk" })}
              />

              <TicketDetail
                assigneeDraft={assigneeDraft}
                assigneeWorkload={assigneeWorkload}
                attachmentDraft={attachmentDraft}
                attachments={attachments}
                comments={comments}
                editDraft={editDraft}
                history={history}
                isConfirmingDelete={isConfirmingDelete}
                isEditing={isEditing}
                reply={reply}
                selected={selected}
                schools={schools}
                students={students}
                templates={communicationTemplates}
                onAddAttachment={addAttachment}
                onAddComment={addComment}
                onAssigneeDraftChange={setAssigneeDraft}
                onBrowseAttachment={browseAttachment}
                onCancelDelete={() => setIsConfirmingDelete(false)}
                onCancelEdit={() => setIsEditing(false)}
                onConfirmDelete={deleteSelectedTicket}
                onEditDraftChange={setEditDraft}
                onOpenAttachment={openAttachment}
                onReplyChange={setReply}
                onRequestDelete={() => setIsConfirmingDelete(true)}
                onRequestEdit={() => setIsEditing(true)}
                onRequestStudentTimeline={viewStudentTimeline}
                onSaveEdits={saveTicketEdits}
                onSetAttachmentDraft={setAttachmentDraft}
                onUpdateCommentStatus={updateCommentStatus}
                onUpdateTicket={updateTicket}
              />
            </div>
          </>
        )}
      </section>

      {isCreating ? (
        <CreateTicketModal
          draft={draft}
          schools={schools}
          students={students}
          onCancel={() => setIsCreating(false)}
          onDraftChange={setDraft}
          onSubmit={createTicket}
        />
      ) : null}

      {studentTimeline ? (
        <StudentTimelinePanel
          timeline={studentTimeline}
          onClose={() => setStudentTimeline(null)}
        />
      ) : null}

      {showUsers ? (
        <UserManagementPanel
          users={users}
          schools={schools}
          currentUser={currentUser}
          onClose={() => setShowUsers(false)}
          onCreateUser={handleCreateUser}
          onUpdateUser={handleUpdateUser}
          onDeleteUser={handleDeleteUser}
          onChangePassword={handleChangePassword}
        />
      ) : null}

      <BottomNav
        activeFilter={activeFilter}
        currentUserRole={currentUser.role}
        filterCounts={filterCounts}
        showingAdmin={adminView !== null}
        onFilterChange={(filter) => {
          setAdminView(null);
          setShowMobileDetail(false);
          setActiveFilter(filter);
        }}
        onCreateClick={() => setIsCreating(true)}
        onMasterDataClick={() => {
          setShowMobileDetail(false);
          setAdminView("master-data");
        }}
        onMoreClick={() => setMobileMoreOpen(true)}
      />

      {mobileMoreOpen && (
        <MobileMoreMenu
          currentUserRole={currentUser.role}
          onClose={() => setMobileMoreOpen(false)}
          onAuditLogClick={() => { void loadAuditLog(); setAdminView("audit-log"); setShowMobileDetail(false); }}
          onCommunicationOpsClick={() => { setAdminView("communications"); setShowMobileDetail(false); }}
          onCsvExportClick={() => { void exportTicketCsvBundle(); }}
          onDirectoryClick={() => { setAdminView("directory"); setShowMobileDetail(false); }}
          onDroppedSchoolsClick={() => { void loadDroppedSchools(); setAdminView("dropped-schools"); setShowMobileDetail(false); }}
          onEscalationPolicyClick={() => { setAdminView("escalation"); setShowMobileDetail(false); }}
          onProgramDashboardClick={() => { void loadProgramDashboard(); setAdminView("program-dashboard"); setShowMobileDetail(false); }}
          onRegionLogClick={() => { void loadSchoolRegionHistory(); setAdminView("region-log"); setShowMobileDetail(false); }}
          onReportsClick={() => { void loadProgramDashboard(); setAdminView("reports"); setShowMobileDetail(false); }}
          onRoutingRulesClick={() => { setAdminView("routing"); setShowMobileDetail(false); }}
          onSlaSettingsClick={() => { setAdminView("sla"); setShowMobileDetail(false); }}
          onTemplatesClick={() => { setAdminView("templates"); setShowMobileDetail(false); }}
          onUsersClick={() => { void loadUsers(); setShowUsers(true); }}
          onFacultyAssignmentsClick={() => {
            void loadFacultyAssignments();
            void loadSubjects();
            setAdminView("faculty-assignments");
            setShowMobileDetail(false);
          }}
          onSubjectsClick={() => {
            void loadSubjects();
            setAdminView("subjects");
            setShowMobileDetail(false);
          }}
          onTimetableClick={() => {
            void loadSubjects();
            void loadUsers();
            setAdminView("timetable");
            setShowMobileDetail(false);
          }}
          onLogout={handleLogout}
        />
      )}
    </main>
  );
}

function uniqueTicketValues(tickets: Ticket[], field: keyof Ticket) {
  return Array.from(
    new Set(tickets.map((ticket) => String(ticket[field] ?? "")).filter(Boolean)),
  ).sort((left, right) => left.localeCompare(right));
}

function schoolToProfileDraft(school: School): SchoolProfileDraft {
  return {
    name: school.name,
    region_id: school.region_id,
    region_name: school.region_name,
    program_model: school.program_model,
    distance_classification: school.distance_classification,
    sip_academic_owner_role: school.sip_academic_owner_role,
    sip_academic_owner_name: school.sip_academic_owner_name,
    sip_academic_owner_mobile: school.sip_academic_owner_mobile,
    sip_academic_owner_email: school.sip_academic_owner_email,
    center_head_name: school.center_head_name,
    center_head_mobile: school.center_head_mobile,
    center_head_email: school.center_head_email,
    principal_name: school.principal_name,
    principal_mobile: school.principal_mobile,
    principal_email: school.principal_email,
    school_spoc_name: school.school_spoc_name,
    school_spoc_mobile: school.school_spoc_mobile,
    school_spoc_email: school.school_spoc_email,
    central_academic_spoc_name: school.central_academic_spoc_name,
    central_academic_spoc_mobile: school.central_academic_spoc_mobile,
    central_academic_spoc_email: school.central_academic_spoc_email,
    central_business_spoc_name: school.central_business_spoc_name,
    central_business_spoc_mobile: school.central_business_spoc_mobile,
    central_business_spoc_email: school.central_business_spoc_email,
    bh_name: school.bh_name,
    bh_mobile: school.bh_mobile,
    bh_email: school.bh_email,
    aom_name: school.aom_name,
    aom_mobile: school.aom_mobile,
    aom_email: school.aom_email,
    mapped_vp_center: school.mapped_vp_center,
  };
}
