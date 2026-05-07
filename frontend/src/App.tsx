import React from "react";
import { api, login as apiLogin, logout as apiLogout } from "./api";
import {
  BottomNav,
  ChangePasswordModal,
  CreateTicketModal,
  LoginScreen,
  Metrics,
  MobileMoreMenu,
  ProgramFilters,
  Sidebar,
  SlaBreachAlert,
  TicketDetail,
  TicketList,
  Topbar,
  UserManagementPanel,
} from "./components";
import { AdminPanelRouter } from "./components/admin/AdminPanelRouter";
import { FacultyApp } from "./components/faculty/FacultyApp";
import { ApproverDashboard } from "./components/approver/ApproverDashboard";
import { useTicketState } from "./hooks/useTicketState";
import { useMasterDataState } from "./hooks/useMasterDataState";
import { useFacultyState } from "./hooks/useFacultyState";
import { useAdminState } from "./hooks/useAdminState";
import { canAccessTool } from "./toolRegistry";
import type { AdminView } from "./toolRegistry";

import type {
  CurrentUser,
  LoginDraft,
} from "./types";

function localToday() {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function App() {
  const [currentUser, setCurrentUser] = React.useState<CurrentUser | null>(null);
  const [loginDraft, setLoginDraft] = React.useState<LoginDraft>({ username: "", password: "" });
  const [loginError, setLoginError] = React.useState("");
  const [error, setError] = React.useState("");
  const [notice, setNotice] = React.useState("");
  const [isMobile, setIsMobile] = React.useState(() => window.innerWidth <= 768);
  const [showMobileDetail, setShowMobileDetail] = React.useState(false);
  const [mobileMoreOpen, setMobileMoreOpen] = React.useState(false);
  const [adminView, setAdminView] = React.useState<AdminView>(null);
  const [showChangePassword, setShowChangePassword] = React.useState(false);
  const admin = useAdminState({
    currentUser,
    onError: setError,
    onNotice: setNotice,
  });

  const master = useMasterDataState({
    onError: setError,
    onNotice: setNotice,
    onLoadAuditLog: admin.loadAuditLog,
  });

  const faculty = useFacultyState({
    currentUser,
    onError: setError,
    onNotice: setNotice,
  });

  const ticket = useTicketState({
    currentUser,
    schools: master.schools,
    students: master.students,
    communicationTemplates: admin.communicationTemplates,
    weeklySlots: faculty.weeklyTimetableSlots,
    lectureSessions: faculty.lectureSessions,
    isMobile,
    onError: setError,
    onNotice: setNotice,
    onLoadAuditLog: admin.loadAuditLog,
    onSetMobileView: (view: "home" | "work" | "detail") => {
      if (view === "detail") setShowMobileDetail(true);
      else setShowMobileDetail(false);
    },
  });

  React.useEffect(() => {
    void (async () => {
      try {
        const user = await api<CurrentUser>("get_current_user");
        setCurrentUser(user);
      } catch {
        // no session yet — stay on login screen
      }
    })();
    void ticket.loadTickets();
    void admin.loadAssignmentRules();
    void admin.loadCommunicationTemplates();
    void admin.loadEscalationPolicy();
    void admin.loadSlaPolicies();
    void master.loadSchools();
    void master.loadDroppedSchools();
    void master.loadSchoolRegionHistory();
    void admin.loadAuditLog();
    void ticket.loadAllComments();
    void master.loadRegions();
    void master.loadLectureModels();
    void master.loadClassPlans();
    void master.loadProgramDashboard();
    void master.loadBatches();
    void faculty.loadSubjects();
    void faculty.loadFacultyAssignments();
  }, []);

  React.useEffect(() => {
    function handleResize() { setIsMobile(window.innerWidth <= 768); }
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  React.useEffect(() => {
    if (currentUser?.role !== "head") return;
    void faculty.loadLeaveRequests();
    void faculty.loadPendingSubstitutionRequests();
    void faculty.loadAdminSessions();
    void faculty.loadTimetableHealth();
    void faculty.loadFacultyMembers();
  }, [
    currentUser?.role,
    currentUser?.id,
    faculty.loadLeaveRequests,
    faculty.loadPendingSubstitutionRequests,
    faculty.loadAdminSessions,
    faculty.loadTimetableHealth,
    faculty.loadFacultyMembers,
  ]);

  async function handleLogin() {
    try {
      const user = await apiLogin(loginDraft.username, loginDraft.password);
      setCurrentUser(user as CurrentUser);
      setLoginDraft({ username: "", password: "" });
      setLoginError("");
      setError("");
      void ticket.loadTickets();
      void admin.loadAssignmentRules();
      void admin.loadCommunicationTemplates();
      void admin.loadEscalationPolicy();
      void admin.loadSlaPolicies();
      void master.loadSchools();
      void master.loadDroppedSchools();
      void master.loadSchoolRegionHistory();
      void admin.loadAuditLog();
      void ticket.loadAllComments();
      void master.loadRegions();
      void master.loadLectureModels();
      void master.loadClassPlans();
      void master.loadProgramDashboard();
      void faculty.loadSubjects();
      void faculty.loadFacultyAssignments();
    } catch (caught) {
      setLoginError(String(caught));
    }
  }

  function handleLogout() {
    apiLogout();
    setCurrentUser(null);
    setLoginDraft({ username: "", password: "" });
    setLoginError("");
    admin.setShowUsers(false);
  }

  React.useEffect(() => {
    if (!ticket.isCreating || !ticket.draft.school_id) return;
    void master.loadStudents(ticket.draft.school_id);
  }, [ticket.isCreating, ticket.draft.school_id, master.loadStudents]);

  React.useEffect(() => {
    if (!ticket.isEditing || !ticket.editDraft.school_id) return;
    void master.loadStudents(ticket.editDraft.school_id);
  }, [ticket.isEditing, ticket.editDraft.school_id, master.loadStudents]);

  async function handleSaveEscalationPolicy(input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) {
    await admin.saveEscalationPolicy(input);
    await ticket.loadTickets(ticket.selected?.id);
    if (ticket.selected?.id) {
      await ticket.loadHistory(ticket.selected.id);
    }
  }

  function handleToolClick(toolId: string) {
    if (!currentUser || !canAccessTool(toolId, currentUser.role)) {
      setError("You do not have access to that tool.");
      return;
    }
    setError("");
    switch (toolId) {
      case "master-data":
        void master.loadBatches();
        setAdminView("master-data");
        break;
      case "program-dashboard":
        void master.loadProgramDashboard();
        setAdminView("program-dashboard");
        break;
      case "reports":
        void master.loadProgramDashboard();
        void master.loadSchools();
        void faculty.loadDasReport(localToday(), localToday(), "school");
        setAdminView("reports");
        break;
      case "communications":
        setAdminView("communications");
        break;
      case "directory":
        void faculty.loadFacultyMembers();
        void admin.loadUsers();
        setAdminView("directory");
        break;
      case "dropped-schools":
        void master.loadDroppedSchools();
        setAdminView("dropped-schools");
        break;
      case "region-log":
        void master.loadSchoolRegionHistory();
        setAdminView("region-log");
        break;
      case "audit-log":
        void admin.loadAuditLog();
        setAdminView("audit-log");
        break;
      case "routing":
        setAdminView("routing");
        break;
      case "escalation":
        setAdminView("escalation");
        break;
      case "sla":
        setAdminView("sla");
        break;
      case "templates":
        setAdminView("templates");
        break;
      case "export-csv":
        void ticket.exportTicketCsvBundle();
        break;
      case "users":
        void admin.loadUsers();
        admin.setShowUsers(true);
        break;
      case "faculty-assignments":
        void faculty.loadFacultyAssignments();
        void faculty.loadSubjects();
        void faculty.loadFacultyMembers();
        void master.loadBatches();
        setAdminView("faculty-assignments");
        break;
      case "subjects":
        void faculty.loadSubjects();
        setAdminView("subjects");
        break;
      case "faculty-members":
        void faculty.loadFacultyMembers();
        void admin.loadUsers();
        setAdminView("faculty-members");
        break;
      case "timetable":
        void faculty.loadTimetableSlots({});
        void master.loadBatches();
        void faculty.loadSubjects();
        void faculty.loadFacultyMembers();
        setAdminView("timetable");
        break;
      case "substitutions":
        setAdminView("substitutions");
        break;
      case "leave-swap":
        void faculty.loadFacultyMembers();
        setAdminView("leave-swap");
        break;
      case "alert-inbox":
        setAdminView("alert-inbox");
        break;
      case "control-tower":
        setAdminView("control-tower");
        break;
      case "compliance-scorecard":
        setAdminView("compliance-scorecard");
        break;
      case "deviation-scoreboard":
        setAdminView("deviation-scoreboard");
        break;
      case "holidays":
        void faculty.loadHolidays();
        setAdminView("holidays");
        break;
      case "batches":
        setAdminView("batches");
        break;
    }
  }

  // File-picker helper: attaches the <input> to the DOM (some desktop browsers
  // require this for programmatic .click() to open the dialog), runs the
  // handler, then cleans up.

  React.useEffect(() => {
    function handleKey(event: KeyboardEvent) {
      const tag = (event.target as HTMLElement).tagName.toLowerCase();
      const isTyping = tag === "input" || tag === "textarea" || tag === "select";
      if (event.key === "Escape") {
        if (ticket.isCreating) { ticket.setIsCreating(false); return; }
        if (adminView) { setAdminView(null); return; }
        if (admin.showUsers) { admin.setShowUsers(false); return; }
        if (ticket.isEditing) { ticket.setIsEditing(false); return; }
      }
      if (isTyping || event.metaKey || event.ctrlKey || event.altKey) return;
      if (event.key === "n" && !ticket.isCreating && currentUser?.role !== "viewer") {
        ticket.setIsCreating(true);
      } else if (event.key === "/") {
        event.preventDefault();
        (document.querySelector('input[aria-label="Search tickets"]') as HTMLInputElement | null)?.focus();
      } else if ((event.key === "j" || event.key === "k") && !adminView) {
        const idx = ticket.visibleTickets.findIndex((t) => t.id === ticket.selectedId);
        const next = event.key === "j" ? ticket.visibleTickets[idx + 1] : ticket.visibleTickets[idx - 1];
        if (next) ticket.setSelectedId(next.id);
      }
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [ticket, adminView, admin.showUsers, currentUser]);
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

  if (currentUser.role === "faculty") {
    return (
      <FacultyApp
        user={currentUser}
        onLogout={handleLogout}
        weeklySlots={faculty.facultyWeeklySlots}
        substitutions={faculty.substitutions}
        pendingRequests={faculty.pendingSubstitutionRequests}
        onLoadWeeklySlots={() =>
          faculty.loadFacultyWeeklySlots(currentUser.id, localToday())
        }
        onLoadSubstitutions={faculty.loadSubstitutions}
        onAcceptSubstitution={faculty.handleAcceptSubstitution}
        onDeclineSubstitution={faculty.handleDeclineSubstitution}
      />
    );
  }

  if (currentUser.role === "head") {
    return (
      <ApproverDashboard
        currentUser={currentUser}
        schools={master.schools}
        leaveRequests={faculty.leaveRequests}
        pendingSubstitutions={faculty.pendingSubstitutionRequests}
        alerts={[]}
        adminSessions={faculty.adminSessions}
        timetableHealth={faculty.timetableHealthData ? [faculty.timetableHealthData] : []}
        facultyMembers={faculty.facultyMembers}
      />
    );
  }

  const adminContent = (
    <AdminPanelRouter
      adminView={adminView}
      admin={admin}
      master={master}
      ticket={ticket}
      faculty={faculty}
      onClose={() => setAdminView(null)}
      onSaveEscalationPolicy={handleSaveEscalationPolicy}
      onRefreshSla={ticket.refreshSlaStatus}
      currentUserRole={currentUser.role}
      currentUser={currentUser}
    />
  );

  return (
    <main className="app-shell">
      <Sidebar
        activeFilter={ticket.activeFilter}
        currentUserRole={currentUser.role}
        filterCounts={ticket.filterCounts}
        onFilterChange={(filter) => {
          setAdminView(null);
          ticket.setActiveFilter(filter);
        }}
        onToolClick={handleToolClick}
      />

      <section className="workspace">
        <Topbar
          search={ticket.search}
          currentUser={currentUser}
          latestUpdate={ticket.latestUpdate}
          onSearchChange={ticket.setSearch}
          onCreateClick={() => ticket.setIsCreating(true)}
          onLogout={handleLogout}
          onChangePasswordClick={() => setShowChangePassword(true)}
          mobileBackLabel={showMobileDetail ? "Tickets" : undefined}
          onMobileBack={isMobile && showMobileDetail ? () => setShowMobileDetail(false) : undefined}
        />

        {adminView ? null : (
          <ProgramFilters
            filters={ticket.programScopeFilters}
            schoolOptions={ticket.schoolOptions}
            dateFrom={ticket.dateFrom}
            dateTo={ticket.dateTo}
            onChange={ticket.setProgramScopeFilters}
            onDateFromChange={ticket.setDateFrom}
            onDateToChange={ticket.setDateTo}
            onReset={() => {
              ticket.setProgramScopeFilters({ school_name: "", grade_level: "", program_track: "", issue_category: "", queue: "" });
              ticket.setDateFrom("");
              ticket.setDateTo("");
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
              newBreachCount={ticket.newBreachCount}
              onView={() => {
                ticket.setActiveFilter("Pending SLA");
                setAdminView(null);
                ticket.setNewBreachCount(0);
              }}
              onDismiss={() => ticket.setNewBreachCount(0)}
            />
            <Metrics
              openCount={ticket.openCount}
              activeSchoolCount={ticket.activeSchoolCount}
              activeQueueCount={ticket.activeQueueCount}
              escalatedCount={ticket.escalatedCount}
              pendingSlaCount={ticket.pendingSlaCount}
              unassignedCount={ticket.unassignedCount}
            />

            <div className={`content-grid${showMobileDetail ? " mobile-show-detail" : ""}`}>
              <TicketList
                activeFilter={ticket.activeFilter}
                currentUser={currentUser}
                selectedId={ticket.selected?.id ?? null}
                tickets={ticket.visibleTickets}
                onSelectTicket={ticket.selectTicket}
                onQuickResolve={(id) => void ticket.quickUpdateTicket(id, { status: "Resolved" })}
                onQuickAssign={(id) => void ticket.quickUpdateTicket(id, { assignee: currentUser?.display_name ?? "Service Desk" })}
              />

              <TicketDetail
                assigneeDraft={ticket.assigneeDraft}
                assigneeWorkload={ticket.assigneeWorkload}
                attachments={ticket.comments.length > 0 ? [] : ticket.attachments}
                comments={ticket.comments}
                editDraft={ticket.editDraft}
                history={ticket.history}
                isConfirmingDelete={ticket.isConfirmingDelete}
                isEditing={ticket.isEditing}
                reply={ticket.reply}
                selected={ticket.selected}
                schools={master.schools}
                students={master.students}
                templates={admin.communicationTemplates}
                weeklySlots={faculty.weeklyTimetableSlots}
                lectureSessions={faculty.lectureSessions}
                onViewFullTimetable={() => handleToolClick("timetable")}
                onAddComment={ticket.addComment}
                onAssigneeDraftChange={ticket.setAssigneeDraft}
                onCancelDelete={() => ticket.setIsConfirmingDelete(false)}
                onCancelEdit={() => ticket.setIsEditing(false)}
                onConfirmDelete={ticket.deleteSelectedTicket}
                onEditDraftChange={ticket.setEditDraft}
                onReplyChange={ticket.setReply}
                onRequestDelete={() => ticket.setIsConfirmingDelete(true)}
                onRequestEdit={() => ticket.setIsEditing(true)}
                onSaveEdits={ticket.saveTicketEdits}

                onUpdateCommentStatus={ticket.updateCommentStatus}
                onUpdateTicket={ticket.updateTicket}
              />
            </div>
          </>
        )}
      </section>

      {ticket.isCreating ? (
        <CreateTicketModal
          draft={ticket.draft}
          schools={master.schools}
          students={master.students}
          onCancel={() => ticket.setIsCreating(false)}
          onDraftChange={ticket.setDraft}
          onSubmit={ticket.createTicket}
        />
      ) : null}

      {admin.showUsers ? (
        <UserManagementPanel
          users={admin.users}
          schools={master.schools}
          currentUser={currentUser}
          onClose={() => admin.setShowUsers(false)}
          onCreateUser={admin.handleCreateUser}
          onUpdateUser={admin.handleUpdateUser}
          onDeleteUser={admin.handleDeleteUser}
          onChangePassword={admin.handleChangePassword}
          onResetPassword={admin.handleResetPassword}
        />
      ) : null}

      {showChangePassword ? (
        <ChangePasswordModal
          onClose={() => setShowChangePassword(false)}
          onSubmit={admin.handleChangePassword}
        />
      ) : null}

      <BottomNav
        currentUserRole={currentUser.role}
        currentSection={adminView !== null ? "admin" : "work"}
        filterCounts={ticket.filterCounts}
        mobileView={showMobileDetail ? "detail" : "work"}
        onHomeClick={() => {
          setAdminView(null);
          setShowMobileDetail(false);
          ticket.setActiveFilter("Inbox");
        }}
        onWorkClick={() => {
          setAdminView(null);
          setShowMobileDetail(false);
          ticket.setActiveFilter("Inbox");
        }}
        onCreateClick={() => ticket.setIsCreating(true)}
        onMoreClick={() => setMobileMoreOpen(true)}
      />

      {mobileMoreOpen && (
        <MobileMoreMenu
          currentUserRole={currentUser.role}
          onClose={() => setMobileMoreOpen(false)}
          onToolClick={(id) => { handleToolClick(id); setShowMobileDetail(false); }}
          onLogout={handleLogout}
          onChangePassword={() => setShowChangePassword(true)}
        />
      )}
    </main>
  );
}
