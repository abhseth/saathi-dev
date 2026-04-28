import React from "react";
import { createPortal } from "react-dom";
import {
  academicTracks,
  batchPatterns,
  filters,
  gradeLevels,
  issueCategories,
  priorities,
  programTracks,
  queues,
  statuses,
  trackEligibleGrades,
} from "./constants";
import { formatBytes, formatField, formatSlaDue, formatSlaCountdown, formatTimestamp, getSlaState } from "./formatters";
import type {
  AppUser,
  AuditLogEntry,
  AttachmentDraft,
  AssignmentRule,
  CommunicationTemplate,
  CreateTicketDraft,
  CreateUserDraft,
  CurrentUser,
  EscalationPolicy,
  Filter,
  LectureModel,
  LoginDraft,
  Priority,
  ProgramScopeFilters,
  Queue,
  Region,
  ReplyDraft,
  School,
  SchoolClassPlan,
  SchoolProfileDraft,
  SchoolProgramDashboard,
  SchoolRegionHistory,
  SipMasterImportPreview,
  SlaPolicy,
  Status,
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

type LoginScreenProps = {
  draft: LoginDraft;
  error: string;
  onDraftChange: (draft: LoginDraft) => void;
  onSubmit: () => void;
};

export function LoginScreen({ draft, error, onDraftChange, onSubmit }: LoginScreenProps) {
  function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    onSubmit();
  }
  return (
    <div className="login-backdrop">
      <section className="login-card">
        <div className="login-header">
          <span className="brand-mark">S</span>
          <div>
            <strong>SAATHI</strong>
            <small>School Integrated Program</small>
          </div>
        </div>
        <form onSubmit={handleSubmit} className="login-form">
          <h2>Sign in</h2>
          {error ? <div className="error-banner">{error}</div> : null}
          <label>
            Username
            <input
              autoFocus
              value={draft.username}
              onChange={(e) => onDraftChange({ ...draft, username: e.target.value })}
              placeholder="username"
              autoComplete="username"
            />
          </label>
          <label>
            Password
            <input
              type="password"
              value={draft.password}
              onChange={(e) => onDraftChange({ ...draft, password: e.target.value })}
              placeholder="password"
              autoComplete="current-password"
            />
          </label>
          <button type="submit" className="primary-action">
            Sign in
          </button>
          <small className="login-hint">Default: admin / admin123 — change on first login.</small>
        </form>
      </section>
    </div>
  );
}

type UserManagementPanelProps = {
  users: AppUser[];
  schools: School[];
  currentUser: CurrentUser;
  onClose: () => void;
  onCreateUser: (draft: CreateUserDraft) => void;
  onUpdateUser: (draft: UpdateUserDraft) => void;
  onDeleteUser: (id: number) => void;
  onChangePassword: (currentPassword: string, newPassword: string) => void;
};

const ROLES_NEEDING_SCHOOL_SCOPE = new Set(["aom", "faculty"]);
const ROLE_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "admin", label: "Admin (full access)" },
  { value: "agent", label: "Agent" },
  { value: "viewer", label: "Viewer (read-only)" },
  { value: "aom", label: "AOM (school-scoped admin)" },
  { value: "faculty", label: "Faculty" },
];

export function UserManagementPanel({
  users,
  schools,
  currentUser,
  onClose,
  onCreateUser,
  onUpdateUser,
  onDeleteUser,
  onChangePassword,
}: UserManagementPanelProps) {
  const [mode, setMode] = React.useState<"list" | "create" | "edit" | "password">("list");
  const [editTarget, setEditTarget] = React.useState<AppUser | null>(null);
  const [createDraft, setCreateDraft] = React.useState<CreateUserDraft>({
    username: "",
    display_name: "",
    role: "agent",
    password: "",
    school_ids: [],
  });
  const [editDraft, setEditDraft] = React.useState<UpdateUserDraft>({
    id: 0,
    username: "",
    display_name: "",
    role: "agent",
    is_active: true,
    school_ids: [],
  });
  const [currentPw, setCurrentPw] = React.useState("");
  const [newPw, setNewPw] = React.useState("");
  const [confirmPw, setConfirmPw] = React.useState("");
  const [localError, setLocalError] = React.useState("");

  function startEdit(user: AppUser) {
    setEditTarget(user);
    setEditDraft({
      id: user.id,
      username: user.username,
      display_name: user.display_name,
      role: user.role,
      is_active: user.is_active,
      school_ids: user.school_ids ?? [],
    });
    setMode("edit");
    setLocalError("");
  }

  function handleCreate(event: React.FormEvent) {
    event.preventDefault();
    if (createDraft.password.length < 6) {
      setLocalError("Password must be at least 6 characters");
      return;
    }
    if (ROLES_NEEDING_SCHOOL_SCOPE.has(createDraft.role) && createDraft.school_ids.length === 0) {
      setLocalError(`A ${createDraft.role.toUpperCase()} must be assigned to at least one school`);
      return;
    }
    setLocalError("");
    onCreateUser(createDraft);
    setCreateDraft({
      username: "",
      display_name: "",
      role: "agent",
      password: "",
      school_ids: [],
    });
    setMode("list");
  }

  function handleEdit(event: React.FormEvent) {
    event.preventDefault();
    setLocalError("");
    onUpdateUser(editDraft);
    setMode("list");
  }

  function handleChangePassword(event: React.FormEvent) {
    event.preventDefault();
    if (newPw !== confirmPw) {
      setLocalError("New passwords do not match");
      return;
    }
    if (newPw.length < 6) {
      setLocalError("Password must be at least 6 characters");
      return;
    }
    setLocalError("");
    onChangePassword(currentPw, newPw);
    setCurrentPw("");
    setNewPw("");
    setConfirmPw("");
    setMode("list");
  }

  if (mode === "create") {
    return (
      <div className="modal-backdrop" role="presentation">
        <section className="ticket-modal" aria-label="Create user">
          <header>
            <h2>Add User</h2>
            <button type="button" onClick={() => setMode("list")}>Cancel</button>
          </header>
          <form onSubmit={handleCreate} className="form-stack">
            {localError ? <div className="error-banner">{localError}</div> : null}
            <label>Username <input value={createDraft.username} onChange={(e) => setCreateDraft({ ...createDraft, username: e.target.value })} /></label>
            <label>Display name <input value={createDraft.display_name} onChange={(e) => setCreateDraft({ ...createDraft, display_name: e.target.value })} /></label>
            <label>Role
              <select value={createDraft.role} onChange={(e) => setCreateDraft({ ...createDraft, role: e.target.value })}>
                {ROLE_OPTIONS.map((r) => (
                  <option key={r.value} value={r.value}>{r.label}</option>
                ))}
              </select>
            </label>
            {ROLES_NEEDING_SCHOOL_SCOPE.has(createDraft.role) ? (
              <UserSchoolPicker
                schools={schools}
                selected={createDraft.school_ids}
                onChange={(ids) => setCreateDraft({ ...createDraft, school_ids: ids })}
              />
            ) : null}
            <label>Password <input type="password" value={createDraft.password} onChange={(e) => setCreateDraft({ ...createDraft, password: e.target.value })} /></label>
            <button type="submit" className="primary-action">Create User</button>
          </form>
        </section>
      </div>
    );
  }

  if (mode === "edit" && editTarget) {
    return (
      <div className="modal-backdrop" role="presentation">
        <section className="ticket-modal" aria-label="Edit user">
          <header>
            <h2>Edit User: {editTarget.display_name}</h2>
            <button type="button" onClick={() => setMode("list")}>Cancel</button>
          </header>
          <form onSubmit={handleEdit} className="form-stack">
            {localError ? <div className="error-banner">{localError}</div> : null}
            <label>Username <input value={editDraft.username} onChange={(e) => setEditDraft({ ...editDraft, username: e.target.value })} /></label>
            <label>Display name <input value={editDraft.display_name} onChange={(e) => setEditDraft({ ...editDraft, display_name: e.target.value })} /></label>
            <label>Role
              <select value={editDraft.role} onChange={(e) => setEditDraft({ ...editDraft, role: e.target.value })}>
                {ROLE_OPTIONS.map((r) => (
                  <option key={r.value} value={r.value}>{r.label}</option>
                ))}
              </select>
            </label>
            {ROLES_NEEDING_SCHOOL_SCOPE.has(editDraft.role) ? (
              <UserSchoolPicker
                schools={schools}
                selected={editDraft.school_ids}
                onChange={(ids) => setEditDraft({ ...editDraft, school_ids: ids })}
              />
            ) : null}
            <label>
              <input type="checkbox" checked={editDraft.is_active} onChange={(e) => setEditDraft({ ...editDraft, is_active: e.target.checked })} />
              {" "}Active account
            </label>
            <button type="submit" className="primary-action">Save Changes</button>
          </form>
        </section>
      </div>
    );
  }

  if (mode === "password") {
    return (
      <div className="modal-backdrop" role="presentation">
        <section className="ticket-modal" aria-label="Change password">
          <header>
            <h2>Change Password</h2>
            <button type="button" onClick={() => setMode("list")}>Cancel</button>
          </header>
          <form onSubmit={handleChangePassword} className="form-stack">
            {localError ? <div className="error-banner">{localError}</div> : null}
            <label>Current password <input type="password" value={currentPw} onChange={(e) => setCurrentPw(e.target.value)} /></label>
            <label>New password <input type="password" value={newPw} onChange={(e) => setNewPw(e.target.value)} /></label>
            <label>Confirm new password <input type="password" value={confirmPw} onChange={(e) => setConfirmPw(e.target.value)} /></label>
            <button type="submit" className="primary-action">Change Password</button>
          </form>
        </section>
      </div>
    );
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal reports-modal" aria-label="User management">
        <header>
          <div>
            <h2>User Management</h2>
            <p>Manage user accounts and roles. Admins have full access. Agents can manage tickets and communications. Viewers have read-only access.</p>
          </div>
          <button type="button" onClick={onClose}>Close</button>
        </header>
        <div className="report-actions">
          <button type="button" className="primary-action" onClick={() => { setLocalError(""); setMode("create"); }}>Add User</button>
          <button type="button" className="secondary-button" onClick={() => { setLocalError(""); setMode("password"); }}>Change My Password</button>
        </div>
        <table className="data-table">
          <thead>
            <tr>
              <th>Display Name</th>
              <th>Username</th>
              <th>Role</th>
              <th>Status</th>
              <th>Last Login</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id} className={!user.is_active ? "row-inactive" : ""}>
                <td>{user.display_name}{user.id === currentUser.id ? " (you)" : ""}</td>
                <td>{user.username}</td>
                <td>{user.role}</td>
                <td>{user.is_active ? "Active" : "Disabled"}</td>
                <td>{user.last_login_at || "Never"}</td>
                <td>
                  <button type="button" className="secondary-button" onClick={() => startEdit(user)}>Edit</button>
                  {user.id !== currentUser.id ? (
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => { if (confirm(`Delete ${user.display_name}?`)) onDeleteUser(user.id); }}
                    >Delete</button>
                  ) : null}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}

function UserSchoolPicker({
  schools,
  selected,
  onChange,
}: {
  schools: School[];
  selected: number[];
  onChange: (ids: number[]) => void;
}) {
  const [query, setQuery] = React.useState("");
  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = schools.filter((s) => !s.is_dropped);
    return q ? list.filter((s) => s.name.toLowerCase().includes(q)) : list;
  }, [schools, query]);
  const selectedSet = React.useMemo(() => new Set(selected), [selected]);

  function toggle(id: number) {
    if (selectedSet.has(id)) {
      onChange(selected.filter((x) => x !== id));
    } else {
      onChange([...selected, id]);
    }
  }

  return (
    <div className="user-school-picker">
      <div className="user-school-picker-header">
        <strong>Assigned schools</strong>
        <span>{selected.length} selected</span>
      </div>
      <input
        type="search"
        placeholder="Search schools..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="user-school-picker-list">
        {filtered.length === 0 ? (
          <p className="empty-state compact">No schools match.</p>
        ) : (
          filtered.map((s) => (
            <label key={s.id} className="user-school-picker-row">
              <input
                type="checkbox"
                checked={selectedSet.has(s.id)}
                onChange={() => toggle(s.id)}
              />
              <span>{s.name}</span>
              {s.region_name ? <small>{s.region_name}</small> : null}
            </label>
          ))
        )}
      </div>
    </div>
  );
}

// ── Bottom navigation (mobile) ────────────────────────────────────────────

type BottomNavProps = {
  activeFilter: Filter;
  currentUserRole: string;
  filterCounts: Record<Filter, number>;
  onFilterChange: (filter: Filter) => void;
  onCreateClick: () => void;
  onMasterDataClick: () => void;
  onMoreClick: () => void;
  showingAdmin: boolean;
};

export function BottomNav({
  activeFilter,
  currentUserRole,
  filterCounts,
  onFilterChange,
  onCreateClick,
  onMasterDataClick,
  onMoreClick,
  showingAdmin,
}: BottomNavProps) {
  const isViewer = currentUserRole === "viewer";
  const inboxCount = filterCounts["Inbox"];
  const mineCount = filterCounts["My Tickets"];
  const InboxIcon = () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="22 12 16 12 14 15 10 15 8 12 2 12" />
      <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z" />
    </svg>
  );
  const MineIcon = () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
      <circle cx="12" cy="7" r="4" />
    </svg>
  );
  const SchoolIcon = () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="7" width="20" height="14" rx="2" />
      <path d="M16 21V5a2 2 0 00-2-2h-4a2 2 0 00-2 2v16" />
    </svg>
  );
  const MoreIcon = () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round">
      <line x1="3" y1="12" x2="21" y2="12" />
      <line x1="3" y1="6" x2="21" y2="6" />
      <line x1="3" y1="18" x2="21" y2="18" />
    </svg>
  );
  return createPortal(
    <nav className="bottom-nav">
      <button
        className={`bottom-nav-tab ${!showingAdmin && activeFilter === "Inbox" ? "active" : ""}`}
        onClick={() => onFilterChange("Inbox")}
      >
        {inboxCount > 0 && <span className="bottom-nav-badge">{inboxCount > 99 ? "99+" : inboxCount}</span>}
        <InboxIcon />
        <span>Inbox</span>
      </button>
      <button
        className={`bottom-nav-tab ${!showingAdmin && activeFilter === "My Tickets" ? "active" : ""}`}
        onClick={() => onFilterChange("My Tickets")}
      >
        {mineCount > 0 && <span className="bottom-nav-badge">{mineCount > 99 ? "99+" : mineCount}</span>}
        <MineIcon />
        <span>Mine</span>
      </button>
      {!isViewer && (
        <button className="bottom-nav-tab bottom-nav-tab-new" onClick={onCreateClick}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      )}
      <button
        className={`bottom-nav-tab ${showingAdmin ? "active" : ""}`}
        onClick={onMasterDataClick}
      >
        <SchoolIcon />
        <span>Schools</span>
      </button>
      <button className="bottom-nav-tab" onClick={onMoreClick}>
        <MoreIcon />
        <span>More</span>
      </button>
    </nav>,
    document.body,
  );
}

// ── Mobile more menu ──────────────────────────────────────────────────────

type MobileMoreMenuProps = {
  currentUserRole: string;
  onClose: () => void;
  onAuditLogClick: () => void;
  onCommunicationOpsClick: () => void;
  onCsvExportClick: () => void;
  onDirectoryClick: () => void;
  onDroppedSchoolsClick: () => void;
  onEscalationPolicyClick: () => void;
  onProgramDashboardClick: () => void;
  onRegionLogClick: () => void;
  onReportsClick: () => void;
  onRoutingRulesClick: () => void;
  onSlaSettingsClick: () => void;
  onTemplatesClick: () => void;
  onUsersClick: () => void;
  onFacultyAssignmentsClick: () => void;
  onSubjectsClick: () => void;
  onTimetableClick: () => void;
  onLogout: () => void;
};

export function MobileMoreMenu({
  currentUserRole,
  onClose,
  onAuditLogClick,
  onCommunicationOpsClick,
  onCsvExportClick,
  onDirectoryClick,
  onDroppedSchoolsClick,
  onEscalationPolicyClick,
  onProgramDashboardClick,
  onRegionLogClick,
  onReportsClick,
  onRoutingRulesClick,
  onSlaSettingsClick,
  onTemplatesClick,
  onUsersClick,
  onFacultyAssignmentsClick,
  onSubjectsClick,
  onTimetableClick,
  onLogout,
}: MobileMoreMenuProps) {
  const isAdmin = currentUserRole === "admin";
  const isAom = currentUserRole === "aom";
  const isViewer = currentUserRole === "viewer";
  function handle(fn: () => void) {
    fn();
    onClose();
  }
  return (
    <>
      <div className="mobile-more-backdrop" onClick={onClose} />
      <div className="mobile-more-sheet">
        <div className="mobile-more-handle" />
        <div className="mobile-more-section">Views</div>
        <button className="mobile-more-item" onClick={() => handle(onProgramDashboardClick)}>Program Dashboard</button>
        <button className="mobile-more-item" onClick={() => handle(onReportsClick)}>Reports</button>
        <button className="mobile-more-item" onClick={() => handle(onDirectoryClick)}>Directory</button>
        {!isViewer && (
          <button className="mobile-more-item" onClick={() => handle(onCommunicationOpsClick)}>Communication Ops</button>
        )}
        <button className="mobile-more-item" onClick={() => handle(onDroppedSchoolsClick)}>Dropped Schools</button>
        <button className="mobile-more-item" onClick={() => handle(onRegionLogClick)}>Region Log</button>
        <button className="mobile-more-item" onClick={() => handle(onAuditLogClick)}>Audit Log</button>
        <button className="mobile-more-item" onClick={() => handle(onCsvExportClick)}>Export CSV</button>
        {(isAdmin || isAom) && (
          <>
            <div className="mobile-more-section">Faculty</div>
            <button className="mobile-more-item" onClick={() => handle(onFacultyAssignmentsClick)}>Faculty Assignments</button>
            <button className="mobile-more-item" onClick={() => handle(onSubjectsClick)}>Subjects</button>
            <button className="mobile-more-item" onClick={() => handle(onTimetableClick)}>Timetable</button>
          </>
        )}
        {isAdmin && (
          <>
            <div className="mobile-more-section">Settings</div>
            <button className="mobile-more-item" onClick={() => handle(onRoutingRulesClick)}>Routing Rules</button>
            <button className="mobile-more-item" onClick={() => handle(onEscalationPolicyClick)}>Escalation Policy</button>
            <button className="mobile-more-item" onClick={() => handle(onSlaSettingsClick)}>SLA Settings</button>
            <button className="mobile-more-item" onClick={() => handle(onTemplatesClick)}>Templates</button>
            <button className="mobile-more-item" onClick={() => handle(onUsersClick)}>Users</button>
          </>
        )}
        <div className="mobile-more-section">Account</div>
        <button className="mobile-more-item mobile-more-item-signout" onClick={() => handle(onLogout)}>Sign out</button>
      </div>
    </>
  );
}

type SidebarProps = {
  activeFilter: Filter;
  currentUserRole: string;
  filterCounts: Record<Filter, number>;
  onFilterChange: (filter: Filter) => void;
  onAuditLogClick: () => void;
  onBackupClick: () => void;
  onCommunicationOpsClick: () => void;
  onCsvExportClick: () => void;
  onEscalationPolicyClick: () => void;
  onDirectoryClick: () => void;
  onDroppedSchoolsClick: () => void;
  onMasterDataClick: () => void;
  onProgramDashboardClick: () => void;
  onRegionLogClick: () => void;
  onReportsClick: () => void;
  onRoutingRulesClick: () => void;
  onSlaSettingsClick: () => void;
  onTemplatesClick: () => void;
  onSyncClick: () => void;
  onUsersClick: () => void;
  onFacultyAssignmentsClick: () => void;
  onSubjectsClick: () => void;
  onTimetableClick: () => void;
};

export function Sidebar({
  activeFilter,
  currentUserRole,
  filterCounts,
  onFilterChange,
  onAuditLogClick,
  onBackupClick,
  onCommunicationOpsClick,
  onCsvExportClick,
  onEscalationPolicyClick,
  onDirectoryClick,
  onDroppedSchoolsClick,
  onMasterDataClick,
  onProgramDashboardClick,
  onRegionLogClick,
  onReportsClick,
  onRoutingRulesClick,
  onSlaSettingsClick,
  onTemplatesClick,
  onSyncClick,
  onUsersClick,
  onFacultyAssignmentsClick,
  onSubjectsClick,
  onTimetableClick,
}: SidebarProps) {
  const isAdmin = currentUserRole === "admin";
  const isAom = currentUserRole === "aom";
  const isViewer = currentUserRole === "viewer";
  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="brand-mark">S</span>
        <div>
          <strong>SAATHI</strong>
          <small>School integrated program</small>
        </div>
      </div>
      <nav aria-label="Ticket views" className="sidebar-section">
        <span className="sidebar-heading">Views</span>
        <div className="nav-list">
          {filters.map((filter) => (
            <button
              className={`nav-item ${activeFilter === filter ? "active" : ""}`}
              key={filter}
              onClick={() => onFilterChange(filter)}
            >
              <span>{filter}</span>
              {filterCounts[filter] > 0 ? (
                <span className={`filter-badge${filter === "Pending SLA" || filter === "Escalated" ? " filter-badge-urgent" : ""}`}>
                  {filterCounts[filter]}
                </span>
              ) : null}
            </button>
          ))}
        </div>
      </nav>
      <nav aria-label="Administration" className="sidebar-section">
        <span className="sidebar-heading">Admin</span>
        <div className="nav-list">
          <button className="nav-item" onClick={onMasterDataClick}>
            Master Data
          </button>
          <button className="nav-item" onClick={onProgramDashboardClick}>
            Program Dashboard
          </button>
          <button className="nav-item" onClick={onReportsClick}>
            Reports
          </button>
          {!isViewer ? (
            <button className="nav-item" onClick={onCommunicationOpsClick}>
              Communication Ops
            </button>
          ) : null}
          <button className="nav-item" onClick={onDirectoryClick}>
            Directory
          </button>
          <button className="nav-item" onClick={onDroppedSchoolsClick}>
            Dropped Schools
          </button>
          <button className="nav-item" onClick={onRegionLogClick}>
            Region Log
          </button>
          <button className="nav-item" onClick={onAuditLogClick}>
            Audit Log
          </button>
          {isAdmin ? (
            <>
              <button className="nav-item" onClick={onRoutingRulesClick}>
                Routing
              </button>
              <button className="nav-item" onClick={onEscalationPolicyClick}>
                Escalation
              </button>
              <button className="nav-item" onClick={onSlaSettingsClick}>
                SLA Settings
              </button>
              <button className="nav-item" onClick={onTemplatesClick}>
                Templates
              </button>
              <button className="nav-item" onClick={onSyncClick}>
                Daily Sync
              </button>
              <button className="nav-item" onClick={onCsvExportClick}>
                Export CSV
              </button>
              <button className="nav-item" onClick={onBackupClick}>
                Backup
              </button>
              <button className="nav-item" onClick={onUsersClick}>
                Users
              </button>
            </>
          ) : null}
          {(isAdmin || isAom) ? (
            <>
              <button className="nav-item" onClick={onFacultyAssignmentsClick}>
                Faculty Assignments
              </button>
              <button className="nav-item" onClick={onSubjectsClick}>
                Subjects
              </button>
              <button className="nav-item" onClick={onTimetableClick}>
                Timetable
              </button>
            </>
          ) : null}
        </div>
      </nav>
    </aside>
  );
}

