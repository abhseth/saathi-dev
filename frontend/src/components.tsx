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
import { APP_TOOLS, isToolVisible } from "./toolRegistry";
import { api } from "./api";
import type {
  AppUser,
  AttendanceRecord,
  AttendanceSummaryRow,
  Batch,
  ChronicAbsentee,
  CommunicationTemplate,
  CreateUserDraft,
  CreateVpCenterBuildingInput,
  CreateVpCenterInput,
  CurrentUser,
  DasGroupBy,
  DasReportRow,
  EffectiveSubject,
  EscalationPolicy,
  FacultyAssignment,
  FacultyMember,
  FacultyProfile,
  FacultySchoolMembership,
  FacultyTodaySession,
  FacultyWeeklySlot,
  CreateFacultyLoginInput,
  CreateFacultyMemberInput,
  CreateFacultySchoolMembershipInput,
  UpdateFacultyMemberInput,
  Filter,
  BulkCreateHolidayInput,
  CreateHolidayInput,
  Holiday,
  LectureModel,
  LectureSession,
  ProgramScopeFilters,
  Region,
  School,
  SchoolClassPlan,
  SchoolDeleteImpact,
  SchoolProfileDraft,
  SipMasterImportPreview,
  SlaPolicy,
  Student,
  Subject,
  SubjectAttendanceRow,
  SubstitutionRecord,
  SwapRequest,
  Ticket,
  TicketAttachment,
  TicketComment,
  TicketHistory,
  TimetableSlot,
  UpdateUserDraft,
  UpdateVpCenterBuildingInput,
  UpdateVpCenterInput,
  UpsertFacultyProfileInput,
  VpCenter,
  VpCenterBuilding,
  WeeklyTimetableSlot,
} from "./types";

/* ── Re-exports from feature folders (append-only) ─────────────────────── */
export { TicketList } from "./components/tickets/TicketList";
export { TicketDetail } from "./components/tickets/TicketDetail";
export { CreateTicketModal } from "./components/tickets/CreateTicketModal";

/* ── Icons ─────────────────────────────────────────────────────────────── */

function IconHome({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
      <polyline points="9 22 9 12 15 12 15 22" />
    </svg>
  );
}

function IconWork({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 12h-6l-2-3H10L8 12H2" />
      <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z" />
    </svg>
  );
}

function IconSchool({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 10v6M2 10l10-5 10 5-10 5z" />
      <path d="M6 12v5c0 1.66 1.79 3 4 3s4-1.34 4-3v-5" />
    </svg>
  );
}

function IconFaculty({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
      <circle cx="12" cy="7" r="4" />
    </svg>
  );
}

function IconTimetable({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  );
}

function IconReports({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  );
}

function IconAdmin({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 2a10 10 0 100 20 10 10 0 000-20z" />
      <path d="M12 6v6l4 2" />
    </svg>
  );
}

function IconLogout({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" />
      <polyline points="16 17 21 12 16 7" />
      <line x1="21" y1="12" x2="9" y2="12" />
    </svg>
  );
}

function IconChevronRight({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}

/* ── Ticket row helpers ────────────────────────────────────────────────── */

function PriorityIcon({ priority }: { priority: string }) {
  const color =
    priority === "Critical" ? "#ef4444" : priority === "High" ? "#f97316" : priority === "Medium" ? "#f59e0b" : "#94a3b8";
  return <span className="priority-dot" style={{ background: color }} aria-hidden />;
}

function StatusChip({ status }: { status: string }) {
  const cls = `status-chip status-${status.toLowerCase().replace(/\s/g, "-")}`;
  return <span className={cls}>{status}</span>;
}

/* ── Error boundary ────────────────────────────────────────────────────── */

export class ErrorBoundary extends React.Component<{ children: React.ReactNode; fallback?: React.ReactNode }, { hasError: boolean }> {
  constructor(props: { children: React.ReactNode; fallback?: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  render() {
    if (this.state.hasError) {
      return this.props.fallback ?? <div className="error-fallback">Something went wrong.</div>;
    }
    return this.props.children;
  }
}

/* ── Topbar ────────────────────────────────────────────────────────────── */

type TopbarProps = {
  search: string;
  currentUser: CurrentUser;
  latestUpdate: string;
  onSearchChange: (value: string) => void;
  onCreateClick: () => void;
  onLogout: () => void;
  onChangePasswordClick: () => void;
  mobileBackLabel?: string;
  onMobileBack?: () => void;
  hideSearch?: boolean;
};

export function Topbar({
  search,
  currentUser,
  latestUpdate,
  onSearchChange,
  onCreateClick,
  onLogout,
  onChangePasswordClick,
  mobileBackLabel,
  onMobileBack,
  hideSearch,
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
            {!hideSearch && (
              <input
                type="search"
                placeholder="Search tickets…"
                value={search}
                onChange={(e) => onSearchChange(e.target.value)}
              />
            )}
            {!isViewer && !onMobileBack && (
              <button type="button" className="primary-action" onClick={onCreateClick}>
                + New Ticket
              </button>
            )}
          </div>
          <div className="topbar-user">
            <span className="user-badge">
              {currentUser.display_name}
              <em>{currentUser.role}</em>
            </span>
            <button type="button" className="ghost-button" onClick={onChangePasswordClick}>
              Change Password
            </button>
            <button type="button" className="ghost-button" onClick={onLogout}>
              Sign out
            </button>
            <span className="latest-update" title="Last updated">
              {latestUpdate}
            </span>
          </div>
        </>
      )}
    </header>
  );
}

/* ── Change Password Modal ─────────────────────────────────────────────── */

export function ChangePasswordModal({
  onClose,
  onSubmit,
}: {
  onClose: () => void;
  onSubmit: (currentPassword: string, newPassword: string) => Promise<void>;
}) {
  const [currentPassword, setCurrentPassword] = React.useState("");
  const [newPassword, setNewPassword] = React.useState("");
  const [confirmPassword, setConfirmPassword] = React.useState("");
  const [error, setError] = React.useState("");
  const [submitting, setSubmitting] = React.useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (newPassword.length < 6) {
      setError("New password must be at least 6 characters.");
      return;
    }
    if (newPassword !== confirmPassword) {
      setError("New passwords do not match.");
      return;
    }
    setSubmitting(true);
    try {
      await onSubmit(currentPassword, newPassword);
      onClose();
    } catch (caught) {
      setError(String(caught));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section
        className="modal-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="change-password-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header>
          <h3 id="change-password-title">Change Password</h3>
          <button className="ghost-button" onClick={onClose} aria-label="Close">
            Close
          </button>
        </header>
        <form onSubmit={handleSubmit} className="master-form">
          <label>
            Current Password
            <input
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              required
            />
          </label>
          <label>
            New Password
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              required
              minLength={6}
            />
          </label>
          <label>
            Confirm New Password
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              required
              minLength={6}
            />
          </label>
          {error ? <p className="form-error">{error}</p> : null}
          <div className="form-actions">
            <button type="button" className="secondary-button" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="primary-action" disabled={submitting}>
              {submitting ? "Saving…" : "Change Password"}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

/* ── Metrics ───────────────────────────────────────────────────────────── */

export function Metrics({
  openCount,
  activeSchoolCount,
  activeQueueCount,
  escalatedCount,
  pendingSlaCount,
  unassignedCount,
}: {
  openCount: number;
  activeSchoolCount: number;
  activeQueueCount: number;
  escalatedCount: number;
  pendingSlaCount: number;
  unassignedCount: number;
}) {
  return (
    <div className="metrics">
      <div className="metric-box">
        <strong>{openCount}</strong>
        <span>Open</span>
      </div>
      <div className="metric-box">
        <strong>{activeSchoolCount}</strong>
        <span>Schools</span>
      </div>
      <div className="metric-box">
        <strong>{activeQueueCount}</strong>
        <span>Queues</span>
      </div>
      <div className="metric-box">
        <strong>{escalatedCount}</strong>
        <span>Escalated</span>
      </div>
      <div className="metric-box">
        <strong>{pendingSlaCount}</strong>
        <span>Pending SLA</span>
      </div>
      <div className="metric-box">
        <strong>{unassignedCount}</strong>
        <span>Unassigned</span>
      </div>
    </div>
  );
}

/* ── Login screen ──────────────────────────────────────────────────────── */

export function LoginScreen({
  draft,
  error,
  onDraftChange,
  onSubmit,
}: {
  draft: { username: string; password: string };
  error: string | null;
  onDraftChange: (draft: { username: string; password: string }) => void;
  onSubmit: () => void;
}) {
  return (
    <div className="login-backdrop">
      <div className="login-card">
        <div className="login-header">
          <div>
            <strong>SAATHI</strong>
            <small>School Integrated Program</small>
          </div>
        </div>
        <form
          className="login-form"
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit();
          }}
        >
          <h2>Sign In</h2>
          <label>
            Username
            <input
              autoFocus
              value={draft.username}
              onChange={(e) =>
                onDraftChange({ ...draft, username: e.target.value })
              }
            />
          </label>
          <label>
            Password
            <input
              type="password"
              value={draft.password}
              onChange={(e) =>
                onDraftChange({ ...draft, password: e.target.value })
              }
            />
          </label>
          {error ? <div className="error-banner">{error}</div> : null}
          <button className="primary-action" type="submit">
            Sign In
          </button>
        </form>
      </div>
    </div>
  );
}

/* ── Offline cache banner ──────────────────────────────────────────────── */

export function OfflineBanner({
  isOnline,
  needsSync,
  onSync,
}: {
  isOnline: boolean;
  needsSync: boolean;
  onSync: () => void;
}) {
  if (isOnline && !needsSync) return null;
  return (
    <div className={`offline-banner ${needsSync ? "needs-sync" : ""}`}>
      {!isOnline ? (
        <span>You are offline. Changes will sync when you reconnect.</span>
      ) : needsSync ? (
        <>
          <span>Local changes pending.</span>
          <button className="primary-action" onClick={onSync}>
            Sync now
          </button>
        </>
      ) : null}
    </div>
  );
}

/* ── Bottom navigation ─────────────────────────────────────────────────── */

type BottomNavProps = {
  currentUserRole: string;
  currentSection: string;
  filterCounts: Record<Filter, number>;
  mobileView: "home" | "work" | "detail";
  onHomeClick: () => void;
  onWorkClick: () => void;
  onCreateClick: () => void;
  onMoreClick: () => void;
};

export function BottomNav({
  currentUserRole,
  currentSection,
  filterCounts,
  mobileView,
  onHomeClick,
  onWorkClick,
  onCreateClick,
  onMoreClick,
}: BottomNavProps) {
  const isViewer = currentUserRole === "viewer";
  const inboxCount = filterCounts["Inbox"];
  const inWorkSection = currentSection === "work";
  return createPortal(
    <nav className="bottom-nav">
      <button
        className={`bottom-nav-tab ${inWorkSection && mobileView === "home" ? "active" : ""}`}
        onClick={onHomeClick}
      >
        <IconHome size={22} />
        <span>Home</span>
      </button>
      <button
        className={`bottom-nav-tab ${inWorkSection && (mobileView === "work" || mobileView === "detail") ? "active" : ""}`}
        onClick={onWorkClick}
      >
        {inboxCount > 0 && <span className="bottom-nav-badge">{inboxCount > 99 ? "99+" : inboxCount}</span>}
        <IconWork size={22} />
        <span>Work</span>
      </button>
      {!isViewer && (
        <button className="bottom-nav-tab bottom-nav-tab-new" onClick={onCreateClick} aria-label="Create ticket">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      )}
      <button className={`bottom-nav-tab ${!inWorkSection ? "active-more" : ""}`} onClick={onMoreClick}>
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round">
          <line x1="3" y1="12" x2="21" y2="12" />
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
        <span>More</span>
      </button>
    </nav>,
    document.body,
  );
}

/* ── Mobile more menu ──────────────────────────────────────────────────── */

type MobileMoreMenuProps = {
  currentUserRole: string;
  onClose: () => void;
  onToolClick: (toolId: string) => void;
  onLogout: () => void;
  onChangePassword: () => void;
};

export function MobileMoreMenu({
  currentUserRole,
  onClose,
  onToolClick,
  onLogout,
  onChangePassword,
}: MobileMoreMenuProps) {
  function handle(fn?: () => void) {
    if (fn) fn();
    onClose();
  }

  const visibleTools = APP_TOOLS.filter((t) => isToolVisible(t, currentUserRole));

  return (
    <>
      <div role="button" tabIndex={0} aria-label="Close menu" className="mobile-more-backdrop" onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onClose(); } }} onClick={onClose} />
      <div className="mobile-more-sheet">
        <div className="mobile-more-handle" />

        <div className="mobile-more-group">
          <div className="mobile-more-heading">Tools</div>
          <div className="mobile-more-grid">
            {visibleTools.map((tool) => (
              <button
                key={tool.id}
                className="mobile-more-tile"
                onClick={() => handle(() => onToolClick(tool.id))}
              >
                <span className="mobile-more-tile-icon">{tool.icon}</span>
                <span className="mobile-more-tile-label">{tool.label}</span>
                <IconChevronRight size={14} />
              </button>
            ))}
          </div>
        </div>

        <div className="mobile-more-group">
          <button
            className="mobile-more-row"
            onClick={() => handle(onChangePassword)}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            <span>Change Password</span>
          </button>
          <button
            className="mobile-more-row mobile-more-row-signout"
            onClick={() => handle(onLogout)}
          >
            <IconLogout size={18} />
            <span>Sign out</span>
          </button>
        </div>
      </div>
    </>
  );
}

/* ── Alert banner ──────────────────────────────────────────────────────── */

export function AlertBanner({
  alerts,
  onDismiss,
}: {
  alerts: Array<{ id: string; severity: "info" | "warning" | "critical"; message: string }>;
  onDismiss: (id: string) => void;
}) {
  if (alerts.length === 0) return null;
  return (
    <div className="alert-banner-stack">
      {alerts.map((alert) => (
        <div key={alert.id} className={`alert-banner alert-banner-${alert.severity}`}>
          <span>{alert.message}</span>
          <button className="ghost-button" onClick={() => onDismiss(alert.id)}>
            Dismiss
          </button>
        </div>
      ))}
    </div>
  );
}

/* ── SLA breach alert ──────────────────────────────────────────────────── */

export function SlaBreachAlert({
  newBreachCount,
  onView,
  onDismiss,
}: {
  newBreachCount: number;
  onView: () => void;
  onDismiss: () => void;
}) {
  if (newBreachCount === 0) return null;
  return (
    <div className="sla-breach-banner" role="alert">
      <span>
        {newBreachCount} ticket{newBreachCount !== 1 ? "s" : ""} breached SLA
      </span>
      <div className="sla-breach-actions">
        <button className="primary-action" onClick={onView}>
          View
        </button>
        <button className="ghost-button" onClick={onDismiss}>
          Dismiss
        </button>
      </div>
    </div>
  );
}

/* ── School contacts bar ───────────────────────────────────────────────── */

export function SchoolContactsBar({ school }: { school: School | null }) {
  if (!school) return null;
  return (
    <div className="school-contacts-bar">
      <strong>{school.name}</strong>
      <div className="school-contact-line">
        {school.principal_name && <span>Principal: {school.principal_name}</span>}
        {school.principal_mobile && <span>{school.principal_mobile}</span>}
      </div>
      <div className="school-contact-line">
        {school.center_head_name && <span>Center Head: {school.center_head_name}</span>}
        {school.center_head_mobile && <span>{school.center_head_mobile}</span>}
      </div>
    </div>
  );
}

/* ── Program filters ───────────────────────────────────────────────────── */

export function ProgramFilters({
  filters,
  schoolOptions,
  dateFrom,
  dateTo,
  onChange,
  onDateFromChange,
  onDateToChange,
  onReset,
}: {
  filters: ProgramScopeFilters;
  schoolOptions: string[];
  dateFrom: string;
  dateTo: string;
  onChange: (filters: ProgramScopeFilters) => void;
  onDateFromChange: (value: string) => void;
  onDateToChange: (value: string) => void;
  onReset: () => void;
}) {
  return (
    <div className="program-filters">
      <select
        value={filters.school_name ?? ""}
        onChange={(e) =>
          onChange({
            ...filters,
            school_name: e.target.value,
          })
        }
      >
        <option value="">All schools</option>
        {schoolOptions.map((s) => (
          <option key={s} value={s}>
            {s}
          </option>
        ))}
      </select>
      <input
        type="date"
        value={dateFrom}
        onChange={(e) => onDateFromChange(e.target.value)}
      />
      <input
        type="date"
        value={dateTo}
        onChange={(e) => onDateToChange(e.target.value)}
      />
      <button className="ghost-button" onClick={onReset}>
        Reset
      </button>
    </div>
  );
}

/* ── Announcement banner ───────────────────────────────────────────────── */

export function AnnouncementBanner() {
  const [announcement] = React.useState<string | null>(null);
  if (!announcement) return null;
  return <div className="announcement-banner">{announcement}</div>;
}

/* ── Modals / Panels ───────────────────────────────────────────────────── */

