import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, waitFor, within } from "@testing-library/react";
import { SlaBreachAlert, LoginScreen, ErrorBoundary, MasterDataPanel, CommunicationOperationsPanel, AssignmentRulePanel, SubjectsPanel, FacultyAssignmentsPanel, FacultyMembersPanel, DirectoryPanel, DroppedSchoolsPanel, ReportsPanel } from "./components";
import { AdminPanelRouter } from "./components/admin/AdminPanelRouter";
import type { Batch, School } from "./types";

afterEach(() => {
  cleanup();
});

describe("SlaBreachAlert", () => {
  it("renders null when count is zero", () => {
    const { container } = render(
      <SlaBreachAlert newBreachCount={0} onView={vi.fn()} onDismiss={vi.fn()} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders breach message and actions when count > 0", () => {
    render(
      <SlaBreachAlert newBreachCount={3} onView={vi.fn()} onDismiss={vi.fn()} />
    );
    expect(screen.getByRole("alert")).toHaveTextContent("3 tickets breached SLA");
    expect(screen.getByText("View")).toBeInTheDocument();
    expect(screen.getByText("Dismiss")).toBeInTheDocument();
  });

  it("calls handlers on click", () => {
    const onView = vi.fn();
    const onDismiss = vi.fn();
    render(
      <SlaBreachAlert newBreachCount={1} onView={onView} onDismiss={onDismiss} />
    );
    fireEvent.click(screen.getByText("View"));
    expect(onView).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByText("Dismiss"));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});

describe("LoginScreen", () => {
  it("renders login form with provided error", () => {
    render(
      <LoginScreen
        draft={{ username: "admin", password: "" }}
        error="Invalid credentials"
        onDraftChange={vi.fn()}
        onSubmit={vi.fn()}
      />
    );
    expect(screen.getByText("Invalid credentials")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /sign in/i })).toBeInTheDocument();
  });
});

describe("ErrorBoundary", () => {
  it("renders children when no error", () => {
    render(
      <ErrorBoundary>
        <div data-testid="child">Safe content</div>
      </ErrorBoundary>
    );
    expect(screen.getByTestId("child")).toHaveTextContent("Safe content");
  });

  it("renders fallback when child throws", () => {
    const Thrower = () => { throw new Error("boom"); };
    render(
      <ErrorBoundary fallback={<div data-testid="fallback">Oops</div>}>
        <Thrower />
      </ErrorBoundary>
    );
    expect(screen.getByTestId("fallback")).toHaveTextContent("Oops");
  });
});

