import React from "react";
import type { Alert, LeaveRequest, SubstitutionRecord, FacultyTodaySession, TimetableHealthStatus, School, CurrentUser, FacultyMember } from "../../types";
import { LeaveSwapPanel } from "../substitution/LeaveSwapPanel";
import { SubstitutionCommandCenter } from "../substitution/SubstitutionCommandCenter";
import { AlertInboxPanel } from "../automation/AlertInboxPanel";
import { MultiSchoolDayAtAGlance } from "../automation/MultiSchoolDayAtAGlance";

type ApproverView =
  | "dashboard"
  | "leave-swap"
  | "substitutions"
  | "alerts"
  | "day-at-glance"
  | "timetable-health";

type ApproverDashboardProps = {
  currentUser: CurrentUser;
  schools: School[];
  facultyMembers: FacultyMember[];
  leaveRequests: LeaveRequest[];
  pendingSubstitutions: SubstitutionRecord[];
  alerts: Alert[];
  adminSessions: FacultyTodaySession[];
  timetableHealth: TimetableHealthStatus[];
};

function TimetableHealthDetail({
  data,
  onClose,
}: {
  data: TimetableHealthStatus[];
  onClose: () => void;
}) {
  return (
    <section className="ticket-modal" role="dialog" aria-modal="true" aria-labelledby="health-title">
      <header>
        <div>
          <h2 id="health-title">Timetable Health Report</h2>
          <p>{data.length} school{data.length !== 1 ? "s" : ""}</p>
        </div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>
          Close
        </button>
      </header>
      <div style={{ margin: "0 24px 24px" }}>
        {data.length === 0 ? (
          <p>No health data available.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>School</th>
                <th>Region</th>
                <th>AOM</th>
                <th>Class Plans</th>
                <th>Master TT</th>
                <th>Sessions</th>
                <th>Gaps</th>
              </tr>
            </thead>
            <tbody>
              {data.map((h) => (
                <tr key={h.school_id}>
                  <td>{h.school_name}</td>
                  <td>{h.region_name}</td>
                  <td>{h.aom_name}</td>
                  <td>{h.class_plans_configured ? "✅" : "❌"}</td>
                  <td>{h.master_timetable_complete ? "✅" : "❌"}</td>
                  <td>{h.sessions_generated ? "✅" : "❌"}</td>
                  <td>
                    {h.gaps_count > 0 ? (
                      <span style={{ color: "#dc2626", fontWeight: 600 }}>{h.gaps_count}</span>
                    ) : (
                      <span style={{ color: "#16a34a" }}>0</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  );
}

export function ApproverDashboard({
  currentUser,
  schools,
  facultyMembers,
  leaveRequests,
  pendingSubstitutions,
  alerts,
  adminSessions,
  timetableHealth,
}: ApproverDashboardProps) {
  const [view, setView] = React.useState<ApproverView>("dashboard");

  const pendingLeaves = leaveRequests.filter((lr) => lr.status === "Pending");
  const criticalAlerts = alerts.filter((a) => a.severity === "critical");
  const warningAlerts = alerts.filter((a) => a.severity === "warning");
  const todayIssues = adminSessions.filter(
    (s) => s.status === "Cancelled" || s.status === "Needs Substitution"
  );
  const healthRisks = timetableHealth.filter((h) => h.status !== "Green");

  const schoolName =
    currentUser.school_ids.length === 1
      ? schools.find((s) => s.id === currentUser.school_ids[0])?.name
      : null;

  if (view === "leave-swap") {
    return (
      <div className="approver-dashboard">
        <header className="approver-header">
          <div>
            <button type="button" className="ghost-button" onClick={() => setView("dashboard")}>
              ← Back
            </button>
            <h1 style={{ marginTop: 8 }}>Leave & Swap</h1>
          </div>
        </header>
        <LeaveSwapPanel
          schools={schools}
          faculty={facultyMembers.map((f) => ({ id: f.id, display_name: f.name }))}
          currentUser={currentUser}
        />
      </div>
    );
  }

  if (view === "substitutions") {
    return (
      <div className="approver-dashboard">
        <header className="approver-header">
          <div>
            <button type="button" className="ghost-button" onClick={() => setView("dashboard")}>
              ← Back
            </button>
            <h1 style={{ marginTop: 8 }}>Substitution Command Center</h1>
          </div>
        </header>
        <SubstitutionCommandCenter schools={schools} />
      </div>
    );
  }

  if (view === "alerts") {
    return (
      <div className="approver-dashboard">
        <AlertInboxPanel onClose={() => setView("dashboard")} />
      </div>
    );
  }

  if (view === "day-at-glance") {
    return (
      <div className="approver-dashboard">
        <MultiSchoolDayAtAGlance
          schools={schools}
          onClose={() => setView("dashboard")}
        />
      </div>
    );
  }

  if (view === "timetable-health") {
    return (
      <div className="approver-dashboard">
        <TimetableHealthDetail
          data={timetableHealth}
          onClose={() => setView("dashboard")}
        />
      </div>
    );
  }

  return (
    <div className="approver-dashboard">
      <header className="approver-header">
        <div>
          <h1>{schoolName ?? "Approver Workspace"}</h1>
          <span className="approver-role">{currentUser.role === "head" ? "Head" : "AOM"}</span>
        </div>
      </header>

      {/* Quick actions */}
      <div className="approver-quick-actions">
        {pendingLeaves.length > 0 && (
          <button type="button" className="quick-action-chip chip-warning" onClick={() => setView("leave-swap")}>
            {pendingLeaves.length} leave request{pendingLeaves.length !== 1 ? "s" : ""}
          </button>
        )}
        {pendingSubstitutions.length > 0 && (
          <button type="button" className="quick-action-chip chip-critical" onClick={() => setView("substitutions")}>
            {pendingSubstitutions.length} substitution{pendingSubstitutions.length !== 1 ? "s" : ""}
          </button>
        )}
        {(criticalAlerts.length > 0 || warningAlerts.length > 0) && (
          <button type="button" className="quick-action-chip chip-info" onClick={() => setView("alerts")}>
            {criticalAlerts.length + warningAlerts.length} alert{criticalAlerts.length + warningAlerts.length !== 1 ? "s" : ""}
          </button>
        )}
      </div>

      <div className="approver-grid">
        {/* Needs Approval */}
        <div className="approver-card">
          <div className="approver-card-header">
            <h3>Needs Approval</h3>
            {pendingLeaves.length > 0 && (
              <span className="approver-badge">{pendingLeaves.length}</span>
            )}
          </div>
          {pendingLeaves.length === 0 ? (
            <p className="approver-empty">No pending leave requests.</p>
          ) : (
            <ul className="approver-list">
              {pendingLeaves.slice(0, 5).map((lr) => (
                <li key={lr.id} className="approver-list-item approver-list-item--action" onClick={() => setView("leave-swap")}>
                  <strong>{lr.faculty_name}</strong>
                  <span>
                    {lr.start_date} → {lr.end_date}
                  </span>
                  <small>{lr.reason}</small>
                </li>
              ))}
            </ul>
          )}
          <button type="button" className="approver-card-action" onClick={() => setView("leave-swap")}>
            Review Leave Requests →
          </button>
        </div>

        {/* Needs Substitution */}
        <div className="approver-card">
          <div className="approver-card-header">
            <h3>Needs Substitution</h3>
            {pendingSubstitutions.length > 0 && (
              <span className="approver-badge">{pendingSubstitutions.length}</span>
            )}
          </div>
          {pendingSubstitutions.length === 0 ? (
            <p className="approver-empty">No pending substitutions.</p>
          ) : (
            <ul className="approver-list">
              {pendingSubstitutions.slice(0, 5).map((sub) => (
                <li key={sub.session_id} className="approver-list-item approver-list-item--action" onClick={() => setView("substitutions")}>
                  <strong>
                    {sub.grade_level} {sub.subject_name}
                  </strong>
                  <span>{sub.session_date}</span>
                  <small>{sub.original_faculty_name}</small>
                </li>
              ))}
            </ul>
          )}
          <button type="button" className="approver-card-action" onClick={() => setView("substitutions")}>
            Assign Substitutes →
          </button>
        </div>

        {/* Today&apos;s Issues */}
        <div className="approver-card">
          <div className="approver-card-header">
            <h3>Today&apos;s Issues</h3>
            {todayIssues.length > 0 && (
              <span className="approver-badge warning">{todayIssues.length}</span>
            )}
          </div>
          {todayIssues.length === 0 && alerts.length === 0 ? (
            <p className="approver-empty">No issues today.</p>
          ) : (
            <ul className="approver-list">
              {todayIssues.slice(0, 3).map((s) => (
                <li key={s.session_id} className={`approver-list-item ${s.status === "Cancelled" ? "issue-cancelled" : "issue-warning"}`}>
                  <strong>
                    {s.grade_level} {s.subject_name}
                  </strong>
                  <span>
                    {s.start_time} — {s.status}
                  </span>
                </li>
              ))}
              {alerts.slice(0, 3).map((a) => (
                <li key={a.id} className={`approver-list-item alert-${a.severity}`}>
                  <strong>{a.category}</strong>
                  <span>{a.message}</span>
                </li>
              ))}
            </ul>
          )}
          <button type="button" className="approver-card-action" onClick={() => setView("day-at-glance")}>
            Day at a Glance →
          </button>
        </div>

        {/* School Risks */}
        <div className="approver-card">
          <div className="approver-card-header">
            <h3>School Risks</h3>
            {healthRisks.length > 0 && (
              <span className="approver-badge critical">{healthRisks.length}</span>
            )}
          </div>
          {healthRisks.length === 0 && criticalAlerts.length === 0 ? (
            <p className="approver-empty">No active risks.</p>
          ) : (
            <ul className="approver-list">
              {healthRisks.slice(0, 5).map((h) => (
                <li key={h.school_id} className={`approver-list-item health-${h.status.toLowerCase()}`}>
                  <strong>{h.school_name}</strong>
                  <span>
                    {h.gaps_count} gap{h.gaps_count !== 1 ? "s" : ""} · {h.master_timetable_complete ? "Complete" : "Incomplete"}
                  </span>
                </li>
              ))}
              {criticalAlerts.slice(0, 3).map((a) => (
                <li key={a.id} className="approver-list-item alert-critical">
                  <strong>{a.category}</strong>
                  <span>{a.message}</span>
                </li>
              ))}
            </ul>
          )}
          <button type="button" className="approver-card-action" onClick={() => setView("timetable-health")}>
            View Health Report →
          </button>
        </div>
      </div>
    </div>
  );
}