type SyncPanelProps = {
  snapshot: SyncSnapshotInfo | null;
  pendingSnapshot: SyncSnapshotInfo | null;
  onClose: () => void;
  onExport: () => void;
  onImport: () => void;
  onConfirmImport: () => void;
  onCancelImport: () => void;
};

export function SyncPanel({
  snapshot,
  pendingSnapshot,
  onClose,
  onExport,
  onImport,
  onConfirmImport,
  onCancelImport,
}: SyncPanelProps) {
  return (
    <section className="ticket-modal sync-modal" aria-label="Daily database sync">
      <header>
        <div>
          <h2>Daily Sync</h2>
          <p>Share one database snapshot from the primary machine, then import it on member desktops.</p>
        </div>
        <button type="button" className="ghost-button" onClick={onClose}>
          Close
        </button>
      </header>

      {pendingSnapshot ? (
        <div className="sync-confirm-card">
          <h3>Confirm Import</h3>
          <p>
            This will replace your local database with the snapshot below.
            A backup of the current database will be saved automatically before importing.
          </p>
          <dl className="sync-summary">
            <div>
              <dt>Snapshot</dt>
              <dd className="path-truncate">{pendingSnapshot.path}</dd>
            </div>
            <div><dt>Tickets</dt><dd>{pendingSnapshot.ticket_count}</dd></div>
            <div><dt>Comments</dt><dd>{pendingSnapshot.comment_count}</dd></div>
            <div><dt>History entries</dt><dd>{pendingSnapshot.history_count}</dd></div>
            <div><dt>Attachments</dt><dd>{pendingSnapshot.attachment_count}</dd></div>
            <div><dt>Latest ticket update</dt><dd>{formatField(pendingSnapshot.latest_ticket_update) || "—"}</dd></div>
            <div><dt>Size</dt><dd>{formatBytes(pendingSnapshot.size_bytes)}</dd></div>
          </dl>
          <div className="sync-confirm-actions">
            <button type="button" className="primary-action" onClick={onConfirmImport}>
              Yes, import and replace
            </button>
            <button type="button" onClick={onCancelImport}>
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <>
          <div className="sync-actions">
            <button type="button" className="primary-action" onClick={onExport}>
              Export Snapshot
            </button>
            <button type="button" className="secondary-button" onClick={onImport}>
              Import Snapshot
            </button>
          </div>

          <div className="sync-guidance">
            <strong>Recommended daily routine</strong>
            <p>
              Choose one primary desktop for edits during the local phase. At day end, export a
              snapshot and place it in a shared Drive, OneDrive, or network folder. Other desktops
              import that snapshot before starting work.
            </p>
          </div>

          {snapshot ? (
            <>
              <h3 className="sync-section-heading">Last completed operation</h3>
              <dl className="sync-summary">
                <div>
                  <dt>Path</dt>
                  <dd className="path-truncate">{snapshot.path}</dd>
                </div>
                <div><dt>Tickets</dt><dd>{snapshot.ticket_count}</dd></div>
                <div><dt>Comments</dt><dd>{snapshot.comment_count}</dd></div>
                <div><dt>History entries</dt><dd>{snapshot.history_count}</dd></div>
                <div><dt>Attachments</dt><dd>{snapshot.attachment_count}</dd></div>
                <div><dt>Latest ticket update</dt><dd>{formatField(snapshot.latest_ticket_update) || "—"}</dd></div>
                <div><dt>Size</dt><dd>{formatBytes(snapshot.size_bytes)}</dd></div>
              </dl>
            </>
          ) : (
            <p className="empty-state compact">No sync snapshot selected yet. Export to share this database or import one from another machine.</p>
          )}
        </>
      )}
    </section>
  );
}

type TopbarProps = {
  search: string;
  currentUser: CurrentUser | null;
  latestUpdate: string;
  onSearchChange: (value: string) => void;
  onCreateClick: () => void;
  onLogout: () => void;
  mobileBackLabel?: string;
  onMobileBack?: () => void;
};

export function Topbar({
  search,
  currentUser,
  latestUpdate,
  onSearchChange,
  onCreateClick,
  onLogout,
  mobileBackLabel,
  onMobileBack,
}: TopbarProps) {
  const isViewer = currentUser?.role === "viewer";
  return (
    <header className="topbar">
      {onMobileBack ? (
        <button className="mobile-back-btn" onClick={onMobileBack}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6" />
          </svg>
          {mobileBackLabel ?? "Back"}
        </button>
      ) : (
        <>
        <span className="topbar-brand">SAATHI</span>
        <div className="search-strip">
          <input
            aria-label="Search tickets"
            placeholder="Search tickets, requesters, assignees"
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
          />
          {!isViewer ? (
            <button className="primary-action" onClick={onCreateClick}>
              New Ticket
            </button>
          ) : null}
        </div>
        </>
      )}
      {currentUser ? (
        <div className="topbar-user">
          {!onMobileBack && latestUpdate ? (
            <span className="data-freshness" title={`Latest ticket update: ${latestUpdate}`}>
              As of {latestUpdate.slice(0, 10)}
            </span>
          ) : null}
          {!onMobileBack && (
            <span className="user-badge">
              {currentUser.display_name}
              <em>{currentUser.role}</em>
            </span>
          )}
          {!onMobileBack && (
            <button type="button" className="ghost-button" onClick={onLogout}>
              Sign out
            </button>
          )}
        </div>
      ) : null}
    </header>
  );
}

type MasterDataPanelProps = {
  classPlans: SchoolClassPlan[];
  lectureModels: LectureModel[];
  regions: Region[];
  schools: School[];
  sipImportPreview: { sourcePath: string; preview: SipMasterImportPreview } | null;
  students: Student[];
  onClose: () => void;
  onCancelSipMasterImport: () => void;
  onConfirmSipMasterImport: (conflictAction: "skip_existing" | "update_existing") => void;
  onCreateSchool: (input: SchoolProfileDraft) => void;
  onDeleteSchool: (id: number) => void;
  onDropSchool: (id: number, reason: string) => void;
  onExportSipMaster: () => void;
  onImportSipMaster: () => void;
  onCreateLectureModel: (input: {
    name: string;
    days_per_week: number;
    lectures_per_day: number;
  }) => void;
  onSaveRegion: (input: {
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
  }) => void;
  onDeleteRegion: (id: number) => void;
  onRemapAndDeleteRegion: (
    regionId: number,
    mappings: Array<{
      school_id: number;
      target_region_id?: number;
      new_region_name?: string;
    }>,
  ) => void;
  onImportSchools: () => void;
  onSaveClassPlan: (input: {
    school_id: number;
    grade_level: string;
    track: string;
    lecture_model_id: number;
    batch_pattern: string;
    aop_admissions: number;
    registrations: number;
    actual_admissions: number;
  }) => void;
  onCreateStudent: (input: {
    school_id: number;
    name: string;
    grade_level: string;
    program_track: string;
  }) => void;
};

type MasterDataTab = "schools" | "regions" | "program" | "students" | "imports";

const emptyRegionDraft: {
  id?: number;
  name: string;
  regional_academic_head_name: string;
  regional_academic_head_mobile: string;
  regional_academic_head_email: string;
  regional_business_head_name: string;
  regional_business_head_mobile: string;
  regional_business_head_email: string;
} = {
  id: undefined,
  name: "",
  regional_academic_head_name: "",
  regional_academic_head_mobile: "",
  regional_academic_head_email: "",
  regional_business_head_name: "",
  regional_business_head_mobile: "",
  regional_business_head_email: "",
};