describe("MasterDataPanel", () => {
  const baseProps = {
    schools: [],
    regions: [],
    lectureModels: [],
    classPlans: [],
    batches: [],
    students: [],
    studentTotalCount: 0,
    studentPage: 1,
    studentPageSize: 100,
    studentSearch: "",
    sipImportPreview: null,
    onClose: vi.fn(),
    onCreateSchool: vi.fn(),
    onSaveRegion: vi.fn(),
    onCreateLectureModel: vi.fn(),
    onSaveClassPlan: vi.fn(),
    onCreateBatch: vi.fn(),
    onUpdateBatch: vi.fn(),
    onArchiveBatch: vi.fn(),
    onImportSchools: vi.fn(),
    onImportSipMaster: vi.fn(),
    onExportSipMaster: vi.fn(),
    onDeleteSchool: vi.fn(),
    onLoadSchoolDeleteImpact: vi.fn().mockResolvedValue({
      school_id: 1,
      school_name: "Green Valley",
      total_linked_records: 3,
      items: [
        { label: "Students", count: 2 },
        { label: "Tickets", count: 1 },
        { label: "Batches", count: 0 },
      ],
    }),
    onDropSchool: vi.fn(),
    onDeleteRegion: vi.fn(),
    onRemapAndDeleteRegion: vi.fn(),
    onCancelSipMasterImport: vi.fn(),
    onConfirmSipMasterImport: vi.fn(),
    onCreateStudent: vi.fn(),
    onUpdateStudent: vi.fn(),
    onDeleteStudent: vi.fn(),
    onStudentSearchChange: vi.fn(),
    onLoadStudents: vi.fn(),
    onImportStudentsCsv: vi.fn(),
  };

  it("shows all admin-only controls for admin", () => {
    render(<MasterDataPanel {...baseProps} currentUserRole="admin" />);
    expect(screen.getByRole("button", { name: /add school/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add region/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add lecture model/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /import schools/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /import sip master/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add class offering/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add batch/i })).toBeInTheDocument();
  });

  it("hides admin-only controls for aom", () => {
    render(<MasterDataPanel {...baseProps} currentUserRole="aom" />);
    expect(screen.queryByRole("button", { name: /add school/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add region/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add lecture model/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /import schools/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /import sip master/i })).not.toBeInTheDocument();
    // AOM should still see scoped controls
    expect(screen.getByRole("button", { name: /add class offering/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add batch/i })).toBeInTheDocument();
  });

  it("drops active schools only after a reason is entered", () => {
    const onDropSchool = vi.fn();
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        schools={[{ id: 1, name: "Green Valley", region_name: "North", program_model: "SIP", principal_name: "Principal" } as unknown as School]}
        onDropSchool={onDropSchool}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /^drop$/i }));
    const dropButton = screen.getByRole("button", { name: /drop school/i });
    expect(dropButton).toBeDisabled();
    fireEvent.change(screen.getByLabelText(/reason/i), { target: { value: "Partnership ended" } });
    fireEvent.click(dropButton);
    expect(onDropSchool).toHaveBeenCalledWith(1, "Partnership ended");
  });

  it("keeps permanent school delete admin-only and impact-review gated", async () => {
    const onDeleteSchool = vi.fn();
    const onLoadSchoolDeleteImpact = vi.fn().mockResolvedValue({
      school_id: 1,
      school_name: "Green Valley",
      total_linked_records: 5,
      items: [
        { label: "Students", count: 4 },
        { label: "Tickets", count: 1 },
      ],
    });
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        schools={[{ id: 1, name: "Green Valley", region_name: "North", program_model: "SIP", principal_name: "Principal" } as unknown as School]}
        onDeleteSchool={onDeleteSchool}
        onLoadSchoolDeleteImpact={onLoadSchoolDeleteImpact}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    expect(onDeleteSchool).not.toHaveBeenCalled();
    await waitFor(() => expect(screen.getByText(/5/)).toBeInTheDocument());
    const dialog = screen.getByRole("dialog", { name: /permanent delete review/i });
    expect(within(dialog).getByText("Students")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /permanently delete school/i }));
    expect(onDeleteSchool).toHaveBeenCalledWith(1);
  });

  it("hides permanent school delete for aom", () => {
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="aom"
        schools={[{ id: 1, name: "Green Valley", region_name: "North", program_model: "SIP", principal_name: "Principal" } as unknown as School]}
        onLoadSchoolDeleteImpact={vi.fn()}
      />
    );
    expect(screen.getByRole("button", { name: /^drop$/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^delete$/i })).not.toBeInTheDocument();
  });

  it("submits class offerings with canonical grade and delivery pattern", async () => {
    const onSaveClassPlan = vi.fn();
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        schools={[{ id: 1, name: "Green Valley" } as School]}
        lectureModels={[{ id: 2, name: "3x3", days_per_week: 3, lectures_per_day: 3, created_at: "" }]}
        onSaveClassPlan={onSaveClassPlan}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /add class offering/i }));
    const form = screen.getByRole("heading", { name: /add class offering/i }).closest(".master-form") as HTMLElement;
    fireEvent.change(within(form).getByLabelText(/school/i), { target: { value: "1" } });
    fireEvent.change(within(form).getByLabelText(/grade level/i), { target: { value: "Grade 11" } });
    fireEvent.change(within(form).getByLabelText(/^track$/i), { target: { value: "JEE" } });
    fireEvent.change(within(form).getByLabelText(/lecture model/i), { target: { value: "2" } });
    fireEvent.change(within(form).getByLabelText(/delivery pattern/i), { target: { value: "Weekday" } });
    fireEvent.change(within(form).getByLabelText(/aop admissions/i), { target: { value: "80" } });
    fireEvent.change(within(form).getByLabelText(/registrations/i), { target: { value: "70" } });
    fireEvent.change(within(form).getByLabelText(/actual admissions/i), { target: { value: "60" } });
    fireEvent.click(within(form).getByRole("button", { name: /^save$/i }));

    await waitFor(() => expect(onSaveClassPlan).toHaveBeenCalledWith({
      school_id: 1,
      grade_level: "Grade 11",
      track: "JEE",
      lecture_model_id: 2,
      batch_pattern: "Weekday",
      aop_admissions: 80,
      registrations: 70,
      actual_admissions: 60,
    }));
  });

  it("shows configured batches in Master Data", () => {
    const batches: Batch[] = [
      {
        id: 1,
        school_id: 1,
        school_name: "Green Valley",
        batch_id: "XI-JEE-WD-A",
        grade_level: "Grade 11",
        track: "JEE",
        batch_pattern: "Weekday",
        capacity: 40,
        created_at: "",
      },
    ];

    render(<MasterDataPanel {...baseProps} currentUserRole="admin" batches={batches} />);

    expect(screen.getAllByRole("heading", { name: /batches/i }).length).toBeGreaterThan(0);
    expect(screen.getByText("XI-JEE-WD-A")).toBeInTheDocument();
    expect(screen.getByText("Green Valley")).toBeInTheDocument();
    expect(screen.getByText("Grade 11")).toBeInTheDocument();
    expect(screen.getByText("Weekday")).toBeInTheDocument();
    expect(screen.getByText("40")).toBeInTheDocument();
  });

  it("creates batches under a selected class offering", async () => {
    const onCreateBatch = vi.fn();
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        classPlans={[
          {
            id: 9,
            school_id: 1,
            school_name: "Green Valley",
            grade_level: "Grade 11",
            track: "JEE",
            lecture_model_id: 2,
            lecture_model_name: "3x3",
            days_per_week: 3,
            lectures_per_day: 3,
            batch_pattern: "Weekday",
            aop_admissions: 80,
            registrations: 70,
            actual_admissions: 60,
            admission_gap: 20,
            admission_attainment_percent: 75,
            updated_at: "",
          },
        ]}
        onCreateBatch={onCreateBatch}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /add batch/i }));
    fireEvent.change(screen.getByLabelText(/class offering/i), { target: { value: "9" } });
    fireEvent.change(screen.getByLabelText(/batch name/i), { target: { value: "XI-JEE-WD-A" } });
    fireEvent.change(screen.getByLabelText(/capacity/i), { target: { value: "40" } });
    fireEvent.click(screen.getByRole("button", { name: /create batch/i }));

    await waitFor(() => expect(onCreateBatch).toHaveBeenCalledWith({
      school_id: 1,
      batch_id: "XI-JEE-WD-A",
      grade_level: "Grade 11",
      track: "JEE",
      batch_pattern: "Weekday",
      capacity: 40,
    }));
  });

  it("archives batches only after confirmation", () => {
    const onArchiveBatch = vi.fn();
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        batches={[
          {
            id: 1,
            school_id: 1,
            school_name: "Green Valley",
            batch_id: "XI-JEE-WD-A",
            grade_level: "Grade 11",
            track: "JEE",
            batch_pattern: "Weekday",
            capacity: 40,
            created_at: "",
          },
        ]}
        onArchiveBatch={onArchiveBatch}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /^archive$/i }));
    expect(onArchiveBatch).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: /confirm archive/i }));
    expect(onArchiveBatch).toHaveBeenCalledWith(1);
  });

  it("shows students with concrete batch names in Master Data", () => {
    render(
      <MasterDataPanel
        {...baseProps}
        currentUserRole="admin"
        schools={[{ id: 1, name: "Green Valley" } as unknown as import("./types").School]}
        studentTotalCount={1}
        students={[
          {
            id: 1,
            school_id: 1,
            school_name: "Green Valley",
            name: "Aarav Shah",
            registration_number: "GV001",
            grade_level: "Grade 11",
            program_track: "SIP",
            track: "JEE",
            student_mobile: "9000000001",
            student_email: "aarav@example.com",
            father_name: "",
            father_email: "",
            father_mobile: "",
            mother_name: "",
            mother_email: "",
            mother_mobile: "",
            batch_ref_id: 1,
            batch_name: "XI-JEE-WD-A",
            batch_id: "XI-JEE-WD-A",
            created_at: "",
          },
        ]}
      />
    );

    expect(screen.getAllByRole("heading", { name: /students/i }).length).toBeGreaterThan(0);
    fireEvent.change(screen.getByLabelText("Student list filter"), { target: { value: "1" } });
    expect(screen.getByText("Aarav Shah")).toBeInTheDocument();
    expect(screen.getByText("XI-JEE-WD-A")).toBeInTheDocument();
    expect(screen.getByText("GV001")).toBeInTheDocument();
  });
});