export function MasterDataPanel({
  schools,
  regions,
  lectureModels,
  classPlans,
  batches,
  students,
  studentTotalCount,
  studentPage,
  studentPageSize,
  studentSearch,
  sipImportPreview,
  onClose,
  onCreateSchool,
  onSaveRegion,
  onCreateLectureModel,
  onSaveClassPlan,
  onCreateBatch,
  onUpdateBatch,
  onArchiveBatch,
  onImportSchools,
  onImportSipMaster,
  onExportSipMaster,
  onDeleteSchool,
  onLoadSchoolDeleteImpact,
  onDropSchool,
  onDeleteRegion,
  onRemapAndDeleteRegion,
  onCancelSipMasterImport,
  onConfirmSipMasterImport,
  onCreateStudent,
  onUpdateStudent,
  onDeleteStudent,
  onStudentSearchChange,
  onLoadStudents,
  onImportStudentsCsv,
  currentUserRole,
}: {
  schools: School[];
  regions: Region[];
  lectureModels: LectureModel[];
  classPlans: SchoolClassPlan[];
  batches: Batch[];
  students: Student[];
  studentTotalCount: number;
  studentPage: number;
  studentPageSize: number;
  studentSearch: string;
  sipImportPreview: { sourcePath: string; preview: SipMasterImportPreview } | null;
  onClose: () => void;
  onCreateSchool: (input: SchoolProfileDraft) => void;
  onUpdateSchool: (input: SchoolProfileDraft & { id: number }) => void;
  onSaveRegion: (input: {
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
    regional_deputy_academic_head_name: string;
    regional_deputy_academic_head_mobile: string;
    regional_deputy_academic_head_email: string;
  }) => void;
  onCreateLectureModel: (input: { name: string; days_per_week: number; lectures_per_day: number }) => void;
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
  onCreateBatch: (input: {
    school_id: number;
    batch_id: string;
    grade_level: string;
    track: string;
    batch_pattern: string;
    capacity: number;
  }) => void;
  onUpdateBatch: (input: {
    id: number;
    school_id: number;
    batch_id: string;
    grade_level: string;
    track: string;
    batch_pattern: string;
    capacity: number;
  }) => void;
  onArchiveBatch: (id: number) => void;
  onImportSchools: () => void;
  onImportSipMaster: () => void;
  onExportSipMaster: () => void;
  onDeleteSchool: (id: number) => void;
  onLoadSchoolDeleteImpact: (id: number) => Promise<SchoolDeleteImpact | null>;
  onDropSchool: (id: number, reason: string) => void;
  onDeleteRegion: (id: number) => void;
  onRemapAndDeleteRegion: (oldId: number, newId: number) => void;
  onCancelSipMasterImport: () => void;
  onConfirmSipMasterImport: () => void;
  onCreateStudent: (input: Record<string, unknown>) => void;
  onUpdateStudent: (input: Record<string, unknown>) => void;
  onDeleteStudent: (id: number) => void;
  onStudentSearchChange: (search: string) => void;
  onLoadStudents: (schoolId?: number, page?: number, search?: string) => void;
  onImportStudentsCsv: (schoolId: number) => void;
  currentUserRole: string;
}) {
  const isAdmin = currentUserRole === "admin";
  const [showSchoolForm, setShowSchoolForm] = React.useState(false);
  const [editingSchoolId, setEditingSchoolId] = React.useState<number | null>(null);
  const [showRegionForm, setShowRegionForm] = React.useState(false);
  const [editingRegionId, setEditingRegionId] = React.useState<number | null>(null);
  const [confirmDeleteRegionId, setConfirmDeleteRegionId] = React.useState<number | null>(null);
  const [collapsedSections, setCollapsedSections] = React.useState({
    schools: false,
    regions: false,
    batches: false,
    students: false,
  });
  function toggleSection(key: "schools" | "regions" | "batches" | "students") {
    setCollapsedSections((s) => ({ ...s, [key]: !s[key] }));
  }
  const [showLectureModelForm, setShowLectureModelForm] = React.useState(false);
  const [showClassPlanForm, setShowClassPlanForm] = React.useState(false);
  const [showBatchForm, setShowBatchForm] = React.useState(false);
  const [editingBatchId, setEditingBatchId] = React.useState<number | null>(null);
  const [confirmArchiveBatchId, setConfirmArchiveBatchId] = React.useState<number | null>(null);
  const [dropSchoolId, setDropSchoolId] = React.useState<number | null>(null);
  const [dropReason, setDropReason] = React.useState("");
  const [confirmDeleteSchoolId, setConfirmDeleteSchoolId] = React.useState<number | null>(null);
  const [deleteImpact, setDeleteImpact] = React.useState<SchoolDeleteImpact | null>(null);
  const [isLoadingDeleteImpact, setIsLoadingDeleteImpact] = React.useState(false);
  const [studentSchoolId, setStudentSchoolId] = React.useState<number | "">("");
  const [studentImportSchoolId, setStudentImportSchoolId] = React.useState<number | "">("");

  const [schoolTab, setSchoolTab] = React.useState<"basic" | "sip" | "center" | "principal">("basic");
  const [isSaving, setIsSaving] = React.useState(false);
  const [formError, setFormError] = React.useState("");

  const [schoolForm, setSchoolForm] = React.useState({
    name: "",
    region_id: null as number | null,
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
    vp_tagging: "",
  });

  const [regionForm, setRegionForm] = React.useState<{
    id?: number;
    name: string;
    regional_academic_head_name: string;
    regional_academic_head_mobile: string;
    regional_academic_head_email: string;
    regional_business_head_name: string;
    regional_business_head_mobile: string;
    regional_business_head_email: string;
    regional_deputy_academic_head_name: string;
    regional_deputy_academic_head_mobile: string;
    regional_deputy_academic_head_email: string;
  }>({
    name: "",
    regional_academic_head_name: "",
    regional_academic_head_mobile: "",
    regional_academic_head_email: "",
    regional_business_head_name: "",
    regional_business_head_mobile: "",
    regional_business_head_email: "",
    regional_deputy_academic_head_name: "",
    regional_deputy_academic_head_mobile: "",
    regional_deputy_academic_head_email: "",
  });

  const [lectureModelForm, setLectureModelForm] = React.useState({
    name: "",
    days_per_week: "",
    lectures_per_day: "",
  });

  const [classPlanForm, setClassPlanForm] = React.useState({
    school_id: "",
    grade_level: "",
    track: "",
    lecture_model_id: "",
    batch_pattern: "",
    aop_admissions: "",
    registrations: "",
    actual_admissions: "",
  });

  const [batchForm, setBatchForm] = React.useState({
    class_plan_id: "",
    batch_id: "",
    capacity: "",
  });

  const selectedBatchClassPlan = classPlans.find((plan) => String(plan.id) === batchForm.class_plan_id);
  const schoolPendingDelete = schools.find((school) => school.id === confirmDeleteSchoolId) ?? null;

  async function previewSchoolDelete(school: School) {
    setIsLoadingDeleteImpact(true);
    setConfirmDeleteSchoolId(school.id);
    setDropSchoolId(null);
    setDeleteImpact(null);
    try {
      setDeleteImpact(await onLoadSchoolDeleteImpact(school.id));
    } finally {
      setIsLoadingDeleteImpact(false);
    }
  }

  function cancelSchoolDelete() {
    setConfirmDeleteSchoolId(null);
    setDeleteImpact(null);
    setIsLoadingDeleteImpact(false);
  }

  async function submitSchool(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    setFormError("");
    try {
      const draft = {
        ...schoolForm,
        region_name: "",
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
      if (editingSchoolId != null) {
        await onUpdateSchool({ ...draft, id: editingSchoolId });
      } else {
        await onCreateSchool(draft);
      }
      setShowSchoolForm(false);
      setEditingSchoolId(null);
      setSchoolForm({
        name: "",
        region_id: null,
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
        vp_tagging: "",
      });
      setSchoolTab("basic");
    } catch (caught) {
      setFormError(String(caught));
      return;
    } finally {
      setIsSaving(false);
    }
  }

  async function submitRegion(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    setFormError("");
    try {
      await onSaveRegion(regionForm);
      setShowRegionForm(false);
      setEditingRegionId(null);
      setRegionForm({
        name: "",
        regional_academic_head_name: "",
        regional_academic_head_mobile: "",
        regional_academic_head_email: "",
        regional_business_head_name: "",
        regional_business_head_mobile: "",
        regional_business_head_email: "",
        regional_deputy_academic_head_name: "",
        regional_deputy_academic_head_mobile: "",
        regional_deputy_academic_head_email: "",
      });
    } catch (caught) {
      setFormError(String(caught));
      return;
    } finally {
      setIsSaving(false);
    }
  }

  async function submitLectureModel(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    setFormError("");
    try {
      await onCreateLectureModel({
        name: lectureModelForm.name,
        days_per_week: Number(lectureModelForm.days_per_week),
        lectures_per_day: Number(lectureModelForm.lectures_per_day),
      });
      setShowLectureModelForm(false);
      setLectureModelForm({ name: "", days_per_week: "", lectures_per_day: "" });
    } catch (caught) {
      setFormError(String(caught));
      return;
    } finally {
      setIsSaving(false);
    }
  }

  async function submitClassPlan(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    setFormError("");
    try {
      await onSaveClassPlan({
        school_id: Number(classPlanForm.school_id),
        grade_level: classPlanForm.grade_level,
        track: classPlanForm.track,
        lecture_model_id: Number(classPlanForm.lecture_model_id),
        batch_pattern: classPlanForm.batch_pattern,
        aop_admissions: Number(classPlanForm.aop_admissions),
        registrations: Number(classPlanForm.registrations),
        actual_admissions: Number(classPlanForm.actual_admissions),
      });
      setShowClassPlanForm(false);
      setClassPlanForm({
        school_id: "",
        grade_level: "",
        track: "",
        lecture_model_id: "",
        batch_pattern: "",
        aop_admissions: "",
        registrations: "",
        actual_admissions: "",
      });
    } catch (caught) {
      setFormError(String(caught));
      return;
    } finally {
      setIsSaving(false);
    }
  }

  function resetBatchForm() {
    setBatchForm({ class_plan_id: "", batch_id: "", capacity: "" });
    setEditingBatchId(null);
    setShowBatchForm(false);
  }

  function startEditBatch(batch: Batch) {
    const matchingPlan = classPlans.find(
      (plan) =>
        plan.school_id === batch.school_id &&
        plan.grade_level === batch.grade_level &&
        plan.track === batch.track &&
        plan.batch_pattern === batch.batch_pattern,
    );
    setBatchForm({
      class_plan_id: matchingPlan ? String(matchingPlan.id) : "",
      batch_id: batch.batch_id,
      capacity: String(batch.capacity),
    });
    setEditingBatchId(batch.id);
    setConfirmArchiveBatchId(null);
    setShowBatchForm(true);
  }

  async function submitBatch(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedBatchClassPlan) return;
    setIsSaving(true);
    setFormError("");
    const payload = {
      school_id: selectedBatchClassPlan.school_id,
      batch_id: batchForm.batch_id,
      grade_level: selectedBatchClassPlan.grade_level,
      track: selectedBatchClassPlan.track,
      batch_pattern: selectedBatchClassPlan.batch_pattern,
      capacity: Number(batchForm.capacity || 0),
    };
    try {
      if (editingBatchId) {
        await onUpdateBatch({ ...payload, id: editingBatchId });
      } else {
        await onCreateBatch(payload);
      }
      resetBatchForm();
    } catch (caught) {
      setFormError(String(caught));
      return;
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <section className="ticket-modal master-data-modal" aria-label="Master data">
      <header>
        <h2>Master Data</h2>
        <div className="actions">
          {isAdmin && (
            <button className="primary-action" onClick={() => { setEditingSchoolId(null); setShowSchoolForm(true); }}>
              Add School
            </button>
          )}
          {isAdmin && (
            <button className="primary-action" onClick={() => {
              setEditingRegionId(null);
              setRegionForm({
                name: "",
                regional_academic_head_name: "",
                regional_academic_head_mobile: "",
                regional_academic_head_email: "",
                regional_business_head_name: "",
                regional_business_head_mobile: "",
                regional_business_head_email: "",
                regional_deputy_academic_head_name: "",
                regional_deputy_academic_head_mobile: "",
                regional_deputy_academic_head_email: "",
              });
              setShowRegionForm(true);
            }}>
              Add Region
            </button>
          )}
          {isAdmin && (
            <button className="primary-action" onClick={() => setShowLectureModelForm(true)}>
              Add Lecture Model
            </button>
          )}
          <button className="primary-action" onClick={() => setShowClassPlanForm(true)}>
            Add Class Offering
          </button>
          <button className="primary-action" onClick={() => {
            setEditingBatchId(null);
            setBatchForm({ class_plan_id: "", batch_id: "", capacity: "" });
            setShowBatchForm(true);
          }}>
            Add Batch
          </button>
          {isAdmin && (
            <button className="secondary-button" onClick={onImportSchools}>
              Import Schools
            </button>
          )}
          {isAdmin && (
            <button className="secondary-button" onClick={onImportSipMaster}>
              Import SIP Master
            </button>
          )}
        </div>
      </header>
      <div className="master-data-grid">
        <div className="master-data-card">
          <h3>Schools</h3>
          <span className="master-data-count">{schools.length}</span>
        </div>
        <div className="master-data-card">
          <h3>Regions</h3>
          <span className="master-data-count">{regions.length}</span>
        </div>
        <div className="master-data-card">
          <h3>Lecture Models</h3>
          <span className="master-data-count">{lectureModels.length}</span>
        </div>
        <div className="master-data-card">
          <h3>Class Offerings / Plans</h3>
          <span className="master-data-count">{classPlans.length}</span>
        </div>
        <div className="master-data-card">
          <h3>Batches</h3>
          <span className="master-data-count">{batches.length}</span>
        </div>
        <div className="master-data-card">
          <h3>Students</h3>
          <span className="master-data-count">{students.length}</span>
        </div>
      </div>

      <section className="master-data-section" aria-label="Active schools">
        <header className="section-header">
          <div>
            <h3>Active Schools</h3>
            <p>Drop preserves history and hides a school from active operations. Permanent delete is admin-only and should be used only for mistaken records.</p>
          </div>
          <button
            type="button"
            className="ghost-button"
            onClick={() => toggleSection("schools")}
            aria-label={collapsedSections.schools ? "Expand schools" : "Collapse schools"}
          >
            {collapsedSections.schools ? "▶" : "▼"}
          </button>
        </header>
        {!collapsedSections.schools && (schools.length === 0 ? (
          <p className="empty-state">No active schools.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>School</th>
                <th>Region</th>
                <th>Program</th>
                <th>Principal</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {schools.map((school) => (
                <tr key={school.id}>
                  <td>{school.name}</td>
                  <td>{school.region_name || "—"}</td>
                  <td>{school.program_model || "—"}</td>
                  <td>{school.principal_name || "—"}</td>
                  <td>
                    <div className="actions">
                      <button
                        className="secondary-button"
                        onClick={() => {
                          setDropSchoolId(school.id);
                          setDropReason("");
                          setConfirmDeleteSchoolId(null);
                        }}
                      >
                        Drop
                      </button>
                      {isAdmin && (
                        <>
                          <button
                            className="secondary-button"
                            onClick={() => {
                              setEditingSchoolId(school.id);
                              setSchoolForm({
                                name: school.name,
                                region_id: school.region_id,
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
                                vp_tagging: school.vp_tagging,
                              });
                              setSchoolTab("basic");
                              setShowSchoolForm(true);
                            }}
                          >
                            Edit
                          </button>
                          <button
                            className="secondary-button"
                            onClick={() => void previewSchoolDelete(school)}
                          >
                            Delete
                          </button>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ))}
      </section>

      {schoolPendingDelete && (
        <div className="modal-backdrop" role="presentation" onClick={cancelSchoolDelete}>
          <section
            className="modal-card delete-impact-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="school-delete-impact-title"
            onClick={(event) => event.stopPropagation()}
          >
            <header>
              <div>
                <h3 id="school-delete-impact-title">Permanent Delete Review</h3>
                <p>
                  Deleting {schoolPendingDelete.name} will permanently remove the school and cascade linked operational records.
                </p>
              </div>
              <button className="ghost-button" onClick={cancelSchoolDelete} aria-label="Close delete review">
                Close
              </button>
            </header>
            {isLoadingDeleteImpact ? (
              <p className="empty-state">Checking linked records...</p>
            ) : deleteImpact ? (
              <>
                <div className="impact-summary">
                  <strong>{deleteImpact.total_linked_records}</strong>
                  <span>linked records will be deleted or detached by this action.</span>
                </div>
                <table className="data-table compact-table">
                  <thead>
                    <tr>
                      <th>Area</th>
                      <th>Records</th>
                    </tr>
                  </thead>
                  <tbody>
                    {deleteImpact.items.filter((item) => item.count > 0).map((item) => (
                      <tr key={item.label}>
                        <td>{item.label}</td>
                        <td>{item.count}</td>
                      </tr>
                    ))}
                    {deleteImpact.items.every((item) => item.count === 0) && (
                      <tr>
                        <td colSpan={2}>No linked records found.</td>
                      </tr>
                    )}
                  </tbody>
                </table>
                <p className="form-error">
                  Prefer Drop for real schools. Use permanent delete only for mistaken duplicate/test school records.
                </p>
              </>
            ) : (
              <p className="form-error">Could not load delete impact. Try again before deleting.</p>
            )}
            <div className="actions">
              <button
                className="danger-button"
                disabled={!deleteImpact}
                onClick={() => {
                  onDeleteSchool(schoolPendingDelete.id);
                  cancelSchoolDelete();
                }}
              >
                Permanently Delete School
              </button>
              <button className="secondary-button" onClick={cancelSchoolDelete}>
                Cancel
              </button>
            </div>
          </section>
        </div>
      )}

      <section className="master-data-section" aria-label="Regions">
        <header className="section-header">
          <div>
            <h3>Regions</h3>
            <p>Regions group schools geographically. Deleting a region requires moving its schools to another region first.</p>
          </div>
          <button
            type="button"
            className="ghost-button"
            onClick={() => toggleSection("regions")}
            aria-label={collapsedSections.regions ? "Expand regions" : "Collapse regions"}
          >
            {collapsedSections.regions ? "▶" : "▼"}
          </button>
        </header>
        {!collapsedSections.regions && (regions.length === 0 ? (
          <p className="empty-state">No regions configured yet.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>RAH</th>
                <th>RBH</th>
                <th>Deputy RAH</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {regions.map((region) => (
                <tr key={region.id}>
                  <td>{region.name}</td>
                  <td>{region.regional_academic_head_name || "—"}</td>
                  <td>{region.regional_business_head_name || "—"}</td>
                  <td>{region.regional_deputy_academic_head_name || "—"}</td>
                  <td>
                    <div className="actions">
                      <button
                        className="secondary-button"
                        onClick={() => {
                          setEditingRegionId(region.id);
                          setRegionForm({
                            id: region.id,
                            name: region.name,
                            regional_academic_head_name: region.regional_academic_head_name,
                            regional_academic_head_mobile: region.regional_academic_head_mobile,
                            regional_academic_head_email: region.regional_academic_head_email,
                            regional_business_head_name: region.regional_business_head_name,
                            regional_business_head_mobile: region.regional_business_head_mobile,
                            regional_business_head_email: region.regional_business_head_email,
                            regional_deputy_academic_head_name: region.regional_deputy_academic_head_name,
                            regional_deputy_academic_head_mobile: region.regional_deputy_academic_head_mobile,
                            regional_deputy_academic_head_email: region.regional_deputy_academic_head_email,
                          });
                          setShowRegionForm(true);
                        }}
                      >
                        Edit
                      </button>
                      {isAdmin && (
                        <button
                          className="secondary-button"
                          onClick={() => setConfirmDeleteRegionId(region.id)}
                        >
                          Delete
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ))}
      </section>

      {confirmDeleteRegionId != null && (
        <div className="modal-backdrop" role="presentation" onClick={() => setConfirmDeleteRegionId(null)}>
          <section
            className="modal-card delete-impact-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="region-delete-title"
            onClick={(event) => event.stopPropagation()}
          >
            <header>
              <div>
                <h3 id="region-delete-title">Delete Region</h3>
                <p>
                  Region <strong>{regions.find((r) => r.id === confirmDeleteRegionId)?.name}</strong> will be permanently deleted.
                </p>
              </div>
              <button className="ghost-button" onClick={() => setConfirmDeleteRegionId(null)} aria-label="Close">
                Close
              </button>
            </header>
            {regions.find((r) => r.id === confirmDeleteRegionId) && schools.some((s) => s.region_id === confirmDeleteRegionId) ? (
              <>
                <p className="form-error">This region has schools mapped to it. Move those schools to another region first.</p>
                <div className="actions">
                  <button className="secondary-button" onClick={() => setConfirmDeleteRegionId(null)}>
                    Cancel
                  </button>
                </div>
              </>
            ) : (
              <div className="actions">
                <button
                  className="danger-button"
                  onClick={() => {
                    onDeleteRegion(confirmDeleteRegionId);
                    setConfirmDeleteRegionId(null);
                  }}
                >
                  Permanently Delete Region
                </button>
                <button className="secondary-button" onClick={() => setConfirmDeleteRegionId(null)}>
                  Cancel
                </button>
              </div>
            )}
          </section>
        </div>
      )}

      <section className="master-data-section" aria-label="Batches">
        <header className="section-header">
          <div>
            <h3>Batches</h3>
            <p>Concrete teachable groups under class offerings. Example: Class XI JEE Weekday can have Batch A and Batch B.</p>
          </div>
          <button
            type="button"
            className="ghost-button"
            onClick={() => toggleSection("batches")}
            aria-label={collapsedSections.batches ? "Expand batches" : "Collapse batches"}
          >
            {collapsedSections.batches ? "▶" : "▼"}
          </button>
        </header>
        {!collapsedSections.batches && (
          <>
            {showBatchForm && (
          <form className="inline-edit-form" onSubmit={submitBatch}>
            <label>
              Class Offering
              <select
                required
                value={batchForm.class_plan_id}
                onChange={(e) => setBatchForm((s) => ({ ...s, class_plan_id: e.target.value }))}
              >
                <option value="">Select class offering</option>
                {classPlans.map((plan) => (
                  <option key={plan.id} value={plan.id}>
                    {plan.school_name} — {plan.grade_level}{plan.track ? ` ${plan.track}` : ""} — {plan.batch_pattern}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Batch Name / ID
              <input
                required
                value={batchForm.batch_id}
                onChange={(e) => setBatchForm((s) => ({ ...s, batch_id: e.target.value }))}
                placeholder="XI-JEE-WD-A"
              />
            </label>
            <label>
              Capacity
              <input
                type="number"
                min="0"
                value={batchForm.capacity}
                onChange={(e) => setBatchForm((s) => ({ ...s, capacity: e.target.value }))}
                placeholder="40"
              />
            </label>
            {formError ? <p className="form-error">{formError}</p> : null}
            <div className="actions">
              <button className="primary-action" disabled={isSaving || !selectedBatchClassPlan} type="submit">
                {editingBatchId ? "Save Batch" : "Create Batch"}
              </button>
              <button type="button" className="ghost-button" onClick={resetBatchForm}>
                Cancel
              </button>
            </div>
          </form>
        )}
        {batches.length === 0 ? (
          <p className="empty-state">No batches configured yet.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>School</th>
                <th>Batch</th>
                <th>Class</th>
                <th>Track</th>
                <th>Delivery Pattern</th>
                <th>Capacity</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {batches.map((batch) => (
                <tr key={batch.id}>
                  <td>{batch.school_name}</td>
                  <td>{batch.batch_id}</td>
                  <td>{batch.grade_level}</td>
                  <td>{batch.track || "Foundation"}</td>
                  <td>{batch.batch_pattern}</td>
                  <td>{batch.capacity || "—"}</td>
                  <td>
                    <div className="actions">
                      <button type="button" className="secondary-button" onClick={() => startEditBatch(batch)}>
                        Edit
                      </button>
                      <button
                        type="button"
                        className={confirmArchiveBatchId === batch.id ? "danger-button" : "secondary-button"}
                        onClick={() => {
                          if (confirmArchiveBatchId === batch.id) {
                            onArchiveBatch(batch.id);
                            setConfirmArchiveBatchId(null);
                          } else {
                            setConfirmArchiveBatchId(batch.id);
                          }
                        }}
                      >
                        {confirmArchiveBatchId === batch.id ? "Confirm Archive" : "Archive"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
          </>
        )}
      </section>

      <section className="master-data-section" aria-label="Students">
        <header className="section-header">
          <div>
            <h3>Students</h3>
            <p>Select a school to load students. This keeps the Master Data view fast at 10,000+ student scale.</p>
          </div>
          <div className="actions">
            <select
              aria-label="Student list filter"
              value={studentSchoolId}
              onChange={(e) => {
                const next = e.target.value ? Number(e.target.value) : "";
                setStudentSchoolId(next);
                if (next) onLoadStudents(next);
              }}
            >
              <option value="">Select school to view</option>
              {schools.map((school) => (
                <option key={school.id} value={school.id}>{school.name}</option>
              ))}
            </select>
            <input
              aria-label="Student search"
              placeholder="Search students"
              value={studentSearch}
              disabled={!studentSchoolId}
              onChange={(e) => onStudentSearchChange(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && studentSchoolId) {
                  onLoadStudents(Number(studentSchoolId), 1, studentSearch);
                }
              }}
            />
            <button
              className="secondary-button"
              disabled={!studentSchoolId}
              onClick={() => {
                if (studentSchoolId) onLoadStudents(Number(studentSchoolId), 1, studentSearch);
              }}
            >
              Search
            </button>
            <select
              aria-label="Student import target"
              value={studentImportSchoolId}
              onChange={(e) => setStudentImportSchoolId(e.target.value ? Number(e.target.value) : "")}
            >
              <option value="">Select school for import</option>
              {schools.map((school) => (
                <option key={school.id} value={school.id}>{school.name}</option>
              ))}
            </select>
            <button
              className="secondary-button"
              disabled={!studentImportSchoolId}
              onClick={() => {
                if (studentImportSchoolId) onImportStudentsCsv(Number(studentImportSchoolId));
              }}
            >
              Import Students
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() => toggleSection("students")}
              aria-label={collapsedSections.students ? "Expand students" : "Collapse students"}
            >
              {collapsedSections.students ? "▶" : "▼"}
            </button>
          </div>
        </header>
        {!collapsedSections.students && (
          <>
            {!studentSchoolId ? (
          <p className="empty-state">Select a school to view its students. Avoid loading all schools into one table.</p>
        ) : students.length === 0 ? (
          <p className="empty-state">No students imported yet.</p>
        ) : (
          <>
            <p className="table-note">
              Showing {students.length} of {studentTotalCount} students, page {studentPage} of {Math.max(1, Math.ceil(studentTotalCount / studentPageSize))}.
            </p>
            <table className="data-table">
              <thead>
                <tr>
                  <th>School</th>
                  <th>Student</th>
                  <th>Registration</th>
                  <th>Class</th>
                  <th>Track</th>
                  <th>Batch</th>
                  <th>Mobile</th>
                  <th>Email</th>
                </tr>
              </thead>
              <tbody>
                {students.map((student) => (
                  <tr key={student.id}>
                    <td>{student.school_name}</td>
                    <td>{student.name}</td>
                    <td>{student.registration_number || "—"}</td>
                    <td>{student.grade_level}</td>
                    <td>{student.track || "Foundation"}</td>
                    <td>{student.batch_name || student.batch_id || "—"}</td>
                    <td>{student.student_mobile || "—"}</td>
                    <td>{student.student_email || "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <div className="pagination-controls">
              <button
                className="secondary-button"
                disabled={studentPage <= 1}
                onClick={() => onLoadStudents(Number(studentSchoolId), studentPage - 1, studentSearch)}
              >
                Previous
              </button>
              <button
                className="secondary-button"
                disabled={studentPage * studentPageSize >= studentTotalCount}
                onClick={() => onLoadStudents(Number(studentSchoolId), studentPage + 1, studentSearch)}
              >
                Next
              </button>
            </div>
          </>
        )}
          </>
        )}
      </section>

      {showSchoolForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => { setShowSchoolForm(false); setEditingSchoolId(null); }}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>{editingSchoolId != null ? "Edit School" : "Add School"}</h3>
              <form onSubmit={submitSchool} className="school-profile-form">
                <div className="form-tabs">
                  {(["basic", "sip", "center", "principal"] as const).map((t) => (
                    <button key={t} type="button" className={schoolTab === t ? "active" : ""} onClick={() => setSchoolTab(t)}>
                      {t === "basic" ? "Basic Info" : t === "sip" ? "SAH/SAL" : t === "center" ? "Center Head" : "Principal"}
                    </button>
                  ))}
                </div>
                {schoolTab === "basic" && (
                  <fieldset>
                    <legend>Basic Info</legend>
                    <label>
                      Name
                      <input
                        required
                        value={schoolForm.name}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, name: e.target.value }))}
                      />
                    </label>
                    <label>
                      Region
                      <select
                        value={schoolForm.region_id ?? ""}
                        onChange={(e) =>
                          setSchoolForm((s) => ({ ...s, region_id: e.target.value ? Number(e.target.value) : null }))
                        }
                      >
                        <option value="">—</option>
                        {regions.map((r) => (
                          <option key={r.id} value={r.id}>
                            {r.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Program Model
                      <input
                        value={schoolForm.program_model}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, program_model: e.target.value }))}
                      />
                    </label>
                    <label>
                      Distance Classification
                      <input
                        value={schoolForm.distance_classification}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, distance_classification: e.target.value }))}
                      />
                    </label>
                    <label>
                      VP - Tagging
                      <input
                        value={schoolForm.vp_tagging}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, vp_tagging: e.target.value }))}
                      />
                    </label>
                  </fieldset>
                )}
                {schoolTab === "sip" && (
                  <fieldset>
                    <legend>SAH / SAL</legend>
                    <label>
                      Role
                      <input
                        value={schoolForm.sip_academic_owner_role}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, sip_academic_owner_role: e.target.value }))}
                      />
                    </label>
                    <label>
                      Name
                      <input
                        value={schoolForm.sip_academic_owner_name}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, sip_academic_owner_name: e.target.value }))}
                      />
                    </label>
                    <label>
                      Mobile
                      <input
                        value={schoolForm.sip_academic_owner_mobile}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, sip_academic_owner_mobile: e.target.value }))}
                      />
                    </label>
                    <label>
                      Email
                      <input
                        value={schoolForm.sip_academic_owner_email}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, sip_academic_owner_email: e.target.value }))}
                      />
                    </label>
                  </fieldset>
                )}
                {schoolTab === "center" && (
                  <fieldset>
                    <legend>Center Head</legend>
                    <label>
                      Name
                      <input
                        value={schoolForm.center_head_name}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, center_head_name: e.target.value }))}
                      />
                    </label>
                    <label>
                      Mobile
                      <input
                        value={schoolForm.center_head_mobile}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, center_head_mobile: e.target.value }))}
                      />
                    </label>
                    <label>
                      Email
                      <input
                        value={schoolForm.center_head_email}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, center_head_email: e.target.value }))}
                      />
                    </label>
                  </fieldset>
                )}
                {schoolTab === "principal" && (
                  <fieldset>
                    <legend>Principal</legend>
                    <label>
                      Name
                      <input
                        value={schoolForm.principal_name}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, principal_name: e.target.value }))}
                      />
                    </label>
                    <label>
                      Mobile
                      <input
                        value={schoolForm.principal_mobile}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, principal_mobile: e.target.value }))}
                      />
                    </label>
                    <label>
                      Email
                      <input
                        value={schoolForm.principal_email}
                        onChange={(e) => setSchoolForm((s) => ({ ...s, principal_email: e.target.value }))}
                      />
                    </label>
                  </fieldset>
                )}
                {formError && <div className="form-error">{formError}</div>}
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowSchoolForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}

      {dropSchoolId &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setDropSchoolId(null)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>Drop School</h3>
              <p className="read-only-notice">Dropping preserves school history and removes it from active operational lists.</p>
              <label>
                Reason
                <textarea
                  required
                  value={dropReason}
                  onChange={(e) => setDropReason(e.target.value)}
                  placeholder="Example: SIP partnership ended for 2026 session"
                />
              </label>
              <div className="form-actions">
                <button
                  className="danger-button"
                  disabled={!dropReason.trim()}
                  onClick={() => {
                    onDropSchool(dropSchoolId, dropReason);
                    setDropSchoolId(null);
                    setDropReason("");
                  }}
                >
                  Drop School
                </button>
                <button className="secondary-button" onClick={() => setDropSchoolId(null)}>
                  Cancel
                </button>
              </div>
            </div>
          </div>,
          document.body,
        )}

      {showRegionForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowRegionForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>{editingRegionId ? "Edit Region" : "Add Region"}</h3>
              <form onSubmit={submitRegion} className="region-form">
                <fieldset>
                  <legend>Region Info</legend>
                  <label>
                    Name
                    <input
                      required
                      value={regionForm.name}
                      onChange={(e) => setRegionForm((s) => ({ ...s, name: e.target.value }))}
                    />
                  </label>
                  <label>
                    RAH Name
                    <input
                      value={regionForm.regional_academic_head_name}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_academic_head_name: e.target.value }))}
                    />
                  </label>
                  <label>
                    RAH Mobile
                    <input
                      value={regionForm.regional_academic_head_mobile}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_academic_head_mobile: e.target.value }))}
                    />
                  </label>
                  <label>
                    RAH Email
                    <input
                      value={regionForm.regional_academic_head_email}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_academic_head_email: e.target.value }))}
                    />
                  </label>
                </fieldset>
                <fieldset>
                  <legend>Regional Business Head (RBH)</legend>
                  <label>
                    RBH Name
                    <input
                      value={regionForm.regional_business_head_name}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_business_head_name: e.target.value }))}
                    />
                  </label>
                  <label>
                    RBH Mobile
                    <input
                      value={regionForm.regional_business_head_mobile}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_business_head_mobile: e.target.value }))}
                    />
                  </label>
                  <label>
                    RBH Email
                    <input
                      value={regionForm.regional_business_head_email}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_business_head_email: e.target.value }))}
                    />
                  </label>
                </fieldset>
                <fieldset>
                  <legend>Deputy Regional Academic Head (Deputy RAH)</legend>
                  <label>
                    Deputy RAH Name
                    <input
                      value={regionForm.regional_deputy_academic_head_name}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_deputy_academic_head_name: e.target.value }))}
                    />
                  </label>
                  <label>
                    Deputy RAH Mobile
                    <input
                      value={regionForm.regional_deputy_academic_head_mobile}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_deputy_academic_head_mobile: e.target.value }))}
                    />
                  </label>
                  <label>
                    Deputy RAH Email
                    <input
                      value={regionForm.regional_deputy_academic_head_email}
                      onChange={(e) => setRegionForm((s) => ({ ...s, regional_deputy_academic_head_email: e.target.value }))}
                    />
                  </label>
                </fieldset>
                {formError && <div className="form-error">{formError}</div>}
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowRegionForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}

      {showLectureModelForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowLectureModelForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>Add Lecture Model</h3>
              <form onSubmit={submitLectureModel} className="lecture-model-form">
                <label>
                  Name
                  <input
                    required
                    value={lectureModelForm.name}
                    onChange={(e) => setLectureModelForm((s) => ({ ...s, name: e.target.value }))}
                  />
                </label>
                <label>
                  Days per Week
                  <input
                    required
                    type="number"
                    min={1}
                    value={lectureModelForm.days_per_week}
                    onChange={(e) => setLectureModelForm((s) => ({ ...s, days_per_week: e.target.value }))}
                  />
                </label>
                <label>
                  Lectures per Day
                  <input
                    required
                    type="number"
                    min={1}
                    value={lectureModelForm.lectures_per_day}
                    onChange={(e) => setLectureModelForm((s) => ({ ...s, lectures_per_day: e.target.value }))}
                  />
                </label>
                {formError && <div className="form-error">{formError}</div>}
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowLectureModelForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}

      {showClassPlanForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowClassPlanForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>Add Class Offering</h3>
              <form onSubmit={submitClassPlan} className="class-plan-form">
                <fieldset>
                  <legend>Metadata</legend>
                  <label>
                    School
                    <select
                      required
                      value={classPlanForm.school_id}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, school_id: e.target.value }))}
                    >
                      <option value="">—</option>
                      {schools.map((s) => (
                        <option key={s.id} value={s.id}>
                          {s.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Grade Level
                    <select
                      required
                      value={classPlanForm.grade_level}
                      onChange={(e) => {
                        const grade = e.target.value;
                        const needsTrack = trackEligibleGrades.has(grade);
                        setClassPlanForm((s) => ({
                          ...s,
                          grade_level: grade,
                          track: needsTrack ? s.track : "",
                        }));
                      }}
                    >
                      <option value="">—</option>
                      {gradeLevels.map((g) => (
                        <option key={g} value={g}>
                          {g}
                        </option>
                      ))}
                    </select>
                  </label>
                  {trackEligibleGrades.has(classPlanForm.grade_level) && (
                    <label>
                      Track
                      <select
                        required
                        value={classPlanForm.track}
                        onChange={(e) => setClassPlanForm((s) => ({ ...s, track: e.target.value }))}
                      >
                        <option value="">—</option>
                        {["JEE", "NEET", "Regional"].map((t) => (
                          <option key={t} value={t}>
                            {t}
                          </option>
                        ))}
                      </select>
                    </label>
                  )}
                </fieldset>
                <fieldset>
                  <legend>Configuration</legend>
                  <label>
                    Lecture Model
                    <select
                      required
                      value={classPlanForm.lecture_model_id}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, lecture_model_id: e.target.value }))}
                    >
                      <option value="">—</option>
                      {lectureModels.map((lm) => (
                        <option key={lm.id} value={lm.id}>
                          {lm.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Delivery Pattern
                    <select
                      required
                      value={classPlanForm.batch_pattern}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, batch_pattern: e.target.value }))}
                    >
                      <option value="">—</option>
                      {batchPatterns.map((bp) => (
                        <option key={bp} value={bp}>
                          {bp}
                        </option>
                      ))}
                    </select>
                  </label>
                </fieldset>
                <fieldset>
                  <legend>Admissions</legend>
                  <label>
                    AOP Admissions
                    <input
                      required
                      type="number"
                      min={0}
                      value={classPlanForm.aop_admissions}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, aop_admissions: e.target.value }))}
                    />
                  </label>
                  <label>
                    Registrations
                    <input
                      required
                      type="number"
                      min={0}
                      value={classPlanForm.registrations}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, registrations: e.target.value }))}
                    />
                  </label>
                  <label>
                    Actual Admissions
                    <input
                      required
                      type="number"
                      min={0}
                      value={classPlanForm.actual_admissions}
                      onChange={(e) => setClassPlanForm((s) => ({ ...s, actual_admissions: e.target.value }))}
                    />
                  </label>
                </fieldset>
                {formError && <div className="form-error">{formError}</div>}
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowClassPlanForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}
    </section>
  );
}