export function MasterDataPanel({
  classPlans,
  lectureModels,
  regions,
  schools,
  sipImportPreview,
  students,
  onClose,
  onCancelSipMasterImport,
  onConfirmSipMasterImport,
  onCreateLectureModel,
  onCreateSchool,
  onDeleteSchool,
  onDropSchool,
  onExportSipMaster,
  onImportSipMaster,
  onImportSchools,
  onSaveRegion,
  onDeleteRegion,
  onRemapAndDeleteRegion,
  onSaveClassPlan,
  onCreateStudent,
}: MasterDataPanelProps) {
  const [schoolDraft, setSchoolDraft] = React.useState<SchoolProfileDraft>(emptySchoolProfileDraft);
  const [activeMasterTab, setActiveMasterTab] = React.useState<MasterDataTab>("schools");
  const [schoolSearch, setSchoolSearch] = React.useState("");
  const [selectedSchoolId, setSelectedSchoolId] = React.useState<number | null>(schools[0]?.id ?? null);
  const [regionDraft, setRegionDraft] = React.useState(emptyRegionDraft);
  const [pendingRegionDelete, setPendingRegionDelete] = React.useState<{
    region: Region;
    mappings: Record<number, { target_region_id: string; new_region_name: string }>;
  } | null>(null);
  const [pendingSchoolDrop, setPendingSchoolDrop] = React.useState<{
    school: School;
    reason: string;
  } | null>(null);
  const [pendingSchoolDelete, setPendingSchoolDelete] = React.useState<School | null>(null);
  const [studentDraft, setStudentDraft] = React.useState({
    school_id: schools[0]?.id ?? 0,
    name: "",
    grade_level: gradeLevels[0],
    program_track: programTracks[0],
  });
  const [lectureModelDraft, setLectureModelDraft] = React.useState({
    name: "",
    days_per_week: 3,
    lectures_per_day: 3,
  });
  const [classPlanDraft, setClassPlanDraft] = React.useState({
    school_id: schools[0]?.id ?? 0,
    grade_level: gradeLevels[0],
    track: "",
    lecture_model_id: lectureModels[0]?.id ?? 0,
    batch_pattern: batchPatterns[0],
    aop_admissions: 0,
    registrations: 0,
    actual_admissions: 0,
  });

  React.useEffect(() => {
    setStudentDraft((current) => ({
      ...current,
      school_id: current.school_id || schools[0]?.id || 0,
    }));
    setClassPlanDraft((current) => ({
      ...current,
      school_id: current.school_id || schools[0]?.id || 0,
    }));
    setSelectedSchoolId((current) =>
      current && schools.some((school) => school.id === current) ? current : schools[0]?.id ?? null,
    );
  }, [schools]);

  React.useEffect(() => {
    setClassPlanDraft((current) => ({
      ...current,
      lecture_model_id: current.lecture_model_id || lectureModels[0]?.id || 0,
    }));
  }, [lectureModels]);

  React.useEffect(() => {
    if (sipImportPreview) {
      setActiveMasterTab("imports");
    }
  }, [sipImportPreview]);

  const startRegionDelete = (region: Region) => {
    const mappedSchools = schools.filter((school) => school.region_id === region.id);
    if (mappedSchools.length === 0) {
      onDeleteRegion(region.id);
      return;
    }

    const fallbackRegion = regions.find((item) => item.id !== region.id);
    setPendingRegionDelete({
      region,
      mappings: Object.fromEntries(
        mappedSchools.map((school) => [
          school.id,
          {
            target_region_id: fallbackRegion ? String(fallbackRegion.id) : "__new__",
            new_region_name: "",
          },
        ]),
      ),
    });
  };
  const filteredSchools = schools.filter((school) => {
    const query = schoolSearch.trim().toLocaleLowerCase();
    if (!query) {
      return true;
    }

    return [
      school.name,
      school.region_name,
      school.program_model,
      school.distance_classification,
      school.sip_academic_owner_name,
      school.center_head_name,
      school.aom_name,
    ]
      .join(" ")
      .toLocaleLowerCase()
      .includes(query);
  });
  const selectedSchool = schools.find((school) => school.id === selectedSchoolId) ?? null;
  const selectedSchoolClassPlans = selectedSchool
    ? classPlans.filter((plan) => plan.school_id === selectedSchool.id)
    : [];
  const selectedSchoolStudents = selectedSchool
    ? students.filter((student) => student.school_id === selectedSchool.id)
    : [];

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal master-data-modal" aria-label="School and student master data">
        <header>
          <div>
            <h2>Master Data</h2>
            <p>Manage school profiles, regions, class plans, students, and SIP master files.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="master-data-list">
          <strong>{schools.length} active schools</strong>
          <strong>{regions.length} regions</strong>
          <strong>{classPlans.length} class plans</strong>
          <strong>{students.length} students</strong>
        </div>

        <div className="master-data-tabs" role="tablist" aria-label="Master data sections">
          {[
            ["schools", "Schools"],
            ["regions", "Regions"],
            ["program", "Class Plans"],
            ["students", "Students"],
            ["imports", "Import / Export"],
          ].map(([tab, label]) => (
            <button
              aria-selected={activeMasterTab === tab}
              className={activeMasterTab === tab ? "active" : ""}
              key={tab}
              onClick={() => setActiveMasterTab(tab as MasterDataTab)}
              role="tab"
              type="button"
            >
              {label}
            </button>
          ))}
        </div>

        {activeMasterTab === "imports" ? (
          <div className="master-data-section">
            <header>
              <div>
                <h3>Import / Export</h3>
                <p>Move schoolwise SIP master data in and out of the app.</p>
              </div>
            </header>
            <div className="master-data-actions">
          <button type="button" className="secondary-button" onClick={onImportSchools}>
            Import Schools CSV
          </button>
          <button type="button" className="secondary-button" onClick={onImportSipMaster}>
            Import SIP Master
          </button>
          <button type="button" className="secondary-button" onClick={onExportSipMaster}>
            Export SIP Master Excel
          </button>
          <small>
            CSV headers can use names like school_name, model, distance_classification,
            sip_academic_owner_name, center_head_email, principal_mobile, and school_spoc_name.
          </small>
        </div>
        {sipImportPreview ? (
          <section className="import-review-panel" aria-label="SIP master import review">
            <header>
              <div>
                <strong>Review SIP master import</strong>
                <small>{sipImportPreview.sourcePath}</small>
              </div>
              <button type="button" className="secondary-button" onClick={onCancelSipMasterImport}>
                Cancel
              </button>
            </header>
            <div className="import-review-metrics">
              <span>{sipImportPreview.preview.total_rows} rows</span>
              <span>{sipImportPreview.preview.new_school_count} new schools</span>
              <span>{sipImportPreview.preview.existing_school_count} existing schools</span>
              <span>{sipImportPreview.preview.skipped_row_count} skipped rows</span>
            </div>
            {sipImportPreview.preview.existing_schools.length > 0 ? (
              <div className="import-conflict-list">
                <strong>Existing schools detected</strong>
                <span>{sipImportPreview.preview.existing_schools.slice(0, 10).join(", ")}</span>
              </div>
            ) : null}
            <div className="actions">
              <button type="button" onClick={() => onConfirmSipMasterImport("skip_existing")}>
                Import New Only
              </button>
              <button
                type="button"
                className="primary-action"
                onClick={() => onConfirmSipMasterImport("update_existing")}
              >
                Update Existing And Import
              </button>
            </div>
          </section>
        ) : null}
          </div>
        ) : null}

        {activeMasterTab === "regions" ? (
          <div className="master-data-section">
        <form
          className="region-form"
          onSubmit={(event) => {
            event.preventDefault();
            onSaveRegion(regionDraft);
            setRegionDraft(emptyRegionDraft);
          }}
        >
          <fieldset>
            <legend>Region configuration</legend>
            <label>
              Region name
              <input
                required
                value={regionDraft.name}
                onChange={(event) =>
                  setRegionDraft((current) => ({ ...current, name: event.target.value }))
                }
              />
            </label>
            <label>
              Regional Academic Head
              <input
                value={regionDraft.regional_academic_head_name}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_academic_head_name: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              RAH mobile
              <input
                type="tel"
                value={regionDraft.regional_academic_head_mobile}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_academic_head_mobile: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              RAH email
              <input
                type="email"
                value={regionDraft.regional_academic_head_email}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_academic_head_email: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              Regional Business Head
              <input
                value={regionDraft.regional_business_head_name}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_business_head_name: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              RBH mobile
              <input
                type="tel"
                value={regionDraft.regional_business_head_mobile}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_business_head_mobile: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              RBH email
              <input
                type="email"
                value={regionDraft.regional_business_head_email}
                onChange={(event) =>
                  setRegionDraft((current) => ({
                    ...current,
                    regional_business_head_email: event.target.value,
                  }))
                }
              />
            </label>
            <button type="submit">{regionDraft.id ? "Update Region" : "Save Region"}</button>
            {regionDraft.id ? (
              <button
                type="button"
                className="secondary-button"
                onClick={() => setRegionDraft(emptyRegionDraft)}
              >
                Cancel Edit
              </button>
            ) : null}
          </fieldset>
        </form>

        <div className="region-list">
          {regions.map((region) => (
            <article key={region.id}>
              <div>
                <strong>{region.name}</strong>
                <small>RAH: {formatField(region.regional_academic_head_name)}</small>
                <small>RBH: {formatField(region.regional_business_head_name)}</small>
              </div>
              <div className="region-actions">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() =>
                    setRegionDraft({
                      id: region.id,
                      name: region.name,
                      regional_academic_head_name: region.regional_academic_head_name,
                      regional_academic_head_mobile: region.regional_academic_head_mobile,
                      regional_academic_head_email: region.regional_academic_head_email,
                      regional_business_head_name: region.regional_business_head_name,
                      regional_business_head_mobile: region.regional_business_head_mobile,
                      regional_business_head_email: region.regional_business_head_email,
                    })
                  }
                >
                  Edit
                </button>
                <button type="button" onClick={() => startRegionDelete(region)}>
                  Delete
                </button>
              </div>
            </article>
          ))}
        </div>

        {pendingRegionDelete ? (
          <form
            className="region-remap-panel"
            onSubmit={(event) => {
              event.preventDefault();
              onRemapAndDeleteRegion(
                pendingRegionDelete.region.id,
                schools
                  .filter((school) => school.region_id === pendingRegionDelete.region.id)
                  .map((school) => {
                    const mapping = pendingRegionDelete.mappings[school.id];
                    return {
                      school_id: school.id,
                      target_region_id:
                        mapping.target_region_id === "__new__"
                          ? undefined
                          : Number(mapping.target_region_id),
                      new_region_name:
                        mapping.target_region_id === "__new__"
                          ? mapping.new_region_name
                          : undefined,
                    };
                  }),
              );
              setPendingRegionDelete(null);
            }}
          >
            <header>
              <div>
                <strong>Move schools from {pendingRegionDelete.region.name}</strong>
                <small>Select an existing region or create a new one for each mapped school.</small>
              </div>
              <button
                type="button"
                className="secondary-button"
                onClick={() => setPendingRegionDelete(null)}
              >
                Cancel
              </button>
            </header>
            {schools
              .filter((school) => school.region_id === pendingRegionDelete.region.id)
              .map((school) => {
                const mapping = pendingRegionDelete.mappings[school.id];
                return (
                  <div className="region-remap-row" key={school.id}>
                    <strong>{school.name}</strong>
                    <label>
                      New mapping
                      <select
                        value={mapping.target_region_id}
                        onChange={(event) =>
                          setPendingRegionDelete((current) =>
                            current
                              ? {
                                  ...current,
                                  mappings: {
                                    ...current.mappings,
                                    [school.id]: {
                                      ...current.mappings[school.id],
                                      target_region_id: event.target.value,
                                    },
                                  },
                                }
                              : current,
                          )
                        }
                      >
                        {regions
                          .filter((region) => region.id !== pendingRegionDelete.region.id)
                          .map((region) => (
                            <option key={region.id} value={region.id}>
                              {region.name}
                            </option>
                          ))}
                        <option value="__new__">Create new region</option>
                      </select>
                    </label>
                    {mapping.target_region_id === "__new__" ? (
                      <label>
                        New region name
                        <input
                          required
                          value={mapping.new_region_name}
                          onChange={(event) =>
                            setPendingRegionDelete((current) =>
                              current
                                ? {
                                    ...current,
                                    mappings: {
                                      ...current.mappings,
                                      [school.id]: {
                                        ...current.mappings[school.id],
                                        new_region_name: event.target.value,
                                      },
                                    },
                                  }
                                : current,
                            )
                          }
                        />
                      </label>
                    ) : null}
                  </div>
                );
              })}
            <button type="submit">Move Schools And Delete Region</button>
          </form>
        ) : null}
          </div>
        ) : null}

        {activeMasterTab === "schools" ? (
          <div className="master-data-section">
            <header>
              <div>
                <h3>Schools</h3>
                <p>Create school profiles and manage active school master records.</p>
              </div>
            </header>
        <form
          className="school-profile-form"
          onSubmit={(event) => {
            event.preventDefault();
            onCreateSchool(schoolDraft);
            setSchoolDraft(emptySchoolProfileDraft);
          }}
        >
          <fieldset>
            <legend>School profile</legend>
            <label>
              School name
              <input
                required
                value={schoolDraft.name}
                onChange={(event) =>
                  setSchoolDraft((current) => ({ ...current, name: event.target.value }))
                }
              />
            </label>
            <label>
              Region
              <select
                value={schoolDraft.region_id ?? ""}
                onChange={(event) =>
                  setSchoolDraft((current) => ({
                    ...current,
                    region_id: event.target.value ? Number(event.target.value) : null,
                    region_name:
                      regions.find((region) => region.id === Number(event.target.value))?.name ??
                      "",
                  }))
                }
              >
                <option value="">Select region</option>
                {regions.map((region) => (
                  <option key={region.id} value={region.id}>
                    {region.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Model
              <select
                value={schoolDraft.program_model}
                onChange={(event) =>
                  setSchoolDraft((current) => ({ ...current, program_model: event.target.value }))
                }
              >
                <option value="">Select model</option>
                <option>Aspire</option>
                <option>Minimum Guarantee</option>
              </select>
            </label>
            <label>
              Distance classification
              <select
                value={schoolDraft.distance_classification}
                onChange={(event) =>
                  setSchoolDraft((current) => ({
                    ...current,
                    distance_classification: event.target.value,
                    sip_academic_owner_role:
                      event.target.value === "Remote"
                        ? "SIP Academic Head"
                        : event.target.value === "Near Proximity"
                          ? "SIP Academic Lead"
                          : current.sip_academic_owner_role,
                  }))
                }
              >
                <option value="">Select classification</option>
                <option>Remote</option>
                <option>Near Proximity</option>
              </select>
            </label>
            <label>
              Mapped VP Center
              <input
                value={schoolDraft.mapped_vp_center}
                placeholder="e.g., Mumbai West VP"
                onChange={(event) =>
                  setSchoolDraft((current) => ({
                    ...current,
                    mapped_vp_center: event.target.value,
                  }))
                }
              />
            </label>
            <label>
              SIP Academic role
              <input
                value={schoolDraft.sip_academic_owner_role}
                onChange={(event) =>
                  setSchoolDraft((current) => ({
                    ...current,
                    sip_academic_owner_role: event.target.value,
                  }))
                }
              />
            </label>
          </fieldset>

          {schoolContactGroups.map((group) => (
            <fieldset key={group.title}>
              <legend>{group.title}</legend>
              {group.fields.map((field) => (
                <label key={field.key}>
                  {field.label}
                  <input
                    type={field.type}
                    value={String(schoolDraft[field.key] ?? "")}
                    onChange={(event) =>
                      setSchoolDraft((current) => ({
                        ...current,
                        [field.key]: event.target.value,
                      }))
                    }
                  />
                </label>
              ))}
            </fieldset>
          ))}

          <button type="submit">Save School Profile</button>
        </form>
          </div>
        ) : null}

        {activeMasterTab === "program" ? (
          <div className="master-data-section">
            <header>
              <div>
                <h3>Class Plans</h3>
                <p>Configure lecture models and classwise admissions visibility.</p>
              </div>
            </header>
        <form
          className="lecture-model-form"
          onSubmit={(event) => {
            event.preventDefault();
            onCreateLectureModel(lectureModelDraft);
            setLectureModelDraft({ name: "", days_per_week: 3, lectures_per_day: 3 });
          }}
        >
          <label>
            New lecture model
            <input
              placeholder="Example: 6x2"
              required
              value={lectureModelDraft.name}
              onChange={(event) =>
                setLectureModelDraft((current) => ({ ...current, name: event.target.value }))
              }
            />
          </label>
          <label>
            Days per week
            <input
              min={1}
              type="number"
              value={lectureModelDraft.days_per_week}
              onChange={(event) =>
                setLectureModelDraft((current) => ({
                  ...current,
                  days_per_week: Number(event.target.value),
                }))
              }
            />
          </label>
          <label>
            Lectures per day
            <input
              min={1}
              type="number"
              value={lectureModelDraft.lectures_per_day}
              onChange={(event) =>
                setLectureModelDraft((current) => ({
                  ...current,
                  lectures_per_day: Number(event.target.value),
                }))
              }
            />
          </label>
          <button type="submit">Save Lecture Model</button>
        </form>

        <form
          className="class-plan-form"
          onSubmit={(event) => {
            event.preventDefault();
            onSaveClassPlan(classPlanDraft);
          }}
        >
          <label>
            School
            <select
              value={classPlanDraft.school_id}
              onChange={(event) =>
                setClassPlanDraft((current) => ({
                  ...current,
                  school_id: Number(event.target.value),
                }))
              }
            >
              {schools.map((school) => (
                <option key={school.id} value={school.id}>
                  {school.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Class
            <select
              value={classPlanDraft.grade_level}
              onChange={(event) => {
                const next = event.target.value;
                setClassPlanDraft((current) => ({
                  ...current,
                  grade_level: next,
                  track: trackEligibleGrades.has(next)
                    ? (current.track || academicTracks[0])
                    : "",
                }));
              }}
            >
              {gradeLevels.map((grade) => (
                <option key={grade}>{grade}</option>
              ))}
            </select>
          </label>
          {trackEligibleGrades.has(classPlanDraft.grade_level) ? (
            <label>
              Track
              <select
                value={classPlanDraft.track || academicTracks[0]}
                onChange={(event) =>
                  setClassPlanDraft((current) => ({ ...current, track: event.target.value }))
                }
              >
                {academicTracks.map((t) => (
                  <option key={t}>{t}</option>
                ))}
              </select>
            </label>
          ) : null}
          <label>
            Lecture model
            <select
              value={classPlanDraft.lecture_model_id}
              onChange={(event) =>
                setClassPlanDraft((current) => ({
                  ...current,
                  lecture_model_id: Number(event.target.value),
                }))
              }
            >
              {lectureModels.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Batch pattern
            <select
              value={classPlanDraft.batch_pattern}
              onChange={(event) =>
                setClassPlanDraft((current) => ({ ...current, batch_pattern: event.target.value }))
              }
            >
              {batchPatterns.map((pattern) => (
                <option key={pattern}>{pattern}</option>
              ))}
            </select>
          </label>
          <label>
            AOP admissions
            <input
              min={0}
              type="number"
              value={classPlanDraft.aop_admissions}
              onChange={(event) =>
                setClassPlanDraft((current) => ({
                  ...current,
                  aop_admissions: Number(event.target.value),
                }))
              }
            />
          </label>
          <label>
            Registrations
            <input
              min={0}
              type="number"
              value={classPlanDraft.registrations}
              onChange={(event) =>
                setClassPlanDraft((current) => ({
                  ...current,
                  registrations: Number(event.target.value),
                }))
              }
            />
          </label>
          <label>
            Actual admissions
            <input
              min={0}
              type="number"
              value={classPlanDraft.actual_admissions}
              onChange={(event) =>
                setClassPlanDraft((current) => ({
                  ...current,
                  actual_admissions: Number(event.target.value),
                }))
              }
            />
          </label>
          <button type="submit" disabled={!classPlanDraft.school_id || !classPlanDraft.lecture_model_id}>
            Save Class Plan
          </button>
        </form>
          </div>
        ) : null}

        {activeMasterTab === "students" ? (
          <div className="master-data-section">
            <header>
              <div>
                <h3>Students</h3>
                <p>Add student records linked to active schools.</p>
              </div>
            </header>
        <form
          className="master-data-form"
          onSubmit={(event) => {
            event.preventDefault();
            onCreateStudent(studentDraft);
            setStudentDraft((current) => ({ ...current, name: "" }));
          }}
        >
          <label>
            School
            <select
              value={studentDraft.school_id}
              onChange={(event) =>
                setStudentDraft((current) => ({
                  ...current,
                  school_id: Number(event.target.value),
                }))
              }
            >
              {schools.map((school) => (
                <option key={school.id} value={school.id}>
                  {school.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Student name
            <input
              required
              value={studentDraft.name}
              onChange={(event) =>
                setStudentDraft((current) => ({ ...current, name: event.target.value }))
              }
            />
          </label>
          <label>
            Grade
            <select
              value={studentDraft.grade_level}
              onChange={(event) =>
                setStudentDraft((current) => ({ ...current, grade_level: event.target.value }))
              }
            >
              {gradeLevels.map((grade) => (
                <option key={grade}>{grade}</option>
              ))}
            </select>
          </label>
          <label>
            Program
            <select
              value={studentDraft.program_track}
              onChange={(event) =>
                setStudentDraft((current) => ({ ...current, program_track: event.target.value }))
              }
            >
              {programTracks.map((track) => (
                <option key={track}>{track}</option>
              ))}
            </select>
          </label>
          <button type="submit" disabled={!studentDraft.school_id}>
            Add Student
          </button>
        </form>
          </div>
        ) : null}

        {activeMasterTab === "program" ? (
        <div className="class-plan-list">
          {classPlans.slice(0, 8).map((plan) => (
            <article key={plan.id}>
              <strong>
                {plan.school_name} - {plan.grade_level}
                {plan.track ? ` (${plan.track})` : ""}
              </strong>
              <span>
                {plan.lecture_model_name} - {plan.batch_pattern}
              </span>
              <small>
                AOP {plan.aop_admissions} / Reg {plan.registrations} / Actual{" "}
                {plan.actual_admissions} / Gap {plan.admission_gap}
              </small>
              <button
                type="button"
                className="secondary-button"
                onClick={() =>
                  setClassPlanDraft({
                    school_id: plan.school_id,
                    grade_level: plan.grade_level,
                    track: plan.track,
                    lecture_model_id: plan.lecture_model_id,
                    batch_pattern: plan.batch_pattern,
                    aop_admissions: plan.aop_admissions,
                    registrations: plan.registrations,
                    actual_admissions: plan.actual_admissions,
                  })
                }
              >
                Edit Plan
              </button>
            </article>
          ))}
        </div>
        ) : null}

        {activeMasterTab === "schools" ? (
          <>
            <div className="master-data-search">
              <label>
                Search schools
                <input
                  value={schoolSearch}
                  onChange={(event) => setSchoolSearch(event.target.value)}
                  placeholder="Search by school, region, model, or owner"
                />
              </label>
            </div>
            {pendingSchoolDrop ? (
              <div className="modal-backdrop" role="presentation" onClick={() => setPendingSchoolDrop(null)}>
                <form
                  className="modal drop-school-modal"
                  onClick={(e) => e.stopPropagation()}
                  onSubmit={(event) => {
                    event.preventDefault();
                    onDropSchool(pendingSchoolDrop.school.id, pendingSchoolDrop.reason.trim());
                    setPendingSchoolDrop(null);
                  }}
                >
                  <header>
                    <div>
                      <h3>Drop {pendingSchoolDrop.school.name}?</h3>
                      <p>The school will leave active views but remain available in Dropped Schools.</p>
                    </div>
                  </header>
                  <label>
                    Drop reason
                    <textarea
                      required
                      autoFocus
                      value={pendingSchoolDrop.reason}
                      onChange={(event) =>
                        setPendingSchoolDrop((current) =>
                          current ? { ...current, reason: event.target.value } : current,
                        )
                      }
                    />
                  </label>
                  <div className="drop-school-modal-actions">
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => setPendingSchoolDrop(null)}
                    >
                      Cancel
                    </button>
                    <button type="submit" className="primary-action danger-action">
                      Confirm Drop
                    </button>
                  </div>
                </form>
              </div>
            ) : null}

            {pendingSchoolDelete ? (
              <div className="modal-backdrop" role="presentation" onClick={() => setPendingSchoolDelete(null)}>
                <form
                  className="modal drop-school-modal"
                  onClick={(e) => e.stopPropagation()}
                  onSubmit={(event) => {
                    event.preventDefault();
                    onDeleteSchool(pendingSchoolDelete.id);
                    setPendingSchoolDelete(null);
                  }}
                >
                  <header>
                    <div>
                      <h3>Delete {pendingSchoolDelete.name}?</h3>
                      <p>
                        Use this only for dummy or wrongly-created schools. Student records, class
                        plans, and region-change logs will be removed. Ticket history is preserved.
                      </p>
                    </div>
                  </header>
                  <div className="drop-school-modal-actions">
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => setPendingSchoolDelete(null)}
                    >
                      Cancel
                    </button>
                    <button type="submit" className="primary-action danger-action">
                      Confirm Delete
                    </button>
                  </div>
                </form>
              </div>
            ) : null}
            {selectedSchool ? (
              <section className="school-detail-panel" aria-label="Selected school profile">
                <header>
                  <div>
                    <h3>{selectedSchool.name}</h3>
                    <p>
                      {formatField(selectedSchool.program_model)} -{" "}
                      {formatField(selectedSchool.distance_classification)} - Region{" "}
                      {formatField(selectedSchool.region_name)}
                    </p>
                  </div>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => setSchoolDraft(selectedSchool)}
                  >
                    Load In Form
                  </button>
                </header>
                <div className="school-detail-grid">
                  <article>
                    <strong>Academic ownership</strong>
                    <span>
                      {formatField(selectedSchool.sip_academic_owner_role)}:{" "}
                      {formatField(selectedSchool.sip_academic_owner_name)}
                    </span>
                    <small><ContactLink kind="tel" value={selectedSchool.sip_academic_owner_mobile} /></small>
                    <small><ContactLink kind="mail" value={selectedSchool.sip_academic_owner_email} /></small>
                  </article>
                  <article>
                    <strong>School leadership</strong>
                    <span>Principal: {formatField(selectedSchool.principal_name)}</span>
                    <span>SPOC: {formatField(selectedSchool.school_spoc_name)}</span>
                    <small>Center Head: {formatField(selectedSchool.center_head_name)}</small>
                  </article>
                  <article>
                    <strong>Business and operations</strong>
                    <span>BH: {formatField(selectedSchool.bh_name)}</span>
                    <span>AOM: {formatField(selectedSchool.aom_name)}</span>
                    <small>
                      Central business: {formatField(selectedSchool.central_business_spoc_name)}
                    </small>
                  </article>
                  <article>
                    <strong>Class plan summary</strong>
                    <span>{selectedSchoolClassPlans.length} configured classes</span>
                    <span>{selectedSchoolStudents.length} student records</span>
                    <small>
                      AOP{" "}
                      {selectedSchoolClassPlans.reduce(
                        (total, plan) => total + plan.aop_admissions,
                        0,
                      )}{" "}
                      / Actual{" "}
                      {selectedSchoolClassPlans.reduce(
                        (total, plan) => total + plan.actual_admissions,
                        0,
                      )}
                    </small>
                  </article>
                </div>
                <div className="class-plan-grid" role="table" aria-label="Class plans by grade">
                  {gradeLevels.flatMap((grade) => {
                    const tracks = trackEligibleGrades.has(grade) ? academicTracks : [""];
                    return tracks.map((track) => {
                      const plan = selectedSchoolClassPlans.find(
                        (item) => item.grade_level === grade && (item.track ?? "") === track,
                      );
                      const label = track ? `${grade} (${track})` : grade;
                      const key = track ? `${grade}|${track}` : grade;
                      return (
                        <div className="class-plan-grid-row" role="row" key={key}>
                          <strong>{label}</strong>
                          <span>{plan?.lecture_model_name ?? "Not configured"}</span>
                          <span>{plan?.batch_pattern ?? "-"}</span>
                          <span>AOP {plan?.aop_admissions ?? 0}</span>
                          <span>Reg {plan?.registrations ?? 0}</span>
                          <span>Actual {plan?.actual_admissions ?? 0}</span>
                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => {
                              setClassPlanDraft({
                                school_id: selectedSchool.id,
                                grade_level: grade,
                                track,
                                lecture_model_id: plan?.lecture_model_id ?? lectureModels[0]?.id ?? 0,
                                batch_pattern: plan?.batch_pattern ?? batchPatterns[0],
                                aop_admissions: plan?.aop_admissions ?? 0,
                                registrations: plan?.registrations ?? 0,
                                actual_admissions: plan?.actual_admissions ?? 0,
                              });
                              setActiveMasterTab("program");
                            }}
                          >
                            Edit
                          </button>
                        </div>
                      );
                    });
                  })}
                </div>
              </section>
            ) : null}
        <div className="school-profile-list">
          {filteredSchools.length === 0 ? <p>No schools match this search.</p> : null}
          {filteredSchools.map((school) => (
            <article
              className={school.id === selectedSchoolId ? "selected-school-card" : ""}
              key={school.id}
            >
              <div className="school-profile-title">
                <strong>{school.name}</strong>
                <div className="school-profile-actions">
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => setSelectedSchoolId(school.id)}
                  >
                    View
                  </button>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => {
                      setPendingSchoolDelete(null);
                      setPendingSchoolDrop({ school, reason: "" });
                    }}
                  >
                    Drop
                  </button>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => {
                      setPendingSchoolDrop(null);
                      setPendingSchoolDelete(school);
                    }}
                  >
                    Delete
                  </button>
                </div>
              </div>
              <span>
                {formatField(school.program_model)} - {formatField(school.distance_classification)}
              </span>
              <small>Region: {formatField(school.region_name)}</small>
              <small>
                {formatField(school.sip_academic_owner_role)}:{" "}
                {formatField(school.sip_academic_owner_name)}
              </small>
              <small>Center Head: {formatField(school.center_head_name)}</small>
              <small>BH: {formatField(school.bh_name)}</small>
              <small>AOM: {formatField(school.aom_name)}</small>
            </article>
          ))}
        </div>
          </>
        ) : null}
      </section>
    </div>
  );
}

type DroppedSchoolsPanelProps = {
  schools: School[];
  onClose: () => void;
  onRestore: (id: number) => void;
};

export function DroppedSchoolsPanel({ schools, onClose, onRestore }: DroppedSchoolsPanelProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Dropped schools">
        <header>
          <div>
            <h2>Dropped Schools</h2>
            <p>Review schools removed from active operations without deleting their history.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="school-profile-list dropped-school-list">
          {schools.length === 0 ? <p>No dropped schools.</p> : null}
          {schools.map((school) => (
            <article key={school.id}>
              <div className="school-profile-title">
                <strong>{school.name}</strong>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => onRestore(school.id)}
                >
                  Restore
                </button>
              </div>
              <span>
                {formatField(school.program_model)} - {formatField(school.distance_classification)}
              </span>
              <small>Region: {formatField(school.region_name)}</small>
              <small>Dropped: {formatField(school.dropped_at)}</small>
              <small>Reason: {formatField(school.dropped_reason)}</small>
              <small>
                SIP: {formatField(school.sip_academic_owner_name)} / Center Head:{" "}
                {formatField(school.center_head_name)}
              </small>
              <small>
                Principal: {formatField(school.principal_name)} / SPOC:{" "}
                {formatField(school.school_spoc_name)}
              </small>
              <small>
                BH: {formatField(school.bh_name)} / AOM: {formatField(school.aom_name)}
              </small>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
}

// ── Faculty Assignments panel (Phase 1) ──────────────────────────────────

type FacultyAssignmentsPanelProps = {
  schools: School[];
  users: AppUser[];
  subjects: Subject[];
  assignments: FacultyAssignment[];
  onClose: () => void;
  onCreate: (input: CreateFacultyAssignmentDraft) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
};

export function FacultyAssignmentsPanel({
  schools,
  users,
  subjects,
  assignments,
  onClose,
  onCreate,
  onDelete,
}: FacultyAssignmentsPanelProps) {
  const facultyUsers = React.useMemo(
    () => users.filter((u) => u.role === "faculty" || u.role === "aom"),
    [users],
  );
  const activeSchools = React.useMemo(
    () => schools.filter((s) => !s.is_dropped),
    [schools],
  );

  const [filterFaculty, setFilterFaculty] = React.useState<number | "">("");
  const [filterSchool, setFilterSchool] = React.useState<number | "">("");

  const [draft, setDraft] = React.useState<CreateFacultyAssignmentDraft>({
    faculty_user_id: 0,
    school_id: 0,
    grade_level: gradeLevels[0],
    track: "",
    subject_id: 0,
  });

  // Auto-clear track for non-eligible grades; pre-set track for eligible ones
  React.useEffect(() => {
    if (trackEligibleGrades.has(draft.grade_level)) {
      if (draft.track === "") {
        setDraft((d) => ({ ...d, track: academicTracks[0] }));
      }
    } else if (draft.track !== "") {
      setDraft((d) => ({ ...d, track: "" }));
    }
  }, [draft.grade_level]);  // eslint-disable-line react-hooks/exhaustive-deps

  // Subject options derived from grade+track combo:
  // Foundation grades → subjects with track="Foundation"
  // 11/12/Dropper → subjects with track matching the chosen JEE/NEET
  const subjectOptions = React.useMemo(() => {
    const desiredTrack = trackEligibleGrades.has(draft.grade_level)
      ? draft.track || academicTracks[0]
      : "Foundation";
    return subjects
      .filter((s) => s.track === desiredTrack && s.is_default) // skip optional unless toggled per school (future)
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }, [subjects, draft.grade_level, draft.track]);

  // Reset subject_id when subject options change (current selection may not apply)
  React.useEffect(() => {
    if (subjectOptions.length === 0) {
      if (draft.subject_id !== 0) setDraft((d) => ({ ...d, subject_id: 0 }));
      return;
    }
    if (!subjectOptions.find((s) => s.id === draft.subject_id)) {
      setDraft((d) => ({ ...d, subject_id: subjectOptions[0].id }));
    }
  }, [subjectOptions]);  // eslint-disable-line react-hooks/exhaustive-deps

  const filtered = React.useMemo(() => {
    return assignments.filter((a) => {
      if (filterFaculty && a.faculty_user_id !== filterFaculty) return false;
      if (filterSchool && a.school_id !== filterSchool) return false;
      return true;
    });
  }, [assignments, filterFaculty, filterSchool]);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    if (!draft.faculty_user_id || !draft.school_id || !draft.subject_id) return;
    await onCreate(draft);
    setDraft((d) => ({ ...d, subject_id: subjectOptions[0]?.id ?? 0 }));
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Faculty assignments">
        <header>
          <div>
            <h2>Faculty Assignments</h2>
            <p>{assignments.length} total · {filtered.length} visible</p>
          </div>
          <button type="button" onClick={onClose}>Close</button>
        </header>

        <form className="faculty-assignment-form" onSubmit={handleAdd}>
          <strong>Add assignment</strong>
          <div className="faculty-assignment-row">
            <label>
              Faculty
              <select
                value={draft.faculty_user_id || ""}
                onChange={(e) => setDraft({ ...draft, faculty_user_id: Number(e.target.value) })}
                required
              >
                <option value="">Select faculty</option>
                {facultyUsers.map((u) => (
                  <option key={u.id} value={u.id}>
                    {u.display_name} ({u.role})
                  </option>
                ))}
              </select>
            </label>
            <label>
              School
              <select
                value={draft.school_id || ""}
                onChange={(e) => setDraft({ ...draft, school_id: Number(e.target.value) })}
                required
              >
                <option value="">Select school</option>
                {activeSchools.map((s) => (
                  <option key={s.id} value={s.id}>{s.name}</option>
                ))}
              </select>
            </label>
            <label>
              Grade
              <select
                value={draft.grade_level}
                onChange={(e) => setDraft({ ...draft, grade_level: e.target.value })}
              >
                {gradeLevels.map((g) => (<option key={g}>{g}</option>))}
              </select>
            </label>
            {trackEligibleGrades.has(draft.grade_level) ? (
              <label>
                Track
                <select
                  value={draft.track}
                  onChange={(e) => setDraft({ ...draft, track: e.target.value })}
                >
                  {academicTracks.map((t) => (<option key={t}>{t}</option>))}
                </select>
              </label>
            ) : null}
            <label>
              Subject
              <select
                value={draft.subject_id || ""}
                onChange={(e) => setDraft({ ...draft, subject_id: Number(e.target.value) })}
                required
                disabled={subjectOptions.length === 0}
              >
                {subjectOptions.length === 0 ? (
                  <option value="">No subjects for this combination</option>
                ) : (
                  subjectOptions.map((s) => (<option key={s.id} value={s.id}>{s.name}</option>))
                )}
              </select>
            </label>
            <button type="submit" className="primary-action">Add</button>
          </div>
        </form>

        <div className="faculty-assignment-filters">
          <label>
            Filter by faculty
            <select
              value={filterFaculty}
              onChange={(e) => setFilterFaculty(e.target.value ? Number(e.target.value) : "")}
            >
              <option value="">All faculty</option>
              {facultyUsers.map((u) => (
                <option key={u.id} value={u.id}>{u.display_name}</option>
              ))}
            </select>
          </label>
          <label>
            Filter by school
            <select
              value={filterSchool}
              onChange={(e) => setFilterSchool(e.target.value ? Number(e.target.value) : "")}
            >
              <option value="">All schools</option>
              {activeSchools.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </label>
        </div>

        <div className="faculty-assignment-list">
          {filtered.length === 0 ? (
            <p className="empty-state">No assignments match.</p>
          ) : (
            filtered.map((a) => (
              <article key={a.id} className="faculty-assignment-card">
                <div>
                  <strong>{a.faculty_display_name}</strong>
                  <span>
                    {a.school_name} · {a.grade_level}
                    {a.track ? ` (${a.track})` : ""} · {a.subject_name}
                  </span>
                </div>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => onDelete(a.id)}
                >
                  Remove
                </button>
              </article>
            ))
          )}
        </div>
      </section>
    </div>
  );
}

// ── Timetable panel (Phase 1, Step 5) ──────────────────────────────────────

type TimetablePanelProps = {
  schools: School[];
  users: AppUser[];
  subjects: Subject[];
  slots: TimetableSlot[];
  onClose: () => void;
  onLoad: (params: { schoolId: number; gradeLevel: string; track: string; batchPattern: string }) => Promise<void>;
  onUpsert: (input: UpsertTimetableSlotDraft) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
};

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function TimetablePanel({
  schools,
  users,
  subjects,
  slots,
  onClose,
  onLoad,
  onUpsert,
  onDelete,
}: TimetablePanelProps) {
  const activeSchools = React.useMemo(() => schools.filter((s) => !s.is_dropped), [schools]);
  const facultyUsers = React.useMemo(
    () => users.filter((u) => u.role === "faculty" || u.role === "aom"),
    [users],
  );

  const [schoolId, setSchoolId] = React.useState<number>(0);
  const [gradeLevel, setGradeLevel] = React.useState<string>(gradeLevels[0]);
  const [track, setTrack] = React.useState<string>("");
  const [batchPattern, setBatchPattern] = React.useState<string>(batchPatterns[0]);
  const [periodCount, setPeriodCount] = React.useState<number>(6);

  // Auto-set track when grade changes
  React.useEffect(() => {
    if (trackEligibleGrades.has(gradeLevel)) {
      if (!track) setTrack(academicTracks[0]);
    } else if (track) {
      setTrack("");
    }
  }, [gradeLevel]);  // eslint-disable-line react-hooks/exhaustive-deps

  // Load slots when scope changes
  React.useEffect(() => {
    if (!schoolId) return;
    void onLoad({ schoolId, gradeLevel, track, batchPattern });
  }, [schoolId, gradeLevel, track, batchPattern]);  // eslint-disable-line react-hooks/exhaustive-deps

  // Subject options for the chosen grade+track
  const subjectOptions = React.useMemo(() => {
    const desired = trackEligibleGrades.has(gradeLevel) ? track || academicTracks[0] : "Foundation";
    return subjects
      .filter((s) => s.track === desired && s.is_default)
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }, [subjects, gradeLevel, track]);

  // Filter slots to current scope so cross-batch/grade leaks don't show
  const visibleSlots = React.useMemo(
    () => slots.filter((s) =>
      s.school_id === schoolId &&
      s.grade_level === gradeLevel &&
      (s.track ?? "") === track &&
      s.batch_pattern === batchPattern
    ),
    [slots, schoolId, gradeLevel, track, batchPattern],
  );

  function slotAt(day: number, period: number): TimetableSlot | undefined {
    return visibleSlots.find((s) => s.day_of_week === day && s.period === period);
  }

  // Editing state for a single cell
  const [editing, setEditing] = React.useState<{
    day: number;
    period: number;
    subject_id: number;
    faculty_user_id: number | null;
    start_time: string;
    end_time: string;
  } | null>(null);

  function openEditor(day: number, period: number) {
    const existing = slotAt(day, period);
    setEditing({
      day,
      period,
      subject_id: existing?.subject_id ?? subjectOptions[0]?.id ?? 0,
      faculty_user_id: existing?.faculty_user_id ?? null,
      start_time: existing?.start_time ?? "",
      end_time: existing?.end_time ?? "",
    });
  }

  async function saveEdit() {
    if (!editing || !schoolId || !editing.subject_id) return;
    await onUpsert({
      school_id: schoolId,
      grade_level: gradeLevel,
      track,
      batch_pattern: batchPattern,
      day_of_week: editing.day,
      period: editing.period,
      subject_id: editing.subject_id,
      faculty_user_id: editing.faculty_user_id,
      start_time: editing.start_time,
      end_time: editing.end_time,
    });
    setEditing(null);
  }

  async function deleteEdit() {
    if (!editing) return;
    const existing = slotAt(editing.day, editing.period);
    if (existing) await onDelete(existing.id);
    setEditing(null);
  }

  const hasScope = schoolId > 0;

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Timetable">
        <header>
          <div>
            <h2>Timetable</h2>
            <p>{visibleSlots.length} slots configured for current scope</p>
          </div>
          <button type="button" onClick={onClose}>Close</button>
        </header>

        <div className="timetable-scope">
          <label>
            School
            <select value={schoolId || ""} onChange={(e) => setSchoolId(Number(e.target.value))}>
              <option value="">Select school</option>
              {activeSchools.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </label>
          <label>
            Grade
            <select value={gradeLevel} onChange={(e) => setGradeLevel(e.target.value)}>
              {gradeLevels.map((g) => (<option key={g}>{g}</option>))}
            </select>
          </label>
          {trackEligibleGrades.has(gradeLevel) ? (
            <label>
              Track
              <select value={track} onChange={(e) => setTrack(e.target.value)}>
                {academicTracks.map((t) => (<option key={t}>{t}</option>))}
              </select>
            </label>
          ) : null}
          <label>
            Batch
            <select value={batchPattern} onChange={(e) => setBatchPattern(e.target.value)}>
              {batchPatterns.map((b) => (<option key={b}>{b}</option>))}
            </select>
          </label>
          <label>
            Periods
            <input
              type="number"
              min={1}
              max={12}
              value={periodCount}
              onChange={(e) => setPeriodCount(Math.max(1, Math.min(12, Number(e.target.value) || 1)))}
            />
          </label>
        </div>

        {!hasScope ? (
          <p className="empty-state">Pick a school to view its timetable.</p>
        ) : (
          <div className="timetable-grid-wrap">
            <table className="timetable-grid">
              <thead>
                <tr>
                  <th>Period</th>
                  {DAY_NAMES.map((d, i) => (<th key={i}>{d}</th>))}
                </tr>
              </thead>
              <tbody>
                {Array.from({ length: periodCount }, (_, i) => i + 1).map((p) => (
                  <tr key={p}>
                    <th>P{p}</th>
                    {DAY_NAMES.map((_, day) => {
                      const s = slotAt(day, p);
                      return (
                        <td
                          key={day}
                          className={`timetable-cell${s ? " filled" : ""}`}
                          onClick={() => openEditor(day, p)}
                        >
                          {s ? (
                            <>
                              <strong>{s.subject_name}</strong>
                              {s.faculty_display_name ? (
                                <small>{s.faculty_display_name}</small>
                              ) : (
                                <small className="timetable-cell-empty-faculty">— no faculty —</small>
                              )}
                            </>
                          ) : (
                            <span className="timetable-cell-add">+</span>
                          )}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {editing ? (
          <div className="modal-backdrop nested-modal" onClick={() => setEditing(null)}>
            <section
              className="ticket-modal timetable-edit-modal"
              onClick={(e) => e.stopPropagation()}
            >
              <header>
                <div>
                  <h3>{DAY_NAMES[editing.day]} · Period {editing.period}</h3>
                  <p>{gradeLevel}{track ? ` (${track})` : ""} · {batchPattern}</p>
                </div>
                <button type="button" onClick={() => setEditing(null)}>Cancel</button>
              </header>
              <div className="timetable-edit-body">
                <label>
                  Subject
                  <select
                    value={editing.subject_id || ""}
                    onChange={(e) => setEditing({ ...editing, subject_id: Number(e.target.value) })}
                  >
                    {subjectOptions.length === 0 ? (
                      <option value="">No subjects available</option>
                    ) : (
                      subjectOptions.map((s) => (<option key={s.id} value={s.id}>{s.name}</option>))
                    )}
                  </select>
                </label>
                <label>
                  Faculty (optional)
                  <select
                    value={editing.faculty_user_id ?? ""}
                    onChange={(e) =>
                      setEditing({ ...editing, faculty_user_id: e.target.value ? Number(e.target.value) : null })
                    }
                  >
                    <option value="">— unassigned —</option>
                    {facultyUsers.map((u) => (
                      <option key={u.id} value={u.id}>{u.display_name}</option>
                    ))}
                  </select>
                </label>
                <label>
                  Start time (optional)
                  <input
                    type="time"
                    value={editing.start_time}
                    onChange={(e) => setEditing({ ...editing, start_time: e.target.value })}
                  />
                </label>
                <label>
                  End time (optional)
                  <input
                    type="time"
                    value={editing.end_time}
                    onChange={(e) => setEditing({ ...editing, end_time: e.target.value })}
                  />
                </label>
              </div>
              <div className="timetable-edit-actions">
                {slotAt(editing.day, editing.period) ? (
                  <button type="button" className="secondary-button" onClick={deleteEdit}>
                    Clear slot
                  </button>
                ) : null}
                <button type="button" className="primary-action" onClick={saveEdit}>
                  Save
                </button>
              </div>
            </section>
          </div>
        ) : null}
      </section>
    </div>
  );
}

type SubjectsPanelProps = {
  schools: School[];
  subjects: Subject[];
  onClose: () => void;
  onLoadEffective: (schoolId: number, track: string) => Promise<EffectiveSubject[]>;
  onToggleOptional: (schoolId: number, subjectId: number, enabled: boolean) => Promise<void>;
};

export function SubjectsPanel({
  schools,
  subjects,
  onClose,
  onLoadEffective,
  onToggleOptional,
}: SubjectsPanelProps) {
  const activeSchools = React.useMemo(
    () => schools.filter((s) => !s.is_dropped).sort((a, b) => a.name.localeCompare(b.name)),
    [schools],
  );

  const [selectedSchoolId, setSelectedSchoolId] = React.useState<number | "">("");
  const [effective, setEffective] = React.useState<EffectiveSubject[]>([]);
  const [loading, setLoading] = React.useState(false);

  const optionalSubjects = React.useMemo(
    () => subjects.filter((s) => s.track === "Foundation" && !s.is_default),
    [subjects],
  );

  React.useEffect(() => {
    if (!selectedSchoolId) {
      setEffective([]);
      return;
    }
    setLoading(true);
    onLoadEffective(selectedSchoolId, "Foundation")
      .then(setEffective)
      .finally(() => setLoading(false));
  }, [selectedSchoolId]); // eslint-disable-line react-hooks/exhaustive-deps

  async function handleToggle(subjectId: number, enabled: boolean) {
    if (!selectedSchoolId) return;
    await onToggleOptional(selectedSchoolId, subjectId, enabled);
    const updated = await onLoadEffective(selectedSchoolId, "Foundation");
    setEffective(updated);
  }

  const selectedSchool = activeSchools.find((s) => s.id === selectedSchoolId);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Subjects configuration">
        <header>
          <div>
            <h2>Subjects</h2>
            <p>Manage optional subjects (English / SST) per Foundation school.</p>
          </div>
          <button type="button" onClick={onClose}>Close</button>
        </header>

        <div className="faculty-assignment-filters">
          <label>
            School
            <select
              value={selectedSchoolId}
              onChange={(e) => setSelectedSchoolId(e.target.value ? Number(e.target.value) : "")}
            >
              <option value="">Select a school</option>
              {activeSchools.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </label>
        </div>

        {selectedSchool && (
          <div style={{ padding: "0 1rem" }}>
            <h3>{selectedSchool.name} — Foundation Track</h3>
            {loading ? (
              <p>Loading subjects…</p>
            ) : (
              <>
                <div className="faculty-assignment-list">
                  <h4>Default Subjects</h4>
                  {effective
                    .filter((s) => s.is_default)
                    .map((s) => (
                      <div key={s.id} className="faculty-assignment-row">
                        <span>{s.name}</span>
                        <span className="badge">Default</span>
                      </div>
                    ))}
                </div>
                <div className="faculty-assignment-list">
                  <h4>Optional Subjects</h4>
                  {optionalSubjects.length === 0 ? (
                    <p className="empty-state">No optional subjects configured.</p>
                  ) : (
                    optionalSubjects.map((subj) => {
                      const eff = effective.find((e) => e.id === subj.id);
                      const enabled = eff?.is_offered ?? false;
                      return (
                        <div key={subj.id} className="faculty-assignment-row">
                          <span>{subj.name}</span>
                          <label className="toggle-switch">
                            <input
                              type="checkbox"
                              checked={enabled}
                              onChange={(e) => handleToggle(subj.id, e.target.checked)}
                            />
                            <span>{enabled ? "Offered" : "Not offered"}</span>
                          </label>
                        </div>
                      );
                    })
                  )}
                </div>
              </>
            )}
          </div>
        )}
      </section>
    </div>
  );
}

type RegionHistoryPanelProps = {
  history: SchoolRegionHistory[];
  onClose: () => void;
};

export function RegionHistoryPanel({ history, onClose }: RegionHistoryPanelProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Region change log">
        <header>
          <div>
            <h2>Region Change Log</h2>
            <p>Track every school region reassignment with the change date.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="region-history-list">
          {history.length === 0 ? <p>No region changes recorded.</p> : null}
          {history.map((item) => (
            <article key={item.id}>
              <strong>{item.school_name}</strong>
              <span>
                {formatField(item.old_region_name)} -&gt; {formatField(item.new_region_name)}
              </span>
              <small>{formatField(item.changed_at)}</small>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
}

type AuditLogPanelProps = {
  entries: AuditLogEntry[];
  onClose: () => void;
};

export function AuditLogPanel({ entries, onClose }: AuditLogPanelProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="Audit log">
        <header>
          <div>
            <h2>Audit Log</h2>
            <p>Production audit trail for ticket, school, and region changes.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="region-history-list">
          {entries.length === 0 ? <p>No audit entries recorded yet.</p> : null}
          {entries.map((entry) => (
            <article key={entry.id}>
              <strong>
                {formatField(entry.entity_type)} #{entry.entity_id} - {formatField(entry.action)}
              </strong>
              <span>{entry.summary || "No summary recorded"}</span>
              <small>
                {entry.actor} - {formatTimestamp(entry.created_at)}
              </small>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
}

const emptySchoolProfileDraft: SchoolProfileDraft = {
  name: "",
  region_id: null,
  region_name: "",
  program_model: "",
  distance_classification: "",
  sip_academic_owner_role: "",
  sip_academic_owner_name: "",
  sip_academic_owner_mobile: "",
  sip_academic_owner_email: "",
  center_head_name: "",
  center_head_mobile: "",
  center_head_email: "",
  principal_name: "",
  principal_mobile: "",
  principal_email: "",
  school_spoc_name: "",
  school_spoc_mobile: "",
  school_spoc_email: "",
  central_academic_spoc_name: "",
  central_academic_spoc_mobile: "",
  central_academic_spoc_email: "",
  central_business_spoc_name: "",
  central_business_spoc_mobile: "",
  central_business_spoc_email: "",
  bh_name: "",
  bh_mobile: "",
  bh_email: "",
  aom_name: "",
  aom_mobile: "",
  aom_email: "",
  mapped_vp_center: "",
};

const schoolContactGroups: Array<{
  title: string;
  fields: Array<{ key: keyof SchoolProfileDraft; label: string; type: "text" | "email" | "tel" }>;
}> = [
  {
    title: "SIP Academic Head / Lead",
    fields: [
      { key: "sip_academic_owner_name", label: "Name", type: "text" },
      { key: "sip_academic_owner_mobile", label: "Mobile", type: "tel" },
      { key: "sip_academic_owner_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "Center Head",
    fields: [
      { key: "center_head_name", label: "Name", type: "text" },
      { key: "center_head_mobile", label: "Mobile", type: "tel" },
      { key: "center_head_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "School Principal",
    fields: [
      { key: "principal_name", label: "Name", type: "text" },
      { key: "principal_mobile", label: "Mobile", type: "tel" },
      { key: "principal_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "School SPOC",
    fields: [
      { key: "school_spoc_name", label: "Name", type: "text" },
      { key: "school_spoc_mobile", label: "Mobile", type: "tel" },
      { key: "school_spoc_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "BH Details",
    fields: [
      { key: "bh_name", label: "Name", type: "text" },
      { key: "bh_mobile", label: "Mobile", type: "tel" },
      { key: "bh_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "AOM Details",
    fields: [
      { key: "aom_name", label: "Name", type: "text" },
      { key: "aom_mobile", label: "Mobile", type: "tel" },
      { key: "aom_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "Central SIP Academic Team SPOC",
    fields: [
      { key: "central_academic_spoc_name", label: "Name", type: "text" },
      { key: "central_academic_spoc_mobile", label: "Mobile", type: "tel" },
      { key: "central_academic_spoc_email", label: "Email", type: "email" },
    ],
  },
  {
    title: "Central Business Team SPOC",
    fields: [
      { key: "central_business_spoc_name", label: "Name", type: "text" },
      { key: "central_business_spoc_mobile", label: "Mobile", type: "tel" },
      { key: "central_business_spoc_email", label: "Email", type: "email" },
    ],
  },
];

type ProgramDashboardPanelProps = {
  dashboard: SchoolProgramDashboard;
  onClose: () => void;
};

export function ProgramDashboardPanel({ dashboard, onClose }: ProgramDashboardPanelProps) {
  const groupedByModel = groupPlansBy(dashboard.class_plans, "lecture_model_name");
  const groupedByClass = groupPlansBy(dashboard.class_plans, "grade_level");

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal program-dashboard-modal" aria-label="Program dashboard">
        <header>
          <div>
            <h2>Program Dashboard</h2>
            <p>Class coverage, lecture delivery, and admission visibility.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="dashboard-metrics">
          <DashboardMetric label="Schools" value={dashboard.total_schools} />
          <DashboardMetric label="Planned schools" value={dashboard.schools_with_class_plans} />
          <DashboardMetric label="Classes running" value={dashboard.total_classes} />
          <DashboardMetric label="AOP admissions" value={dashboard.total_aop_admissions} />
          <DashboardMetric label="Actual admissions" value={dashboard.total_actual_admissions} />
          <DashboardMetric label="Attainment" value={`${dashboard.admission_attainment_percent}%`} />
        </div>

        <div className="dashboard-sections">
          <section>
            <h3>School mix</h3>
            <dl className="dashboard-breakdown">
              <div>
                <dt>Aspire</dt>
                <dd>{dashboard.aspire_school_count}</dd>
              </div>
              <div>
                <dt>Minimum Guarantee</dt>
                <dd>{dashboard.minimum_guarantee_school_count}</dd>
              </div>
              <div>
                <dt>Remote</dt>
                <dd>{dashboard.remote_school_count}</dd>
              </div>
              <div>
                <dt>Near Proximity</dt>
                <dd>{dashboard.near_proximity_school_count}</dd>
              </div>
            </dl>
          </section>

          <section>
            <h3>Lecture model coverage</h3>
            <DashboardGroupList groups={groupedByModel} />
          </section>

          <section>
            <h3>Class-wise admissions</h3>
            <DashboardGroupList groups={groupedByClass} />
          </section>
        </div>

        <div className="class-plan-table" role="table" aria-label="Class plan details">
          <div role="row">
            <strong>School</strong>
            <strong>Class</strong>
            <strong>Delivery</strong>
            <strong>AOP</strong>
            <strong>Actual</strong>
            <strong>Gap</strong>
          </div>
          {dashboard.class_plans.map((plan) => (
            <div role="row" key={plan.id}>
              <span>{plan.school_name}</span>
              <span>{plan.grade_level}</span>
              <span>
                {plan.lecture_model_name}, {plan.batch_pattern}
              </span>
              <span>{plan.aop_admissions}</span>
              <span>{plan.actual_admissions}</span>
              <span>{plan.admission_gap}</span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function DashboardMetric({ label, value }: { label: string; value: number | string }) {
  return (
    <div>
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function DashboardGroupList({ groups }: { groups: Record<string, SchoolClassPlan[]> }) {
  return (
    <div className="dashboard-breakdown">
      {Object.entries(groups).map(([name, plans]) => {
        const aop = plans.reduce((sum, plan) => sum + plan.aop_admissions, 0);
        const actual = plans.reduce((sum, plan) => sum + plan.actual_admissions, 0);
        return (
          <div key={name}>
            <dt>{name}</dt>
            <dd>
              {plans.length} classes - {actual}/{aop}
            </dd>
          </div>
        );
      })}
    </div>
  );
}

type ReportsPanelProps = {
  comments: TicketComment[];
  tickets: Ticket[];
  schools: School[];
  droppedSchools: School[];
  dashboard: SchoolProgramDashboard;
  onClose: () => void;
  onExportCsv: () => void;
  onExportSipMaster: () => void;
};

type ReportTab = "overview" | "schoolwise" | "regionwise" | "resolution";

export function ReportsPanel({
  comments,
  tickets,
  schools,
  droppedSchools,
  dashboard,
  onClose,
  onExportCsv,
  onExportSipMaster,
}: ReportsPanelProps) {
  const [reportTab, setReportTab] = React.useState<ReportTab>("overview");

  const openTickets = tickets.filter((ticket) => !["Resolved", "Closed"].includes(ticket.status));
  const resolvedTickets = tickets.filter((ticket) => ["Resolved", "Closed"].includes(ticket.status));
  const breachedTickets = tickets.filter((ticket) => getSlaState(ticket) === "Breached");
  const escalatedTickets = tickets.filter((ticket) => ticket.escalation_status === "Escalated");
  const atRiskTickets = tickets.filter((ticket) => ticket.escalation_status === "At Risk");
  const unassignedTickets = tickets.filter((ticket) => ticket.assignee === "Unassigned");
  const statusCounts = countBy(tickets, (ticket) => ticket.status);
  const priorityCounts = countBy(tickets, (ticket) => ticket.priority);
  const queueCounts = countBy(tickets, (ticket) => ticket.queue);
  const assigneeCounts = countBy(tickets, (ticket) => ticket.assignee || "Unassigned");
  const schoolCounts = countBy(tickets, (ticket) => ticket.school_name || "Unmapped school");
  const outboundCommunications = comments.filter((comment) => !comment.is_internal);
  const internalNotes = comments.filter((comment) => comment.is_internal);
  const channelCounts = countBy(outboundCommunications, (comment) => comment.channel || "Local");
  const audienceCounts = countBy(
    outboundCommunications,
    (comment) => comment.audience || "School",
  );
  const deliveryCounts = countBy(
    outboundCommunications,
    (comment) => comment.delivery_status || "Logged",
  );
  const overdueFollowUps = outboundCommunications
    .map((comment) => ({
      comment,
      ticket: tickets.find((ticket) => ticket.id === comment.ticket_id),
    }))
    .filter(
      (item) =>
        item.ticket &&
        item.comment.next_follow_up_due &&
        item.comment.delivery_status !== "Acknowledged" &&
        item.comment.next_follow_up_due < new Date().toISOString().slice(0, 16).replace("T", " "),
    )
    .slice(0, 8);

  // Schoolwise executive report data
  const schoolwiseRows = schools
    .map((school) => {
      const schoolTickets = tickets.filter((t) => t.school_name === school.name);
      const classPlan = dashboard.class_plans.filter((p) => p.school_name === school.name);
      const aopAdmissions = classPlan.reduce((sum, p) => sum + p.aop_admissions, 0);
      const actualAdmissions = classPlan.reduce((sum, p) => sum + p.actual_admissions, 0);
      const attainmentPct = aopAdmissions > 0 ? Math.round((actualAdmissions / aopAdmissions) * 100) : 0;
      return {
        name: school.name,
        region: school.region_name || "—",
        open: schoolTickets.filter((t) => !["Resolved", "Closed"].includes(t.status)).length,
        resolved: schoolTickets.filter((t) => ["Resolved", "Closed"].includes(t.status)).length,
        breached: schoolTickets.filter((t) => getSlaState(t) === "Breached").length,
        escalated: schoolTickets.filter((t) => t.escalation_status === "Escalated").length,
        comms: outboundCommunications.filter((c) => {
          const t = tickets.find((tk) => tk.id === c.ticket_id);
          return t?.school_name === school.name;
        }).length,
        aopAdmissions,
        actualAdmissions,
        attainmentPct,
      };
    })
    .sort((a, b) => b.open - a.open);

  // Time-to-resolution data (based on created_at → updated_at for resolved/closed tickets)
  const resolvedWithTtr = resolvedTickets
    .map((t) => {
      const created = Date.parse(t.created_at.replace(" ", "T"));
      const updated = Date.parse(t.updated_at.replace(" ", "T"));
      const ttrHours = (updated - created) / 3_600_000;
      return { ...t, ttrHours };
    })
    .filter((t) => t.ttrHours >= 0);
  const avgTtrHours = resolvedWithTtr.length > 0
    ? resolvedWithTtr.reduce((sum, t) => sum + t.ttrHours, 0) / resolvedWithTtr.length
    : 0;
  function formatTtrHours(h: number): string {
    if (h < 1) return `${Math.round(h * 60)}m`;
    if (h < 48) return `${h.toFixed(1)}h`;
    return `${(h / 24).toFixed(1)}d`;
  }
  function groupAvgTtr(field: keyof Ticket): Array<{ label: string; avgHours: number; count: number }> {
    const buckets = new Map<string, number[]>();
    for (const t of resolvedWithTtr) {
      const key = String(t[field] || "Unknown");
      const arr = buckets.get(key) ?? [];
      arr.push(t.ttrHours);
      buckets.set(key, arr);
    }
    return Array.from(buckets.entries())
      .map(([label, hours]) => ({ label, avgHours: hours.reduce((s, h) => s + h, 0) / hours.length, count: hours.length }))
      .sort((a, b) => a.avgHours - b.avgHours);
  }
  const ttrByPriority = groupAvgTtr("priority");
  const ttrByCategory = groupAvgTtr("issue_category");
  const fastestTickets = [...resolvedWithTtr].sort((a, b) => a.ttrHours - b.ttrHours).slice(0, 5);
  const slowestTickets = [...resolvedWithTtr].sort((a, b) => b.ttrHours - a.ttrHours).slice(0, 5);

  // Regionwise executive report data
  const regionMap = new Map<string, {
    schools: number; open: number; resolved: number; breached: number;
    escalated: number; comms: number; aopAdmissions: number; actualAdmissions: number;
  }>();
  for (const row of schoolwiseRows) {
    const key = row.region || "No Region";
    const existing = regionMap.get(key) ?? { schools: 0, open: 0, resolved: 0, breached: 0, escalated: 0, comms: 0, aopAdmissions: 0, actualAdmissions: 0 };
    regionMap.set(key, {
      schools: existing.schools + 1,
      open: existing.open + row.open,
      resolved: existing.resolved + row.resolved,
      breached: existing.breached + row.breached,
      escalated: existing.escalated + row.escalated,
      comms: existing.comms + row.comms,
      aopAdmissions: existing.aopAdmissions + row.aopAdmissions,
      actualAdmissions: existing.actualAdmissions + row.actualAdmissions,
    });
  }
  const regionwiseRows = Array.from(regionMap.entries())
    .map(([region, data]) => ({
      region,
      ...data,
      attainmentPct: data.aopAdmissions > 0 ? Math.round((data.actualAdmissions / data.aopAdmissions) * 100) : 0,
    }))
    .sort((a, b) => b.open - a.open);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal reports-modal" aria-label="Reports">
        <header>
          <div>
            <h2>Reports</h2>
            <p>Operational, SLA, school program, and workload reporting from current app data.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="report-actions">
          <button type="button" className="secondary-button" onClick={onExportCsv}>
            Export Ticket Data CSV
          </button>
          <button type="button" className="secondary-button" onClick={onExportSipMaster}>
            Export SIP Master Excel
          </button>
          <small>Exports ticket history CSV or a schoolwise SIP master Excel file.</small>
        </div>

        <div className="report-tab-bar">
          {(["overview", "schoolwise", "regionwise", "resolution"] as ReportTab[]).map((tab) => (
            <button
              key={tab}
              type="button"
              className={reportTab === tab ? "secondary-button active-chip" : "secondary-button"}
              onClick={() => setReportTab(tab)}
            >
              {tab === "resolution" ? "Time to Resolution" : tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        {reportTab === "overview" ? (
          <>
            <div className="dashboard-metrics">
              <DashboardMetric label="Total tickets" value={tickets.length} />
              <DashboardMetric label="Open workload" value={openTickets.length} />
              <DashboardMetric label="Resolved/closed" value={resolvedTickets.length} />
              <DashboardMetric label="SLA breached" value={breachedTickets.length} />
              <DashboardMetric label="Escalated" value={escalatedTickets.length} />
              <DashboardMetric label="Unassigned" value={unassignedTickets.length} />
              <DashboardMetric label="Outbound comms" value={outboundCommunications.length} />
              <DashboardMetric label="Internal notes" value={internalNotes.length} />
            </div>

            <div className="report-sections">
              <section>
                <h3>Ticket status</h3>
                <ReportBreakdown counts={statusCounts} />
              </section>
              <section>
                <h3>Priority mix</h3>
                <ReportBreakdown counts={priorityCounts} />
              </section>
              <section>
                <h3>SLA and escalation</h3>
                <dl className="dashboard-breakdown">
                  <ReportRow label="Breached" value={breachedTickets.length} />
                  <ReportRow label="At risk" value={atRiskTickets.length} />
                  <ReportRow label="Escalated" value={escalatedTickets.length} />
                  <ReportRow label="Unassigned" value={unassignedTickets.length} />
                </dl>
              </section>
              <section>
                <h3>Queue workload</h3>
                <ReportBreakdown counts={queueCounts} limit={6} />
              </section>
              <section>
                <h3>Assignee workload</h3>
                <ReportBreakdown counts={assigneeCounts} limit={6} />
              </section>
              <section>
                <h3>Top schools by tickets</h3>
                <ReportBreakdown counts={schoolCounts} limit={6} />
              </section>
              <section>
                <h3>Communication channels</h3>
                <ReportBreakdown counts={channelCounts} />
              </section>
              <section>
                <h3>Communication audiences</h3>
                <ReportBreakdown counts={audienceCounts} />
              </section>
              <section>
                <h3>Communication status</h3>
                <ReportBreakdown counts={deliveryCounts} />
              </section>
              <section>
                <h3>Overdue follow-ups</h3>
                {overdueFollowUps.length > 0 ? (
                  <div className="report-list">
                    {overdueFollowUps.map(({ comment, ticket }) => (
                      <article key={comment.id}>
                        <strong>
                          #{ticket?.id} {ticket?.title}
                        </strong>
                        <span>
                          {comment.channel} to {comment.recipient_name || comment.audience}
                        </span>
                        <small>
                          {comment.delivery_status} - due {formatTimestamp(comment.next_follow_up_due)}
                        </small>
                      </article>
                    ))}
                  </div>
                ) : (
                  <p className="empty-state compact">No overdue follow-up communications.</p>
                )}
              </section>
              <section>
                <h3>SIP program snapshot</h3>
                <dl className="dashboard-breakdown">
                  <ReportRow label="Active schools" value={schools.length} />
                  <ReportRow label="Dropped schools" value={droppedSchools.length} />
                  <ReportRow label="Classes running" value={dashboard.total_classes} />
                  <ReportRow
                    label="Admission attainment"
                    value={`${dashboard.admission_attainment_percent}%`}
                  />
                </dl>
              </section>
              <section>
                <h3>Admissions</h3>
                <dl className="dashboard-breakdown">
                  <ReportRow label="AOP admissions" value={dashboard.total_aop_admissions} />
                  <ReportRow label="Actual admissions" value={dashboard.total_actual_admissions} />
                  <ReportRow label="Admission gap" value={dashboard.admission_gap} />
                  <ReportRow label="Planned schools" value={dashboard.schools_with_class_plans} />
                </dl>
              </section>
            </div>
          </>
        ) : reportTab === "schoolwise" ? (
          <div className="executive-report-wrap">
            <table className="data-table executive-table">
              <thead>
                <tr>
                  <th>School</th>
                  <th>Region</th>
                  <th>Open</th>
                  <th>Resolved</th>
                  <th>Breached</th>
                  <th>Escalated</th>
                  <th>Comms</th>
                  <th>AOP</th>
                  <th>Actual</th>
                  <th>Attainment</th>
                </tr>
              </thead>
              <tbody>
                {schoolwiseRows.length === 0 ? (
                  <tr><td colSpan={10} className="empty-state compact">No school data available.</td></tr>
                ) : schoolwiseRows.map((row) => (
                  <tr key={row.name} className={row.breached > 0 ? "row-attention" : ""}>
                    <td>{row.name}</td>
                    <td>{row.region}</td>
                    <td>{row.open}</td>
                    <td>{row.resolved}</td>
                    <td className={row.breached > 0 ? "cell-alert" : ""}>{row.breached}</td>
                    <td className={row.escalated > 0 ? "cell-alert" : ""}>{row.escalated}</td>
                    <td>{row.comms}</td>
                    <td>{row.aopAdmissions || "—"}</td>
                    <td>{row.actualAdmissions || "—"}</td>
                    <td>{row.aopAdmissions > 0 ? `${row.attainmentPct}%` : "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : reportTab === "regionwise" ? (
          <div className="executive-report-wrap">
            <table className="data-table executive-table">
              <thead>
                <tr>
                  <th>Region</th>
                  <th>Schools</th>
                  <th>Open</th>
                  <th>Resolved</th>
                  <th>Breached</th>
                  <th>Escalated</th>
                  <th>Comms</th>
                  <th>AOP</th>
                  <th>Actual</th>
                  <th>Attainment</th>
                </tr>
              </thead>
              <tbody>
                {regionwiseRows.length === 0 ? (
                  <tr><td colSpan={10} className="empty-state compact">No region data available.</td></tr>
                ) : regionwiseRows.map((row) => (
                  <tr key={row.region} className={row.breached > 0 ? "row-attention" : ""}>
                    <td>{row.region}</td>
                    <td>{row.schools}</td>
                    <td>{row.open}</td>
                    <td>{row.resolved}</td>
                    <td className={row.breached > 0 ? "cell-alert" : ""}>{row.breached}</td>
                    <td className={row.escalated > 0 ? "cell-alert" : ""}>{row.escalated}</td>
                    <td>{row.comms}</td>
                    <td>{row.aopAdmissions || "—"}</td>
                    <td>{row.actualAdmissions || "—"}</td>
                    <td>{row.aopAdmissions > 0 ? `${row.attainmentPct}%` : "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="report-sections">
            <section>
              <h3>Overall</h3>
              <dl className="dashboard-breakdown">
                <ReportRow label="Resolved tickets" value={resolvedWithTtr.length} />
                <ReportRow label="Avg time to resolution" value={resolvedWithTtr.length > 0 ? formatTtrHours(avgTtrHours) : "—"} />
              </dl>
            </section>
            <section>
              <h3>By priority</h3>
              {ttrByPriority.length === 0 ? (
                <p className="empty-state compact">No resolved tickets yet.</p>
              ) : (
                <table className="data-table">
                  <thead><tr><th>Priority</th><th>Avg TTR</th><th>Count</th></tr></thead>
                  <tbody>
                    {ttrByPriority.map((row) => (
                      <tr key={row.label}>
                        <td>{row.label}</td>
                        <td>{formatTtrHours(row.avgHours)}</td>
                        <td>{row.count}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </section>
            <section>
              <h3>By category</h3>
              {ttrByCategory.length === 0 ? (
                <p className="empty-state compact">No resolved tickets yet.</p>
              ) : (
                <table className="data-table">
                  <thead><tr><th>Category</th><th>Avg TTR</th><th>Count</th></tr></thead>
                  <tbody>
                    {ttrByCategory.map((row) => (
                      <tr key={row.label}>
                        <td>{row.label}</td>
                        <td>{formatTtrHours(row.avgHours)}</td>
                        <td>{row.count}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </section>
            <section>
              <h3>Fastest resolutions</h3>
              {fastestTickets.length === 0 ? (
                <p className="empty-state compact">No resolved tickets yet.</p>
              ) : (
                <table className="data-table">
                  <thead><tr><th>#</th><th>Title</th><th>Priority</th><th>TTR</th></tr></thead>
                  <tbody>
                    {fastestTickets.map((t) => (
                      <tr key={t.id}>
                        <td>{t.id}</td>
                        <td>{t.title}</td>
                        <td>{t.priority}</td>
                        <td>{formatTtrHours(t.ttrHours)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </section>
            <section>
              <h3>Slowest resolutions</h3>
              {slowestTickets.length === 0 ? (
                <p className="empty-state compact">No resolved tickets yet.</p>
              ) : (
                <table className="data-table">
                  <thead><tr><th>#</th><th>Title</th><th>Priority</th><th>TTR</th></tr></thead>
                  <tbody>
                    {slowestTickets.map((t) => (
                      <tr key={t.id}>
                        <td>{t.id}</td>
                        <td>{t.title}</td>
                        <td>{t.priority}</td>
                        <td>{formatTtrHours(t.ttrHours)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </section>
          </div>
        )}
      </section>
    </div>
  );
}

type CommunicationOperationsPanelProps = {
  comments: TicketComment[];
  schools: School[];
  tickets: Ticket[];
  onClose: () => void;
  onExport: () => void;
  onOpenTicket: (ticketId: number) => void;
  onUpdateCommentStatus: (id: number, deliveryStatus: string, nextFollowUpDue?: string) => void;
};

type CommunicationQueueFilter =
  | "all"
  | "due-today"
  | "overdue"
  | "awaiting-ack"
  | "follow-up-due";

export function CommunicationOperationsPanel({
  comments,
  schools,
  tickets,
  onClose,
  onExport,
  onOpenTicket,
  onUpdateCommentStatus,
}: CommunicationOperationsPanelProps) {
  const [queueFilter, setQueueFilter] = React.useState<CommunicationQueueFilter>("overdue");
  const [schoolFilter, setSchoolFilter] = React.useState("");
  const [search, setSearch] = React.useState("");
  const [dateFrom, setDateFrom] = React.useState("");
  const [dateTo, setDateTo] = React.useState("");

  const outboundComments = comments.filter((comment) => !comment.is_internal);
  const now = new Date();
  const nowKey = toLocalDateTimeKey(now);
  const todayKey = nowKey.slice(0, 10);
  const schoolHistoryCounts = countBy(outboundComments, (comment) => {
    const ticket = tickets.find((item) => item.id === comment.ticket_id);
    return ticket?.school_name || "Unmapped school";
  });

  const queuedComments = outboundComments
    .map((comment) => ({
      comment,
      ticket: tickets.find((ticket) => ticket.id === comment.ticket_id),
    }))
    .filter((item) => item.ticket)
    .filter((item) => {
      if (!schoolFilter) {
        return true;
      }
      return item.ticket?.school_name === schoolFilter;
    })
    .filter((item) => {
      const created = item.comment.created_at.slice(0, 10);
      if (dateFrom && created < dateFrom) return false;
      if (dateTo && created > dateTo) return false;
      return true;
    })
    .filter((item) => {
      const haystack = [
        item.ticket?.title || "",
        item.ticket?.school_name || "",
        item.comment.author,
        item.comment.recipient_name,
        item.comment.recipient_contact,
        item.comment.channel,
        item.comment.delivery_status,
        item.comment.body,
      ]
        .join(" ")
        .toLocaleLowerCase();
      return haystack.includes(search.trim().toLocaleLowerCase());
    })
    .filter((item) => {
      const due = item.comment.next_follow_up_due;
      if (queueFilter === "all") {
        return true;
      }
      if (queueFilter === "due-today") {
        return Boolean(due && due.startsWith(todayKey));
      }
      if (queueFilter === "overdue") {
        return Boolean(due && due < nowKey && item.comment.delivery_status !== "Acknowledged");
      }
      if (queueFilter === "awaiting-ack") {
        return item.comment.delivery_status === "Sent";
      }
      if (queueFilter === "follow-up-due") {
        return item.comment.delivery_status === "Follow-up Due";
      }
      return true;
    })
    .sort((left, right) => {
      const leftDue = left.comment.next_follow_up_due || "9999-12-31 23:59";
      const rightDue = right.comment.next_follow_up_due || "9999-12-31 23:59";
      return leftDue.localeCompare(rightDue);
    });

  const dueTodayCount = outboundComments.filter(
    (comment) => comment.next_follow_up_due && comment.next_follow_up_due.startsWith(todayKey),
  ).length;
  const overdueCount = outboundComments.filter(
    (comment) =>
      comment.next_follow_up_due &&
      comment.next_follow_up_due < nowKey &&
      comment.delivery_status !== "Acknowledged",
  ).length;
  const awaitingAckCount = outboundComments.filter(
    (comment) => comment.delivery_status === "Sent",
  ).length;
  const followUpDueCount = outboundComments.filter(
    (comment) => comment.delivery_status === "Follow-up Due",
  ).length;

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal reports-modal" aria-label="Communication operations">
        <header>
          <div>
            <h2>Communication Operations</h2>
            <p>Monitor follow-ups, acknowledgements, schoolwise communication history, and export logs.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="report-actions">
          <button type="button" className="secondary-button" onClick={onExport}>
            Export Communication CSV
          </button>
          <small>Exports the communication log with recipients, status, last contact, and follow-up due.</small>
        </div>

        <div className="dashboard-metrics">
          <DashboardMetric label="Outbound" value={outboundComments.length} />
          <DashboardMetric label="Due today" value={dueTodayCount} />
          <DashboardMetric label="Overdue" value={overdueCount} />
          <DashboardMetric label="Awaiting ack" value={awaitingAckCount} />
          <DashboardMetric label="Follow-up due" value={followUpDueCount} />
          <DashboardMetric label="Schools touched" value={Object.keys(schoolHistoryCounts).length} />
        </div>

        <div className="communication-toolbar">
          <div className="communication-filter-chips">
            {[
              ["overdue", "Overdue"],
              ["due-today", "Due Today"],
              ["awaiting-ack", "Awaiting Ack"],
              ["follow-up-due", "Follow-up Due"],
              ["all", "All"],
            ].map(([key, label]) => (
              <button
                key={key}
                type="button"
                className={queueFilter === key ? "secondary-button active-chip" : "secondary-button"}
                onClick={() => setQueueFilter(key as CommunicationQueueFilter)}
              >
                {label}
              </button>
            ))}
          </div>
          <div className="communication-filter-row">
            <label>
              School
              <select value={schoolFilter} onChange={(event) => setSchoolFilter(event.target.value)}>
                <option value="">All schools</option>
                {schools.map((school) => (
                  <option key={school.id} value={school.name}>
                    {school.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              From
              <input type="date" value={dateFrom} onChange={(e) => setDateFrom(e.target.value)} />
            </label>
            <label>
              To
              <input type="date" value={dateTo} onChange={(e) => setDateTo(e.target.value)} />
            </label>
            <label>
              Search
              <input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Author, recipient, school, ticket"
              />
            </label>
          </div>
        </div>

        <div className="report-sections">
          <section>
            <h3>Schoolwise communication history</h3>
            <ReportBreakdown counts={schoolHistoryCounts} limit={10} />
          </section>
          <section className="communication-queue-section">
            <h3>Action queue</h3>
            {queuedComments.length > 0 ? (
              <div className="communication-queue">
                {queuedComments.map(({ comment, ticket }) => {
                  const isOverdue =
                    comment.next_follow_up_due &&
                    comment.next_follow_up_due < nowKey &&
                    comment.delivery_status !== "Acknowledged";
                  return (
                  <article key={comment.id} className={isOverdue ? "comm-overdue" : ""}>
                    <div className="communication-queue-header">
                      <div>
                        <strong>
                          #{ticket?.id} {ticket?.title}
                        </strong>
                        <span>{ticket?.school_name}</span>
                      </div>
                      <button type="button" className="secondary-button" onClick={() => ticket && onOpenTicket(ticket.id)}>
                        Open Ticket
                      </button>
                    </div>
                    <div className="comment-meta">
                      <span>{comment.author}</span>
                      <span>{comment.channel}</span>
                      <span className={`delivery-status-badge delivery-status-${comment.delivery_status.toLowerCase().replace(/\s+/g, "-")}`}>{comment.delivery_status}</span>
                      <span>{comment.recipient_name || comment.audience}</span>
                      {comment.next_follow_up_due ? (
                        <span className={isOverdue ? "overdue-label" : ""}>{isOverdue ? "Overdue — " : "Due "}
                          {formatTimestamp(comment.next_follow_up_due)}
                        </span>
                      ) : null}
                    </div>
                    {comment.recipient_contact ? (
                      <small className="comment-contact">{comment.recipient_contact}</small>
                    ) : null}
                    <p>{comment.body}</p>
                    <div className="comment-actions">
                      {["Sent", "Acknowledged", "Failed", "Follow-up Due"].map((status) => (
                        <button
                          key={status}
                          type="button"
                          className={comment.delivery_status === status ? "secondary-button active-chip" : "secondary-button"}
                          onClick={() =>
                            onUpdateCommentStatus(comment.id, status, comment.next_follow_up_due)
                          }
                        >
                          {status}
                        </button>
                      ))}
                      <label className="follow-up-input">
                        <span>Next follow-up</span>
                        <input
                          type="datetime-local"
                          value={
                            comment.next_follow_up_due
                              ? comment.next_follow_up_due.replace(" ", "T")
                              : ""
                          }
                          onChange={(event) =>
                            onUpdateCommentStatus(
                              comment.id,
                              event.target.value ? "Follow-up Due" : comment.delivery_status,
                              event.target.value,
                            )
                          }
                        />
                      </label>
                    </div>
                  </article>
                  );
                })}
              </div>
            ) : (
              <p className="empty-state compact">No communications match this queue.</p>
            )}
          </section>
        </div>
      </section>
    </div>
  );
}

function ReportBreakdown({
  counts,
  limit = 8,
}: {
  counts: Record<string, number>;
  limit?: number;
}) {
  const entries = Object.entries(counts)
    .sort(([, left], [, right]) => right - left)
    .slice(0, limit);

  if (entries.length === 0) {
    return <p>No data available.</p>;
  }

  return (
    <dl className="dashboard-breakdown">
      {entries.map(([label, value]) => (
        <ReportRow key={label} label={label} value={value} />
      ))}
    </dl>
  );
}

function ReportRow({ label, value }: { label: string; value: number | string }) {
  return (
    <div>
      <dt>{formatField(label)}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function countBy<T>(items: T[], getKey: (item: T) => string): Record<string, number> {
  return items.reduce<Record<string, number>>((counts, item) => {
    const key = getKey(item).trim() || "Unspecified";
    counts[key] = (counts[key] ?? 0) + 1;
    return counts;
  }, {});
}

function toLocalDateTimeKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day} ${hours}:${minutes}`;
}

function groupPlansBy(
  plans: SchoolClassPlan[],
  key: keyof Pick<SchoolClassPlan, "grade_level" | "lecture_model_name">,
) {
  return plans.reduce<Record<string, SchoolClassPlan[]>>((groups, plan) => {
    const groupName = String(plan[key] || "Unspecified");
    groups[groupName] = [...(groups[groupName] ?? []), plan];
    return groups;
  }, {});
}

type DirectoryPanelProps = {
  regions: Region[];
  schools: School[];
  onClose: () => void;
};

function contactKey(c: DirectoryContact) {
  return c.key;
}

export function DirectoryPanel({ regions, schools, onClose }: DirectoryPanelProps) {
  const contacts = React.useMemo(
    () => buildDirectoryContacts(regions, schools),
    [regions, schools],
  );

  const [filterRegions, setFilterRegions] = React.useState<Set<string>>(new Set());
  const [filterSchools, setFilterSchools] = React.useState<Set<string>>(new Set());
  const [filterVpCenters, setFilterVpCenters] = React.useState<Set<string>>(new Set());
  const [filterRoles, setFilterRoles] = React.useState<Set<string>>(new Set());
  const [nameQuery, setNameQuery] = React.useState("");
  const [selected, setSelected] = React.useState<Set<string>>(new Set());
  const [toast, setToast] = React.useState("");

  const regionOptions = React.useMemo(
    () => Array.from(new Set(contacts.flatMap((c) => c.regions).filter(Boolean))).sort(),
    [contacts],
  );
  const schoolOptions = React.useMemo(
    () => Array.from(new Set(contacts.flatMap((c) => c.schools).filter(Boolean))).sort(),
    [contacts],
  );
  const vpCenterOptions = React.useMemo(
    () => Array.from(new Set(contacts.flatMap((c) => c.vp_centers).filter(Boolean))).sort(),
    [contacts],
  );
  const roleOptions = React.useMemo(
    () => Array.from(new Set(contacts.flatMap((c) => c.roles).filter(Boolean))).sort(),
    [contacts],
  );

  const filtered = React.useMemo(() => {
    const q = nameQuery.trim().toLowerCase();
    return contacts.filter((c) => {
      if (filterRegions.size > 0 && !c.regions.some((r) => filterRegions.has(r))) return false;
      if (filterSchools.size > 0 && !c.schools.some((s) => filterSchools.has(s))) return false;
      if (filterVpCenters.size > 0 && !c.vp_centers.some((v) => filterVpCenters.has(v))) return false;
      if (filterRoles.size > 0 && !c.roles.some((r) => filterRoles.has(r))) return false;
      if (q && !c.name.toLowerCase().includes(q) && !c.email.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [contacts, filterRegions, filterSchools, filterVpCenters, filterRoles, nameQuery]);

  const filteredKeys = React.useMemo(() => filtered.map(contactKey), [filtered]);
  const selectableFiltered = React.useMemo(
    () => filtered.filter((c) => c.email),
    [filtered],
  );
  const allFilteredSelected =
    selectableFiltered.length > 0 &&
    selectableFiltered.every((c) => selected.has(contactKey(c)));

  function toggleAllFiltered() {
    setSelected((prev) => {
      const next = new Set(prev);
      if (allFilteredSelected) {
        for (const c of selectableFiltered) next.delete(contactKey(c));
      } else {
        for (const c of selectableFiltered) next.add(contactKey(c));
      }
      return next;
    });
  }

  function toggleOne(c: DirectoryContact) {
    if (!c.email) return;
    setSelected((prev) => {
      const next = new Set(prev);
      const key = contactKey(c);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  const selectedEmails = React.useMemo(() => {
    const wanted = new Set(selected);
    return contacts.filter((c) => wanted.has(contactKey(c)) && c.email).map((c) => c.email);
  }, [contacts, selected]);

  function flashToast(message: string) {
    setToast(message);
    setTimeout(() => setToast(""), 1800);
  }

  function copyEmails() {
    if (selectedEmails.length === 0) return;
    void navigator.clipboard.writeText(selectedEmails.join(", ")).then(() => {
      flashToast(`✓ Copied ${selectedEmails.length} email${selectedEmails.length === 1 ? "" : "s"}`);
    });
  }

  function bulkMail() {
    if (selectedEmails.length === 0) return;
    window.location.href = `mailto:${selectedEmails.join(",")}`;
  }

  function clearSelection() {
    setSelected(new Set());
  }

  function resetFilters() {
    setFilterRegions(new Set());
    setFilterSchools(new Set());
    setFilterVpCenters(new Set());
    setFilterRoles(new Set());
    setNameQuery("");
  }

  const anyFilterActive =
    filterRegions.size + filterSchools.size + filterVpCenters.size + filterRoles.size > 0 ||
    nameQuery.trim().length > 0;

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" aria-label="SIP contact directory">
        <header>
          <div>
            <h2>SIP Directory</h2>
            <p>{filtered.length} of {contacts.length} contacts visible</p>
          </div>
          <button type="button" onClick={onClose}>Close</button>
        </header>

        <div className="directory-filters">
          <DirectoryFilter label="Region" options={regionOptions} selected={filterRegions} onChange={setFilterRegions} />
          <DirectoryFilter label="School" options={schoolOptions} selected={filterSchools} onChange={setFilterSchools} />
          <DirectoryFilter label="VP Center" options={vpCenterOptions} selected={filterVpCenters} onChange={setFilterVpCenters} />
          <DirectoryFilter label="Role" options={roleOptions} selected={filterRoles} onChange={setFilterRoles} />
          <input
            type="search"
            className="directory-search"
            placeholder="Search name or email…"
            value={nameQuery}
            onChange={(e) => setNameQuery(e.target.value)}
          />
          {anyFilterActive ? (
            <button type="button" className="directory-filter-reset" onClick={resetFilters}>
              Clear filters
            </button>
          ) : null}
        </div>

        <div className="directory-select-bar">
          <label className="directory-select-all">
            <input
              type="checkbox"
              checked={allFilteredSelected}
              onChange={toggleAllFiltered}
              disabled={selectableFiltered.length === 0}
            />
            Select all visible ({selectableFiltered.length} with email)
          </label>
          {selected.size > 0 ? (
            <button type="button" className="directory-clear-selection" onClick={clearSelection}>
              Clear selection ({selected.size})
            </button>
          ) : null}
        </div>

        <div className="directory-grid">
          {filtered.map((contact) => {
            const key = contactKey(contact);
            const isSelected = selected.has(key);
            const hasEmail = !!contact.email;
            return (
              <article
                key={`${filteredKeys.indexOf(key)}-${key}`}
                className={`directory-card${isSelected ? " selected" : ""}${hasEmail ? "" : " no-email"}`}
              >
                <label className="directory-card-checkbox" title={hasEmail ? "Select" : "No email — cannot bulk-mail"}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    disabled={!hasEmail}
                    onChange={() => toggleOne(contact)}
                  />
                </label>
                <div className="directory-card-body">
                  <strong>{formatField(contact.name)}</strong>
                  <span>{contact.roles.join(" · ")}</span>
                  <small><ContactLink kind="tel" value={contact.mobile} /></small>
                  <small><ContactLink kind="mail" value={contact.email} /></small>
                  {contact.regions.length > 0 ? (
                    <small className="directory-card-meta">
                      <span className="directory-card-meta-label">Region:</span>{" "}
                      {contact.regions.join(", ")}
                    </small>
                  ) : null}
                  {contact.vp_centers.length > 0 ? (
                    <small className="directory-card-meta">
                      <span className="directory-card-meta-label">VP:</span>{" "}
                      {contact.vp_centers.join(", ")}
                    </small>
                  ) : null}
                  {contact.schools.length > 0 ? (
                    <details className="directory-card-schools">
                      <summary>
                        {contact.schools.length} school
                        {contact.schools.length === 1 ? "" : "s"}
                      </summary>
                      <ul>
                        {contact.schools.map((s) => (
                          <li key={s}>{s}</li>
                        ))}
                      </ul>
                    </details>
                  ) : null}
                </div>
              </article>
            );
          })}
          {filtered.length === 0 ? (
            <p className="empty-state">No contacts match the current filters.</p>
          ) : null}
        </div>

        {selected.size > 0 ? (
          <footer className="directory-action-bar">
            <span className="directory-action-count">{selected.size} selected</span>
            <button type="button" className="secondary-button" onClick={copyEmails}>
              Copy emails
            </button>
            <button type="button" className="primary-action" onClick={bulkMail}>
              Open mail (To:)
            </button>
          </footer>
        ) : null}

        {toast ? <div className="directory-toast">{toast}</div> : null}
      </section>
    </div>
  );
}

function DirectoryFilter({
  label,
  options,
  selected,
  onChange,
}: {
  label: string;
  options: string[];
  selected: Set<string>;
  onChange: (next: Set<string>) => void;
}) {
  const [open, setOpen] = React.useState(false);
  const [query, setQuery] = React.useState("");
  const filteredOptions = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    return options.filter((o) => o.toLowerCase().includes(q));
  }, [options, query]);

  function toggle(value: string) {
    const next = new Set(selected);
    if (next.has(value)) next.delete(value);
    else next.add(value);
    onChange(next);
  }

  const summary =
    selected.size === 0
      ? `All ${label.toLowerCase()}s`
      : selected.size === 1
        ? Array.from(selected)[0]
        : `${selected.size} selected`;

  return (
    <div className={`directory-filter${open ? " open" : ""}`}>
      <button
        type="button"
        className="directory-filter-trigger"
        onClick={() => setOpen((v) => !v)}
      >
        <span className="directory-filter-label">{label}:</span>
        <span className="directory-filter-summary">{summary}</span>
        <span className="directory-filter-chevron">▾</span>
      </button>
      {open ? (
        <div className="directory-filter-popover" role="listbox">
          <input
            type="search"
            className="directory-filter-search"
            placeholder={`Search ${label.toLowerCase()}…`}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoFocus
          />
          {selected.size > 0 ? (
            <button
              type="button"
              className="directory-filter-clear"
              onClick={() => onChange(new Set())}
            >
              Clear ({selected.size})
            </button>
          ) : null}
          <div className="directory-filter-options">
            {filteredOptions.map((opt) => (
              <label key={opt} className="directory-filter-option">
                <input
                  type="checkbox"
                  checked={selected.has(opt)}
                  onChange={() => toggle(opt)}
                />
                <span>{opt}</span>
              </label>
            ))}
            {filteredOptions.length === 0 ? (
              <span className="directory-filter-empty">No matches</span>
            ) : null}
          </div>
          <button
            type="button"
            className="directory-filter-done"
            onClick={() => setOpen(false)}
          >
            Done
          </button>
        </div>
      ) : null}
    </div>
  );
}

// Aggregated directory entry — one card per unique person, listing every
// (role, school) association so shared contacts (Central SPOCs, Center Heads
// covering multiple schools, Regional Heads spanning a whole region) appear
// once with full coverage shown.
type DirectoryContact = {
  key: string;
  name: string;
  mobile: string;
  email: string;
  roles: string[];
  schools: string[];
  regions: string[];
  vp_centers: string[];
};

function identityKey(name: string, mobile: string, email: string): string {
  const e = email.trim().toLowerCase();
  if (e) return `e:${e}`;
  const n = name.trim().toLowerCase();
  const m = mobile.trim();
  if (n && m) return `nm:${n}|${m}`;
  if (n) return `n:${n}`;
  if (m) return `m:${m}`;
  return "";
}

function buildDirectoryContacts(regions: Region[], schools: School[]): DirectoryContact[] {
  const groups = new Map<string, DirectoryContact>();

  function addAssociation(
    name: string,
    mobile: string,
    email: string,
    role: string,
    school: string,
    region: string,
    vp_center: string,
  ) {
    if (!name && !mobile && !email) return;
    const key = identityKey(name, mobile, email);
    if (!key) return;
    let entry = groups.get(key);
    if (!entry) {
      entry = {
        key,
        name: name.trim(),
        mobile: mobile.trim(),
        email: email.trim(),
        roles: [],
        schools: [],
        regions: [],
        vp_centers: [],
      };
      groups.set(key, entry);
    } else {
      // Promote name/mobile if the first association left them blank
      if (!entry.name && name) entry.name = name.trim();
      if (!entry.mobile && mobile) entry.mobile = mobile.trim();
      if (!entry.email && email) entry.email = email.trim();
    }
    if (role && !entry.roles.includes(role)) entry.roles.push(role);
    if (school && !entry.schools.includes(school)) entry.schools.push(school);
    if (region && !entry.regions.includes(region)) entry.regions.push(region);
    if (vp_center && !entry.vp_centers.includes(vp_center)) entry.vp_centers.push(vp_center);
  }

  // Regional heads — associate with every active school in that region.
  for (const region of regions) {
    const regionSchools = schools.filter((s) => s.region_id === region.id);
    const regionSchoolNames = regionSchools.map((s) => s.name);
    const regionVpCenters = Array.from(
      new Set(regionSchools.map((s) => s.mapped_vp_center).filter(Boolean)),
    );
    for (const role of ["Regional Academic Head", "Regional Business Head"] as const) {
      const isAcademic = role === "Regional Academic Head";
      const name = isAcademic ? region.regional_academic_head_name : region.regional_business_head_name;
      const mobile = isAcademic ? region.regional_academic_head_mobile : region.regional_business_head_mobile;
      const email = isAcademic ? region.regional_academic_head_email : region.regional_business_head_email;
      if (!name && !mobile && !email) continue;
      const key = identityKey(name, mobile, email);
      if (!key) continue;
      // Merge all schools from the region into this contact's entry
      let entry = groups.get(key);
      if (!entry) {
        entry = {
          key,
          name: name.trim(),
          mobile: mobile.trim(),
          email: email.trim(),
          roles: [],
          schools: [],
          regions: [],
          vp_centers: [],
        };
        groups.set(key, entry);
      }
      if (!entry.roles.includes(role)) entry.roles.push(role);
      if (region.name && !entry.regions.includes(region.name)) entry.regions.push(region.name);
      for (const sn of regionSchoolNames) {
        if (!entry.schools.includes(sn)) entry.schools.push(sn);
      }
      for (const vp of regionVpCenters) {
        if (!entry.vp_centers.includes(vp)) entry.vp_centers.push(vp);
      }
    }
  }

  // School-level contacts — every role on every school.
  for (const school of schools) {
    const region = school.region_name;
    const vp = school.mapped_vp_center;
    const slots: Array<[string, string, string, string]> = [
      [school.sip_academic_owner_role || "SIP Academic Head / Lead", school.sip_academic_owner_name, school.sip_academic_owner_mobile, school.sip_academic_owner_email],
      ["Academic Operations Manager", school.aom_name, school.aom_mobile, school.aom_email],
      ["Center Head", school.center_head_name, school.center_head_mobile, school.center_head_email],
      ["BH", school.bh_name, school.bh_mobile, school.bh_email],
      ["School Principal", school.principal_name, school.principal_mobile, school.principal_email],
      ["School SPOC", school.school_spoc_name, school.school_spoc_mobile, school.school_spoc_email],
      ["Central SIP Academic Team SPOC", school.central_academic_spoc_name, school.central_academic_spoc_mobile, school.central_academic_spoc_email],
      ["Central Business Team SPOC", school.central_business_spoc_name, school.central_business_spoc_mobile, school.central_business_spoc_email],
    ];
    for (const [role, name, mobile, email] of slots) {
      addAssociation(name, mobile, email, role, school.name, region, vp);
    }
  }

  // Sort each entry's lists for stable display
  const out = Array.from(groups.values());
  for (const c of out) {
    c.roles.sort();
    c.schools.sort();
    c.regions.sort();
    c.vp_centers.sort();
  }
  out.sort((a, b) => a.name.localeCompare(b.name));
  return out;
}

type SlaPolicyPanelProps = {
  policies: SlaPolicy[];
  onClose: () => void;
  onSave: (issueCategory: string, hours: number) => void;
};

export function SlaPolicyPanel({ policies, onClose, onSave }: SlaPolicyPanelProps) {
  const [drafts, setDrafts] = React.useState<Record<string, number>>({});

  React.useEffect(() => {
    setDrafts(
      Object.fromEntries(policies.map((policy) => [policy.issue_category, policy.hours])),
    );
  }, [policies]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal sla-policy-modal" aria-label="SLA policy settings">
        <header>
          <div>
            <h2>SLA Policies</h2>
            <p>Set response windows for school-program issue categories.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="sla-policy-grid">
          {policies.map((policy) => (
            <form
              className="sla-policy-row"
              key={policy.issue_category}
              onSubmit={(event) => {
                event.preventDefault();
                onSave(policy.issue_category, drafts[policy.issue_category] ?? policy.hours);
              }}
            >
              <strong>{policy.issue_category}</strong>
              <label>
                Hours
                <input
                  min={1}
                  max={720}
                  type="number"
                  value={drafts[policy.issue_category] ?? policy.hours}
                  onChange={(event) =>
                    setDrafts((current) => ({
                      ...current,
                      [policy.issue_category]: Number(event.target.value),
                    }))
                  }
                />
              </label>
              <button type="submit">Save</button>
            </form>
          ))}
        </div>
      </section>
    </div>
  );
}

type AssignmentRulePanelProps = {
  rules: AssignmentRule[];
  onClose: () => void;
  onSave: (queue: string, assignee: string, isActive: boolean) => void;
};

export function AssignmentRulePanel({ rules, onClose, onSave }: AssignmentRulePanelProps) {
  const [drafts, setDrafts] = React.useState<Record<string, { assignee: string; is_active: boolean }>>(
    {},
  );

  React.useEffect(() => {
    setDrafts(
      Object.fromEntries(
        rules.map((rule) => [
          rule.queue,
          { assignee: rule.assignee, is_active: rule.is_active },
        ]),
      ),
    );
  }, [rules]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal assignment-rule-modal" aria-label="Routing rules">
        <header>
          <div>
            <h2>Routing Rules</h2>
            <p>Set default owners for queues when new tickets are created.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="assignment-rule-grid">
          {rules.map((rule) => {
            const draft = drafts[rule.queue] ?? {
              assignee: rule.assignee,
              is_active: rule.is_active,
            };

            return (
              <form
                className="assignment-rule-row"
                key={rule.queue}
                onSubmit={(event) => {
                  event.preventDefault();
                  onSave(rule.queue, draft.assignee, draft.is_active);
                }}
              >
                <strong>{rule.queue}</strong>
                <label>
                  Default owner
                  <input
                    required
                    value={draft.assignee}
                    onChange={(event) =>
                      setDrafts((current) => ({
                        ...current,
                        [rule.queue]: {
                          ...draft,
                          assignee: event.target.value,
                        },
                      }))
                    }
                  />
                </label>
                <label className="inline-toggle">
                  <input
                    checked={draft.is_active}
                    type="checkbox"
                    onChange={(event) =>
                      setDrafts((current) => ({
                        ...current,
                        [rule.queue]: {
                          ...draft,
                          is_active: event.target.checked,
                        },
                      }))
                    }
                  />
                  Active
                </label>
                <button type="submit">Save</button>
              </form>
            );
          })}
        </div>
      </section>
    </div>
  );
}

type EscalationPolicyPanelProps = {
  policy: EscalationPolicy;
  onClose: () => void;
  onSave: (input: {
    at_risk_hours: number;
    escalation_assignee: string;
    auto_assign_on_breach: boolean;
  }) => void;
};

export function EscalationPolicyPanel({
  policy,
  onClose,
  onSave,
}: EscalationPolicyPanelProps) {
  const [draft, setDraft] = React.useState({
    at_risk_hours: policy.at_risk_hours,
    escalation_assignee: policy.escalation_assignee,
    auto_assign_on_breach: policy.auto_assign_on_breach,
  });

  React.useEffect(() => {
    setDraft({
      at_risk_hours: policy.at_risk_hours,
      escalation_assignee: policy.escalation_assignee,
      auto_assign_on_breach: policy.auto_assign_on_breach,
    });
  }, [policy]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal escalation-policy-modal" aria-label="Escalation policy">
        <header>
          <div>
            <h2>Escalation Policy</h2>
            <p>Configure SLA at-risk threshold and breach ownership.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <form
          className="escalation-policy-form"
          onSubmit={(event) => {
            event.preventDefault();
            onSave(draft);
          }}
        >
          <label>
            At-risk threshold hours
            <input
              min={1}
              max={720}
              required
              type="number"
              value={draft.at_risk_hours}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  at_risk_hours: Number(event.target.value),
                }))
              }
            />
          </label>
          <label>
            Escalation owner
            <input
              required
              value={draft.escalation_assignee}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  escalation_assignee: event.target.value,
                }))
              }
            />
          </label>
          <label className="inline-toggle">
            <input
              checked={draft.auto_assign_on_breach}
              type="checkbox"
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  auto_assign_on_breach: event.target.checked,
                }))
              }
            />
            Auto-assign breached tickets
          </label>
          <button type="submit">Save Policy</button>
        </form>
      </section>
    </div>
  );
}

type CommunicationTemplatePanelProps = {
  templates: CommunicationTemplate[];
  onClose: () => void;
  onSave: (input: {
    id?: number;
    name: string;
    audience: string;
    body: string;
    is_active: boolean;
  }) => void;
};

const emptyTemplateDraft = {
  id: undefined as number | undefined,
  name: "",
  audience: "Parent",
  body: "",
  is_active: true,
};

export function CommunicationTemplatePanel({
  templates,
  onClose,
  onSave,
}: CommunicationTemplatePanelProps) {
  const [draft, setDraft] = React.useState(emptyTemplateDraft);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal template-modal" aria-label="Communication templates">
        <header>
          <div>
            <h2>Communication Templates</h2>
            <p>Manage reusable replies for parent, student, school, and internal updates.</p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <form
          className="template-editor"
          onSubmit={(event) => {
            event.preventDefault();
            onSave(draft);
            setDraft(emptyTemplateDraft);
          }}
        >
          <label>
            Template name
            <input
              required
              value={draft.name}
              onChange={(event) =>
                setDraft((current) => ({ ...current, name: event.target.value }))
              }
            />
          </label>
          <label>
            Audience
            <select
              value={draft.audience}
              onChange={(event) =>
                setDraft((current) => ({ ...current, audience: event.target.value }))
              }
            >
              <option>Parent</option>
              <option>Student</option>
              <option>School</option>
              <option>Internal</option>
            </select>
          </label>
          <label className="inline-toggle">
            <input
              checked={draft.is_active}
              type="checkbox"
              onChange={(event) =>
                setDraft((current) => ({ ...current, is_active: event.target.checked }))
              }
            />
            Active
          </label>
          <label className="template-body-field">
            Body
            <textarea
              required
              value={draft.body}
              onChange={(event) =>
                setDraft((current) => ({ ...current, body: event.target.value }))
              }
            />
          </label>
          <div className="actions">
            <button type="button" onClick={() => setDraft(emptyTemplateDraft)}>
              New
            </button>
            <button className="primary-action" type="submit">
              Save Template
            </button>
          </div>
        </form>

        <div className="template-list">
          {templates.map((template) => (
            <button
              aria-label={`${template.name} ${template.audience} ${template.is_active ? "Active" : "Inactive"}`}
              className="template-row"
              key={template.id}
              onClick={() =>
                setDraft({
                  id: template.id,
                  name: template.name,
                  audience: template.audience,
                  body: template.body,
                  is_active: template.is_active,
                })
              }
            >
              <strong>{template.name}</strong>
              <small>
                {template.audience} - {template.is_active ? "Active" : "Inactive"}
              </small>
            </button>
          ))}
        </div>
      </section>
    </div>
  );
}

type SlaBreachAlertProps = {
  newBreachCount: number;
  onView: () => void;
  onDismiss: () => void;
};

export function SlaBreachAlert({ newBreachCount, onView, onDismiss }: SlaBreachAlertProps) {
  if (newBreachCount === 0) return null;
  return (
    <div className="sla-breach-alert" role="alert">
      <strong>
        {newBreachCount} ticket{newBreachCount > 1 ? "s" : ""} just breached SLA
      </strong>
      <div className="sla-breach-actions">
        <button type="button" className="primary-action" onClick={onView}>
          View
        </button>
        <button type="button" onClick={onDismiss}>
          Dismiss
        </button>
      </div>
    </div>
  );
}

type MetricsProps = {
  activeSchoolCount: number;
  activeQueueCount: number;
  escalatedCount: number;
  openCount: number;
  pendingSlaCount: number;
  unassignedCount: number;
};

export function Metrics({
  activeSchoolCount,
  activeQueueCount,
  escalatedCount,
  openCount,
  pendingSlaCount,
  unassignedCount,
}: MetricsProps) {
  return (
    <div className="metrics">
      <div>
        <strong>{openCount}</strong>
        <span>Open workload</span>
      </div>
      <div>
        <strong>{activeSchoolCount}</strong>
        <span>Active schools</span>
      </div>
      <div>
        <strong>{activeQueueCount}</strong>
        <span>Active queues</span>
      </div>
      <div>
        <strong>{pendingSlaCount}</strong>
        <span>Pending SLA</span>
      </div>
      <div>
        <strong>{escalatedCount}</strong>
        <span>Escalated</span>
      </div>
      <div>
        <strong>{unassignedCount}</strong>
        <span>Unassigned</span>
      </div>
    </div>
  );
}

type ProgramFiltersProps = {
  filters: ProgramScopeFilters;
  schoolOptions: string[];
  dateFrom: string;
  dateTo: string;
  onChange: React.Dispatch<React.SetStateAction<ProgramScopeFilters>>;
  onDateFromChange: (v: string) => void;
  onDateToChange: (v: string) => void;
  onReset: () => void;
};

export function ProgramFilters({
  filters,
  schoolOptions,
  dateFrom,
  dateTo,
  onChange,
  onDateFromChange,
  onDateToChange,
  onReset,
}: ProgramFiltersProps) {
  return (
    <details className="filter-panel">
      <summary>Filters</summary>
      <section className="program-filters" aria-label="School program filters">
        <fieldset>
          <legend>Student context</legend>
          <div className="filter-group-grid">
            <label>
              School
              <select
                value={filters.school_name}
                onChange={(event) =>
                  onChange((current) => ({
                    ...current,
                    school_name: event.target.value,
                  }))
                }
              >
                <option value="">All schools</option>
                {schoolOptions.map((school) => (
                  <option key={school}>{school}</option>
                ))}
              </select>
            </label>
            <label>
              Grade
              <select
                value={filters.grade_level}
                onChange={(event) =>
                  onChange((current) => ({
                    ...current,
                    grade_level: event.target.value,
                  }))
                }
              >
                <option value="">All grades</option>
                {gradeLevels.map((grade) => (
                  <option key={grade}>{grade}</option>
                ))}
              </select>
            </label>
            <label>
              Program
              <select
                value={filters.program_track}
                onChange={(event) =>
                  onChange((current) => ({
                    ...current,
                    program_track: event.target.value,
                  }))
                }
              >
                <option value="">All programs</option>
                {programTracks.map((track) => (
                  <option key={track}>{track}</option>
                ))}
              </select>
            </label>
          </div>
        </fieldset>
        <fieldset>
          <legend>Operational routing</legend>
          <div className="filter-group-grid compact-filter-grid">
            <label>
              Category
              <select
                value={filters.issue_category}
                onChange={(event) =>
                  onChange((current) => ({
                    ...current,
                    issue_category: event.target.value,
                  }))
                }
              >
                <option value="">All categories</option>
                {issueCategories.map((category) => (
                  <option key={category}>{category}</option>
                ))}
              </select>
            </label>
            <label>
              Queue
              <select
                value={filters.queue}
                onChange={(event) =>
                  onChange((current) => ({
                    ...current,
                    queue: event.target.value,
                  }))
                }
              >
                <option value="">All queues</option>
                {queues.map((queue) => (
                  <option key={queue}>{queue}</option>
                ))}
              </select>
            </label>
          </div>
        </fieldset>
        <fieldset>
          <legend>Date range (created)</legend>
          <div className="filter-group-grid compact-filter-grid">
            <label>
              From
              <input
                type="date"
                value={dateFrom}
                onChange={(e) => onDateFromChange(e.target.value)}
              />
            </label>
            <label>
              To
              <input
                type="date"
                value={dateTo}
                onChange={(e) => onDateToChange(e.target.value)}
              />
            </label>
          </div>
        </fieldset>
        <div className="filter-actions">
          <button type="button" onClick={onReset}>
            Reset Filters
          </button>
        </div>
      </section>
    </details>
  );
}

type TicketListProps = {
  activeFilter: Filter;
  currentUser: import("./types").CurrentUser | null;
  selectedId: number | null;
  tickets: Ticket[];
  onSelectTicket: (id: number) => void;
  onQuickResolve: (id: number) => void;
  onQuickAssign: (id: number) => void;
};

const emptyStateMessages: Record<Filter, string> = {
  Inbox: "No open tickets — everything is resolved.",
  "My Tickets": "No tickets assigned to you right now.",
  Unassigned: "All tickets are assigned.",
  "Pending SLA": "No SLA issues — all tickets are within policy.",
  Escalated: "No escalated tickets.",
  Resolved: "No resolved or closed tickets yet.",
};

function ticketAgeClass(ticket: Ticket): string {
  if (["Resolved", "Closed"].includes(ticket.status)) return "";
  const created = new Date(ticket.created_at.replace(" ", "T"));
  const ageHours = (Date.now() - created.getTime()) / 3_600_000;
  if (ageHours > 72) return "age-old";
  if (ageHours > 24) return "age-aging";
  return "";
}

export function TicketList({
  activeFilter,
  currentUser,
  selectedId,
  tickets,
  onSelectTicket,
  onQuickResolve,
  onQuickAssign,
}: TicketListProps) {
  return (
    <section className="ticket-list" aria-label="Ticket list">
      <div className="section-heading">
        <h1>{activeFilter}</h1>
        <span>{tickets.length} tickets</span>
      </div>
      {tickets.length > 0 ? (
        tickets.map((ticket) => {
          const isResolved = ["Resolved", "Closed"].includes(ticket.status);
          const isAssignedToMe = currentUser && ticket.assignee === currentUser.display_name;
          return (
            <button
              className={`ticket-row ${ticket.id === selectedId ? "selected" : ""} ${ticketAgeClass(ticket)}`}
              key={ticket.id}
              onClick={() => onSelectTicket(ticket.id)}
            >
              <span className={`priority priority-${ticket.priority.toLowerCase()}`} />
              <span>
                <strong>{ticket.title}</strong>
                <small>#{ticket.id} · {ticket.requester} · {formatTimestamp(ticket.updated_at)}</small>
                <small>{ticket.school_name} · {ticket.queue} · SLA {getSlaState(ticket)}</small>
              </span>
              <em className={`ticket-status ticket-status-${ticket.status.toLowerCase().replace(/\s+/g, "-")}`}>
                {ticket.status}
              </em>
              {!isResolved && (
                <span className="ticket-row-actions" onClick={(e) => e.stopPropagation()}>
                  {!isAssignedToMe && currentUser && (
                    <button
                      type="button"
                      className="quick-action"
                      title="Assign to me"
                      onClick={(e) => { e.stopPropagation(); onQuickAssign(ticket.id); }}
                    >
                      Assign to me
                    </button>
                  )}
                  <button
                    type="button"
                    className="quick-action quick-action-resolve"
                    title="Resolve ticket"
                    onClick={(e) => { e.stopPropagation(); onQuickResolve(ticket.id); }}
                  >
                    Resolve
                  </button>
                </span>
              )}
            </button>
          );
        })
      ) : (
        <p className="empty-state">{emptyStateMessages[activeFilter] ?? "No tickets match this view."}</p>
      )}
    </section>
  );
}

type StudentTimelinePanelProps = {
  timeline: StudentTimeline;
  onClose: () => void;
};

export function StudentTimelinePanel({ timeline, onClose }: StudentTimelinePanelProps) {
  const openTickets = timeline.tickets.filter((ticket) => ticket.status !== "Closed").length;
  const escalatedTickets = timeline.tickets.filter(
    (ticket) => ticket.escalation_status === "Escalated" && ticket.status !== "Closed",
  ).length;

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal student-timeline-modal" aria-label="Student timeline">
        <header>
          <div>
            <h2>{timeline.student_name}</h2>
            <p>
              {timeline.school_name} - {timeline.tickets.length} tickets, {openTickets} open,{" "}
              {escalatedTickets} escalated
            </p>
          </div>
          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="timeline-summary">
          <strong>{timeline.comments.length}</strong>
          <span>Comments</span>
          <strong>{timeline.history.length}</strong>
          <span>History events</span>
          <strong>{timeline.attachments.length}</strong>
          <span>Attachments</span>
        </div>

        <div className="student-timeline-list">
          {timeline.tickets.length > 0 ? (
            timeline.tickets.map((ticket) => (
              <article className="student-timeline-item" key={ticket.id}>
                <div>
                  <strong>
                    #{ticket.id} {ticket.title}
                  </strong>
                  <small>
                    {ticket.issue_category} - {ticket.queue} - {ticket.priority}
                  </small>
                </div>
                <div className="timeline-status">
                  <span>{ticket.status}</span>
                  <span>SLA {getSlaState(ticket)}</span>
                  <span>{ticket.escalation_status}</span>
                </div>
                <p>{ticket.description}</p>
                <small>
                  Created {formatTimestamp(ticket.created_at)} - Updated{" "}
                  {formatTimestamp(ticket.updated_at)}
                </small>
              </article>
            ))
          ) : (
            <p className="empty-state compact">No tickets recorded for this student.</p>
          )}
        </div>
      </section>
    </div>
  );
}

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = React.useState(false);
  function handleCopy() {
    void navigator.clipboard.writeText(value).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }
  return (
    <button type="button" className="copy-btn" onClick={handleCopy} title={`Copy ${value}`}>
      {copied ? "✓" : "⎘"}
    </button>
  );
}

// Phone numbers as tel: links and emails as mailto: links — clickable on mobile
// to launch dialer/mail, and accompanied by a copy button so users can
// alternate between calling/mailing and copying the bare value.
function ContactLink({
  kind,
  value,
  withCopy = true,
}: {
  kind: "tel" | "mail";
  value: string | null | undefined;
  withCopy?: boolean;
}) {
  const trimmed = (value ?? "").trim();
  if (!trimmed) {
    return <span className="contact-link contact-link-empty">—</span>;
  }
  // For tel: strip spaces, dashes, parens; keep leading + for country code
  const href =
    kind === "tel"
      ? `tel:${trimmed.replace(/[^\d+]/g, "")}`
      : `mailto:${trimmed}`;
  return (
    <span className={`contact-link-wrap contact-link-${kind}`}>
      <a
        href={href}
        className="contact-link"
        onClick={(e) => e.stopPropagation()}
      >
        {trimmed}
      </a>
      {withCopy ? <CopyButton value={trimmed} /> : null}
    </span>
  );
}

function SchoolContactsBar({ school }: { school: School | null }) {
  if (!school) return null;
  const contacts = [
    { role: "SPOC", name: school.school_spoc_name, mobile: school.school_spoc_mobile, email: school.school_spoc_email },
    { role: "Principal", name: school.principal_name, mobile: school.principal_mobile, email: school.principal_email },
    { role: "Center Head", name: school.center_head_name, mobile: school.center_head_mobile, email: school.center_head_email },
  ].filter((c) => c.name);
  if (contacts.length === 0) return null;
  return (
    <details className="school-contacts-bar">
      <summary>{school.name} — contacts</summary>
      <div className="school-contacts-list">
        {contacts.map((c) => (
          <div key={c.role} className="school-contact-row">
            <span className="contact-role">{c.role}</span>
            <span className="contact-name">{c.name}</span>
            {c.mobile ? (
              <span className="contact-field">
                <ContactLink kind="tel" value={c.mobile} />
              </span>
            ) : null}
            {c.email ? (
              <span className="contact-field">
                <ContactLink kind="mail" value={c.email} />
              </span>
            ) : null}
          </div>
        ))}
      </div>
    </details>
  );
}

type TicketDetailProps = {
  assigneeDraft: string;
  assigneeWorkload: Record<string, number>;
  attachmentDraft: AttachmentDraft;
  attachments: TicketAttachment[];
  comments: TicketComment[];
  editDraft: TicketEditDraft;
  history: TicketHistory[];
  isConfirmingDelete: boolean;
  isEditing: boolean;
  reply: ReplyDraft;
  selected: Ticket | null;
  schools: School[];
  students: Student[];
  templates: CommunicationTemplate[];
  onAddAttachment: (event: React.FormEvent<HTMLFormElement>) => void;
  onAddComment: (isInternal: boolean) => void;
  onAssigneeDraftChange: (value: string) => void;
  onBrowseAttachment: () => void;
  onCancelDelete: () => void;
  onCancelEdit: () => void;
  onConfirmDelete: () => void;
  onEditDraftChange: React.Dispatch<React.SetStateAction<TicketEditDraft>>;
  onOpenAttachment: (path: string) => void;
  onReplyChange: React.Dispatch<React.SetStateAction<ReplyDraft>>;
  onRequestDelete: () => void;
  onRequestEdit: () => void;
  onRequestStudentTimeline: () => void;
  onSaveEdits: (event: React.FormEvent<HTMLFormElement>) => void;
  onSetAttachmentDraft: React.Dispatch<React.SetStateAction<AttachmentDraft>>;
  onUpdateCommentStatus: (id: number, deliveryStatus: string, nextFollowUpDue?: string) => void;
  onUpdateTicket: (changes: TicketChanges) => void;
};

export function TicketDetail(props: TicketDetailProps) {
  const {
    assigneeDraft,
    assigneeWorkload,
    attachmentDraft,
    attachments,
    comments,
    editDraft,
    history,
    isConfirmingDelete,
    isEditing,
    reply,
    selected,
    schools,
    students,
    templates,
    onAddAttachment,
    onAddComment,
    onAssigneeDraftChange,
    onBrowseAttachment,
    onCancelDelete,
    onCancelEdit,
    onConfirmDelete,
    onEditDraftChange,
    onOpenAttachment,
    onReplyChange,
    onRequestDelete,
    onRequestEdit,
    onRequestStudentTimeline,
    onSaveEdits,
    onSetAttachmentDraft,
    onUpdateCommentStatus,
    onUpdateTicket,
  } = props;

  const [detailTab, setDetailTab] = React.useState<"info" | "conversation">("info");

  return (
    <section className="ticket-detail" aria-label="Ticket detail">
      {selected ? (
        <>
          <div className="detail-header">
            <div className="detail-header-title">
              <span className="ticket-id">#{selected.id}</span>
              <h2>{selected.title}</h2>
            </div>
            <button className="danger-action detail-delete-btn" onClick={onRequestDelete}>
              Delete
            </button>
          </div>

          {isConfirmingDelete ? (
            <div className="confirm-delete">
              <span>This permanently deletes the ticket and its notes.</span>
              <button onClick={onCancelDelete}>Cancel</button>
              <button className="danger-action" onClick={onConfirmDelete}>
                Confirm Delete
              </button>
            </div>
          ) : null}

          <div className="detail-tabs">
            <button
              className={`detail-tab${detailTab === "info" ? " active" : ""}`}
              onClick={() => setDetailTab("info")}
            >
              Info
            </button>
            <button
              className={`detail-tab${detailTab === "conversation" ? " active" : ""}`}
              onClick={() => setDetailTab("conversation")}
            >
              Conversation
            </button>
          </div>

          <div className={`detail-tab-panel detail-tab-info${detailTab === "info" ? " visible" : ""}`}>
          <dl className="metadata">
            <label>
              <dt>Status</dt>
              <dd>
                <select
                  value={selected.status}
                  onChange={(event) => onUpdateTicket({ status: event.target.value as Status })}
                >
                  {statuses.map((status) => (
                    <option key={status}>{status}</option>
                  ))}
                </select>
              </dd>
            </label>
            <label>
              <dt>Priority</dt>
              <dd>
                <select
                  value={selected.priority}
                  onChange={(event) =>
                    onUpdateTicket({ priority: event.target.value as Priority })
                  }
                >
                  {priorities.map((priority) => (
                    <option key={priority}>{priority}</option>
                  ))}
                </select>
              </dd>
            </label>
            <label>
              <dt>Requester</dt>
              <dd>{selected.requester}</dd>
            </label>
            <label>
              <dt>School</dt>
              <dd>{selected.school_name}</dd>
            </label>
            <label>
              <dt>Student</dt>
              <dd>
                <button className="link-action" onClick={onRequestStudentTimeline}>
                  {selected.student_name}
                </button>
              </dd>
            </label>
            <label>
              <dt>Grade</dt>
              <dd>{selected.grade_level}</dd>
            </label>
            <label>
              <dt>Program</dt>
              <dd>{selected.program_track}</dd>
            </label>
            <label>
              <dt>Category</dt>
              <dd>{selected.issue_category}</dd>
            </label>
            <label>
              <dt>Queue</dt>
              <dd>
                <select
                  value={selected.queue}
                  onChange={(event) => onUpdateTicket({ queue: event.target.value as Queue })}
                >
                  {queues.map((queue) => (
                    <option key={queue}>{queue}</option>
                  ))}
                </select>
              </dd>
            </label>
            <label>
              <dt>SLA</dt>
              <dd>
                <span className={`sla-state sla-state-${getSlaState(selected).toLowerCase().replace(" ", "-")}`}>
                  {getSlaState(selected)}
                </span>
                {selected.sla_due_at ? (
                  <span className="sla-countdown"> — {formatSlaCountdown(selected.sla_due_at)}</span>
                ) : null}
              </dd>
            </label>
            <label>
              <dt>Escalation</dt>
              <dd>
                {selected.escalation_status}
                {selected.escalated_at ? ` since ${formatTimestamp(selected.escalated_at)}` : ""}
              </dd>
            </label>
            <label>
              <dt>Assignee</dt>
              <dd>
                <input
                  list="assignee-options"
                  value={assigneeDraft}
                  onBlur={() => onUpdateTicket({ assignee: assigneeDraft.trim() || "Unassigned" })}
                  onChange={(event) => onAssigneeDraftChange(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.currentTarget.blur();
                    }
                  }}
                />
                <datalist id="assignee-options">
                  {Object.entries(assigneeWorkload)
                    .sort((a, b) => a[1] - b[1])
                    .map(([name, count]) => (
                      <option key={name} value={name}>
                        {count} open ticket{count !== 1 ? "s" : ""}
                      </option>
                    ))}
                </datalist>
              </dd>
            </label>
          </dl>

          <SchoolContactsBar school={schools.find((s) => s.id === selected.school_id) ?? null} />
          </div>{/* end detail-tab-info */}

          <div className={`detail-tab-panel detail-tab-conversation${detailTab === "conversation" ? " visible" : ""}`}>
          <div className="conversation">
            {isEditing ? (
              <TicketEditForm
                draft={editDraft}
                schools={schools}
                students={students}
                onCancel={onCancelEdit}
                onDraftChange={onEditDraftChange}
                onSubmit={onSaveEdits}
              />
            ) : (
              <article>
                <div className="article-header">
                  <strong>Initial request</strong>
                  <button onClick={onRequestEdit}>Edit</button>
                </div>
                <p>{selected.description}</p>
                <small>
                  Requested by {selected.requester} - Created{" "}
                  {formatTimestamp(selected.created_at)}
                </small>
              </article>
            )}

            {comments.map((comment) => (
              <article className={comment.is_internal ? "internal-note" : ""} key={comment.id}>
                <strong>
                  {comment.author} {comment.is_internal ? "(internal note)" : ""}
                </strong>
                <div className="comment-meta">
                  <span>{comment.channel}</span>
                  <span>{comment.audience}</span>
                  {comment.recipient_name ? <span>To: {comment.recipient_name}</span> : null}
                  <span>{comment.delivery_status}</span>
                  {comment.last_contacted_at ? (
                    <span>Last contact: {formatTimestamp(comment.last_contacted_at)}</span>
                  ) : null}
                  {comment.next_follow_up_due ? (
                    <span>Next follow-up: {formatTimestamp(comment.next_follow_up_due)}</span>
                  ) : null}
                </div>
                {comment.recipient_contact ? (
                  <small className="comment-contact">{comment.recipient_contact}</small>
                ) : null}
                <p>{comment.body}</p>
                {!comment.is_internal ? (
                  <div className="comment-actions">
                    {["Sent", "Failed", "Acknowledged", "Follow-up Due"].map((status) => (
                      <button
                        type="button"
                        className={comment.delivery_status === status ? "secondary-button active-chip" : "secondary-button"}
                        key={status}
                        onClick={() => onUpdateCommentStatus(comment.id, status, comment.next_follow_up_due)}
                      >
                        {status}
                      </button>
                    ))}
                    <label className="follow-up-input">
                      <span>Next follow-up</span>
                      <input
                        type="datetime-local"
                        value={comment.next_follow_up_due ? comment.next_follow_up_due.replace(" ", "T") : ""}
                        onChange={(event) =>
                          onUpdateCommentStatus(
                            comment.id,
                            event.target.value ? "Follow-up Due" : comment.delivery_status,
                            event.target.value,
                          )
                        }
                      />
                    </label>
                  </div>
                ) : null}
                <small>{formatTimestamp(comment.created_at)}</small>
              </article>
            ))}

            <ReplyBox
              draft={reply}
              templates={templates}
              onAddComment={onAddComment}
              onDraftChange={onReplyChange}
            />

            <AttachmentsPanel
              attachmentDraft={attachmentDraft}
              attachments={attachments}
              onAddAttachment={onAddAttachment}
              onBrowseAttachment={onBrowseAttachment}
              onOpenAttachment={onOpenAttachment}
              onSetAttachmentDraft={onSetAttachmentDraft}
            />

            <HistoryPanel history={history} />
          </div>
          </div>{/* end detail-tab-conversation */}
        </>
      ) : (
        <p className="empty-state">Create or select a ticket to begin.</p>
      )}
    </section>
  );
}

type TicketEditFormProps = {
  draft: TicketEditDraft;
  schools: School[];
  students: Student[];
  onCancel: () => void;
  onDraftChange: React.Dispatch<React.SetStateAction<TicketEditDraft>>;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
};

function TicketEditForm({
  draft,
  schools,
  students,
  onCancel,
  onDraftChange,
  onSubmit,
}: TicketEditFormProps) {
  const selectedSchool = schools.find((school) => school.id === draft.school_id);
  const studentOptions = selectedSchool
    ? students.filter((student) => student.school_id === selectedSchool.id)
    : students;

  return (
    <form className="edit-panel" onSubmit={onSubmit}>
      <label>
        Title
        <input
          required
          value={draft.title}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              title: event.target.value,
            }))
          }
        />
      </label>
      <label>
        Requester
        <input
          required
          value={draft.requester}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              requester: event.target.value,
            }))
          }
        />
      </label>
      <label>
        School
        <select
          required
          value={draft.school_id ?? ""}
          onChange={(event) => {
            const school = schools.find((item) => item.id === Number(event.target.value));
            onDraftChange((current) => ({
              ...current,
              school_id: school?.id ?? null,
              school_name: school?.name ?? "",
              student_name: "",
            }));
          }}
        >
          <option value="">Select school</option>
          {schools.map((school) => (
            <option key={school.id} value={school.id}>
              {school.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        Student
        <select
          required
          value={draft.student_name}
          onChange={(event) => {
            const student = studentOptions.find((item) => item.name === event.target.value);
            onDraftChange((current) => ({
              ...current,
              student_name: event.target.value,
              grade_level: student?.grade_level ?? current.grade_level,
              program_track: student?.program_track ?? current.program_track,
            }));
          }}
        >
          <option value="">Select student</option>
          {studentOptions.map((student) => (
            <option key={student.id}>{student.name}</option>
          ))}
        </select>
      </label>
      <label>
        Grade
        <select
          value={draft.grade_level}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              grade_level: event.target.value,
            }))
          }
        >
          {gradeLevels.map((grade) => (
            <option key={grade}>{grade}</option>
          ))}
        </select>
      </label>
      <label>
        Program
        <select
          value={draft.program_track}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              program_track: event.target.value,
            }))
          }
        >
          {programTracks.map((track) => (
            <option key={track}>{track}</option>
          ))}
        </select>
      </label>
      <label>
        Category
        <select
          value={draft.issue_category}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              issue_category: event.target.value,
            }))
          }
        >
          {issueCategories.map((category) => (
            <option key={category}>{category}</option>
          ))}
        </select>
      </label>
      <label>
        Description
        <textarea
          required
          value={draft.description}
          onChange={(event) =>
            onDraftChange((current) => ({
              ...current,
              description: event.target.value,
            }))
          }
        />
      </label>
      <div className="actions">
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
        <button className="primary-action" type="submit">
          Save Changes
        </button>
      </div>
    </form>
  );
}

type ReplyBoxProps = {
  draft: ReplyDraft;
  templates: CommunicationTemplate[];
  onAddComment: (isInternal: boolean) => void;
  onDraftChange: React.Dispatch<React.SetStateAction<ReplyDraft>>;
};

function ReplyBox({ draft, templates, onAddComment, onDraftChange }: ReplyBoxProps) {
  const activeTemplates = templates.filter((template) => template.is_active);

  return (
    <>
      <div className="reply-box">
        <input
          aria-label="Author"
          value={draft.author}
          onChange={(event) =>
            onDraftChange((current) => ({ ...current, author: event.target.value }))
          }
        />
        <textarea
          placeholder="Write a reply or internal note"
          value={draft.body}
          onChange={(event) =>
            onDraftChange((current) => ({ ...current, body: event.target.value }))
          }
        />
        <div className="reply-routing">
          <label>
            Channel
            <select
              value={draft.channel}
              onChange={(event) =>
                onDraftChange((current) => ({ ...current, channel: event.target.value }))
              }
            >
              <option>Local</option>
              <option>Email</option>
              <option>Phone</option>
              <option>WhatsApp</option>
            </select>
          </label>
          <label>
            Audience
            <select
              value={draft.audience}
              onChange={(event) =>
                onDraftChange((current) => ({ ...current, audience: event.target.value }))
              }
            >
              <option>School</option>
              <option>Parent</option>
              <option>Student</option>
              <option>Internal</option>
            </select>
          </label>
          <label>
            Recipient
            <input
              placeholder="Name"
              value={draft.recipient_name}
              onChange={(event) =>
                onDraftChange((current) => ({ ...current, recipient_name: event.target.value }))
              }
            />
          </label>
          <label>
            Contact
            <input
              placeholder="Email or mobile"
              value={draft.recipient_contact}
              onChange={(event) =>
                onDraftChange((current) => ({ ...current, recipient_contact: event.target.value }))
              }
            />
          </label>
          <label>
            Next follow-up
            <input
              type="datetime-local"
              value={draft.next_follow_up_due}
              onChange={(event) =>
                onDraftChange((current) => ({ ...current, next_follow_up_due: event.target.value }))
              }
            />
          </label>
        </div>
        <label>
          Use template
          <select
            value=""
            onChange={(event) => {
              const template = activeTemplates.find(
                (item) => item.id === Number(event.target.value),
              );
              if (template) {
                onDraftChange((current) => ({
                  ...current,
                  body: template.body,
                  audience: template.audience || current.audience,
                }));
              }
            }}
          >
            <option value="">Choose a template</option>
            {activeTemplates.map((template) => (
              <option key={template.id} value={template.id}>
                {template.name} ({template.audience})
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="actions">
        <button
          onClick={() =>
            onDraftChange((current) => ({ ...current, audience: "Internal", channel: "Internal Note" }))
          }
        >
          Mark Internal
        </button>
        <button onClick={() => onAddComment(true)}>Internal Note</button>
        <button className="primary-action" onClick={() => onAddComment(false)}>
          Send Reply
        </button>
      </div>
    </>
  );
}

type AttachmentsPanelProps = {
  attachmentDraft: AttachmentDraft;
  attachments: TicketAttachment[];
  onAddAttachment: (event: React.FormEvent<HTMLFormElement>) => void;
  onBrowseAttachment: () => void;
  onOpenAttachment: (path: string) => void;
  onSetAttachmentDraft: React.Dispatch<React.SetStateAction<AttachmentDraft>>;
};

function AttachmentsPanel({
  attachmentDraft,
  attachments,
  onAddAttachment,
  onBrowseAttachment,
  onOpenAttachment,
  onSetAttachmentDraft,
}: AttachmentsPanelProps) {
  return (
    <details className="attachments-panel">
      <summary>Attachments ({attachments.length})</summary>
      <form className="attachment-form" onSubmit={onAddAttachment}>
        <input
          aria-label="Attachment file path"
          placeholder="/home/abhi/Documents/example.pdf"
          value={attachmentDraft.source_path}
          onChange={(event) =>
            onSetAttachmentDraft((current) => ({
              ...current,
              source_path: event.target.value,
            }))
          }
        />
        <button type="button" onClick={onBrowseAttachment}>
          Browse
        </button>
        <input
          aria-label="Uploaded by"
          value={attachmentDraft.uploaded_by}
          onChange={(event) =>
            onSetAttachmentDraft((current) => ({
              ...current,
              uploaded_by: event.target.value,
            }))
          }
        />
        <button type="submit">Attach File</button>
      </form>
      {attachments.length > 0 ? (
        attachments.map((attachment) => (
          <div className="attachment-row" key={attachment.id}>
            <div className="attachment-title">
              <strong>{attachment.original_filename}</strong>
              <button onClick={() => onOpenAttachment(attachment.stored_path)}>Open</button>
            </div>
            <span>{formatBytes(attachment.size_bytes)}</span>
            <small>
              {attachment.uploaded_by} - {formatTimestamp(attachment.created_at)}
            </small>
            <code>{attachment.stored_path}</code>
          </div>
        ))
      ) : (
        <p className="empty-state compact">No attachments yet.</p>
      )}
    </details>
  );
}

type HistoryPanelProps = {
  history: TicketHistory[];
};

function HistoryPanel({ history }: HistoryPanelProps) {
  return (
    <details className="history-panel">
      <summary>History ({history.length})</summary>
      {history.length > 0 ? (
        history.map((item) => (
          <div className="history-row" key={item.id}>
            <strong>{formatField(item.field)}</strong>
            <span>
              {item.old_value || "Blank"} {"->"} {item.new_value || "Blank"}
            </span>
            <small>
              {item.actor} - {formatTimestamp(item.created_at)}
            </small>
          </div>
        ))
      ) : (
        <p className="empty-state compact">No history recorded yet.</p>
      )}
    </details>
  );
}

type CreateTicketModalProps = {
  draft: CreateTicketDraft;
  schools: School[];
  students: Student[];
  onCancel: () => void;
  onDraftChange: React.Dispatch<React.SetStateAction<CreateTicketDraft>>;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
};

const TICKET_TEMPLATES: Array<{
  label: string;
  issue_category: string;
  description: string;
}> = [
  {
    label: "Academic support request",
    issue_category: "Academic Support",
    description: "Student is experiencing difficulties with academic content. Please describe the specific subject or topic, the nature of the challenge, and any previous support provided.",
  },
  {
    label: "Attendance concern",
    issue_category: "Attendance",
    description: "Reporting an attendance concern for this student. Please include the dates missed, reason if known, and steps taken to follow up with the family.",
  },
  {
    label: "Assessment issue",
    issue_category: "Assessment",
    description: "Issue related to an assessment or evaluation. Please describe the assessment affected, the problem observed, and any impact on the student's grade or progress.",
  },
  {
    label: "Device / hardware problem",
    issue_category: "Device",
    description: "Student's device is not functioning as expected. Please describe the device type, the fault observed, and whether a replacement or repair is needed.",
  },
  {
    label: "Learning platform issue",
    issue_category: "Learning Platform",
    description: "Student is unable to access or use the learning platform. Please describe the error or failure, the steps already attempted, and the impact on learning.",
  },
  {
    label: "Operations request",
    issue_category: "Operations",
    description: "Operational matter requiring coordination. Please describe the request, the expected outcome, and any deadline or urgency.",
  },
  {
    label: "Parent communication needed",
    issue_category: "Parent Communication",
    description: "A communication with the parent or guardian is required. Please describe the topic to be discussed, preferred communication channel, and any prior contact made.",
  },
];

export function CreateTicketModal({
  draft,
  schools,
  students,
  onCancel,
  onDraftChange,
  onSubmit,
}: CreateTicketModalProps) {
  const selectedSchool = schools.find((school) => school.id === draft.school_id);
  const studentOptions = selectedSchool
    ? students.filter((student) => student.school_id === selectedSchool.id)
    : students;

  function applyTemplate(label: string) {
    const template = TICKET_TEMPLATES.find((t) => t.label === label);
    if (!template) return;
    onDraftChange((current) => ({
      ...current,
      issue_category: template.issue_category,
      description: template.description,
    }));
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <form className="ticket-modal" onSubmit={onSubmit}>
        <header>
          <div>
            <h2>New Ticket</h2>
            <p>Create a school-program support request.</p>
          </div>
          <button type="button" onClick={onCancel}>
            Close
          </button>
        </header>

        <label>
          Template (optional)
          <select
            value=""
            onChange={(e) => applyTemplate(e.target.value)}
          >
            <option value="">— Select a template —</option>
            {TICKET_TEMPLATES.map((t) => (
              <option key={t.label} value={t.label}>{t.label}</option>
            ))}
          </select>
        </label>

        <label>
          Title
          <input
            autoFocus
            required
            value={draft.title}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, title: event.target.value }))
            }
          />
        </label>

        <label>
          Requester
          <input
            required
            value={draft.requester}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, requester: event.target.value }))
            }
          />
        </label>
        <label>
          School
          <select
            required
            value={draft.school_id ?? ""}
            onChange={(event) => {
              const school = schools.find((item) => item.id === Number(event.target.value));
              onDraftChange((current) => ({
                ...current,
                school_id: school?.id ?? null,
                school_name: school?.name ?? "",
                student_name: "",
              }));
            }}
          >
            <option value="">Select school</option>
            {schools.map((school) => (
              <option key={school.id} value={school.id}>
                {school.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Student
          <select
            required
            value={draft.student_name}
            onChange={(event) => {
              const student = studentOptions.find((item) => item.name === event.target.value);
              onDraftChange((current) => ({
                ...current,
                student_name: event.target.value,
                grade_level: student?.grade_level ?? current.grade_level,
                program_track: student?.program_track ?? current.program_track,
              }));
            }}
          >
            <option value="">Select student</option>
            {studentOptions.map((student) => (
              <option key={student.id}>{student.name}</option>
            ))}
          </select>
        </label>
        <label>
          Grade
          <select
            value={draft.grade_level}
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                grade_level: event.target.value,
              }))
            }
          >
            {gradeLevels.map((grade) => (
              <option key={grade}>{grade}</option>
            ))}
          </select>
        </label>
        <label>
          Program
          <select
            value={draft.program_track}
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                program_track: event.target.value,
              }))
            }
          >
            {programTracks.map((track) => (
              <option key={track}>{track}</option>
            ))}
          </select>
        </label>
        <label>
          Category
          <select
            value={draft.issue_category}
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                issue_category: event.target.value,
              }))
            }
          >
            {issueCategories.map((category) => (
              <option key={category}>{category}</option>
            ))}
          </select>
        </label>

        <label>
          Priority
          <select
            value={draft.priority}
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                priority: event.target.value as Priority,
              }))
            }
          >
            {priorities.map((priority) => (
              <option key={priority}>{priority}</option>
            ))}
          </select>
        </label>

        <label>
          Description
          <textarea
            required
            value={draft.description}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, description: event.target.value }))
            }
          />
        </label>

        <div className="actions">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button className="primary-action" type="submit">
            Create Ticket
          </button>
        </div>
      </form>
    </div>
  );
}