describe("AdminPanelRouter master-data view", () => {
  const minimalMaster = {
    schools: [],
    droppedSchools: [],
    regions: [],
    lectureModels: [],
    classPlans: [],
    students: [],
    studentTotalCount: 0,
    studentPage: 1,
    studentPageSize: 100,
    studentSearch: "",
    batches: [],
    schoolRegionHistory: [],
    programDashboard: null,
    sipImportReview: null,
    setSchools: vi.fn(),
    setDroppedSchools: vi.fn(),
    setRegions: vi.fn(),
    setLectureModels: vi.fn(),
    setClassPlans: vi.fn(),
    setStudents: vi.fn(),
    setStudentSearch: vi.fn(),
    setBatches: vi.fn(),
    setSchoolRegionHistory: vi.fn(),
    setProgramDashboard: vi.fn(),
    setSipImportReview: vi.fn(),
    loadSchools: vi.fn(),
    loadDroppedSchools: vi.fn(),
    loadSchoolRegionHistory: vi.fn(),
    loadRegions: vi.fn(),
    loadLectureModels: vi.fn(),
    loadClassPlans: vi.fn(),
    loadProgramDashboard: vi.fn(),
    loadStudents: vi.fn(),
    loadBatches: vi.fn(),
    loadSchoolDeleteImpact: vi.fn(),
    createSchool: vi.fn(),
    dropSchool: vi.fn(),
    deleteSchool: vi.fn(),
    restoreSchool: vi.fn(),
    saveRegion: vi.fn(),
    createLectureModel: vi.fn(),
    saveSchoolClassPlan: vi.fn(),
    createBatch: vi.fn(),
    updateBatch: vi.fn(),
    archiveBatch: vi.fn(),
    importSchoolsCsv: vi.fn(),
    importSipMaster: vi.fn(),
    confirmSipMasterImport: vi.fn(),
    cancelSipMasterImport: vi.fn(),
    exportSipMasterExcel: vi.fn(),
    deleteRegion: vi.fn(),
    remapAndDeleteRegion: vi.fn(),
    createStudent: vi.fn(),
    updateStudent: vi.fn(),
    deleteStudent: vi.fn(),
    importStudentsCsv: vi.fn(),
  } as unknown as import("./hooks").UseMasterDataStateReturn;

  const minimalAdmin = {
    users: [],
    showUsers: false,
    assignmentRules: [],
    communicationTemplates: [],
    escalationPolicy: null,
    slaPolicies: [],
    auditLog: [],
    setShowUsers: vi.fn(),
    loadUsers: vi.fn(),
    loadAssignmentRules: vi.fn(),
    loadCommunicationTemplates: vi.fn(),
    loadEscalationPolicy: vi.fn(),
    loadSlaPolicies: vi.fn(),
    loadAuditLog: vi.fn(),
    createUser: vi.fn(),
    updateUser: vi.fn(),
    deleteUser: vi.fn(),
    resetPassword: vi.fn(),
    changePassword: vi.fn(),
    saveEscalationPolicy: vi.fn(),
    saveAssignmentRule: vi.fn(),
    saveCommunicationTemplate: vi.fn(),
    saveSlaPolicy: vi.fn(),
  } as unknown as import("./hooks").UseAdminStateReturn;

  const minimalTicket = {
    tickets: [],
    ticketsLoading: false,
    comments: [],
    allComments: [],
    history: [],
    attachments: [],
    selectedId: null,
    selected: null,
    activeFilter: "all" as const,
    programScopeFilters: { school_name: "", grade_level: "", program_track: "", issue_category: "", queue: "" },
    search: "",
    dateFrom: "",
    dateTo: "",
    newBreachCount: 0,
    isCreating: false,
    isEditing: false,
    isConfirmingDelete: false,
    draft: {} as Record<string, unknown>,
    reply: {} as Record<string, unknown>,
    filterCounts: { all: 0, open: 0, pending_sla: 0, escalated: 0 },
    schoolOptions: [],
    latestUpdate: "",
    ticketPage: 1,
    ticketPageSize: 20,
    ticketTotalCount: 0,
    commentPage: 1,
    commentPageSize: 20,
    commentTotalCount: 0,
    allCommentPage: 1,
    allCommentPageSize: 20,
    allCommentTotalCount: 0,
    historyPage: 1,
    historyPageSize: 20,
    historyTotalCount: 0,
    slaStatus: null,
    setActiveFilter: vi.fn(),
    setProgramScopeFilters: vi.fn(),
    setSearch: vi.fn(),
    setDateFrom: vi.fn(),
    setDateTo: vi.fn(),
    setNewBreachCount: vi.fn(),
    setIsCreating: vi.fn(),
    setIsEditing: vi.fn(),
    setIsConfirmingDelete: vi.fn(),
    setDraft: vi.fn(),
    setReply: vi.fn(),
    setTicketPage: vi.fn(),
    setCommentPage: vi.fn(),
    setAllCommentPage: vi.fn(),
    setHistoryPage: vi.fn(),
    setSelectedId: vi.fn(),
    loadTickets: vi.fn(),
    loadComments: vi.fn(),
    loadAllComments: vi.fn(),
    loadHistory: vi.fn(),
    loadAttachments: vi.fn(),
    loadFilterCounts: vi.fn(),
    loadSchoolOptions: vi.fn(),
    createTicket: vi.fn(),
    updateTicket: vi.fn(),
    deleteTicket: vi.fn(),
    addComment: vi.fn(),
    updateCommentStatus: vi.fn(),
    refreshSlaStatus: vi.fn(),
    refreshEscalations: vi.fn(),
  } as unknown as import("./hooks").UseTicketStateReturn;

  const minimalFaculty = {
    subjects: [],
    effectiveSubjects: [],
    facultyAssignments: [],
    timetableSlots: [],
    weeklyTimetableSlots: [],
    lectureSessions: [],
    adminSessions: [],
    facultyWeeklySlots: [],
    leaveRequests: [],
    swapRequests: [],
    substitutions: [],
    pendingSubstitutionRequests: [],
    todaySubstitutions: null,
    substitutionBalance: {},
    substitutionReports: [],
    timetableHealthData: null,
    complianceData: null,
    attendanceSummary: [],
    chronicAbsentees: [],
    subjectAttendance: [],
    vpCenters: [],
    buildings: [],
    holidays: [],
    loadSubjects: vi.fn(),
    loadEffectiveSubjects: vi.fn(),
    loadFacultyAssignments: vi.fn(),
    loadTimetableSlots: vi.fn(),
    loadWeeklyTimetableSlots: vi.fn(),
    loadLectureSessions: vi.fn(),
    loadAdminSessions: vi.fn(),
    loadFacultyWeeklySlots: vi.fn(),
    loadSubstitutions: vi.fn(),
    loadLeaveRequests: vi.fn(),
    loadSwapRequests: vi.fn(),
    loadPendingSubstitutionRequests: vi.fn(),
    loadTodaySubstitutions: vi.fn(),
    loadSubstitutionBalance: vi.fn(),
    loadSubstitutionReports: vi.fn(),
    loadTimetableHealth: vi.fn(),
    loadComplianceData: vi.fn(),
    loadAttendanceSummary: vi.fn(),
    loadChronicAbsentees: vi.fn(),
    loadSubjectAttendance: vi.fn(),
    loadVpCenters: vi.fn(),
    loadBuildings: vi.fn(),
    loadHolidays: vi.fn(),
    createSubject: vi.fn(),
    updateSubject: vi.fn(),
    deleteSubject: vi.fn(),
    createFacultyAssignment: vi.fn(),
    deleteFacultyAssignment: vi.fn(),
    upsertTimetableSlot: vi.fn(),
    deleteTimetableSlot: vi.fn(),
    upsertWeeklyTimetableSlot: vi.fn(),
    deleteWeeklyTimetableSlot: vi.fn(),
    cloneWeek: vi.fn(),
    createHoliday: vi.fn(),
    deleteHoliday: vi.fn(),
    createLeaveRequest: vi.fn(),
    approveLeaveRequest: vi.fn(),
    rejectLeaveRequest: vi.fn(),
    createSwapRequest: vi.fn(),
    acceptSwapRequest: vi.fn(),
    suggestSubstitutes: vi.fn(),
    assignSubstitute: vi.fn(),
    markAttendance: vi.fn(),
    bulkMarkAttendance: vi.fn(),
    createMakeupSession: vi.fn(),
    handleAcceptSubstitution: vi.fn(),
    handleDeclineSubstitution: vi.fn(),
    createVpCenter: vi.fn(),
    updateVpCenter: vi.fn(),
    deleteVpCenter: vi.fn(),
    createBuilding: vi.fn(),
    updateBuilding: vi.fn(),
    deleteBuilding: vi.fn(),
  } as unknown as import("./hooks").UseFacultyStateReturn;

  it("shows admin-only controls through the real router path", () => {
    render(
      <AdminPanelRouter
        adminView="master-data"
        admin={minimalAdmin}
        master={minimalMaster}
        ticket={minimalTicket}
        faculty={minimalFaculty}
        onClose={vi.fn()}
        onSaveEscalationPolicy={vi.fn()}
        currentUserRole="admin"
        currentUser={{ id: 1, username: "admin", display_name: "Admin", role: "admin", school_ids: [] }}
      />
    );
    expect(screen.getByRole("button", { name: /add school/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add region/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add lecture model/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /import schools/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /import sip master/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add class offering/i })).toBeInTheDocument();
  });

  it("hides admin-only controls through the real router path for aom", () => {
    render(
      <AdminPanelRouter
        adminView="master-data"
        admin={minimalAdmin}
        master={minimalMaster}
        ticket={minimalTicket}
        faculty={minimalFaculty}
        onClose={vi.fn()}
        onSaveEscalationPolicy={vi.fn()}
        currentUserRole="aom"
        currentUser={{ id: 2, username: "aom1", display_name: "AOM", role: "aom", school_ids: [1] }}
      />
    );
    expect(screen.queryByRole("button", { name: /add school/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add region/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add lecture model/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /import schools/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /import sip master/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add class offering/i })).toBeInTheDocument();
  });
});