export function VpCentersPanel({
  vpCenters,
  buildings,
  onClose,
  onCreateVpCenter,
  onUpdateVpCenter,
  onDeleteVpCenter,
  onCreateBuilding,
  onUpdateBuilding,
  onDeleteBuilding,
}: {
  vpCenters: VpCenter[];
  buildings: VpCenterBuilding[];
  onClose: () => void;
  onCreateVpCenter: (input: CreateVpCenterInput) => void;
  onUpdateVpCenter: (input: UpdateVpCenterInput) => void;
  onDeleteVpCenter: (id: number) => void;
  onCreateBuilding: (input: CreateVpCenterBuildingInput) => void;
  onUpdateBuilding: (input: UpdateVpCenterBuildingInput) => void;
  onDeleteBuilding: (id: number) => void;
}) {
  const [expandedCenterId, setExpandedCenterId] = React.useState<number | null>(null);
  const [showVpCenterForm, setShowVpCenterForm] = React.useState(false);
  const [showBuildingForm, setShowBuildingForm] = React.useState(false);
  const [editingVpCenter, setEditingVpCenter] = React.useState<VpCenter | null>(null);
  const [editingBuilding, setEditingBuilding] = React.useState<VpCenterBuilding | null>(null);
  const [buildingVpCenterId, setBuildingVpCenterId] = React.useState<number | null>(null);
  const [isSaving, setIsSaving] = React.useState(false);

  const [vpCenterForm, setVpCenterForm] = React.useState({
    name: "",
    location: "",
    contact_name: "",
    contact_mobile: "",
    contact_email: "",
  });

  const [buildingForm, setBuildingForm] = React.useState({
    vp_center_id: "",
    building_name: "",
    address: "",
    center_head_name: "",
    center_head_mobile: "",
    center_head_email: "",
    associate_center_head_name: "",
    associate_center_head_mobile: "",
    associate_center_head_email: "",
  });

  function resetVpCenterForm() {
    setVpCenterForm({
      name: "",
      location: "",
      contact_name: "",
      contact_mobile: "",
      contact_email: "",
    });
  }

  function resetBuildingForm() {
    setBuildingForm({
      vp_center_id: "",
      building_name: "",
      address: "",
      center_head_name: "",
      center_head_mobile: "",
      center_head_email: "",
      associate_center_head_name: "",
      associate_center_head_mobile: "",
      associate_center_head_email: "",
    });
  }

  function openAddVpCenter() {
    setEditingVpCenter(null);
    resetVpCenterForm();
    setShowVpCenterForm(true);
  }

  function openEditVpCenter(center: VpCenter) {
    setEditingVpCenter(center);
    setVpCenterForm({
      name: center.name,
      location: center.location,
      contact_name: center.contact_name,
      contact_mobile: center.contact_mobile,
      contact_email: center.contact_email,
    });
    setShowVpCenterForm(true);
  }

  function openAddBuilding(vpCenterId: number) {
    setEditingBuilding(null);
    setBuildingVpCenterId(vpCenterId);
    resetBuildingForm();
    setBuildingForm((s) => ({ ...s, vp_center_id: String(vpCenterId) }));
    setShowBuildingForm(true);
  }

  function openEditBuilding(building: VpCenterBuilding) {
    setEditingBuilding(building);
    setBuildingVpCenterId(building.vp_center_id);
    setBuildingForm({
      vp_center_id: String(building.vp_center_id),
      building_name: building.building_name,
      address: building.address,
      center_head_name: building.center_head_name,
      center_head_mobile: building.center_head_mobile,
      center_head_email: building.center_head_email,
      associate_center_head_name: building.associate_center_head_name,
      associate_center_head_mobile: building.associate_center_head_mobile,
      associate_center_head_email: building.associate_center_head_email,
    });
    setShowBuildingForm(true);
  }

  function submitVpCenter(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    try {
      if (editingVpCenter) {
        onUpdateVpCenter({
          id: editingVpCenter.id,
          ...vpCenterForm,
        });
      } else {
        onCreateVpCenter(vpCenterForm);
      }
    } finally {
      setIsSaving(false);
    }
    setShowVpCenterForm(false);
    resetVpCenterForm();
    setEditingVpCenter(null);
  }

  function submitBuilding(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    try {
      const base = {
        vp_center_id: Number(buildingForm.vp_center_id),
        building_name: buildingForm.building_name,
        address: buildingForm.address,
        center_head_name: buildingForm.center_head_name,
        center_head_mobile: buildingForm.center_head_mobile,
        center_head_email: buildingForm.center_head_email,
        associate_center_head_name: buildingForm.associate_center_head_name,
        associate_center_head_mobile: buildingForm.associate_center_head_mobile,
        associate_center_head_email: buildingForm.associate_center_head_email,
      };
      if (editingBuilding) {
        onUpdateBuilding({ id: editingBuilding.id, ...base });
      } else {
        onCreateBuilding(base);
      }
    } finally {
      setIsSaving(false);
    }
    setShowBuildingForm(false);
    resetBuildingForm();
    setEditingBuilding(null);
    setBuildingVpCenterId(null);
  }

  function handleDeleteVpCenter(id: number) {
    if (confirm("Are you sure you want to delete this VP Center?")) {
      onDeleteVpCenter(id);
    }
  }

  function handleDeleteBuilding(id: number) {
    if (confirm("Are you sure you want to delete this building?")) {
      onDeleteBuilding(id);
    }
  }

  return (
    <section className="ticket-modal" aria-label="VP Centers">
      <header>
        <h2>VP Centers</h2>
        <div className="actions">
          <button className="primary-action" onClick={openAddVpCenter}>
            Add VP Center
          </button>
          <button type="button" className="ghost-button" onClick={onClose}>
            Close
          </button>
        </div>
      </header>

      {vpCenters.length === 0 ? (
        <p className="empty-state">No VP centers.</p>
      ) : (
        <div className="vp-centers-list">
          {vpCenters.map((center) => {
            const centerBuildings = buildings.filter((b) => b.vp_center_id === center.id);
            const isExpanded = expandedCenterId === center.id;
            return (
              <div key={center.id} className="vp-center-card">
                <div className="vp-center-header">
                  <div className="vp-center-info">
                    <strong>{center.name}</strong>
                    <span>{center.location}</span>
                    <span>
                      {center.contact_name}
                      {center.contact_mobile && ` · ${center.contact_mobile}`}
                      {center.contact_email && ` · ${center.contact_email}`}
                    </span>
                  </div>
                  <div className="vp-center-actions">
                    <button className="secondary-button" onClick={() => openEditVpCenter(center)}>
                      Edit
                    </button>
                    <button className="secondary-button" onClick={() => handleDeleteVpCenter(center.id)}>
                      Delete
                    </button>
                    <button className="secondary-button" onClick={() => openAddBuilding(center.id)}>
                      Add Building
                    </button>
                    <button
                      className="secondary-button"
                      onClick={() => setExpandedCenterId(isExpanded ? null : center.id)}
                    >
                      {isExpanded ? "Collapse" : "Expand"}
                    </button>
                  </div>
                </div>
                {isExpanded && (
                  <div className="vp-center-buildings">
                    {centerBuildings.length === 0 ? (
                      <p className="empty-state">No buildings for this center.</p>
                    ) : (
                      <table className="data-table">
                        <thead>
                          <tr>
                            <th>Building</th>
                            <th>Address</th>
                            <th>Center Head</th>
                            <th>Associate CH</th>
                            <th>Actions</th>
                          </tr>
                        </thead>
                        <tbody>
                          {centerBuildings.map((building) => (
                            <tr key={building.id}>
                              <td>{building.building_name}</td>
                              <td>{building.address || "—"}</td>
                              <td>
                                {building.center_head_name || "—"}
                                {building.center_head_mobile && <div>{building.center_head_mobile}</div>}
                                {building.center_head_email && <div>{building.center_head_email}</div>}
                              </td>
                              <td>
                                {building.associate_center_head_name || "—"}
                                {building.associate_center_head_mobile && <div>{building.associate_center_head_mobile}</div>}
                                {building.associate_center_head_email && <div>{building.associate_center_head_email}</div>}
                              </td>
                              <td>
                                <button className="secondary-button" onClick={() => openEditBuilding(building)}>
                                  Edit
                                </button>
                                <button className="secondary-button" onClick={() => handleDeleteBuilding(building.id)}>
                                  Delete
                                </button>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {showVpCenterForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowVpCenterForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>{editingVpCenter ? "Edit VP Center" : "Add VP Center"}</h3>
              <form onSubmit={submitVpCenter} className="vp-center-form">
                <label>
                  Name
                  <input
                    required
                    value={vpCenterForm.name}
                    onChange={(e) => setVpCenterForm((s) => ({ ...s, name: e.target.value }))}
                  />
                </label>
                <label>
                  Location
                  <input
                    value={vpCenterForm.location}
                    onChange={(e) => setVpCenterForm((s) => ({ ...s, location: e.target.value }))}
                  />
                </label>
                <label>
                  Contact Name
                  <input
                    value={vpCenterForm.contact_name}
                    onChange={(e) => setVpCenterForm((s) => ({ ...s, contact_name: e.target.value }))}
                  />
                </label>
                <label>
                  Contact Mobile
                  <input
                    value={vpCenterForm.contact_mobile}
                    onChange={(e) => setVpCenterForm((s) => ({ ...s, contact_mobile: e.target.value }))}
                  />
                </label>
                <label>
                  Contact Email
                  <input
                    type="email"
                    value={vpCenterForm.contact_email}
                    onChange={(e) => setVpCenterForm((s) => ({ ...s, contact_email: e.target.value }))}
                  />
                </label>
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowVpCenterForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}

      {showBuildingForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowBuildingForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>{editingBuilding ? "Edit Building" : "Add Building"}</h3>
              <form onSubmit={submitBuilding} className="building-form">
                <label>
                  VP Center
                  <select
                    required
                    value={buildingForm.vp_center_id}
                    onChange={(e) => setBuildingForm((s) => ({ ...s, vp_center_id: e.target.value }))}
                  >
                    <option value="">—</option>
                    {vpCenters.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Building Name
                  <input
                    required
                    value={buildingForm.building_name}
                    onChange={(e) => setBuildingForm((s) => ({ ...s, building_name: e.target.value }))}
                  />
                </label>
                <label>
                  Address
                  <input
                    value={buildingForm.address}
                    onChange={(e) => setBuildingForm((s) => ({ ...s, address: e.target.value }))}
                  />
                </label>
                <fieldset>
                  <legend>Center Head</legend>
                  <label>
                    Name
                    <input
                      value={buildingForm.center_head_name}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, center_head_name: e.target.value }))}
                    />
                  </label>
                  <label>
                    Mobile
                    <input
                      value={buildingForm.center_head_mobile}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, center_head_mobile: e.target.value }))}
                    />
                  </label>
                  <label>
                    Email
                    <input
                      type="email"
                      value={buildingForm.center_head_email}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, center_head_email: e.target.value }))}
                    />
                  </label>
                </fieldset>
                <fieldset>
                  <legend>Associate Center Head</legend>
                  <label>
                    Name
                    <input
                      value={buildingForm.associate_center_head_name}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, associate_center_head_name: e.target.value }))}
                    />
                  </label>
                  <label>
                    Mobile
                    <input
                      value={buildingForm.associate_center_head_mobile}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, associate_center_head_mobile: e.target.value }))}
                    />
                  </label>
                  <label>
                    Email
                    <input
                      type="email"
                      value={buildingForm.associate_center_head_email}
                      onChange={(e) => setBuildingForm((s) => ({ ...s, associate_center_head_email: e.target.value }))}
                    />
                  </label>
                </fieldset>
                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowBuildingForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}
    </section>
  );
}