describe("DroppedSchoolsPanel", () => {
  const droppedSchool = {
    id: 1,
    name: "Dropped School",
    region_name: "North",
    program_model: "SIP",
    dropped_at: "2026-05-04 10:00:00",
    dropped_reason: "Partnership ended",
  } as unknown as School;

  it("restores dropped schools and hides permanent delete for aom", () => {
    const onRestoreSchool = vi.fn();
    render(
      <DroppedSchoolsPanel
        schools={[droppedSchool]}
        currentUserRole="aom"
        onRestoreSchool={onRestoreSchool}
        onDeleteSchool={vi.fn()}
        onLoadSchoolDeleteImpact={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /restore/i }));
    expect(onRestoreSchool).toHaveBeenCalledWith(1);
    expect(screen.queryByRole("button", { name: /^delete$/i })).not.toBeInTheDocument();
  });

  it("allows admin permanent delete only after impact review", async () => {
    const onDeleteSchool = vi.fn();
    const onLoadSchoolDeleteImpact = vi.fn().mockResolvedValue({
      school_id: 1,
      school_name: "Dropped School",
      total_linked_records: 2,
      items: [{ label: "Students", count: 2 }],
    });
    render(
      <DroppedSchoolsPanel
        schools={[droppedSchool]}
        currentUserRole="admin"
        onRestoreSchool={vi.fn()}
        onDeleteSchool={onDeleteSchool}
        onLoadSchoolDeleteImpact={onLoadSchoolDeleteImpact}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    expect(onDeleteSchool).not.toHaveBeenCalled();
    await waitFor(() => expect(screen.getByText(/linked records/)).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: /permanently delete school/i }));
    expect(onDeleteSchool).toHaveBeenCalledWith(1);
  });
});

describe("ReportsPanel", () => {
  const reportData = {
    tickets_by_status: { Open: 2 },
    tickets_by_school: { "Green Valley": 2 },
  };

  it("renders DAS controls and rows for admin users", () => {
    const onLoadDasReport = vi.fn();
    render(
      <ReportsPanel
        data={reportData}
        currentUserRole="admin"
        schools={[{ id: 1, name: "Green Valley" } as School]}
        dasRows={[{
          group_by: "school",
          label: "Green Valley",
          school_id: 1,
          school_name: "Green Valley",
          grade_level: "",
          cohort: "",
          batch_id: "",
          student_id: null,
          student_name: "",
          scheduled_lectures: 10,
          present_lectures: 8,
          das_percent: 80,
        }]}
        onLoadDasReport={onLoadDasReport}
      />
    );

    expect(screen.getByText(/DAS Attendance Index/i)).toBeInTheDocument();
    expect(screen.getByText("80%")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /calculate das/i }));
    expect(onLoadDasReport).toHaveBeenCalledWith(expect.any(String), expect.any(String), "school", undefined);
  });

  it("does not show DAS controls to viewer users", () => {
    render(<ReportsPanel data={reportData} currentUserRole="viewer" />);
    expect(screen.queryByText(/DAS Attendance Index/i)).not.toBeInTheDocument();
    expect(screen.getByText(/By Status/i)).toBeInTheDocument();
  });
});


describe("CommunicationOperationsPanel", () => {
  const baseTemplates = [
    { id: 1, name: "Welcome", audience: "School", body: "Welcome message", is_active: true, updated_at: "2024-01-01T00:00:00Z" },
    { id: 2, name: "Reminder", audience: "Parent", body: "Reminder message", is_active: false, updated_at: "2024-01-02T00:00:00Z" },
  ];

  it("admin sees Add Template button and toggle controls", () => {
    const onAdd = vi.fn();
    const onToggle = vi.fn();
    render(
      <CommunicationOperationsPanel
        templates={baseTemplates}
        onAddTemplate={onAdd}
        onToggleTemplate={onToggle}
        currentUserRole="admin"
      />
    );
    expect(screen.getByRole("button", { name: /add template/i })).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /deactivate|activate/i }).length).toBe(2);
  });

  it("admin can trigger add and toggle actions", () => {
    const onAdd = vi.fn();
    const onToggle = vi.fn();
    render(
      <CommunicationOperationsPanel
        templates={baseTemplates}
        onAddTemplate={onAdd}
        onToggleTemplate={onToggle}
        currentUserRole="admin"
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add template/i }));
    expect(onAdd).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: /deactivate/i }));
    expect(onToggle).toHaveBeenCalledWith(1);
  });

  it("aom sees read-only copy and no write controls", () => {
    render(
      <CommunicationOperationsPanel
        templates={baseTemplates}
        onAddTemplate={vi.fn()}
        onToggleTemplate={vi.fn()}
        currentUserRole="aom"
      />
    );
    expect(screen.getByText(/templates are read-only for your role/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add template/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /deactivate|activate/i })).not.toBeInTheDocument();
  });

  it("agent sees read-only copy and no write controls", () => {
    render(
      <CommunicationOperationsPanel
        templates={baseTemplates}
        onAddTemplate={vi.fn()}
        onToggleTemplate={vi.fn()}
        currentUserRole="agent"
      />
    );
    expect(screen.getByText(/templates are read-only for your role/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add template/i })).not.toBeInTheDocument();
  });

  it("renders empty state when no templates", () => {
    render(
      <CommunicationOperationsPanel
        templates={[]}
        onAddTemplate={vi.fn()}
        onToggleTemplate={vi.fn()}
        currentUserRole="admin"
      />
    );
    expect(screen.getByText(/no templates/i)).toBeInTheDocument();
  });
});


describe("AssignmentRulePanel", () => {
  const baseRules = [
    { id: 1, queue: " Admissions", assignee: "Alice", condition: "Active" },
    { id: 2, queue: "Support", assignee: "Bob", condition: "Inactive" },
  ];

  it("renders existing rule values in editable inputs", () => {
    render(<AssignmentRulePanel rules={baseRules} onSave={vi.fn()} />);
    const queueInputs = screen.getAllByLabelText(/queue for rule/i);
    const assigneeInputs = screen.getAllByLabelText(/assignee for rule/i);
    const conditionSelects = screen.getAllByLabelText(/condition for rule/i);

    expect(queueInputs[0]).toHaveValue(" Admissions");
    expect(assigneeInputs[0]).toHaveValue("Alice");
    expect(conditionSelects[0]).toHaveValue("Active");

    expect(queueInputs[1]).toHaveValue("Support");
    expect(assigneeInputs[1]).toHaveValue("Bob");
    expect(conditionSelects[1]).toHaveValue("Inactive");
  });

  it("calls onSave with updated draft after editing", () => {
    const onSave = vi.fn();
    render(<AssignmentRulePanel rules={baseRules} onSave={onSave} />);

    const queueInputs = screen.getAllByLabelText(/queue for rule/i);
    fireEvent.change(queueInputs[0], { target: { value: "Billing" } });

    const assigneeInputs = screen.getAllByLabelText(/assignee for rule/i);
    fireEvent.change(assigneeInputs[1], { target: { value: "Charlie" } });

    const conditionSelects = screen.getAllByLabelText(/condition for rule/i);
    fireEvent.change(conditionSelects[0], { target: { value: "Inactive" } });

    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    expect(onSave).toHaveBeenCalledTimes(1);
    const saved = onSave.mock.calls[0][0];
    expect(saved[0]).toMatchObject({ id: 1, queue: "Billing", assignee: "Alice", condition: "Inactive" });
    expect(saved[1]).toMatchObject({ id: 2, queue: "Support", assignee: "Charlie", condition: "Inactive" });
  });

  it("does not show Save button when there are no rules", () => {
    render(<AssignmentRulePanel rules={[]} onSave={vi.fn()} />);
    expect(screen.getByText(/no rules/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /save/i })).not.toBeInTheDocument();
  });

  it("syncs draft state when rules prop changes", () => {
    const { rerender } = render(<AssignmentRulePanel rules={baseRules} onSave={vi.fn()} />);
    const queueInputs = screen.getAllByLabelText(/queue for rule/i);
    expect(queueInputs[0]).toHaveValue(" Admissions");

    rerender(<AssignmentRulePanel rules={[{ id: 3, queue: "NewQueue", assignee: "Dana", condition: "Active" }]} onSave={vi.fn()} />);
    const updatedInputs = screen.getAllByLabelText(/queue for rule/i);
    expect(updatedInputs[0]).toHaveValue("NewQueue");
    expect(updatedInputs).toHaveLength(1);
  });
});


describe("SubjectsPanel", () => {
  const baseSubjects = [
    { id: 1, name: "Physics", track: "JEE", is_default: true, sort_order: 1 },
    { id: 2, name: "Chemistry", track: "NEET", is_default: false, sort_order: 2 },
  ];

  it("renders subjects in read-only mode when no handlers provided", () => {
    render(<SubjectsPanel subjects={baseSubjects} />);
    expect(screen.getByText("Physics")).toBeInTheDocument();
    expect(screen.getByText("Chemistry")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add subject/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /edit/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /delete/i })).not.toBeInTheDocument();
  });

  it("shows Add Subject button when onCreateSubject is provided", () => {
    render(<SubjectsPanel subjects={baseSubjects} onCreateSubject={vi.fn()} />);
    expect(screen.getByRole("button", { name: /add subject/i })).toBeInTheDocument();
  });

  it("reveals add form row when Add Subject is clicked", () => {
    render(<SubjectsPanel subjects={baseSubjects} onCreateSubject={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /add subject/i }));
    expect(screen.getByLabelText(/new subject name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/new subject track/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/new subject default/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/new subject sort order/i)).toBeInTheDocument();
  });

  it("calls onCreateSubject with correct payload when saving new subject", () => {
    const onCreate = vi.fn();
    render(<SubjectsPanel subjects={baseSubjects} onCreateSubject={onCreate} />);
    fireEvent.click(screen.getByRole("button", { name: /add subject/i }));

    fireEvent.change(screen.getByLabelText(/new subject name/i), { target: { value: "Biology" } });
    fireEvent.change(screen.getByLabelText(/new subject track/i), { target: { value: "NEET" } });
    fireEvent.change(screen.getByLabelText(/new subject sort order/i), { target: { value: "3" } });

    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onCreate).toHaveBeenCalledTimes(1);
    expect(onCreate).toHaveBeenCalledWith({ name: "Biology", track: "NEET", is_default: true, sort_order: 3 });
  });

  it("shows edit and delete buttons when handlers are provided", () => {
    render(
      <SubjectsPanel
        subjects={baseSubjects}
        onCreateSubject={vi.fn()}
        onUpdateSubject={vi.fn()}
        onDeleteSubject={vi.fn()}
      />
    );
    expect(screen.getAllByRole("button", { name: /edit/i }).length).toBe(2);
    expect(screen.getAllByRole("button", { name: /delete/i }).length).toBe(2);
  });

  it("enters inline edit mode and calls onUpdateSubject on save", () => {
    const onUpdate = vi.fn();
    render(
      <SubjectsPanel
        subjects={baseSubjects}
        onCreateSubject={vi.fn()}
        onUpdateSubject={onUpdate}
        onDeleteSubject={vi.fn()}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /edit/i })[0]);

    fireEvent.change(screen.getByLabelText(/edit name for subject 1/i), { target: { value: "Advanced Physics" } });
    fireEvent.change(screen.getByLabelText(/edit track for subject 1/i), { target: { value: "Foundation" } });

    fireEvent.click(screen.getAllByRole("button", { name: /^save$/i })[0]);
    expect(onUpdate).toHaveBeenCalledTimes(1);
    expect(onUpdate).toHaveBeenCalledWith({
      id: 1,
      name: "Advanced Physics",
      track: "Foundation",
      is_default: true,
      sort_order: 1,
    });
  });

  it("does not fire onDeleteSubject on first delete click (requires confirmation)", () => {
    const onDelete = vi.fn();
    render(
      <SubjectsPanel
        subjects={baseSubjects}
        onCreateSubject={vi.fn()}
        onUpdateSubject={vi.fn()}
        onDeleteSubject={onDelete}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /delete/i })[1]);
    expect(onDelete).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeInTheDocument();
  });

  it("fires onDeleteSubject on second confirmation click", () => {
    const onDelete = vi.fn();
    render(
      <SubjectsPanel
        subjects={baseSubjects}
        onCreateSubject={vi.fn()}
        onUpdateSubject={vi.fn()}
        onDeleteSubject={onDelete}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /delete/i })[1]);
    fireEvent.click(screen.getByRole("button", { name: /confirm delete/i }));
    expect(onDelete).toHaveBeenCalledTimes(1);
    expect(onDelete).toHaveBeenCalledWith(2);
  });

  it("cancels delete confirmation when edit is started on another row", () => {
    const onDelete = vi.fn();
    render(
      <SubjectsPanel
        subjects={baseSubjects}
        onCreateSubject={vi.fn()}
        onUpdateSubject={vi.fn()}
        onDeleteSubject={onDelete}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /delete/i })[1]);
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: /edit/i })[0]);
    expect(screen.queryByRole("button", { name: /confirm delete/i })).not.toBeInTheDocument();
  });

  it("renders empty state when no subjects", () => {
    render(<SubjectsPanel subjects={[]} onCreateSubject={vi.fn()} />);
    expect(screen.getByText(/no subjects/i)).toBeInTheDocument();
  });
});