export function ProgramDashboardPanel({
  dashboard,
}: {
  dashboard: { total_schools: number; total_batches: number; total_students: number; active_tickets: number } | null;
}) {
  if (!dashboard) return <p className="empty-state">No dashboard data.</p>;
  return (
    <section className="ticket-modal" aria-label="Program dashboard">
      <header>
        <h2>Program Dashboard</h2>
      </header>
      <div className="metrics">
        <div className="metric-box">
          <strong>{dashboard.total_schools}</strong>
          <span>Schools</span>
        </div>
        <div className="metric-box">
          <strong>{dashboard.total_batches}</strong>
          <span>Batches</span>
        </div>
        <div className="metric-box">
          <strong>{dashboard.total_students}</strong>
          <span>Students</span>
        </div>
        <div className="metric-box">
          <strong>{dashboard.active_tickets}</strong>
          <span>Active Tickets</span>
        </div>
      </div>
    </section>
  );
}

export function RegionHistoryPanel({
  history,
  schools,
}: {
  history: Array<{ school_id: number; old_region_id: number | null; new_region_id: number | null; changed_at: string; changed_by: string }>;
  schools: School[];
}) {
  return (
    <section className="ticket-modal" aria-label="Region history">
      <header>
        <h2>Region History</h2>
      </header>
      {history.length === 0 ? (
        <p className="empty-state">No region changes recorded.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Old Region</th>
              <th>New Region</th>
              <th>Batch</th>
              <th>By</th>
            </tr>
          </thead>
          <tbody>
            {history.map((item, i) => (
              <tr key={i}>
                <td>{schools.find((s) => s.id === item.school_id)?.name ?? item.school_id}</td>
                <td>{item.old_region_id ?? "—"}</td>
                <td>{item.new_region_id ?? "—"}</td>
                <td>{formatTimestamp(item.changed_at)}</td>
                <td>{item.changed_by}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function AuditLogPanel({
  entries,
}: {
  entries: Array<{ id: number; entity_type: string; entity_id: number; action: string; actor: string; created_at: string; details: string | null }>;
}) {
  return (
    <section className="ticket-modal" aria-label="Audit log">
      <header>
        <h2>Audit Log</h2>
      </header>
      {entries.length === 0 ? (
        <p className="empty-state">No audit entries.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Actor</th>
              <th>Action</th>
              <th>Entity</th>
              <th>Details</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((entry) => (
              <tr key={entry.id}>
                <td>{formatTimestamp(entry.created_at)}</td>
                <td>{entry.actor}</td>
                <td>{entry.action}</td>
                <td>
                  {entry.entity_type} #{entry.entity_id}
                </td>
                <td>{entry.details ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function CommunicationOperationsPanel({
  templates,
  onAddTemplate,
  onToggleTemplate,
  currentUserRole,
}: {
  templates: CommunicationTemplate[];
  onAddTemplate: () => void;
  onToggleTemplate: (id: number) => void;
  currentUserRole: string;
}) {
  const isAdmin = currentUserRole === "admin";
  return (
    <section className="ticket-modal" aria-label="Communication operations">
      <header>
        <h2>Communication Operations</h2>
        {isAdmin && (
          <button className="primary-action" onClick={onAddTemplate}>
            Add Template
          </button>
        )}
      </header>
      {!isAdmin && (
        <p className="read-only-notice">Templates are read-only for your role.</p>
      )}
      {templates.length === 0 ? (
        <p className="empty-state">No templates.</p>
      ) : (
        <div className="template-list">
          {templates.map((template) => (
            <div key={template.id} className={`template-card ${template.is_active ? "active" : ""}`}>
              <div className="template-header">
                <strong>{template.name}</strong>
                {isAdmin && (
                  <button
                    className={template.is_active ? "secondary-button" : "primary-action"}
                    onClick={() => onToggleTemplate(template.id)}
                  >
                    {template.is_active ? "Deactivate" : "Activate"}
                  </button>
                )}
              </div>
              <span className="template-audience">{template.audience}</span>
              <p className="template-body">{template.body}</p>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

export function CommunicationTemplatePanel({
  templates,
  onAddTemplate,
}: {
  templates: CommunicationTemplate[];
  onAddTemplate: () => void;
}) {
  return (
    <section className="ticket-modal" aria-label="Communication templates">
      <header>
        <h2>Communication Templates</h2>
        <button className="primary-action" onClick={onAddTemplate}>
          Add Template
        </button>
      </header>
      {templates.length === 0 ? (
        <p className="empty-state">No templates.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Audience</th>
              <th>Body</th>
            </tr>
          </thead>
          <tbody>
            {templates.map((template) => (
              <tr key={template.id}>
                <td>{template.name}</td>
                <td>{template.audience}</td>
                <td>{template.body}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function normalizeFacultyRole(role: string | null | undefined) {
  const trimmed = (role ?? "").trim();
  return trimmed.toLowerCase() === "teacher" || !trimmed ? "Faculty" : trimmed;
}

export function DirectoryPanel({
  schools,
  regions = [],
  facultyMembers = [],
  facultyMemberships = {},
  users = [],
  onExport,
  onLoadFacultyMemberships,
}: {
  schools: School[];
  regions?: Region[];
  facultyMembers?: FacultyMember[];
  facultyMemberships?: Record<number, FacultySchoolMembership[]>;
  users?: AppUser[];
  onExport: () => void;
  onLoadFacultyMemberships?: (facultyId: number) => void;
}) {
  type DirectoryContact = {
    id: string;
    name: string;
    role: string;
    contactType: "Internal" | "External";
    schoolName: string;
    regionName: string;
    source: string;
    mobileNumbers: string[];
    emails: string[];
    tags: string[];
    active: boolean;
  };

  const [search, setSearch] = React.useState("");
  const [typeFilter, setTypeFilter] = React.useState<"All" | "Internal" | "External">("All");
  const [roleFilter, setRoleFilter] = React.useState("All");
  const [schoolFilter, setSchoolFilter] = React.useState("All");
  const [selectedIds, setSelectedIds] = React.useState<Set<string>>(new Set());
  const [copyNotice, setCopyNotice] = React.useState("");

  React.useEffect(() => {
    if (!onLoadFacultyMemberships) return;
    facultyMembers.forEach((member) => {
      if (!facultyMemberships[member.id]) {
        onLoadFacultyMemberships(member.id);
      }
    });
  }, [facultyMembers, facultyMemberships, onLoadFacultyMemberships]);

  const splitContacts = (value: string | null | undefined) =>
    (value ?? "")
      .split(/[;,/|]+/)
      .map((item) => item.trim())
      .filter(Boolean);
  const addSchoolContact = (
    contacts: DirectoryContact[],
    school: School,
    role: string,
    name: string,
    mobiles: string,
    emails: string,
    contactType: "Internal" | "External",
    source: string,
  ) => {
    const mobileNumbers = splitContacts(mobiles);
    const emailList = splitContacts(emails);
    if (!name.trim() && mobileNumbers.length === 0 && emailList.length === 0) return;
    contacts.push({
      id: `${school.id}-${role}-${name || mobileNumbers.join("-") || emailList.join("-")}`,
      name: name.trim() || "Unnamed contact",
      role,
      contactType,
      schoolName: school.name,
      regionName: school.region_name || "",
      source,
      mobileNumbers,
      emails: emailList,
      tags: [school.name, school.region_name, role, source].filter(Boolean),
      active: !school.is_dropped,
    });
  };

  const contacts = React.useMemo(() => {
    const rows: DirectoryContact[] = [];
    regions.forEach((region) => {
      if ((region.regional_academic_head_name ?? "").trim() || (region.regional_academic_head_mobile ?? "").trim() || (region.regional_academic_head_email ?? "").trim()) {
        rows.push({
          id: `region-${region.id}-rah`,
          name: (region.regional_academic_head_name ?? "").trim() || "Unnamed contact",
          role: "Regional Academic Head",
          contactType: "Internal",
          schoolName: region.name,
          regionName: region.name,
          source: "Region Master",
          mobileNumbers: splitContacts(region.regional_academic_head_mobile ?? ""),
          emails: splitContacts(region.regional_academic_head_email ?? ""),
          tags: [region.name, "Regional Academic Head", "Region Master"],
          active: true,
        });
      }
      if ((region.regional_business_head_name ?? "").trim() || (region.regional_business_head_mobile ?? "").trim() || (region.regional_business_head_email ?? "").trim()) {
        rows.push({
          id: `region-${region.id}-rbh`,
          name: (region.regional_business_head_name ?? "").trim() || "Unnamed contact",
          role: "Regional Business Head",
          contactType: "Internal",
          schoolName: region.name,
          regionName: region.name,
          source: "Region Master",
          mobileNumbers: splitContacts(region.regional_business_head_mobile ?? ""),
          emails: splitContacts(region.regional_business_head_email ?? ""),
          tags: [region.name, "Regional Business Head", "Region Master"],
          active: true,
        });
      }
      if ((region.regional_deputy_academic_head_name ?? "").trim() || (region.regional_deputy_academic_head_mobile ?? "").trim() || (region.regional_deputy_academic_head_email ?? "").trim()) {
        rows.push({
          id: `region-${region.id}-deputy-rah`,
          name: (region.regional_deputy_academic_head_name ?? "").trim() || "Unnamed contact",
          role: "Deputy Regional Academic Head",
          contactType: "Internal",
          schoolName: region.name,
          regionName: region.name,
          source: "Region Master",
          mobileNumbers: splitContacts(region.regional_deputy_academic_head_mobile ?? ""),
          emails: splitContacts(region.regional_deputy_academic_head_email ?? ""),
          tags: [region.name, "Deputy Regional Academic Head", "Region Master"],
          active: true,
        });
      }
    });
    schools.forEach((school) => {
      addSchoolContact(rows, school, "Principal", school.principal_name, school.principal_mobile, school.principal_email, "External", "School Master");
      addSchoolContact(rows, school, "School SPOC", school.school_spoc_name, school.school_spoc_mobile, school.school_spoc_email, "External", "School Master");
      addSchoolContact(rows, school, "Center Head", school.center_head_name, school.center_head_mobile, school.center_head_email, "Internal", "School Master");
      addSchoolContact(rows, school, (school.sip_academic_owner_role ?? "").trim() || "SIP Academic Head", school.sip_academic_owner_name ?? "", school.sip_academic_owner_mobile ?? "", school.sip_academic_owner_email ?? "", "Internal", "School Master");
      addSchoolContact(rows, school, "RAH / Academic SPOC", school.central_academic_spoc_name ?? "", school.central_academic_spoc_mobile ?? "", school.central_academic_spoc_email ?? "", "Internal", "School Master");
      addSchoolContact(rows, school, "RBH / Business SPOC", (school.central_business_spoc_name ?? "").trim() || (school.bh_name ?? "").trim(), (school.central_business_spoc_mobile ?? "").trim() || (school.bh_mobile ?? "").trim(), (school.central_business_spoc_email ?? "").trim() || (school.bh_email ?? "").trim(), "Internal", "School Master");
      addSchoolContact(rows, school, "AOM", school.aom_name ?? "", school.aom_mobile ?? "", school.aom_email ?? "", "Internal", "School Master");
    });

    facultyMembers.forEach((member) => {
      const membershipsForMember = facultyMemberships[member.id] ?? [];
      const membershipRows = membershipsForMember.length > 0 ? membershipsForMember : [null];
      membershipRows.forEach((membership) => {
        const school = membership ? schools.find((s) => s.id === membership.school_id) : null;
        rows.push({
          id: `faculty-${member.id}-${membership?.school_id ?? "unassigned"}`,
          name: member.name,
          role: normalizeFacultyRole(membership?.role_at_school),
          contactType: "Internal",
          schoolName: membership?.school_name || "Unassigned",
          regionName: school?.region_name || "",
          source: "Faculty Master",
          mobileNumbers: splitContacts(member.mobile),
          emails: splitContacts(member.email),
          tags: [
            member.name,
            member.pwid,
            member.designation,
            member.specialization,
            member.user_id ? "Linked login" : "No login",
            membership?.school_name,
          ].filter((tag): tag is string => Boolean(tag)),
          active: member.is_active,
        });
      });
    });

    users
      .filter((user) => user.role !== "faculty")
      .forEach((user) => {
        rows.push({
          id: `user-${user.id}`,
          name: user.display_name,
          role: (user.role ?? "").toUpperCase(),
          contactType: "Internal",
          schoolName: (user.school_ids ?? []).map((id) => schools.find((s) => s.id === id)?.name ?? id).join(", ") || "All schools",
          regionName: "",
          source: "User Account",
          mobileNumbers: [],
          emails: [],
          tags: [user.username, user.role, user.display_name],
          active: user.is_active,
        });
      });

    return rows.sort((a, b) => a.name.localeCompare(b.name));
  }, [facultyMembers, facultyMemberships, schools, users]);

  const roles = Array.from(new Set(contacts.map((contact) => contact.role))).sort();
  const schoolNames = Array.from(new Set(contacts.map((contact) => contact.schoolName).filter(Boolean))).sort();
  const filteredContacts = contacts.filter((contact) => {
    const haystack = [
      contact.name,
      contact.role,
      contact.contactType,
      contact.schoolName,
      contact.regionName,
      contact.source,
      ...contact.mobileNumbers,
      ...contact.emails,
      ...contact.tags,
    ].join(" ").toLowerCase();
    return (
      (!search.trim() || haystack.includes(search.trim().toLowerCase())) &&
      (typeFilter === "All" || contact.contactType === typeFilter) &&
      (roleFilter === "All" || contact.role === roleFilter) &&
      (schoolFilter === "All" || contact.schoolName === schoolFilter)
    );
  });

  const selectedContacts = filteredContacts.filter((contact) => selectedIds.has(contact.id));
  const selectedEmails = Array.from(new Set(selectedContacts.flatMap((contact) => contact.emails))).filter(Boolean);
  const selectedMissingEmails = selectedContacts.filter((contact) => contact.emails.length === 0).length;

  const copyText = async (text: string, notice: string) => {
    if (!text) return;
    try {
      await navigator.clipboard?.writeText(text);
      setCopyNotice(notice);
    } catch {
      setCopyNotice("Copy failed. Select the text manually.");
    }
  };

  const fullContactBlock = (contact: DirectoryContact) => [
    contact.name,
    `${contact.role}${contact.schoolName ? `, ${contact.schoolName}` : ""}`,
    contact.mobileNumbers.length ? `Mobile: ${contact.mobileNumbers.join(", ")}` : "",
    contact.emails.length ? `Email: ${contact.emails.join(", ")}` : "",
  ].filter(Boolean).join("\n");

  const selectedContactTable = selectedContacts
    .map((contact) => [contact.name, contact.role, contact.schoolName, contact.mobileNumbers.join(", "), contact.emails.join(", ")].join("\t"))
    .join("\n");

  const toggleSelected = (id: string) => {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const selectVisible = () => setSelectedIds(new Set(filteredContacts.map((contact) => contact.id)));
  const clearSelected = () => setSelectedIds(new Set());
  const openBulkEmail = () => {
    if (selectedEmails.length > 0) {
      window.location.href = `mailto:?bcc=${encodeURIComponent(selectedEmails.join(","))}`;
    }
  };

  return (
    <section className="ticket-modal" aria-label="Directory">
      <header>
        <div>
          <h2>Directory</h2>
          <p>Operational contact book for internal and external school coordination.</p>
        </div>
        <div className="directory-actions">
          <button className="secondary-button" onClick={selectVisible}>Select visible</button>
          <button className="secondary-button" onClick={clearSelected}>Clear</button>
          <button className="secondary-button" onClick={onExport}>Export</button>
        </div>
      </header>
      <div className="directory-toolbar">
        <input
          className="directory-search"
          type="search"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search name, role, school, mobile, email..."
          aria-label="Search directory"
        />
        <select value={typeFilter} onChange={(e) => setTypeFilter(e.target.value as "All" | "Internal" | "External")} aria-label="Contact type">
          <option>All</option>
          <option>Internal</option>
          <option>External</option>
        </select>
        <select value={roleFilter} onChange={(e) => setRoleFilter(e.target.value)} aria-label="Role">
          <option>All</option>
          {roles.map((role) => <option key={role}>{role}</option>)}
        </select>
        <select value={schoolFilter} onChange={(e) => setSchoolFilter(e.target.value)} aria-label="School">
          <option>All</option>
          {schoolNames.map((school) => <option key={school}>{school}</option>)}
        </select>
      </div>
      {selectedContacts.length > 0 && (
        <div className="directory-bulk-bar">
          <strong>{selectedContacts.length} selected</strong>
          <span>{selectedEmails.length} email(s), {selectedMissingEmails} missing email</span>
          <button className="primary-action" onClick={openBulkEmail} disabled={selectedEmails.length === 0}>Open email client</button>
          <button className="secondary-button" onClick={() => copyText(selectedEmails.join(", "), "Copied selected emails.")} disabled={selectedEmails.length === 0}>Copy emails</button>
          <button className="secondary-button" onClick={() => copyText(`bcc: ${selectedEmails.join(", ")}`, "Copied BCC list.")} disabled={selectedEmails.length === 0}>Copy BCC</button>
          <button className="secondary-button" onClick={() => copyText(selectedContactTable, "Copied contact table.")}>Copy table</button>
        </div>
      )}
      {copyNotice && <p className="read-only-notice">{copyNotice}</p>}
      <div className="directory-list">
        {filteredContacts.length === 0 ? (
          <p className="empty-state">No contacts match the current filters.</p>
        ) : filteredContacts.map((contact) => (
          <article key={contact.id} className={`directory-card ${contact.active ? "" : "inactive"}`}>
            <label className="directory-select">
              <input type="checkbox" checked={selectedIds.has(contact.id)} onChange={() => toggleSelected(contact.id)} />
              <span>Select</span>
            </label>
            <div className="directory-card-main">
              <strong>{contact.name}</strong>
              <span>{contact.role} · {contact.contactType}</span>
              <small>{contact.schoolName}{contact.regionName ? ` · ${contact.regionName}` : ""}</small>
              <small>{contact.source}{contact.active ? "" : " · Inactive"}</small>
            </div>
            <div className="directory-contact-lines">
              {contact.mobileNumbers.length === 0 && contact.emails.length === 0 && <span className="muted-helper">No contact details saved.</span>}
              {contact.mobileNumbers.map((mobile, index) => (
                <div key={`${contact.id}-mobile-${mobile}`} className="directory-contact-line">
                  <span>{index === 0 ? "Mobile" : `Mobile ${index + 1}`}</span>
                  <a href={`tel:${mobile}`}>{mobile}</a>
                  <button className="secondary-button" onClick={() => copyText(mobile, "Copied mobile.")}>Copy</button>
                </div>
              ))}
              {contact.emails.map((email, index) => (
                <div key={`${contact.id}-email-${email}`} className="directory-contact-line">
                  <span>{index === 0 ? "Email" : `Email ${index + 1}`}</span>
                  <a href={`mailto:${email}`}>{email}</a>
                  <button className="secondary-button" onClick={() => copyText(email, "Copied email.")}>Copy</button>
                </div>
              ))}
            </div>
            <div className="directory-card-actions">
              {contact.mobileNumbers[0] && <a className="secondary-button" href={`tel:${contact.mobileNumbers[0]}`}>Call</a>}
              {contact.emails[0] && <a className="secondary-button" href={`mailto:${contact.emails[0]}`}>Email</a>}
              <button className="secondary-button" onClick={() => copyText(fullContactBlock(contact), "Copied full contact.")}>Copy full</button>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

export function DroppedSchoolsPanel({
  schools,
  currentUserRole,
  onRestoreSchool,
  onDeleteSchool,
  onLoadSchoolDeleteImpact,
}: {
  schools: School[];
  currentUserRole: string;
  onRestoreSchool: (id: number) => void;
  onDeleteSchool: (id: number) => void;
  onLoadSchoolDeleteImpact: (id: number) => Promise<SchoolDeleteImpact | null>;
}) {
  const isAdmin = currentUserRole === "admin";
  const [confirmDeleteId, setConfirmDeleteId] = React.useState<number | null>(null);
  const [deleteImpact, setDeleteImpact] = React.useState<SchoolDeleteImpact | null>(null);
  const [isLoadingDeleteImpact, setIsLoadingDeleteImpact] = React.useState(false);
  const schoolPendingDelete = schools.find((school) => school.id === confirmDeleteId) ?? null;

  async function previewDelete(school: School) {
    setConfirmDeleteId(school.id);
    setDeleteImpact(null);
    setIsLoadingDeleteImpact(true);
    try {
      setDeleteImpact(await onLoadSchoolDeleteImpact(school.id));
    } finally {
      setIsLoadingDeleteImpact(false);
    }
  }

  function cancelDelete() {
    setConfirmDeleteId(null);
    setDeleteImpact(null);
    setIsLoadingDeleteImpact(false);
  }

  return (
    <section className="ticket-modal" aria-label="Dropped schools">
      <header>
        <h2>Dropped Schools</h2>
        <p>Dropped schools are inactive but preserved. Restore returns them to active operations.</p>
      </header>
      {schools.length === 0 ? (
        <p className="empty-state">No dropped schools.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Region</th>
              <th>Program</th>
              <th>Dropped</th>
              <th>Reason</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {schools.map((school) => (
              <tr key={school.id}>
                <td>{school.name}</td>
                <td>{school.region_name}</td>
                <td>{school.program_model}</td>
                <td>{school.dropped_at || "—"}</td>
                <td>{school.dropped_reason || "—"}</td>
                <td>
                  <div className="actions">
                    <button className="secondary-button" onClick={() => onRestoreSchool(school.id)}>
                      Restore
                    </button>
                    {isAdmin && (
                      <button
                        className="secondary-button"
                        onClick={() => void previewDelete(school)}
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {schoolPendingDelete && (
        <div className="modal-backdrop" role="presentation" onClick={cancelDelete}>
          <section
            className="modal-card delete-impact-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="dropped-school-delete-impact-title"
            onClick={(event) => event.stopPropagation()}
          >
            <header>
              <div>
                <h3 id="dropped-school-delete-impact-title">Permanent Delete Review</h3>
                <p>
                  Review all linked data before permanently deleting {schoolPendingDelete.name}.
                </p>
              </div>
              <button className="ghost-button" onClick={cancelDelete} aria-label="Close delete review">
                Close
              </button>
            </header>
            {isLoadingDeleteImpact ? (
              <p className="empty-state">Checking linked records...</p>
            ) : deleteImpact ? (
              <>
                <div className="impact-summary">
                  <strong>{deleteImpact.total_linked_records}</strong>
                  <span>linked records will be deleted or detached by this action.</span>
                </div>
                <table className="data-table compact-table">
                  <thead>
                    <tr>
                      <th>Area</th>
                      <th>Records</th>
                    </tr>
                  </thead>
                  <tbody>
                    {deleteImpact.items.filter((item) => item.count > 0).map((item) => (
                      <tr key={item.label}>
                        <td>{item.label}</td>
                        <td>{item.count}</td>
                      </tr>
                    ))}
                    {deleteImpact.items.every((item) => item.count === 0) && (
                      <tr>
                        <td colSpan={2}>No linked records found.</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </>
            ) : (
              <p className="form-error">Could not load delete impact. Try again before deleting.</p>
            )}
            <div className="actions">
              <button
                className="danger-button"
                disabled={!deleteImpact}
                onClick={() => {
                  onDeleteSchool(schoolPendingDelete.id);
                  cancelDelete();
                }}
              >
                Permanently Delete School
              </button>
              <button className="secondary-button" onClick={cancelDelete}>
                Cancel
              </button>
            </div>
          </section>
        </div>
      )}
    </section>
  );
}

export function UserManagementPanel({
  users,
  schools,
  currentUser,
  onClose,
  onCreateUser,
  onUpdateUser,
  onDeleteUser,
  onChangePassword,
  onResetPassword,
}: {
  users: AppUser[];
  schools: School[];
  currentUser: CurrentUser | null;
  onClose: () => void;
  onCreateUser: (draft: CreateUserDraft) => Promise<void>;
  onUpdateUser: (draft: UpdateUserDraft) => Promise<void>;
  onDeleteUser: (id: number) => Promise<void>;
  onChangePassword: (currentPassword: string, newPassword: string) => Promise<void>;
  onResetPassword: (id: number, newPassword: string) => Promise<void>;
}) {
  const [resetUserId, setResetUserId] = React.useState<number | null>(null);
  const [resetPassword, setResetPassword] = React.useState("");
  const [resetConfirm, setResetConfirm] = React.useState("");
  const [resetError, setResetError] = React.useState("");
  const [resetting, setResetting] = React.useState(false);
  const isAdmin = currentUser?.role === "admin";

  async function handleResetSubmit(e: React.FormEvent) {
    e.preventDefault();
    setResetError("");
    if (resetPassword.length < 6) {
      setResetError("Password must be at least 6 characters.");
      return;
    }
    if (resetPassword !== resetConfirm) {
      setResetError("Passwords do not match.");
      return;
    }
    if (resetUserId == null) return;
    setResetting(true);
    try {
      await onResetPassword(resetUserId, resetPassword);
      setResetUserId(null);
      setResetPassword("");
      setResetConfirm("");
    } catch (caught) {
      setResetError(String(caught));
    } finally {
      setResetting(false);
    }
  }

  return (
    <section className="ticket-modal" aria-label="User management">
      <header>
        <h2>Users</h2>
        <button className="ghost-button" onClick={onClose}>Close</button>
      </header>
      {users.length === 0 ? (
        <p className="empty-state">No users.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Username</th>
              <th>Role</th>
              <th>Schools</th>
              <th>Active</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id}>
                <td>{user.display_name}</td>
                <td>{user.username}</td>
                <td>{user.role}</td>
                <td>
                  {user.school_ids
                    .map((id) => schools.find((s) => s.id === id)?.name ?? id)
                    .join(", ")}
                </td>
                <td>{user.is_active ? "Yes" : "No"}</td>
                <td style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
                  <button
                    className={user.is_active ? "secondary-button" : "primary-action"}
                    onClick={() =>
                      onUpdateUser({
                        id: user.id,
                        username: user.username,
                        display_name: user.display_name,
                        role: user.role,
                        is_active: !user.is_active,
                        school_ids: user.school_ids,
                      })
                    }
                  >
                    {user.is_active ? "Deactivate" : "Activate"}
                  </button>
                  {isAdmin && user.id !== currentUser?.id && (
                    <button
                      className="secondary-button"
                      onClick={() => {
                        setResetUserId(user.id);
                        setResetPassword("");
                        setResetConfirm("");
                        setResetError("");
                      }}
                    >
                      Reset Password
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {resetUserId != null && (
        <div className="modal-backdrop" role="presentation" onClick={() => setResetUserId(null)}>
          <section
            className="modal-card"
            role="dialog"
            aria-modal="true"
            aria-labelledby="reset-password-title"
            onClick={(event) => event.stopPropagation()}
          >
            <header>
              <h3 id="reset-password-title">
                Reset Password for {users.find((u) => u.id === resetUserId)?.display_name}
              </h3>
              <button className="ghost-button" onClick={() => setResetUserId(null)} aria-label="Close">
                Close
              </button>
            </header>
            <form onSubmit={handleResetSubmit} className="master-form">
              <label>
                New Password
                <input
                  type="password"
                  value={resetPassword}
                  onChange={(e) => setResetPassword(e.target.value)}
                  required
                  minLength={6}
                />
              </label>
              <label>
                Confirm New Password
                <input
                  type="password"
                  value={resetConfirm}
                  onChange={(e) => setResetConfirm(e.target.value)}
                  required
                  minLength={6}
                />
              </label>
              {resetError ? <p className="form-error">{resetError}</p> : null}
              <div className="form-actions">
                <button type="button" className="secondary-button" onClick={() => setResetUserId(null)}>
                  Cancel
                </button>
                <button type="submit" className="primary-action" disabled={resetting}>
                  {resetting ? "Resetting…" : "Reset Password"}
                </button>
              </div>
            </form>
          </section>
        </div>
      )}
    </section>
  );
}

export function FacultyAssignmentsPanel({
  assignments,
  schools,
  batches,
  facultyMembers,
  subjects,
  onCreateAssignment,
  onDeleteAssignment,
}: {
  assignments: FacultyAssignment[];
  schools: School[];
  batches: Batch[];
  facultyMembers: FacultyMember[];
  subjects: Subject[];
  onCreateAssignment?: (input: {
    faculty_id: number;
    batch_id: number;
    subject_id: number;
  }) => void;
  onDeleteAssignment?: (id: number) => void;
}) {
  const [isAdding, setIsAdding] = React.useState(false);
  const [confirmDeleteId, setConfirmDeleteId] = React.useState<number | null>(null);
  const [draft, setDraft] = React.useState({
    faculty_id: 0,
    batch_id: 0,
    subject_id: 0,
  });

  const activeFaculty = facultyMembers.filter((m) => m.is_active);
  const selectedBatch = batches.find((b) => b.id === draft.batch_id);
  const assignmentTrack = selectedBatch ? (selectedBatch.track || "Foundation") : "";
  const subjectOptions = assignmentTrack
    ? subjects.filter((s) => s.track === assignmentTrack)
    : [];
  const selectedSubjectIsValid = subjectOptions.some((s) => s.id === draft.subject_id);

  const canSave =
    draft.faculty_id > 0 &&
    draft.batch_id > 0 &&
    selectedSubjectIsValid &&
    Boolean(selectedBatch);

  const resetDraft = () => {
    setDraft({ faculty_id: 0, batch_id: 0, subject_id: 0 });
  };

  const saveNew = () => {
    if (onCreateAssignment && canSave) {
      onCreateAssignment({
        faculty_id: draft.faculty_id,
        batch_id: draft.batch_id,
        subject_id: draft.subject_id,
      });
      setIsAdding(false);
      setConfirmDeleteId(null);
      resetDraft();
    }
  };

  const hasHandlers = onCreateAssignment || onDeleteAssignment;

  return (
    <section className="ticket-modal" aria-label="Faculty assignments">
      <header>
        <h2>Faculty Assignments</h2>
        {onCreateAssignment && (
          <button className="primary-action" onClick={() => setIsAdding(true)}>
            Add Assignment
          </button>
        )}
      </header>
      {onCreateAssignment && activeFaculty.length === 0 && (
        <p className="read-only-notice">No active faculty members available. Add faculty in Faculty Master first.</p>
      )}
      {onCreateAssignment && batches.length === 0 && (
        <p className="read-only-notice">No batches available. Create batches in Master Data before assigning faculty.</p>
      )}
      <table className="data-table">
        <thead>
          <tr>
            <th>Faculty</th>
            <th>School</th>
            <th>Batch</th>
            <th>Subject</th>
            <th>Grade</th>
            <th>Track</th>
            <th>Delivery</th>
            {hasHandlers && <th>Actions</th>}
          </tr>
        </thead>
        <tbody>
          {isAdding && onCreateAssignment && (
            <tr>
              <td>
                <select
                  value={draft.faculty_id || ""}
                  onChange={(e) => setDraft((prev) => ({ ...prev, faculty_id: Number(e.target.value) }))}
                  aria-label="Faculty"
                >
                  <option value="">Select faculty</option>
                  {activeFaculty.map((m) => (
                    <option key={m.id} value={m.id}>{m.name}{m.user_id ? "" : " (No login)"}</option>
                  ))}
                </select>
              </td>
              <td>
                {selectedBatch?.school_name || "—"}
              </td>
              <td>
                <select
                  value={draft.batch_id || ""}
                  onChange={(e) => setDraft((prev) => ({ ...prev, batch_id: Number(e.target.value), subject_id: 0 }))}
                  aria-label="Batch"
                >
                  <option value="">Select batch</option>
                  {batches.map((b) => (
                    <option key={b.id} value={b.id}>
                      {b.school_name} — {b.batch_id} — {b.grade_level}{b.track ? ` ${b.track}` : ""} — {b.batch_pattern}
                    </option>
                  ))}
                </select>
              </td>
              <td>
                <select
                  value={draft.subject_id || ""}
                  onChange={(e) => setDraft((prev) => ({ ...prev, subject_id: Number(e.target.value) }))}
                  aria-label="Subject"
                  disabled={!assignmentTrack}
                >
                  <option value="">{assignmentTrack ? "Select subject" : "Select batch first"}</option>
                  {subjectOptions.map((s) => (
                    <option key={s.id} value={s.id}>{s.name}</option>
                  ))}
                </select>
                {selectedBatch && selectedBatch.track === "" && (
                  <small className="muted-helper">Foundation subjects only.</small>
                )}
              </td>
              <td>{selectedBatch?.grade_level || "—"}</td>
              <td>{selectedBatch?.track || "—"}</td>
              <td>{selectedBatch?.batch_pattern || "—"}</td>
              <td>
                <button className="primary-action" onClick={saveNew} disabled={!canSave}>
                  Save
                </button>
                <button className="secondary-button" onClick={() => { setIsAdding(false); resetDraft(); }}>
                  Cancel
                </button>
              </td>
            </tr>
          )}
          {assignments.length === 0 && !isAdding ? (
            <tr>
              <td colSpan={hasHandlers ? 8 : 7} className="empty-state">No assignments.</td>
            </tr>
          ) : (
            assignments.map((a) => (
              <tr key={a.id}>
                <td>{a.faculty_display_name}</td>
                <td>{a.school_name}</td>
                <td>{a.batch_name}</td>
                <td>{a.subject_name}</td>
                <td>{a.grade_level}</td>
                <td>{a.track || "—"}</td>
                <td>{a.batch_pattern}</td>
                {hasHandlers && (
                  <td>
                    {onDeleteAssignment && (
                      <button
                        className={confirmDeleteId === a.id ? "danger-button" : "secondary-button"}
                        onClick={() => {
                          if (confirmDeleteId === a.id) {
                            onDeleteAssignment(a.id);
                            setConfirmDeleteId(null);
                          } else {
                            setConfirmDeleteId(a.id);
                          }
                        }}
                      >
                        {confirmDeleteId === a.id ? "Confirm Delete" : "Delete"}
                      </button>
                    )}
                  </td>
                )}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </section>
  );
}

export function FacultyDirectoryPanel({
  facultyProfiles,
  vpCenters,
  schools,
  subjects,
  batches,
  users,
  onClose,
  onUpsertProfile,
}: {
  facultyProfiles: FacultyProfile[];
  vpCenters: VpCenter[];
  schools: School[];
  subjects: Subject[];
  batches: Batch[];
  users: AppUser[];
  onClose: () => void;
  onUpsertProfile: (input: UpsertFacultyProfileInput) => void;
}) {
  const [search, setSearch] = React.useState("");
  const [filterVpCenterId, setFilterVpCenterId] = React.useState<number | "">("");
  const [filterSchoolId, setFilterSchoolId] = React.useState<number | "">("");
  const [filterEmploymentType, setFilterEmploymentType] = React.useState<string>("All");
  const [filterStatus, setFilterStatus] = React.useState<string>("All");

  const [showForm, setShowForm] = React.useState(false);
  const [editingProfile, setEditingProfile] = React.useState<FacultyProfile | null>(null);
  const [formTab, setFormTab] = React.useState<"basic" | "contact" | "professional" | "wings">("basic");
  const [isSaving, setIsSaving] = React.useState(false);

  const facultyUsers = React.useMemo(() => users.filter((u) => u.role === "faculty"), [users]);

  const defaultForm = {
    faculty_user_id: "" as number | "",
    pwid: "",
    email: "",
    mobile: "",
    emergency_contact_name: "",
    emergency_contact_mobile: "",
    vp_center_id: "" as number | "",
    sip_school_id: "" as number | "",
    primary_subject_id: "" as number | "",
    employment_type: "VP Payroll",
    qualification: "",
    experience_years: "" as number | "",
    designation: "",
    specialization: "",
    max_periods_per_week: "" as number | "",
    joining_date: "",
    exit_date: "",
    documents_verified: false,
    is_active: true,
    wings: "",
    batch_ids: [] as number[],
  };

  const [form, setForm] = React.useState({ ...defaultForm });

  function openAdd() {
    setEditingProfile(null);
    setForm({ ...defaultForm });
    setFormTab("basic");
    setShowForm(true);
  }

  function openEdit(profile: FacultyProfile) {
    setEditingProfile(profile);
    setForm({
      faculty_user_id: profile.faculty_user_id,
      pwid: profile.pwid,
      email: profile.email,
      mobile: profile.mobile,
      emergency_contact_name: profile.emergency_contact_name,
      emergency_contact_mobile: profile.emergency_contact_mobile,
      vp_center_id: profile.vp_center_id ?? "",
      sip_school_id: profile.sip_school_id ?? "",
      primary_subject_id: profile.primary_subject_id ?? "",
      employment_type: profile.employment_type,
      qualification: profile.qualification,
      experience_years: profile.experience_years,
      designation: profile.designation,
      specialization: profile.specialization,
      max_periods_per_week: profile.max_periods_per_week,
      joining_date: profile.joining_date ? profile.joining_date.slice(0, 10) : "",
      exit_date: profile.exit_date ? profile.exit_date.slice(0, 10) : "",
      documents_verified: profile.documents_verified,
      is_active: profile.is_active,
      wings: profile.wings.join(", "),
      batch_ids: [...profile.batch_ids],
    });
    setFormTab("basic");
    setShowForm(true);
  }

  function toggleBatchId(id: number) {
    setForm((s) => ({
      ...s,
      batch_ids: s.batch_ids.includes(id) ? s.batch_ids.filter((b) => b !== id) : [...s.batch_ids, id],
    }));
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setIsSaving(true);
    try {
      const input: UpsertFacultyProfileInput = {
        faculty_user_id: Number(form.faculty_user_id),
        pwid: form.pwid,
        email: form.email,
        mobile: form.mobile,
        emergency_contact_name: form.emergency_contact_name,
        emergency_contact_mobile: form.emergency_contact_mobile,
        vp_center_id: form.vp_center_id === "" ? null : Number(form.vp_center_id),
        sip_school_id: form.sip_school_id === "" ? null : Number(form.sip_school_id),
        primary_subject_id: form.primary_subject_id === "" ? null : Number(form.primary_subject_id),
        employment_type: form.employment_type,
        qualification: form.qualification,
        experience_years: Number(form.experience_years) || 0,
        designation: form.designation,
        specialization: form.specialization,
        max_periods_per_week: Number(form.max_periods_per_week) || 0,
        joining_date: form.joining_date,
        exit_date: form.exit_date,
        documents_verified: form.documents_verified,
        is_active: form.is_active,
        wings: form.wings
          .split(",")
          .map((w) => w.trim())
          .filter((w) => w.length > 0),
        batch_ids: form.batch_ids,
      };
      onUpsertProfile(input);
    } finally {
      setIsSaving(false);
    }
    setShowForm(false);
    setEditingProfile(null);
    setForm({ ...defaultForm });
  }

  const filtered = React.useMemo(() => {
    return facultyProfiles.filter((p) => {
      const matchesSearch =
        search.trim() === "" ||
        p.faculty_display_name.toLowerCase().includes(search.toLowerCase()) ||
        p.pwid.toLowerCase().includes(search.toLowerCase());
      const matchesVpCenter = filterVpCenterId === "" || p.vp_center_id === filterVpCenterId;
      const matchesSchool = filterSchoolId === "" || p.sip_school_id === filterSchoolId;
      const matchesEmployment = filterEmploymentType === "All" || p.employment_type === filterEmploymentType;
      const matchesStatus =
        filterStatus === "All" || (filterStatus === "Active" && p.is_active) || (filterStatus === "Exited" && !p.is_active);
      return matchesSearch && matchesVpCenter && matchesSchool && matchesEmployment && matchesStatus;
    });
  }, [facultyProfiles, search, filterVpCenterId, filterSchoolId, filterEmploymentType, filterStatus]);

  return (
    <section className="ticket-modal" aria-label="Faculty directory">
      <header>
        <h2>Faculty Directory</h2>
        <div className="actions">
          <button className="primary-action" onClick={openAdd}>
            Add Faculty
          </button>
          <button type="button" className="ghost-button" onClick={onClose}>
            Close
          </button>
        </div>
      </header>

      <div className="directory-filters">
        <input
          className="directory-search"
          type="search"
          placeholder="Search by name or PWID…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <select
          value={filterVpCenterId}
          onChange={(e) => setFilterVpCenterId(e.target.value === "" ? "" : Number(e.target.value))}
        >
          <option value="">All VP Centers</option>
          {vpCenters.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
        <select
          value={filterSchoolId}
          onChange={(e) => setFilterSchoolId(e.target.value === "" ? "" : Number(e.target.value))}
        >
          <option value="">All Schools</option>
          {schools.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
        <select value={filterEmploymentType} onChange={(e) => setFilterEmploymentType(e.target.value)}>
          <option value="All">All Employment Types</option>
          <option value="VP Payroll">VP Payroll</option>
          <option value="School Payroll">School Payroll</option>
        </select>
        <select value={filterStatus} onChange={(e) => setFilterStatus(e.target.value)}>
          <option value="All">All Statuses</option>
          <option value="Active">Active</option>
          <option value="Exited">Exited</option>
        </select>
        <button
          className="directory-filter-reset"
          onClick={() => {
            setSearch("");
            setFilterVpCenterId("");
            setFilterSchoolId("");
            setFilterEmploymentType("All");
            setFilterStatus("All");
          }}
        >
          Reset
        </button>
      </div>

      {filtered.length === 0 ? (
        <p className="empty-state">No faculty found.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>PWID</th>
              <th>Designation</th>
              <th>VP Center</th>
              <th>SIP School</th>
              <th>Primary Subject</th>
              <th>Employment Type</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((p) => (
              <tr key={p.faculty_user_id}>
                <td>{p.faculty_display_name}</td>
                <td>{p.pwid}</td>
                <td>{p.designation || "—"}</td>
                <td>{p.vp_center_name || "—"}</td>
                <td>{p.sip_school_name || "—"}</td>
                <td>{p.primary_subject_name || "—"}</td>
                <td>{p.employment_type}</td>
                <td>
                  <span className={`status-chip status-${p.is_active ? "active" : "exited"}`}>
                    {p.is_active ? "Active" : "Exited"}
                  </span>
                </td>
                <td>
                  <button className="secondary-button" onClick={() => openEdit(p)}>
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showForm &&
        createPortal(
          <div role="button" tabIndex={0} aria-label="Close dialog" className="master-form-overlay" onKeyDown={(e) => { if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) { e.preventDefault(); } }} onClick={() => setShowForm(false)}>
            <div className="master-form" onClick={(e) => e.stopPropagation()}>
              <h3>{editingProfile ? "Edit Faculty" : "Add Faculty"}</h3>
              <form onSubmit={handleSubmit} className="school-profile-form">
                <div className="form-tabs">
                  {([
                    ["basic", "Basic Info"],
                    ["contact", "Contact"],
                    ["professional", "Professional"],
                    ["wings", "Wings & Batches"],
                  ] as const).map(([key, label]) => (
                    <button
                      key={key}
                      type="button"
                      className={formTab === key ? "active" : ""}
                      onClick={() => setFormTab(key)}
                    >
                      {label}
                    </button>
                  ))}
                </div>

                {formTab === "basic" && (
                  <>
                    <label>
                      Faculty User
                      <select
                        required
                        value={form.faculty_user_id}
                        onChange={(e) => setForm((s) => ({ ...s, faculty_user_id: e.target.value ? Number(e.target.value) : "" }))}
                      >
                        <option value="">—</option>
                        {facultyUsers.map((u) => (
                          <option key={u.id} value={u.id}>
                            {u.display_name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      PWID
                      <input
                        required
                        value={form.pwid}
                        onChange={(e) => setForm((s) => ({ ...s, pwid: e.target.value }))}
                      />
                    </label>
                    <label>
                      Employment Type
                      <select
                        required
                        value={form.employment_type}
                        onChange={(e) => setForm((s) => ({ ...s, employment_type: e.target.value }))}
                      >
                        <option value="VP Payroll">VP Payroll</option>
                        <option value="School Payroll">School Payroll</option>
                      </select>
                    </label>
                    <label>
                      VP Center
                      <select
                        value={form.vp_center_id}
                        onChange={(e) => setForm((s) => ({ ...s, vp_center_id: e.target.value ? Number(e.target.value) : "" }))}
                      >
                        <option value="">—</option>
                        {vpCenters.map((c) => (
                          <option key={c.id} value={c.id}>
                            {c.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      SIP School
                      <select
                        value={form.sip_school_id}
                        onChange={(e) => setForm((s) => ({ ...s, sip_school_id: e.target.value ? Number(e.target.value) : "" }))}
                      >
                        <option value="">—</option>
                        {schools.map((s) => (
                          <option key={s.id} value={s.id}>
                            {s.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Primary Subject
                      <select
                        value={form.primary_subject_id}
                        onChange={(e) => setForm((s) => ({ ...s, primary_subject_id: e.target.value ? Number(e.target.value) : "" }))}
                      >
                        <option value="">—</option>
                        {subjects.map((sub) => (
                          <option key={sub.id} value={sub.id}>
                            {sub.name}
                          </option>
                        ))}
                      </select>
                    </label>
                  </>
                )}

                {formTab === "contact" && (
                  <>
                    <label>
                      Email
                      <input
                        type="email"
                        value={form.email}
                        onChange={(e) => setForm((s) => ({ ...s, email: e.target.value }))}
                      />
                    </label>
                    <label>
                      Mobile
                      <input
                        value={form.mobile}
                        onChange={(e) => setForm((s) => ({ ...s, mobile: e.target.value }))}
                      />
                    </label>
                    <label>
                      Emergency Contact Name
                      <input
                        value={form.emergency_contact_name}
                        onChange={(e) => setForm((s) => ({ ...s, emergency_contact_name: e.target.value }))}
                      />
                    </label>
                    <label>
                      Emergency Contact Mobile
                      <input
                        value={form.emergency_contact_mobile}
                        onChange={(e) => setForm((s) => ({ ...s, emergency_contact_mobile: e.target.value }))}
                      />
                    </label>
                  </>
                )}

                {formTab === "professional" && (
                  <>
                    <label>
                      Designation
                      <input
                        value={form.designation}
                        onChange={(e) => setForm((s) => ({ ...s, designation: e.target.value }))}
                      />
                    </label>
                    <label>
                      Qualification
                      <input
                        value={form.qualification}
                        onChange={(e) => setForm((s) => ({ ...s, qualification: e.target.value }))}
                      />
                    </label>
                    <label>
                      Experience Years
                      <input
                        type="number"
                        min={0}
                        value={form.experience_years}
                        onChange={(e) => setForm((s) => ({ ...s, experience_years: e.target.value === "" ? "" : Number(e.target.value) }))}
                      />
                    </label>
                    <label>
                      Specialization
                      <input
                        value={form.specialization}
                        onChange={(e) => setForm((s) => ({ ...s, specialization: e.target.value }))}
                      />
                    </label>
                    <label>
                      Max Periods / Week
                      <input
                        type="number"
                        min={0}
                        value={form.max_periods_per_week}
                        onChange={(e) => setForm((s) => ({ ...s, max_periods_per_week: e.target.value === "" ? "" : Number(e.target.value) }))}
                      />
                    </label>
                    <label>
                      Joining Date
                      <input
                        type="date"
                        value={form.joining_date}
                        onChange={(e) => setForm((s) => ({ ...s, joining_date: e.target.value }))}
                      />
                    </label>
                    <label>
                      Exit Date
                      <input
                        type="date"
                        value={form.exit_date}
                        onChange={(e) => setForm((s) => ({ ...s, exit_date: e.target.value }))}
                      />
                    </label>
                    <label style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                      <input
                        type="checkbox"
                        checked={form.documents_verified}
                        onChange={(e) => setForm((s) => ({ ...s, documents_verified: e.target.checked }))}
                      />
                      Documents Verified
                    </label>
                    <label style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                      <input
                        type="checkbox"
                        checked={form.is_active}
                        onChange={(e) => setForm((s) => ({ ...s, is_active: e.target.checked }))}
                      />
                      Active
                    </label>
                  </>
                )}

                {formTab === "wings" && (
                  <>
                    <label style={{ gridColumn: "1 / -1" }}>
                      Wings (comma-separated)
                      <input
                        value={form.wings}
                        onChange={(e) => setForm((s) => ({ ...s, wings: e.target.value }))}
                        placeholder="e.g. JEE, NEET"
                      />
                    </label>
                    <fieldset style={{ gridColumn: "1 / -1" }}>
                      <legend>Batches</legend>
                      {batches.length === 0 ? (
                        <p className="empty-state" style={{ margin: 0 }}>No batches available.</p>
                      ) : (
                        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 8, marginTop: 8 }}>
                          {batches.map((b) => (
                            <label key={b.id}>
                              <input
                                type="checkbox"
                                checked={form.batch_ids.includes(b.id)}
                                onChange={() => toggleBatchId(b.id)}
                              />
                              {b.batch_id} — {b.school_name}
                            </label>
                          ))}
                        </div>
                      )}
                    </fieldset>
                  </>
                )}

                <div className="form-actions">
                  <button type="submit" className="primary-action" disabled={isSaving}>
                    {isSaving ? "Saving…" : "Save"}
                  </button>
                  <button type="button" className="secondary-button" onClick={() => setShowForm(false)}>
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>,
          document.body,
        )}
    </section>
  );
}

export function SessionManagerPanel({
  sessions,
  schools,
  onAddSession,
}: {
  sessions: LectureSession[];
  schools: School[];
  onAddSession: () => void;
}) {
  return (
    <section className="ticket-modal" aria-label="Session manager">
      <header>
        <h2>Session Manager</h2>
        <button className="primary-action" onClick={onAddSession}>
          Add Session
        </button>
      </header>
      {sessions.length === 0 ? (
        <p className="empty-state">No sessions.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>School</th>
              <th>Grade</th>
              <th>Subject</th>
              <th>Faculty</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((s) => (
              <tr key={s.id}>
                <td>{s.session_date}</td>
                <td>{schools.find((school) => school.id === s.school_id)?.name ?? s.school_id}</td>
                <td>{s.grade_level}</td>
                <td>{s.subject_id ? `Subject #${s.subject_id}` : "—"}</td>
                <td>{s.actual_faculty_user_id ? `Faculty #${s.actual_faculty_user_id}` : "Unassigned"}</td>
                <td>{s.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function SubjectsPanel({
  subjects,
  onCreateSubject,
  onUpdateSubject,
  onDeleteSubject,
}: {
  subjects: Subject[];
  onCreateSubject?: (input: Omit<Subject, "id">) => void;
  onUpdateSubject?: (input: Subject) => void;
  onDeleteSubject?: (id: number) => void;
}) {
  const [editingId, setEditingId] = React.useState<number | null>(null);
  const [draft, setDraft] = React.useState<Partial<Subject>>({});
  const [isAdding, setIsAdding] = React.useState(false);
  const [confirmDeleteId, setConfirmDeleteId] = React.useState<number | null>(null);
  const [newSubject, setNewSubject] = React.useState<Omit<Subject, "id">>({
    name: "",
    track: "JEE",
    is_default: true,
    sort_order: 0,
  });

  const startEdit = (subject: Subject) => {
    setEditingId(subject.id);
    setConfirmDeleteId(null);
    setDraft({ ...subject });
  };

  const cancelEdit = () => {
    setEditingId(null);
    setDraft({});
  };

  const saveEdit = () => {
    if (onUpdateSubject && draft.id && draft.name && draft.track !== undefined) {
      onUpdateSubject({
        id: draft.id,
        name: draft.name,
        track: draft.track,
        is_default: draft.is_default ?? false,
        sort_order: draft.sort_order ?? 0,
      });
      setEditingId(null);
      setDraft({});
    }
  };

  const saveNew = () => {
    if (onCreateSubject && newSubject.name.trim()) {
      onCreateSubject(newSubject);
      setIsAdding(false);
      setConfirmDeleteId(null);
      setNewSubject({ name: "", track: "JEE", is_default: true, sort_order: 0 });
    }
  };

  const hasHandlers = onCreateSubject || onUpdateSubject || onDeleteSubject;

  return (
    <section className="ticket-modal" aria-label="Subjects">
      <header>
        <h2>Subjects</h2>
        {onCreateSubject && (
          <button className="primary-action" onClick={() => setIsAdding(true)}>
            Add Subject
          </button>
        )}
      </header>
      <table className="data-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Track</th>
            <th>Default</th>
            <th>Sort</th>
            {hasHandlers && <th>Actions</th>}
          </tr>
        </thead>
        <tbody>
          {isAdding && onCreateSubject && (
            <tr>
              <td>
                <input
                  type="text"
                  value={newSubject.name}
                  onChange={(e) => setNewSubject((prev) => ({ ...prev, name: e.target.value }))}
                  placeholder="Subject name"
                  aria-label="New subject name"
                />
              </td>
              <td>
                <select
                  value={newSubject.track}
                  onChange={(e) => setNewSubject((prev) => ({ ...prev, track: e.target.value }))}
                  aria-label="New subject track"
                >
                  <option value="JEE">JEE</option>
                  <option value="NEET">NEET</option>
                  <option value="Foundation">Foundation</option>
                </select>
              </td>
              <td>
                <input
                  type="checkbox"
                  checked={newSubject.is_default}
                  onChange={(e) => setNewSubject((prev) => ({ ...prev, is_default: e.target.checked }))}
                  aria-label="New subject default"
                />
              </td>
              <td>
                <input
                  type="number"
                  value={newSubject.sort_order}
                  onChange={(e) => setNewSubject((prev) => ({ ...prev, sort_order: Number(e.target.value) }))}
                  aria-label="New subject sort order"
                  style={{ width: "60px" }}
                />
              </td>
              <td>
                <button className="primary-action" onClick={saveNew}>Save</button>
                <button className="secondary-button" onClick={() => setIsAdding(false)}>Cancel</button>
              </td>
            </tr>
          )}
          {subjects.length === 0 && !isAdding ? (
            <tr>
              <td colSpan={hasHandlers ? 5 : 4} className="empty-state">No subjects.</td>
            </tr>
          ) : (
            subjects.map((s) => (
              <tr key={s.id}>
                {editingId === s.id ? (
                  <>
                    <td>
                      <input
                        type="text"
                        value={draft.name ?? ""}
                        onChange={(e) => setDraft((prev) => ({ ...prev, name: e.target.value }))}
                        aria-label={`Edit name for subject ${s.id}`}
                      />
                    </td>
                    <td>
                      <select
                        value={draft.track ?? ""}
                        onChange={(e) => setDraft((prev) => ({ ...prev, track: e.target.value }))}
                        aria-label={`Edit track for subject ${s.id}`}
                      >
                        <option value="JEE">JEE</option>
                        <option value="NEET">NEET</option>
                        <option value="Foundation">Foundation</option>
                      </select>
                    </td>
                    <td>
                      <input
                        type="checkbox"
                        checked={draft.is_default ?? false}
                        onChange={(e) => setDraft((prev) => ({ ...prev, is_default: e.target.checked }))}
                        aria-label={`Edit default for subject ${s.id}`}
                      />
                    </td>
                    <td>
                      <input
                        type="number"
                        value={draft.sort_order ?? 0}
                        onChange={(e) => setDraft((prev) => ({ ...prev, sort_order: Number(e.target.value) }))}
                        aria-label={`Edit sort order for subject ${s.id}`}
                        style={{ width: "60px" }}
                      />
                    </td>
                    <td>
                      <button className="primary-action" onClick={saveEdit}>Save</button>
                      <button className="secondary-button" onClick={cancelEdit}>Cancel</button>
                    </td>
                  </>
                ) : (
                  <>
                    <td>{s.name}</td>
                    <td>{s.track}</td>
                    <td>{s.is_default ? "Yes" : "No"}</td>
                    <td>{s.sort_order}</td>
                    {hasHandlers && (
                      <td>
                        {onUpdateSubject && (
                          <button className="secondary-button" onClick={() => startEdit(s)}>Edit</button>
                        )}
                        {onDeleteSubject && (
                          <button
                            className={confirmDeleteId === s.id ? "danger-button" : "secondary-button"}
                            onClick={() => {
                              if (confirmDeleteId === s.id) {
                                onDeleteSubject(s.id);
                                setConfirmDeleteId(null);
                              } else {
                                setConfirmDeleteId(s.id);
                              }
                            }}
                            style={{ marginLeft: "6px" }}
                          >
                            {confirmDeleteId === s.id ? "Confirm Delete" : "Delete"}
                          </button>
                        )}
                      </td>
                    )}
                  </>
                )}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </section>
  );
}

export function FacultyMembersPanel({
  members,
  schools,
  users = [],
  memberships,
  onCreateMember,
  onUpdateMember,
  onDeleteMember,
  onCreateMembership,
  onDeleteMembership,
  onLoadMemberships,
  onImportCsv,
  onCreateLogin,
  onLinkUser,
}: {
  members: FacultyMember[];
  schools: School[];
  users?: AppUser[];
  memberships: Record<number, FacultySchoolMembership[]>;
  onCreateMember?: (input: CreateFacultyMemberInput) => void;
  onUpdateMember?: (input: UpdateFacultyMemberInput) => void;
  onDeleteMember?: (id: number) => void;
  onCreateMembership?: (input: CreateFacultySchoolMembershipInput) => void;
  onDeleteMembership?: (id: number, facultyId: number) => void;
  onLoadMemberships?: (facultyId: number) => void;
  onImportCsv?: (file: File) => void;
  onCreateLogin?: (facultyId: number, input: CreateFacultyLoginInput) => void;
  onLinkUser?: (facultyId: number, userId: number) => void;
}) {
  const fileInputRef = React.useRef<HTMLInputElement>(null);
  const [isAdding, setIsAdding] = React.useState(false);
  const [editingId, setEditingId] = React.useState<number | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = React.useState<number | null>(null);
  const [expandedId, setExpandedId] = React.useState<number | null>(null);
  const [draft, setDraft] = React.useState<Partial<FacultyMember>>({});
  const [newMember, setNewMember] = React.useState<CreateFacultyMemberInput>({
    name: "",
    email: "",
    mobile: "",
    pwid: "",
    qualification: "",
    experience_years: 0,
    designation: "",
    specialization: "",
    employment_type: "VP Payroll",
    is_active: true,
    user_id: null,
    initial_school_id: null,
  });
  const [newMembershipFacultyId, setNewMembershipFacultyId] = React.useState<number | null>(null);
  const [newMembershipSchoolId, setNewMembershipSchoolId] = React.useState<string>("");
  const [newMembershipRole, setNewMembershipRole] = React.useState("Faculty");
  const [loginFacultyId, setLoginFacultyId] = React.useState<number | null>(null);
  const [linkFacultyId, setLinkFacultyId] = React.useState<number | null>(null);
  const [loginDraft, setLoginDraft] = React.useState<CreateFacultyLoginInput>({
    username: "",
    display_name: "",
    password: "",
  });
  const [linkUserId, setLinkUserId] = React.useState<string>("");

  const linkedUserIds = new Set(members.map((m) => m.user_id).filter((id): id is number => id !== null));
  const linkableFacultyUsers = users.filter((u) => u.role === "faculty" && !linkedUserIds.has(u.id));

  const startEdit = (m: FacultyMember) => {
    setEditingId(m.id);
    setConfirmDeleteId(null);
    setDraft({ ...m });
  };

  const cancelEdit = () => {
    setEditingId(null);
    setDraft({});
  };

  const saveEdit = () => {
    if (onUpdateMember && draft.id && draft.name) {
      onUpdateMember({
        id: draft.id,
        name: draft.name,
        email: draft.email ?? "",
        mobile: draft.mobile ?? "",
        pwid: draft.pwid ?? "",
        qualification: draft.qualification ?? "",
        experience_years: draft.experience_years ?? 0,
        designation: draft.designation ?? "",
        specialization: draft.specialization ?? "",
        employment_type: draft.employment_type ?? "VP Payroll",
        is_active: draft.is_active ?? true,
        user_id: draft.user_id ?? null,
      });
      setEditingId(null);
      setDraft({});
    }
  };

  const saveNew = () => {
    if (onCreateMember && newMember.name.trim()) {
      onCreateMember(newMember);
      setIsAdding(false);
      setConfirmDeleteId(null);
      setNewMember({
        name: "",
        email: "",
        mobile: "",
        pwid: "",
        qualification: "",
        experience_years: 0,
        designation: "",
        specialization: "",
        employment_type: "VP Payroll",
        is_active: true,
        user_id: null,
        initial_school_id: null,
      });
    }
  };

  const startCreateLogin = (m: FacultyMember) => {
    setLoginFacultyId(m.id);
    setLinkFacultyId(null);
    setLinkUserId("");
    setLoginDraft({
      username: m.pwid ? m.pwid.toLowerCase() : "",
      display_name: m.name,
      password: "",
    });
  };

  const saveCreateLogin = (facultyId: number) => {
    if (onCreateLogin && loginDraft.username.trim() && loginDraft.display_name.trim() && loginDraft.password.trim()) {
      onCreateLogin(facultyId, loginDraft);
      setLoginFacultyId(null);
      setLoginDraft({ username: "", display_name: "", password: "" });
    }
  };

  const startLinkUser = (facultyId: number) => {
    setLinkFacultyId(facultyId);
    setLoginFacultyId(null);
    setLoginDraft({ username: "", display_name: "", password: "" });
    setLinkUserId("");
  };

  const saveLinkUser = (facultyId: number) => {
    if (onLinkUser && linkUserId) {
      onLinkUser(facultyId, Number(linkUserId));
      setLinkFacultyId(null);
      setLinkUserId("");
    }
  };

  const toggleExpand = (id: number) => {
    if (expandedId === id) {
      setExpandedId(null);
    } else {
      setExpandedId(id);
      onLoadMemberships?.(id);
    }
  };

  const hasHandlers = onCreateMember || onUpdateMember || onDeleteMember || onCreateLogin || onLinkUser;

  const accountBadge = (m: FacultyMember) => {
    if (!m.user_id) return <span style={{ color: "#64748b", fontSize: "12px" }}>No login</span>;
    if (!m.is_active) return <span style={{ color: "#dc2626", fontSize: "12px" }}>Inactive linked user</span>;
    return <span style={{ color: "#16a34a", fontSize: "12px" }}>Linked</span>;
  };

  return (
    <section className="ticket-modal" aria-label="Faculty Members">
      <header>
        <h2>Faculty Master</h2>
        <div style={{ display: "flex", gap: "8px" }}>
          {onImportCsv && (
            <>
              <input
                type="file"
                accept=".csv"
                ref={fileInputRef}
                style={{ display: "none" }}
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) onImportCsv(file);
                  if (fileInputRef.current) fileInputRef.current.value = "";
                }}
              />
              <button className="secondary-button" onClick={() => fileInputRef.current?.click()}>
                Import CSV
              </button>
            </>
          )}
          {onCreateMember && (
            <button className="primary-action" onClick={() => setIsAdding(true)}>
              Add Faculty
            </button>
          )}
        </div>
      </header>
      <p className="read-only-notice">Login is not required for timetable planning. Login is required for faculty self-service, leave, substitutions, and attendance marking.</p>
      <table className="data-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Contact</th>
            <th>PWID</th>
            <th>Designation</th>
            <th>Specialization</th>
            <th>Active</th>
            <th>Account</th>
            {hasHandlers && <th>Actions</th>}
          </tr>
        </thead>
        <tbody>
          {isAdding && onCreateMember && (
            <tr>
              <td><input type="text" value={newMember.name} onChange={(e) => setNewMember((p) => ({ ...p, name: e.target.value }))} placeholder="Name" aria-label="New faculty name" /></td>
              <td>
                <label className="compact-field">
                  <span>Email</span>
                  <input type="email" value={newMember.email} onChange={(e) => setNewMember((p) => ({ ...p, email: e.target.value }))} placeholder="name@example.com" aria-label="New faculty email" />
                </label>
                <label className="compact-field">
                  <span>Mobile</span>
                  <input type="tel" value={newMember.mobile} onChange={(e) => setNewMember((p) => ({ ...p, mobile: e.target.value }))} placeholder="Phone number" aria-label="New faculty mobile" />
                </label>
              </td>
              <td><input type="text" value={newMember.pwid} onChange={(e) => setNewMember((p) => ({ ...p, pwid: e.target.value }))} placeholder="PWID" aria-label="New faculty PWID" style={{ width: "80px" }} /></td>
              <td><input type="text" value={newMember.designation} onChange={(e) => setNewMember((p) => ({ ...p, designation: e.target.value }))} placeholder="Designation" aria-label="New faculty designation" style={{ width: "100px" }} /></td>
              <td><input type="text" value={newMember.specialization} onChange={(e) => setNewMember((p) => ({ ...p, specialization: e.target.value }))} placeholder="Specialization" aria-label="New faculty specialization" style={{ width: "100px" }} /></td>
              <td>
                <input type="checkbox" checked={newMember.is_active} onChange={(e) => setNewMember((p) => ({ ...p, is_active: e.target.checked }))} aria-label="New faculty active" />
              </td>
              <td>
                <select
                  value={newMember.initial_school_id ?? ""}
                  onChange={(e) => setNewMember((p) => ({ ...p, initial_school_id: e.target.value ? Number(e.target.value) : null }))}
                  aria-label="New faculty school"
                >
                  <option value="">Select school</option>
                  {schools.map((s) => (
                    <option key={s.id} value={s.id}>{s.name}</option>
                  ))}
                </select>
              </td>
              <td>
                <button className="primary-action" onClick={saveNew} disabled={!newMember.name.trim() || !newMember.initial_school_id}>Save</button>
                <button className="secondary-button" onClick={() => setIsAdding(false)}>Cancel</button>
              </td>
            </tr>
          )}
          {members.length === 0 && !isAdding ? (
            <tr>
              <td colSpan={hasHandlers ? 8 : 7} className="empty-state">No faculty members.</td>
            </tr>
          ) : (
            members.map((m) => (
              <React.Fragment key={m.id}>
                <tr>
                  {editingId === m.id ? (
                    <>
                      <td><input type="text" value={draft.name ?? ""} onChange={(e) => setDraft((p) => ({ ...p, name: e.target.value }))} aria-label={`Edit name for faculty ${m.id}`} /></td>
                      <td>
                        <label className="compact-field">
                          <span>Email</span>
                          <input type="email" value={draft.email ?? ""} onChange={(e) => setDraft((p) => ({ ...p, email: e.target.value }))} aria-label={`Edit email for faculty ${m.id}`} />
                        </label>
                        <label className="compact-field">
                          <span>Mobile</span>
                          <input type="tel" value={draft.mobile ?? ""} onChange={(e) => setDraft((p) => ({ ...p, mobile: e.target.value }))} aria-label={`Edit mobile for faculty ${m.id}`} />
                        </label>
                      </td>
                      <td><input type="text" value={draft.pwid ?? ""} onChange={(e) => setDraft((p) => ({ ...p, pwid: e.target.value }))} aria-label={`Edit PWID for faculty ${m.id}`} style={{ width: "80px" }} /></td>
                      <td><input type="text" value={draft.designation ?? ""} onChange={(e) => setDraft((p) => ({ ...p, designation: e.target.value }))} aria-label={`Edit designation for faculty ${m.id}`} style={{ width: "100px" }} /></td>
                      <td><input type="text" value={draft.specialization ?? ""} onChange={(e) => setDraft((p) => ({ ...p, specialization: e.target.value }))} aria-label={`Edit specialization for faculty ${m.id}`} style={{ width: "100px" }} /></td>
                      <td><input type="checkbox" checked={draft.is_active ?? false} onChange={(e) => setDraft((p) => ({ ...p, is_active: e.target.checked }))} aria-label={`Edit active for faculty ${m.id}`} /></td>
                      <td>{accountBadge(m)}</td>
                      <td>
                        <button className="primary-action" onClick={saveEdit}>Save</button>
                        <button className="secondary-button" onClick={cancelEdit}>Cancel</button>
                      </td>
                    </>
                  ) : (
                    <>
                      <td>
                        <button className="link-button" onClick={() => toggleExpand(m.id)} style={{ background: "none", border: "none", color: "#2563eb", cursor: "pointer", textDecoration: "underline", padding: 0 }}>
                          {m.name}
                        </button>
                      </td>
                      <td>
                        <div className="directory-contact">
                          {m.email ? <a href={`mailto:${m.email}`}>{m.email}</a> : <span>—</span>}
                          {m.mobile ? <a href={`tel:${m.mobile}`}>{m.mobile}</a> : null}
                        </div>
                      </td>
                      <td>{m.pwid || "—"}</td>
                      <td>{m.designation || "—"}</td>
                      <td>{m.specialization || "—"}</td>
                      <td>{m.is_active ? "Yes" : "No"}</td>
                      <td>{accountBadge(m)}</td>
                      {hasHandlers && (
                        <td>
                          {onUpdateMember && (
                            <button className="secondary-button" onClick={() => startEdit(m)}>Edit</button>
                          )}
                          {onDeleteMember && (
                            <button
                              className={confirmDeleteId === m.id ? "danger-button" : "secondary-button"}
                              onClick={() => {
                                if (confirmDeleteId === m.id) {
                                  onDeleteMember(m.id);
                                  setConfirmDeleteId(null);
                                } else {
                                  setConfirmDeleteId(m.id);
                                }
                              }}
                              style={{ marginLeft: "6px" }}
                            >
                              {confirmDeleteId === m.id ? "Confirm Delete" : "Delete"}
                            </button>
                          )}
                          {!m.user_id && onCreateLogin && (
                            <button className="secondary-button" onClick={() => startCreateLogin(m)} style={{ marginLeft: "6px" }}>Create Login</button>
                          )}
                          {!m.user_id && onLinkUser && (
                            <button className="secondary-button" onClick={() => startLinkUser(m.id)} style={{ marginLeft: "6px" }}>Link User</button>
                          )}
                        </td>
                      )}
                    </>
                  )}
                </tr>
                {loginFacultyId === m.id && (
                  <tr>
                    <td colSpan={hasHandlers ? 8 : 7} style={{ background: "#f8fafc" }}>
                      <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
                        <strong>Create login</strong>
                        <input aria-label={`Login username for faculty ${m.id}`} placeholder="Username" value={loginDraft.username} onChange={(e) => setLoginDraft((p) => ({ ...p, username: e.target.value }))} />
                        <input aria-label={`Login display name for faculty ${m.id}`} placeholder="Display name" value={loginDraft.display_name} onChange={(e) => setLoginDraft((p) => ({ ...p, display_name: e.target.value }))} />
                        <input aria-label={`Login password for faculty ${m.id}`} placeholder="Password" type="password" value={loginDraft.password} onChange={(e) => setLoginDraft((p) => ({ ...p, password: e.target.value }))} />
                        <button className="primary-action" onClick={() => saveCreateLogin(m.id)} disabled={!loginDraft.username.trim() || !loginDraft.display_name.trim() || !loginDraft.password.trim()}>Save Login</button>
                        <button className="secondary-button" onClick={() => setLoginFacultyId(null)}>Cancel</button>
                      </div>
                    </td>
                  </tr>
                )}
                {linkFacultyId === m.id && (
                  <tr>
                    <td colSpan={hasHandlers ? 8 : 7} style={{ background: "#f8fafc" }}>
                      <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
                        <strong>Link existing user</strong>
                        <select value={linkUserId} onChange={(e) => setLinkUserId(e.target.value)} aria-label={`Existing user for faculty ${m.id}`}>
                          <option value="">Select faculty user</option>
                          {linkableFacultyUsers.map((u) => (
                            <option key={u.id} value={u.id}>{u.display_name} ({u.username})</option>
                          ))}
                        </select>
                        <button className="primary-action" onClick={() => saveLinkUser(m.id)} disabled={!linkUserId}>Confirm Link</button>
                        <button className="secondary-button" onClick={() => setLinkFacultyId(null)}>Cancel</button>
                      </div>
                    </td>
                  </tr>
                )}
                {expandedId === m.id && (
                  <tr>
                    <td colSpan={hasHandlers ? 8 : 7} style={{ padding: 0, background: "#f8fafc" }}>
                      <div style={{ padding: "12px 16px" }}>
                        <strong>School Memberships</strong>
                        <table className="data-table" style={{ marginTop: "8px" }}>
                          <thead>
                            <tr>
                              <th>School</th>
                              <th>Role</th>
                              <th>Primary</th>
                              {onDeleteMembership && <th>Actions</th>}
                            </tr>
                          </thead>
                          <tbody>
                            {(memberships[m.id] || []).length === 0 ? (
                              <tr><td colSpan={onDeleteMembership ? 4 : 3} className="empty-state">No school memberships.</td></tr>
                            ) : (
                              (memberships[m.id] || []).map((ms) => (
                                <tr key={ms.id}>
                                  <td>{ms.school_name}</td>
                                  <td>{normalizeFacultyRole(ms.role_at_school)}</td>
                                  <td>{ms.is_primary ? "Yes" : "No"}</td>
                                  {onDeleteMembership && (
                                    <td>
                                      <button className="secondary-button" onClick={() => onDeleteMembership(ms.id, m.id)}>Remove</button>
                                    </td>
                                  )}
                                </tr>
                              ))
                            )}
                          </tbody>
                        </table>
                        {onCreateMembership && (
                          <div style={{ marginTop: "8px", display: "flex", gap: "8px", alignItems: "center" }}>
                            <select
                              value={newMembershipFacultyId === m.id ? newMembershipSchoolId : ""}
                              onChange={(e) => { setNewMembershipFacultyId(m.id); setNewMembershipSchoolId(e.target.value); }}
                              aria-label={`Add school for faculty ${m.id}`}
                            >
                              <option value="">Select school…</option>
                              {schools.map((s) => (
                                <option key={s.id} value={String(s.id)}>{s.name}</option>
                              ))}
                            </select>
                            <input
                              type="text"
                              value={newMembershipFacultyId === m.id ? newMembershipRole : "Faculty"}
                              onChange={(e) => { setNewMembershipFacultyId(m.id); setNewMembershipRole(e.target.value); }}
                              placeholder="Role"
                              aria-label={`Role at school for faculty ${m.id}`}
                              style={{ width: "120px" }}
                            />
                            <button
                              className="primary-action"
                              onClick={() => {
                                if (newMembershipFacultyId === m.id && newMembershipSchoolId) {
                                  onCreateMembership({
                                    faculty_id: m.id,
                                    school_id: Number(newMembershipSchoolId),
                                    role_at_school: newMembershipRole,
                                    is_primary: false,
                                  });
                                  setNewMembershipFacultyId(null);
                                  setNewMembershipSchoolId("");
                                  setNewMembershipRole("Faculty");
                                }
                              }}
                              disabled={newMembershipFacultyId !== m.id || !newMembershipSchoolId}
                            >
                              Add School
                            </button>
                          </div>
                        )}
                      </div>
                    </td>
                  </tr>
                )}
              </React.Fragment>
            ))
          )}
        </tbody>
      </table>
    </section>
  );
}

export function TimetablePanel({
  slots,
  schools,
  batches = [],
  subjects = [],
  facultyMembers = [],
  holidays = [],
  onSaveSlot,
}: {
  slots: TimetableSlot[];
  schools: School[];
  batches?: Batch[];
  subjects?: Subject[];
  facultyMembers?: FacultyMember[];
  holidays?: Holiday[];
  onSaveSlot?: (input: {
    batch_id: number;
    day_of_week: number;
    period: number;
    subject_id: number;
    faculty_user_id: number | null;
    start_time: string;
    end_time: string;
  }) => void;
}) {
  const [isAdding, setIsAdding] = React.useState(false);
  const [draft, setDraft] = React.useState({
    batch_id: 0,
    day_of_week: 1,
    period: 1,
    subject_id: 0,
    faculty_user_id: 0,
    start_time: "",
    end_time: "",
  });
  const selectedBatch = batches.find((b) => b.id === draft.batch_id);
  const selectedTrack = selectedBatch ? selectedBatch.track || "Foundation" : "";
  const subjectOptions = selectedTrack ? subjects.filter((s) => s.track === selectedTrack) : [];
  const canSave = Boolean(selectedBatch && draft.period > 0 && draft.subject_id > 0);

  const relevantHolidays = React.useMemo(() => {
    if (!selectedBatch) return [];
    return holidays.filter((h) => {
      if (h.scope === "global") return true;
      if (h.scope === "school" && h.school_id === selectedBatch.school_id) return true;
      if (h.scope === "region") {
        const school = schools.find((s) => s.id === selectedBatch.school_id);
        if (school && school.region_id === h.region_id) return true;
      }
      return false;
    }).filter((h) => !h.grade_level || h.grade_level === selectedBatch.grade_level);
  }, [holidays, selectedBatch, schools]);

  const conflictingHolidays = React.useMemo(() => {
    if (!selectedBatch || !isAdding) return [];
    return relevantHolidays.filter((h) => {
      const d = new Date(h.date + "T00:00:00");
      return d.getDay() === draft.day_of_week;
    });
  }, [relevantHolidays, draft.day_of_week, isAdding, selectedBatch]);

  function saveSlot() {
    if (!onSaveSlot || !canSave) return;
    if (conflictingHolidays.length > 0) {
      const dates = conflictingHolidays.map((h) => h.date).join(", ");
      const ok = window.confirm(
        `Warning: ${conflictingHolidays.length} holiday(s) fall on this day of the week for ${selectedBatch?.grade_level} at ${selectedBatch?.school_name} (${dates}). Save anyway?`
      );
      if (!ok) return;
    }
    onSaveSlot({
      batch_id: draft.batch_id,
      day_of_week: draft.day_of_week,
      period: draft.period,
      subject_id: draft.subject_id,
      faculty_user_id: draft.faculty_user_id || null,
      start_time: draft.start_time,
      end_time: draft.end_time,
    });
    setIsAdding(false);
    setDraft({ batch_id: 0, day_of_week: 1, period: 1, subject_id: 0, faculty_user_id: 0, start_time: "", end_time: "" });
  }

  return (
    <section className="ticket-modal" aria-label="Timetable">
      <header>
        <h2>Timetable</h2>
        {onSaveSlot && (
          <button className="primary-action" onClick={() => setIsAdding(true)}>
            Add Slot
          </button>
        )}
      </header>
      <p className="read-only-notice">Timetable slots now target concrete batches. Faculty selection is limited to login-linked faculty until attendance supports no-login faculty sessions.</p>
      {isAdding && onSaveSlot && (
        <div className="inline-edit-form">
          <label>
            Batch
            <select
              aria-label="Batch"
              value={draft.batch_id || ""}
              onChange={(e) => setDraft((s) => ({ ...s, batch_id: Number(e.target.value), subject_id: 0 }))}
            >
              <option value="">Select batch</option>
              {batches.map((b) => (
                <option key={b.id} value={b.id}>
                  {b.school_name} — {b.batch_id} — {b.grade_level}{b.track ? ` ${b.track}` : ""} — {b.batch_pattern}
                </option>
              ))}
            </select>
          </label>
          <label>
            Day
            <select aria-label="Day" value={draft.day_of_week} onChange={(e) => setDraft((s) => ({ ...s, day_of_week: Number(e.target.value) }))}>
              {["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"].map((d, idx) => (
                <option key={d} value={idx}>{d}</option>
              ))}
            </select>
          </label>
          <label>
            Period
            <input aria-label="Period" type="number" min="1" value={draft.period} onChange={(e) => setDraft((s) => ({ ...s, period: Number(e.target.value) }))} />
          </label>
          <label>
            Subject
            <select
              aria-label="Subject"
              value={draft.subject_id || ""}
              disabled={!selectedBatch}
              onChange={(e) => setDraft((s) => ({ ...s, subject_id: Number(e.target.value) }))}
            >
              <option value="">{selectedBatch ? "Select subject" : "Select batch first"}</option>
              {subjectOptions.map((subject) => (
                <option key={subject.id} value={subject.id}>{subject.name}</option>
              ))}
            </select>
          </label>
          <label>
            Faculty
            <select aria-label="Faculty" value={draft.faculty_user_id || ""} onChange={(e) => setDraft((s) => ({ ...s, faculty_user_id: Number(e.target.value) }))}>
              <option value="">Unassigned</option>
              {facultyMembers.filter((m) => m.is_active && m.user_id).map((m) => (
                <option key={m.id} value={m.user_id ?? ""}>{m.name}</option>
              ))}
            </select>
          </label>
          <label>
            Start
            <input aria-label="Start" value={draft.start_time} onChange={(e) => setDraft((s) => ({ ...s, start_time: e.target.value }))} />
          </label>
          <label>
            End
            <input aria-label="End" value={draft.end_time} onChange={(e) => setDraft((s) => ({ ...s, end_time: e.target.value }))} />
          </label>
          {selectedBatch && relevantHolidays.length > 0 && (
            <div className="holiday-notice">
              <strong>🗓 Holidays for {selectedBatch.grade_level} at {selectedBatch.school_name}</strong>
              <ul>
                {relevantHolidays.slice(0, 8).map((h) => (
                  <li key={h.id} className={conflictingHolidays.some((c) => c.id === h.id) ? "holiday-conflict" : ""}>
                    {h.date} — {h.name} {conflictingHolidays.some((c) => c.id === h.id) ? "⚠️ conflicts with this day" : ""}
                  </li>
                ))}
                {relevantHolidays.length > 8 && <li>…and {relevantHolidays.length - 8} more</li>}
              </ul>
            </div>
          )}
          <div className="actions">
            <button className="primary-action" disabled={!canSave} onClick={saveSlot}>Save Slot</button>
            <button className="secondary-button" onClick={() => setIsAdding(false)}>Cancel</button>
          </div>
        </div>
      )}
      {slots.length === 0 ? (
        <p className="empty-state">No timetable slots.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Day</th>
              <th>Period</th>
              <th>Batch</th>
              <th>Grade</th>
              <th>Track</th>
              <th>Delivery</th>
              <th>Subject</th>
              <th>Faculty</th>
            </tr>
          </thead>
          <tbody>
            {slots.map((slot) => (
              <tr key={slot.id}>
                <td>{schools.find((s) => s.id === slot.school_id)?.name ?? slot.school_id}</td>
                <td>{slot.day_of_week}</td>
                <td>{slot.period}</td>
                <td>{slot.batch_name}</td>
                <td>{slot.grade_level}</td>
                <td>{slot.track || "—"}</td>
                <td>{slot.batch_pattern}</td>
                <td>{slot.subject_name}</td>
                <td>{slot.faculty_display_name || "Unassigned"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function WeeklyTimetablePanel({
  slots,
  schools,
  onAddSlot,
}: {
  slots: WeeklyTimetableSlot[];
  schools: School[];
  onAddSlot: () => void;
}) {
  return (
    <section className="ticket-modal" aria-label="Weekly timetable">
      <header>
        <h2>Weekly Timetable</h2>
        <button className="primary-action" onClick={onAddSlot}>
          Add Weekly Slot
        </button>
      </header>
      <p className="read-only-notice">Weekly timetable operational views use linked faculty login accounts. No-login faculty remain available in Faculty Assignments for planning.</p>
      {slots.length === 0 ? (
        <p className="empty-state">No weekly slots.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Week</th>
              <th>Day</th>
              <th>Period</th>
              <th>Grade</th>
              <th>Subject</th>
              <th>Faculty</th>
            </tr>
          </thead>
          <tbody>
            {slots.map((slot) => (
              <tr key={slot.id}>
                <td>{schools.find((s) => s.id === slot.school_id)?.name ?? slot.school_id}</td>
                <td>{slot.week_start_date ?? "—"}</td>
                <td>{slot.day_of_week}</td>
                <td>{slot.period}</td>
                <td>{slot.grade_level}</td>
                <td>{slot.subject_name}</td>
                <td>{slot.faculty_display_name || "Unassigned"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function SchoolMasterTimetablePanel({
  slots,
  schools,
  onAddSlot,
}: {
  slots: WeeklyTimetableSlot[];
  schools: School[];
  onAddSlot: () => void;
}) {
  return (
    <section className="ticket-modal" aria-label="School master timetable">
      <header>
        <h2>School Master Timetable</h2>
        <button className="primary-action" onClick={onAddSlot}>
          Add Slot
        </button>
      </header>
      {slots.length === 0 ? (
        <p className="empty-state">No slots.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Day</th>
              <th>Period</th>
              <th>Grade</th>
              <th>Subject</th>
            </tr>
          </thead>
          <tbody>
            {slots.map((slot) => (
              <tr key={slot.id}>
                <td>{schools.find((s) => s.id === slot.school_id)?.name ?? slot.school_id}</td>
                <td>{slot.day_of_week}</td>
                <td>{slot.period}</td>
                <td>{slot.grade_level}</td>
                <td>{slot.subject_name}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function GradeTimetablePanel({
  slots,
  schools,
}: {
  slots: WeeklyTimetableSlot[];
  schools: School[];
}) {
  return (
    <section className="ticket-modal" aria-label="Grade timetable">
      <header>
        <h2>Grade Timetable</h2>
      </header>
      {slots.length === 0 ? (
        <p className="empty-state">No slots.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Grade</th>
              <th>Day</th>
              <th>Period</th>
              <th>Subject</th>
            </tr>
          </thead>
          <tbody>
            {slots.map((slot) => (
              <tr key={slot.id}>
                <td>{schools.find((s) => s.id === slot.school_id)?.name ?? slot.school_id}</td>
                <td>{slot.grade_level}</td>
                <td>{slot.day_of_week}</td>
                <td>{slot.period}</td>
                <td>{slot.subject_name}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function FacultyTimetablePanel({
  slots,
  schools,
}: {
  slots: WeeklyTimetableSlot[];
  schools: School[];
}) {
  return (
    <section className="ticket-modal" aria-label="Faculty timetable">
      <header>
        <h2>Faculty Timetable</h2>
      </header>
      {slots.length === 0 ? (
        <p className="empty-state">No slots.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Faculty</th>
              <th>Day</th>
              <th>Period</th>
              <th>Subject</th>
            </tr>
          </thead>
          <tbody>
            {slots.map((slot) => (
              <tr key={slot.id}>
                <td>{schools.find((s) => s.id === slot.school_id)?.name ?? slot.school_id}</td>
                <td>{slot.faculty_display_name || "Unassigned"}</td>
                <td>{slot.day_of_week}</td>
                <td>{slot.period}</td>
                <td>{slot.subject_name}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function HolidaysPanel({
  holidays,
  schools,
  regions,
  onBulkCreateHoliday,
  onDeleteHoliday,
}: {
  holidays: Holiday[];
  schools: School[];
  regions?: Region[];
  onBulkCreateHoliday: (input: BulkCreateHolidayInput) => Promise<void>;
  onDeleteHoliday: (id: number) => Promise<void>;
}) {
  const [showForm, setShowForm] = React.useState(false);
  const [saving, setSaving] = React.useState(false);
  const [draft, setDraft] = React.useState<BulkCreateHolidayInput>({
    name: "",
    start_date: "",
    end_date: "",
    scope: "global",
    region_id: null,
    school_id: null,
    grade_levels: null,
  });

  function resetDraft() {
    setDraft({ name: "", start_date: "", end_date: "", scope: "global", region_id: null, school_id: null, grade_levels: null });
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    try {
      await onBulkCreateHoliday(draft);
      resetDraft();
      setShowForm(false);
    } finally {
      setSaving(false);
    }
  }

  function toggleGrade(grade: string) {
    setDraft((d) => {
      const current = d.grade_levels ?? [];
      if (current.includes(grade)) {
        const next = current.filter((g) => g !== grade);
        return { ...d, grade_levels: next.length > 0 ? next : null };
      }
      return { ...d, grade_levels: [...current, grade] };
    });
  }

  // Group consecutive dates with same name/scope/school/region/grades for display
  const grouped = React.useMemo(() => {
    if (holidays.length === 0) return [];
    const sorted = [...holidays].sort((a, b) => a.date.localeCompare(b.date));
    const groups: Array<{ start: string; end: string; name: string; scope: string; target: string; grades: string[]; ids: number[] }> = [];
    let current = {
      start: sorted[0].date,
      end: sorted[0].date,
      name: sorted[0].name,
      scope: sorted[0].scope,
      target: sorted[0].school_name ?? sorted[0].region_name ?? "All schools",
      grades: sorted[0].grade_level ? [sorted[0].grade_level] : [],
      ids: [sorted[0].id],
    };
    for (let i = 1; i < sorted.length; i++) {
      const h = sorted[i];
      const target = h.school_name ?? h.region_name ?? "All schools";
      const isConsecutive = new Date(h.date).getTime() - new Date(current.end).getTime() === 86400000;
      const sameMeta = h.name === current.name && h.scope === current.scope && target === current.target;
      const gradeSet = h.grade_level ? [h.grade_level] : [];
      const sameGrades = JSON.stringify(gradeSet.sort()) === JSON.stringify([...current.grades].sort());
      if (isConsecutive && sameMeta && sameGrades) {
        current.end = h.date;
        current.ids.push(h.id);
      } else {
        groups.push(current);
        current = { start: h.date, end: h.date, name: h.name, scope: h.scope, target, grades: gradeSet, ids: [h.id] };
      }
    }
    groups.push(current);
    return groups;
  }, [holidays]);

  return (
    <section className="ticket-modal" aria-label="Holidays">
      <header>
        <h2>Holidays</h2>
        <button className="primary-action" onClick={() => setShowForm((s) => !s)}>
          {showForm ? "Cancel" : "Add Holiday"}
        </button>
      </header>

      {showForm && (
        <form onSubmit={handleSubmit} className="holiday-form">
          <div className="form-row">
            <label>
              Name
              <input
                required
                value={draft.name}
                onChange={(e) => setDraft((d) => ({ ...d, name: e.target.value }))}
              />
            </label>
          </div>
          <div className="form-row">
            <label>
              Start Date
              <input
                required
                type="date"
                value={draft.start_date}
                onChange={(e) => setDraft((d) => ({ ...d, start_date: e.target.value }))}
              />
            </label>
            <label>
              End Date
              <input
                required
                type="date"
                value={draft.end_date}
                onChange={(e) => setDraft((d) => ({ ...d, end_date: e.target.value }))}
              />
            </label>
          </div>
          <div className="form-row">
            <label>
              Scope
              <select
                value={draft.scope}
                onChange={(e) =>
                  setDraft((d) => ({
                    ...d,
                    scope: e.target.value,
                    region_id: e.target.value === "region" ? d.region_id : null,
                    school_id: e.target.value === "school" ? d.school_id : null,
                  }))
                }
              >
                <option value="global">Global</option>
                <option value="region">Region</option>
                <option value="school">School</option>
              </select>
            </label>
            {draft.scope === "region" && (
              <label>
                Region
                <select
                  required
                  value={draft.region_id ?? ""}
                  onChange={(e) => setDraft((d) => ({ ...d, region_id: Number(e.target.value) || null }))}
                >
                  <option value="">Select region</option>
                  {(regions ?? []).map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.name}
                    </option>
                  ))}
                </select>
              </label>
            )}
            {draft.scope === "school" && (
              <label>
                School
                <select
                  required
                  value={draft.school_id ?? ""}
                  onChange={(e) => setDraft((d) => ({ ...d, school_id: Number(e.target.value) || null }))}
                >
                  <option value="">Select school</option>
                  {schools.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </label>
            )}
          </div>
          <fieldset className="grade-fieldset">
            <legend>Grade Levels <small>(leave empty for all grades)</small></legend>
            <div className="grade-checkboxes">
              {gradeLevels.map((g) => (
                <label key={g} className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={(draft.grade_levels ?? []).includes(g)}
                    onChange={() => toggleGrade(g)}
                  />
                  {g}
                </label>
              ))}
            </div>
          </fieldset>
          <div className="form-actions">
            <button type="submit" className="primary-action" disabled={saving}>
              {saving ? "Saving…" : "Save Holiday"}
            </button>
            <button type="button" className="secondary-button" onClick={() => { resetDraft(); setShowForm(false); }}>
              Cancel
            </button>
          </div>
        </form>
      )}

      {holidays.length === 0 ? (
        <p className="empty-state">No holidays.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Dates</th>
              <th>Name</th>
              <th>Scope</th>
              <th>Applies To</th>
              <th>Grades</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {grouped.map((g, idx) => (
              <tr key={`${g.name}-${g.start}-${idx}`}>
                <td>{g.start === g.end ? g.start : `${g.start} → ${g.end}`}</td>
                <td>{g.name}</td>
                <td>{g.scope}</td>
                <td>{g.target}</td>
                <td>{g.grades.length > 0 ? g.grades.join(", ") : "All"}</td>
                <td>
                  <button className="link-action" onClick={() => { g.ids.forEach((id) => onDeleteHoliday(id)); }}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function EscalationPolicyPanel({
  policy,
  onSave,
}: {
  policy: EscalationPolicy | null;
  onSave: (policy: EscalationPolicy) => void;
}) {
  const [draft, setDraft] = React.useState(policy ?? { at_risk_hours: 24, escalation_assignee: "", auto_assign_on_breach: false, updated_at: new Date().toISOString() });
  return (
    <section className="ticket-modal" aria-label="Escalation policy">
      <header>
        <h2>Escalation Policy</h2>
        <button className="primary-action" onClick={() => onSave(draft)}>
          Save
        </button>
      </header>
      <label>
        At-Risk Hours
        <input type="number" value={draft.at_risk_hours} onChange={(e) => setDraft({ ...draft, at_risk_hours: Number(e.target.value) })} />
      </label>
      <label>
        Escalation Assignee
        <input value={draft.escalation_assignee} onChange={(e) => setDraft({ ...draft, escalation_assignee: e.target.value })} />
      </label>
      <label>
        <input type="checkbox" checked={draft.auto_assign_on_breach} onChange={(e) => setDraft({ ...draft, auto_assign_on_breach: e.target.checked })} />
        Auto-assign on breach
      </label>
    </section>
  );
}

export function SlaPolicyPanel({
  policies,
  onSave,
  onRefreshSla,
}: {
  policies: SlaPolicy[];
  onSave: (policies: SlaPolicy[]) => void;
  onRefreshSla?: () => void;
}) {
  const [draft, setDraft] = React.useState(policies);
  return (
    <section className="ticket-modal" aria-label="SLA policies">
      <header>
        <h2>SLA Policies</h2>
        <div style={{ display: "flex", gap: 8 }}>
          {onRefreshSla && (
            <button className="ghost-button" onClick={onRefreshSla}>
              Refresh SLA Status
            </button>
          )}
          <button className="primary-action" onClick={() => onSave(draft)}>
            Save
          </button>
        </div>
      </header>
      {draft.length === 0 ? (
        <p className="empty-state">No SLA policies.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Issue Category</th>
              <th>Hours</th>
            </tr>
          </thead>
          <tbody>
            {draft.map((p) => (
              <tr key={p.issue_category}>
                <td>{p.issue_category}</td>
                <td>
                  <input
                    type="number"
                    value={p.hours}
                    onChange={(e) => {
                      const next = draft.map((item) =>
                        item.issue_category === p.issue_category
                          ? { ...item, hours: Number(e.target.value) }
                          : item,
                      );
                      setDraft(next);
                    }}
                  />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function ReportsPanel({
  data,
  dasRows = [],
  schools = [],
  currentUserRole = "",
  onLoadDasReport,
}: {
  data: { tickets_by_status: Record<string, number>; tickets_by_school: Record<string, number> } | null;
  dasRows?: DasReportRow[];
  schools?: School[];
  currentUserRole?: string;
  onLoadDasReport?: (startDate: string, endDate: string, groupBy: DasGroupBy, schoolId?: number) => void;
}) {
  const today = React.useMemo(() => {
    const d = new Date();
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
  }, []);
  const [startDate, setStartDate] = React.useState(today);
  const [endDate, setEndDate] = React.useState(today);
  const [groupBy, setGroupBy] = React.useState<DasGroupBy>("school");
  const [schoolId, setSchoolId] = React.useState<number | "">("");
  const canViewDas = ["admin", "aom"].includes(currentUserRole);

  const [sortColumn, setSortColumn] = React.useState<string | null>(null);
  const [sortDirection, setSortDirection] = React.useState<"asc" | "desc">("asc");

  function handleSort(column: string) {
    if (sortColumn === column) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
    } else {
      setSortColumn(column);
      setSortDirection("asc");
    }
  }

  const sortedDasRows = React.useMemo(() => {
    if (!sortColumn) return dasRows;
    const col = sortColumn;
    const dir = sortDirection;
    return [...dasRows].sort((a, b) => {
      let aVal: string | number;
      let bVal: string | number;
      switch (col) {
        case "group": aVal = a.label; bVal = b.label; break;
        case "school": aVal = a.school_name; bVal = b.school_name; break;
        case "class": aVal = a.grade_level; bVal = b.grade_level; break;
        case "cohort": aVal = a.cohort; bVal = b.cohort; break;
        case "batch": aVal = a.batch_id; bVal = b.batch_id; break;
        case "scheduled": aVal = a.scheduled_lectures; bVal = b.scheduled_lectures; break;
        case "present": aVal = a.present_lectures; bVal = b.present_lectures; break;
        case "das": aVal = a.das_percent; bVal = b.das_percent; break;
        default: return 0;
      }
      if (typeof aVal === "number" && typeof bVal === "number") {
        return dir === "asc" ? aVal - bVal : bVal - aVal;
      }
      return dir === "asc"
        ? String(aVal).localeCompare(String(bVal))
        : String(bVal).localeCompare(String(aVal));
    });
  }, [dasRows, sortColumn, sortDirection]);

  if (!data) return <p className="empty-state">No report data.</p>;
  return (
    <section className="ticket-modal" aria-label="Reports">
      <header>
        <h2>Reports</h2>
      </header>
      {canViewDas && onLoadDasReport && (
        <section className="master-data-section" aria-label="DAS report">
          <header className="section-header">
            <div>
              <h3>DAS Attendance Index</h3>
              <p>Daily Attendance Score = present lecture-student records divided by scheduled lecture-student opportunities.</p>
            </div>
          </header>
          <div className="inline-edit-form">
            <label>
              Start Date
              <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
            </label>
            <label>
              End Date
              <input type="date" value={endDate} onChange={(e) => setEndDate(e.target.value)} />
            </label>
            <label>
              Group By
              <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as DasGroupBy)}>
                <option value="overall">Overall</option>
                <option value="school">School</option>
                <option value="class">Class / Batch</option>
                <option value="cohort">Cohort</option>
                <option value="student">Student</option>
              </select>
            </label>
            <label>
              School
              <select value={schoolId} onChange={(e) => setSchoolId(e.target.value ? Number(e.target.value) : "")}>
                <option value="">All scoped schools</option>
                {schools.map((school) => (
                  <option key={school.id} value={school.id}>{school.name}</option>
                ))}
              </select>
            </label>
            <div className="actions">
              <button
                className="primary-action"
                onClick={() => onLoadDasReport(startDate, endDate, groupBy, schoolId === "" ? undefined : schoolId)}
              >
                Calculate DAS
              </button>
            </div>
          </div>
          {dasRows.length === 0 ? (
            <p className="empty-state">No DAS data for the selected range.</p>
          ) : (
            <table className="data-table">
              <thead>
                <tr>
                  <th onClick={() => handleSort("group")} style={{ cursor: "pointer" }}>Group {sortColumn === "group" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("school")} style={{ cursor: "pointer" }}>School {sortColumn === "school" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("class")} style={{ cursor: "pointer" }}>Class {sortColumn === "class" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("cohort")} style={{ cursor: "pointer" }}>Cohort {sortColumn === "cohort" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("batch")} style={{ cursor: "pointer" }}>Batch {sortColumn === "batch" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("scheduled")} style={{ cursor: "pointer", textAlign: "right" }}>Scheduled {sortColumn === "scheduled" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("present")} style={{ cursor: "pointer", textAlign: "right" }}>Present {sortColumn === "present" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                  <th onClick={() => handleSort("das")} style={{ cursor: "pointer", textAlign: "right" }}>DAS {sortColumn === "das" ? (sortDirection === "asc" ? "▲" : "▼") : ""}</th>
                </tr>
              </thead>
              <tbody>
                {sortedDasRows.map((row, index) => (
                  <tr key={`${row.group_by}-${row.label}-${index}`}>
                    <td>{row.label}</td>
                    <td>{row.school_name || "—"}</td>
                    <td>{row.grade_level || "—"}</td>
                    <td>{row.cohort || "—"}</td>
                    <td>{row.batch_id || "—"}</td>
                    <td style={{ textAlign: "right" }}>{row.scheduled_lectures}</td>
                    <td style={{ textAlign: "right" }}>{row.present_lectures}</td>
                    <td style={{ textAlign: "right" }}>{row.das_percent}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>
      )}
      <h3>By Status</h3>
      <div className="report-grid">
        {Object.entries(data.tickets_by_status).map(([status, count]) => (
          <div key={status} className="report-cell">
            <strong>{count}</strong>
            <span>{status}</span>
          </div>
        ))}
      </div>
      <h3>By School</h3>
      <div className="report-grid">
        {Object.entries(data.tickets_by_school).map(([school, count]) => (
          <div key={school} className="report-cell">
            <strong>{count}</strong>
            <span>{school}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

export function AttendanceReportsPanel({
  summary,
  chronicAbsentees,
  subjectAttendance,
}: {
  summary: AttendanceSummaryRow[];
  chronicAbsentees: ChronicAbsentee[];
  subjectAttendance: SubjectAttendanceRow[];
}) {
  return (
    <section className="ticket-modal" aria-label="Attendance reports">
      <header>
        <h2>Attendance Reports</h2>
      </header>
      <h3>Summary</h3>
      {summary.length === 0 ? (
        <p className="empty-state">No attendance data.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Batch</th>
              <th>Present</th>
              <th>Absent</th>
              <th>Late</th>
            </tr>
          </thead>
          <tbody>
            {summary.map((row, i) => (
              <tr key={i}>
                <td>{row.school_name}</td>
                <td>{row.batch_id}</td>
                <td>{row.present_count}</td>
                <td>{row.absent_count}</td>
                <td>{row.late_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <h3>Chronic Absentees</h3>
      {chronicAbsentees.length === 0 ? (
        <p className="empty-state">No chronic absentees.</p>
      ) : (
        <ul>
          {chronicAbsentees.map((s, i) => (
            <li key={i}>
              {s.student_name} — {s.total_sessions - s.present_count} days absent
            </li>
          ))}
        </ul>
      )}
      <h3>By Subject</h3>
      {subjectAttendance.length === 0 ? (
        <p className="empty-state">No subject attendance data.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Subject</th>
              <th>Present</th>
              <th>Absent</th>
            </tr>
          </thead>
          <tbody>
            {subjectAttendance.map((row, i) => (
              <tr key={i}>
                <td>{row.subject_name}</td>
                <td>{row.present_count}</td>
                <td>{row.absent_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function AssignmentRulePanel({
  rules,
  onSave,
}: {
  rules: Array<{ id: number; queue: string; assignee: string; condition: string }>;
  onSave: (rules: Array<{ id: number; queue: string; assignee: string; condition: string }>) => void;
}) {
  const [draft, setDraft] = React.useState(rules);

  React.useEffect(() => {
    setDraft(rules);
  }, [rules]);

  const updateRule = (id: number, field: "queue" | "assignee" | "condition", value: string) => {
    setDraft((prev) =>
      prev.map((rule) => (rule.id === id ? { ...rule, [field]: value } : rule))
    );
  };

  return (
    <section className="ticket-modal" aria-label="Assignment rules">
      <header>
        <h2>Assignment Rules</h2>
        {draft.length > 0 && (
          <button className="primary-action" onClick={() => onSave(draft)}>
            Save
          </button>
        )}
      </header>
      {draft.length === 0 ? (
        <p className="empty-state">No rules.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Queue</th>
              <th>Assignee</th>
              <th>Condition</th>
            </tr>
          </thead>
          <tbody>
            {draft.map((rule) => (
              <tr key={rule.id}>
                <td>
                  <input
                    type="text"
                    value={rule.queue}
                    onChange={(e) => updateRule(rule.id, "queue", e.target.value)}
                    aria-label={`Queue for rule ${rule.id}`}
                  />
                </td>
                <td>
                  <input
                    type="text"
                    value={rule.assignee}
                    onChange={(e) => updateRule(rule.id, "assignee", e.target.value)}
                    aria-label={`Assignee for rule ${rule.id}`}
                  />
                </td>
                <td>
                  <select
                    value={rule.condition}
                    onChange={(e) => updateRule(rule.id, "condition", e.target.value)}
                    aria-label={`Condition for rule ${rule.id}`}
                  >
                    <option value="Active">Active</option>
                    <option value="Inactive">Inactive</option>
                  </select>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/* ── Compliance ────────────────────────────────────────────────────────── */

export function ComplianceAnalyticsPanel({
  data,
}: {
  data: Array<{ school_id: number; school_name: string; compliance_score: number; issues: string[] }>;
}) {
  return (
    <section className="ticket-modal" aria-label="Compliance analytics">
      <header>
        <h2>Compliance Analytics</h2>
      </header>
      {data.length === 0 ? (
        <p className="empty-state">No compliance data.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Score</th>
              <th>Issues</th>
            </tr>
          </thead>
          <tbody>
            {data.map((row) => (
              <tr key={row.school_id}>
                <td>{row.school_name}</td>
                <td>{row.compliance_score}%</td>
                <td>{row.issues.join(", ")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/* ── Cross-school faculty ──────────────────────────────────────────────── */

export function CrossSchoolFacultyPanel({
  assignments,
  schools,
  users,
}: {
  assignments: FacultyAssignment[];
  schools: School[];
  users: AppUser[];
}) {
  const crossSchool = assignments.filter((a) => {
    const schoolIds = assignments
      .filter((b) => b.faculty_user_id === a.faculty_user_id)
      .map((b) => b.school_id);
    return new Set(schoolIds).size > 1;
  });
  return (
    <section className="ticket-modal" aria-label="Cross-school faculty">
      <header>
        <h2>Cross-School Faculty</h2>
      </header>
      {crossSchool.length === 0 ? (
        <p className="empty-state">No cross-school assignments.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Faculty</th>
              <th>Schools</th>
              <th>Subjects</th>
            </tr>
          </thead>
          <tbody>
            {crossSchool.map((a) => (
              <tr key={a.id}>
                <td>{users.find((u) => u.id === a.faculty_user_id)?.display_name ?? a.faculty_user_id}</td>
                <td>
                  {[
                    ...new Set(
                      assignments
                        .filter((b) => b.faculty_user_id === a.faculty_user_id)
                        .map((b) => schools.find((s) => s.id === b.school_id)?.name ?? b.school_id),
                    ),
                  ].join(", ")}
                </td>
                <td>
                  {[
                    ...new Set(
                      assignments
                        .filter((b) => b.faculty_user_id === a.faculty_user_id)
                        .map((b) => b.subject_id),
                    ),
                  ].join(", ")}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/* ── Day at a glance ───────────────────────────────────────────────────── */

export function DayAtAGlancePanel({
  sessions,
  schools,
}: {
  sessions: FacultyTodaySession[];
  schools: School[];
}) {
  return (
    <section className="ticket-modal" aria-label="Day at a glance">
      <header>
        <h2>Day at a Glance</h2>
      </header>
      {sessions.length === 0 ? (
        <p className="empty-state">No sessions today.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>School</th>
              <th>Grade</th>
              <th>Subject</th>
              <th>Faculty</th>
              <th>Room</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((s, i) => (
              <tr key={i}>
                <td>
                  {s.start_time} - {s.end_time}
                </td>
                <td>{schools.find((school) => school.id === s.school_id)?.name ?? s.school_id}</td>
                <td>{s.grade_level}</td>
                <td>{s.subject_name}</td>
                <td>{s.faculty_name}</td>
                <td>—</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/* ── Health dashboard ──────────────────────────────────────────────────── */

export function HealthDashboardPanel({
  data,
}: {
  data: Array<{
    school_id: number;
    school_name: string;
    status: string;
    gaps_count: number;
    faculty_shortage: number;
    room_conflicts: number;
  }>;
}) {
  return (
    <section className="ticket-modal" aria-label="Health dashboard">
      <header>
        <h2>Timetable Health</h2>
      </header>
      {data.length === 0 ? (
        <p className="empty-state">No health data.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>School</th>
              <th>Status</th>
              <th>Gaps</th>
              <th>Faculty Shortage</th>
              <th>Room Conflicts</th>
            </tr>
          </thead>
          <tbody>
            {data.map((row) => (
              <tr key={row.school_id}>
                <td>{row.school_name}</td>
                <td>
                  <span className={`health-status health-${row.status.toLowerCase()}`}>{row.status}</span>
                </td>
                <td>{row.gaps_count}</td>
                <td>{row.faculty_shortage}</td>
                <td>{row.room_conflicts}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/* ── Sidebar ───────────────────────────────────────────────────────────── */

type SidebarProps = {
  activeFilter: Filter;
  currentUserRole: string;
  filterCounts: Record<Filter, number>;
  onFilterChange: (filter: Filter) => void;
  onToolClick: (toolId: string) => void;
};

export function Sidebar({
  activeFilter,
  currentUserRole,
  filterCounts,
  onFilterChange,
  onToolClick,
}: SidebarProps) {
  const visibleTools = APP_TOOLS.filter((t) => isToolVisible(t, currentUserRole));
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
      <nav aria-label="Tools" className="sidebar-section">
        <span className="sidebar-heading">Tools</span>
        <div className="nav-list">
          {visibleTools.map((tool) => (
            <button className="nav-item" key={tool.id} onClick={() => onToolClick(tool.id)}>
              {tool.label}
            </button>
          ))}
        </div>
      </nav>
    </aside>
  );
}

/* ── Faculty App Shell ─────────────────────────────────────────────────── */

export function FacultyApp({
  user,
  onLogout,
  weeklySlots,
  substitutions,
  pendingRequests,
  onLoadWeeklySlots,
  onLoadSubstitutions,
  onAcceptSubstitution,
  onDeclineSubstitution,
}: {
  user: CurrentUser;
  onLogout: () => void;
  weeklySlots: FacultyWeeklySlot[];
  substitutions: SubstitutionRecord[];
  pendingRequests: SubstitutionRecord[];
  onLoadWeeklySlots: () => void;
  onLoadSubstitutions: () => void;
  onAcceptSubstitution: (id: number) => void;
  onDeclineSubstitution: (id: number) => void;
}) {
  const [facultyTab, setFacultyTab] = React.useState<"schedule" | "subs" | "history">("schedule");
  React.useEffect(() => {
    onLoadWeeklySlots();
    onLoadSubstitutions();
  }, [onLoadWeeklySlots, onLoadSubstitutions]);

  return (
    <main className="app-shell faculty-app">
      <header className="faculty-header">
        <span className="faculty-greeting">Hello, {user.display_name}</span>
        <button className="ghost-button" onClick={onLogout}>
          Sign out
        </button>
      </header>
      <nav className="faculty-tabs">
        {(
          [
            ["schedule", "My Schedule"],
            ["subs", "Substitutions"],
            ["history", "History"],
          ] as const
        ).map(([key, label]) => (
          <button
            key={key}
            className={`faculty-tab${facultyTab === key ? " active" : ""}`}
            onClick={() => setFacultyTab(key)}
          >
            {label}
          </button>
        ))}
      </nav>
      {facultyTab === "schedule" && (
        <section className="faculty-schedule">
          {weeklySlots.length === 0 ? (
            <p className="empty-state">No schedule found.</p>
          ) : (
            weeklySlots.map((slot) => (
              <div key={slot.id} className="faculty-slot">
                <strong>
                  {slot.day_of_week} {slot.period}
                </strong>
                <span>{slot.subject_name}</span>
                <span>{slot.grade_level}</span>
              </div>
            ))
          )}
        </section>
      )}
      {facultyTab === "subs" && (
        <section className="faculty-subs">
          {pendingRequests.length === 0 ? (
            <p className="empty-state">No pending substitution requests.</p>
          ) : (
            pendingRequests.map((req) => (
              <div key={req.session_id} className="sub-request">
                <span>{req.subject_name}</span>
                <span>{req.grade_level}</span>
                <span>{req.session_date}</span>
                <div className="actions">
                  <button className="primary-action" onClick={() => onAcceptSubstitution(req.session_id)}>
                    Accept
                  </button>
                  <button className="secondary-button" onClick={() => onDeclineSubstitution(req.session_id)}>
                    Decline
                  </button>
                </div>
              </div>
            ))
          )}
        </section>
      )}
      {facultyTab === "history" && (
        <section className="faculty-history">
          {substitutions.length === 0 ? (
            <p className="empty-state">No substitution history.</p>
          ) : (
            substitutions.map((sub) => (
              <div key={sub.session_id} className="sub-history-item">
                <span>{sub.subject_name}</span>
                <span>{sub.session_date}</span>
                <span>{sub.status}</span>
              </div>
            ))
          )}
        </section>
      )}
    </main>
  );
}