describe("FacultyAssignmentsPanel", () => {
  const baseSchools = [
    { id: 1, name: "Alpha School", region_id: 1, phone: "", email: "", principal_name: "", address: "", city: "", state: "", pincode: "", board: "", establishment_year: 0, website: "", aop: "", director_name: "", director_mobile: "", director_email: "", is_active: true, dropped_at: null, region_name: "North" },
    { id: 2, name: "Beta School", region_id: 2, phone: "", email: "", principal_name: "", address: "", city: "", state: "", pincode: "", board: "", establishment_year: 0, website: "", aop: "", director_name: "", director_mobile: "", director_email: "", is_active: true, dropped_at: null, region_name: "South" },
  ] as unknown as import("./types").School[];

  const baseFacultyMembers = [
    { id: 1, name: "Dr. Smith", email: "", mobile: "", pwid: "", qualification: "", experience_years: 0, designation: "", specialization: "", employment_type: "VP Payroll", is_active: true, user_id: 1, user_username: "faculty1", user_display_name: "Dr. Smith", created_at: "", updated_at: "" },
    { id: 2, name: "Prof. Jones", email: "", mobile: "", pwid: "", qualification: "", experience_years: 0, designation: "", specialization: "", employment_type: "VP Payroll", is_active: true, user_id: 3, user_username: "faculty2", user_display_name: "Prof. Jones", created_at: "", updated_at: "" },
    { id: 3, name: "Unlinked Faculty", email: "", mobile: "", pwid: "", qualification: "", experience_years: 0, designation: "", specialization: "", employment_type: "VP Payroll", is_active: true, user_id: null, user_username: null, user_display_name: null, created_at: "", updated_at: "" },
  ];

  const baseSubjects = [
    { id: 1, name: "Physics", track: "JEE", is_default: true, sort_order: 1 },
    { id: 2, name: "Chemistry", track: "NEET", is_default: true, sort_order: 2 },
    { id: 3, name: "Science", track: "Foundation", is_default: true, sort_order: 3 },
  ];

  const baseBatches = [
    { id: 101, school_id: 1, school_name: "Alpha School", batch_id: "XI-JEE-WD-A", grade_level: "Grade 11", track: "JEE", batch_pattern: "Weekday", capacity: 40, created_at: "" },
    { id: 102, school_id: 1, school_name: "Alpha School", batch_id: "X-FDN-WD-A", grade_level: "Grade 10", track: "", batch_pattern: "Weekday", capacity: 35, created_at: "" },
    { id: 103, school_id: 2, school_name: "Beta School", batch_id: "X-NEET-WE-A", grade_level: "Grade 10", track: "", batch_pattern: "Weekend", capacity: 30, created_at: "" },
  ];

  const baseAssignments = [
    { id: 10, faculty_id: 1, faculty_user_id: 1, faculty_display_name: "Dr. Smith", school_id: 1, school_name: "Alpha School", batch_id: 101, batch_name: "XI-JEE-WD-A", grade_level: "Grade 11", track: "JEE", batch_pattern: "Weekday", subject_id: 1, subject_name: "Physics", created_at: "" },
    { id: 11, faculty_id: 2, faculty_user_id: 3, faculty_display_name: "Prof. Jones", school_id: 2, school_name: "Beta School", batch_id: 103, batch_name: "X-NEET-WE-A", grade_level: "Grade 10", track: "", batch_pattern: "Weekend", subject_id: 3, subject_name: "Science", created_at: "" },
  ];

  it("renders read-only when no handlers provided", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
      />
    );
    expect(screen.getByText("Dr. Smith")).toBeInTheDocument();
    expect(screen.getByText("Alpha School")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /add assignment/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /delete/i })).not.toBeInTheDocument();
  });

  it("shows Add Assignment button when onCreateAssignment is provided", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    expect(screen.getByRole("button", { name: /add assignment/i })).toBeInTheDocument();
  });

  it("reveals create form row when Add Assignment is clicked", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));
    expect(screen.getByRole("combobox", { name: /faculty/i })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: /batch/i })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: /subject/i })).toBeInTheDocument();
  });

  it("faculty select includes active faculty members, including no-login faculty", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));
    const facultySelect = screen.getByRole("combobox", { name: /faculty/i });
    expect(facultySelect).toHaveTextContent("Dr. Smith");
    expect(facultySelect).toHaveTextContent("Prof. Jones");
    expect(facultySelect).toHaveTextContent("Unlinked Faculty (No login)");
  });

  it("save is disabled until required fields are selected", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));
    const saveBtn = screen.getByRole("button", { name: /^save$/i });
    expect(saveBtn).toBeDisabled();

    fireEvent.change(screen.getByRole("combobox", { name: /faculty/i }), { target: { value: "1" } });
    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "102" } });
    fireEvent.change(screen.getByRole("combobox", { name: /subject/i }), { target: { value: "3" } });

    expect(saveBtn).not.toBeDisabled();
  });

  it("calls onCreateAssignment with correct payload", () => {
    const onCreate = vi.fn();
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={onCreate}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));

    fireEvent.change(screen.getByRole("combobox", { name: /faculty/i }), { target: { value: "1" } });
    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "101" } });
    fireEvent.change(screen.getByRole("combobox", { name: /subject/i }), { target: { value: "1" } });

    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onCreate).toHaveBeenCalledTimes(1);
    expect(onCreate).toHaveBeenCalledWith({
      faculty_id: 1,
      batch_id: 101,
      subject_id: 1,
    });
  });

  it("uses foundation subjects for foundation batches", () => {
    const onCreate = vi.fn();
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={onCreate}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));

    fireEvent.change(screen.getByRole("combobox", { name: /faculty/i }), { target: { value: "1" } });
    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "102" } });
    fireEvent.change(screen.getByRole("combobox", { name: /subject/i }), { target: { value: "3" } });

    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onCreate).toHaveBeenCalledWith({
      faculty_id: 1,
      batch_id: 102,
      subject_id: 3,
    });
  });

  it("filters subjects by selected batch track", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));

    const subjectSelect = screen.getByRole("combobox", { name: /subject/i }) as HTMLSelectElement;
    expect(subjectSelect).toBeDisabled();

    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "101" } });
    expect(subjectSelect).not.toBeDisabled();
    const jeeOptionLabels = Array.from(subjectSelect.options).map((option) => option.textContent);
    expect(jeeOptionLabels).toContain("Physics");
    expect(jeeOptionLabels).not.toContain("Chemistry");
    expect(jeeOptionLabels).not.toContain("Science");

    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "102" } });
    const foundationOptionLabels = Array.from(subjectSelect.options).map((option) => option.textContent);
    expect(foundationOptionLabels).toContain("Science");
    expect(foundationOptionLabels).not.toContain("Physics");
    expect(foundationOptionLabels).not.toContain("Chemistry");
  });

  it("creates assignment payload for no-login faculty member", () => {
    const onCreate = vi.fn();
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={onCreate}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add assignment/i }));

    fireEvent.change(screen.getByRole("combobox", { name: /faculty/i }), { target: { value: "3" } });
    fireEvent.change(screen.getByRole("combobox", { name: /batch/i }), { target: { value: "102" } });
    fireEvent.change(screen.getByRole("combobox", { name: /subject/i }), { target: { value: "3" } });

    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onCreate).toHaveBeenCalledWith({
      faculty_id: 3,
      batch_id: 102,
      subject_id: 3,
    });
  });

  it("delete requires two-click confirmation", () => {
    const onDelete = vi.fn();
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
        onDeleteAssignment={onDelete}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /delete/i })[0]);
    expect(onDelete).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /confirm delete/i }));
    expect(onDelete).toHaveBeenCalledTimes(1);
    expect(onDelete).toHaveBeenCalledWith(10);
  });

  it("renders empty state when no assignments", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={[]}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={baseFacultyMembers}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    expect(screen.getByText(/no assignments/i)).toBeInTheDocument();
  });

  it("shows empty-faculty notice when no active faculty members exist", () => {
    render(
      <FacultyAssignmentsPanel
        assignments={baseAssignments}
        schools={baseSchools}
        batches={baseBatches}
        facultyMembers={[{ id: 2, name: "Inactive Only", email: "", mobile: "", pwid: "", qualification: "", experience_years: 0, designation: "", specialization: "", employment_type: "VP Payroll", is_active: false, user_id: null, user_username: null, user_display_name: null, created_at: "", updated_at: "" }]}
        subjects={baseSubjects}
        onCreateAssignment={vi.fn()}
      />
    );
    expect(screen.getByText(/no active faculty members available/i)).toBeInTheDocument();
  });
});

describe("DirectoryPanel", () => {
  const schools = [
    {
      id: 1,
      name: "Green Valley",
      region_id: 1,
      region_name: "North",
      program_model: "SIP",
      distance_classification: "",
      sip_academic_owner_role: "SIP Academic Head",
      sip_academic_owner_name: "Academic Lead",
      sip_academic_owner_mobile: "9000000001",
      sip_academic_owner_email: "academic@example.com",
      center_head_name: "Center Head",
      center_head_mobile: "9000000002",
      center_head_email: "center@example.com",
      principal_name: "Principal Rao",
      principal_mobile: "9000000003",
      principal_email: "principal@example.com",
      school_spoc_name: "School SPOC",
      school_spoc_mobile: "9000000004",
      school_spoc_email: "spoc@example.com",
      central_academic_spoc_name: "RAH One",
      central_academic_spoc_mobile: "9000000005",
      central_academic_spoc_email: "rah@example.com",
      central_business_spoc_name: "RBH One",
      central_business_spoc_mobile: "9000000006",
      central_business_spoc_email: "rbh@example.com",
      bh_name: "",
      bh_mobile: "",
      bh_email: "",
      aom_name: "AOM One",
      aom_mobile: "9000000007",
      aom_email: "aom@example.com",
      mapped_vp_center: "",
      vp_tagging: "",
      is_dropped: false,
      dropped_at: "",
      dropped_reason: "",
      created_at: "",
    },
  ] as unknown as School[];

  const facultyMembers = [
    { id: 1, name: "Dr. Sharma", email: "sharma@example.com;alt@example.com", mobile: "9876543210/9876543211", pwid: "PW001", qualification: "PhD", experience_years: 10, designation: "Professor", specialization: "Physics", employment_type: "VP Payroll", is_active: true, user_id: null, user_username: null, user_display_name: null, created_at: "", updated_at: "" },
  ];

  it("aggregates school and faculty contacts with clickable phone and email links", () => {
    render(
      <DirectoryPanel
        schools={schools}
        facultyMembers={facultyMembers}
        facultyMemberships={{ 1: [{ id: 10, faculty_id: 1, school_id: 1, school_name: "Green Valley", role_at_school: "Faculty", is_primary: true, created_at: "" }] }}
        users={[]}
        onExport={vi.fn()}
      />
    );

    expect(screen.getByText("Principal Rao")).toBeInTheDocument();
    expect(screen.getByText("Dr. Sharma")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "9000000003" })).toHaveAttribute("href", "tel:9000000003");
    expect(screen.getByRole("link", { name: "principal@example.com" })).toHaveAttribute("href", "mailto:principal@example.com");
    expect(screen.getByRole("link", { name: "9876543210" })).toHaveAttribute("href", "tel:9876543210");
    expect(screen.getByRole("link", { name: "alt@example.com" })).toHaveAttribute("href", "mailto:alt@example.com");
  });

  it("filters contacts and supports visible bulk selection", () => {
    render(
      <DirectoryPanel
        schools={schools}
        facultyMembers={facultyMembers}
        facultyMemberships={{ 1: [{ id: 10, faculty_id: 1, school_id: 1, school_name: "Green Valley", role_at_school: "Faculty", is_primary: true, created_at: "" }] }}
        users={[]}
        onExport={vi.fn()}
      />
    );

    fireEvent.change(screen.getByLabelText(/search directory/i), { target: { value: "Sharma" } });
    expect(screen.getByText("Dr. Sharma")).toBeInTheDocument();
    expect(screen.queryByText("Principal Rao")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /select visible/i }));
    expect(screen.getByText(/1 selected/i)).toBeInTheDocument();
    expect(screen.getByText(/2 email/i)).toBeInTheDocument();
  });
});

describe("FacultyMembersPanel", () => {
  const baseMembers = [
    { id: 1, name: "Dr. Sharma", email: "sharma@example.com", mobile: "9876543210", pwid: "PW001", qualification: "PhD", experience_years: 10, designation: "Professor", specialization: "Physics", employment_type: "VP Payroll", is_active: true, user_id: 101, user_username: "sharma", user_display_name: "Dr. Sharma", created_at: "", updated_at: "" },
    { id: 2, name: "Ms. Gupta", email: "gupta@example.com", mobile: "", pwid: "", qualification: "", experience_years: 0, designation: "", specialization: "", employment_type: "VP Payroll", is_active: true, user_id: null, user_username: null, user_display_name: null, created_at: "", updated_at: "" },
  ];

  const baseSchools = [
    { id: 1, name: "Green Valley" },
    { id: 2, name: "North City" },
  ] as unknown as School[];

  it("renders read-only when no handlers provided", () => {
    render(
      <FacultyMembersPanel members={baseMembers} schools={baseSchools} memberships={{}} />
    );
    expect(screen.getByText("Dr. Sharma")).toBeInTheDocument();
    expect(screen.queryByText("Add Faculty")).not.toBeInTheDocument();
  });

  it("shows add form when Add Faculty clicked", () => {
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        memberships={{}}
        onCreateMember={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add faculty/i }));
    expect(screen.getByLabelText(/new faculty name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/new faculty school/i)).toBeInTheDocument();
  });

  it("creates member with correct payload", () => {
    const onCreate = vi.fn();
    render(
      <FacultyMembersPanel
        members={[]}
        schools={baseSchools}
        memberships={{}}
        onCreateMember={onCreate}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add faculty/i }));
    fireEvent.change(screen.getByLabelText(/new faculty name/i), { target: { value: "New Faculty" } });
    fireEvent.change(screen.getByLabelText(/new faculty email/i), { target: { value: "new@example.com" } });
    fireEvent.change(screen.getByLabelText(/new faculty mobile/i), { target: { value: "9999999999" } });
    fireEvent.change(screen.getByLabelText(/new faculty pwid/i), { target: { value: "PW999" } });
    fireEvent.change(screen.getByLabelText(/new faculty school/i), { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onCreate).toHaveBeenCalledTimes(1);
    expect(onCreate).toHaveBeenCalledWith(expect.objectContaining({ name: "New Faculty", email: "new@example.com", mobile: "9999999999", pwid: "PW999", is_active: true, initial_school_id: 1 }));
  });

  it("keeps save disabled until new faculty has name and school", () => {
    render(
      <FacultyMembersPanel
        members={[]}
        schools={baseSchools}
        memberships={{}}
        onCreateMember={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /add faculty/i }));
    const save = screen.getByRole("button", { name: /^save$/i });
    expect(save).toBeDisabled();
    fireEvent.change(screen.getByLabelText(/new faculty name/i), { target: { value: "New Faculty" } });
    expect(save).toBeDisabled();
    fireEvent.change(screen.getByLabelText(/new faculty school/i), { target: { value: "1" } });
    expect(save).not.toBeDisabled();
  });

  it("supports inline edit and update", () => {
    const onUpdate = vi.fn();
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        memberships={{}}
        onUpdateMember={onUpdate}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /edit/i })[0]);
    fireEvent.change(screen.getByLabelText(/edit name for faculty 1/i), { target: { value: "Updated Name" } });
    fireEvent.change(screen.getByLabelText(/edit email for faculty 1/i), { target: { value: "updated@example.com" } });
    fireEvent.change(screen.getByLabelText(/edit mobile for faculty 1/i), { target: { value: "8888888888" } });
    fireEvent.click(screen.getByRole("button", { name: /^save$/i }));
    expect(onUpdate).toHaveBeenCalledWith(expect.objectContaining({ id: 1, name: "Updated Name", email: "updated@example.com", mobile: "8888888888" }));
  });

  it("delete requires two-click confirmation", () => {
    const onDelete = vi.fn();
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        memberships={{}}
        onDeleteMember={onDelete}
      />
    );
    fireEvent.click(screen.getAllByRole("button", { name: /delete/i })[0]);
    expect(onDelete).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: /confirm delete/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /confirm delete/i }));
    expect(onDelete).toHaveBeenCalledTimes(1);
    expect(onDelete).toHaveBeenCalledWith(1);
  });

  it("expands row and loads memberships", () => {
    const onLoadMemberships = vi.fn();
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        memberships={{ 1: [{ id: 10, faculty_id: 1, school_id: 1, school_name: "Green Valley", role_at_school: "Faculty", is_primary: true, created_at: "" }] }}
        onLoadMemberships={onLoadMemberships}
      />
    );
    fireEvent.click(screen.getByText("Dr. Sharma"));
    expect(onLoadMemberships).toHaveBeenCalledWith(1);
    expect(screen.getByText("Green Valley")).toBeInTheDocument();
  });

  it("creates login for an unlinked faculty member", () => {
    const onCreateLogin = vi.fn();
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        memberships={{}}
        onCreateLogin={onCreateLogin}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /create login/i }));
    fireEvent.change(screen.getByLabelText(/login username for faculty 2/i), { target: { value: "gupta" } });
    fireEvent.change(screen.getByLabelText(/login display name for faculty 2/i), { target: { value: "Ms. Gupta" } });
    fireEvent.change(screen.getByLabelText(/login password for faculty 2/i), { target: { value: "secret123" } });
    fireEvent.click(screen.getByRole("button", { name: /save login/i }));
    expect(onCreateLogin).toHaveBeenCalledWith(2, {
      username: "gupta",
      display_name: "Ms. Gupta",
      password: "secret123",
    });
  });

  it("links an existing faculty user to an unlinked faculty member", () => {
    const onLinkUser = vi.fn();
    const users = [
      { id: 201, username: "existing", display_name: "Existing Faculty", role: "faculty", is_active: true, created_at: "", last_login_at: "", school_ids: [1] },
      { id: 101, username: "sharma", display_name: "Dr. Sharma", role: "faculty", is_active: true, created_at: "", last_login_at: "", school_ids: [1] },
    ];
    render(
      <FacultyMembersPanel
        members={baseMembers}
        schools={baseSchools}
        users={users}
        memberships={{}}
        onLinkUser={onLinkUser}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: /link user/i }));
    const select = screen.getByLabelText(/existing user for faculty 2/i);
    expect(select).toHaveTextContent("Existing Faculty");
    expect(select).not.toHaveTextContent("Dr. Sharma");
    fireEvent.change(select, { target: { value: "201" } });
    fireEvent.click(screen.getByRole("button", { name: /confirm link/i }));
    expect(onLinkUser).toHaveBeenCalledWith(2, 201);
  });

  it("renders empty state when no members", () => {
    render(
      <FacultyMembersPanel members={[]} schools={baseSchools} memberships={{}} />
    );
    expect(screen.getByText(/no faculty members/i)).toBeInTheDocument();
  });
});
