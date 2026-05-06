import React from "react";
import { api } from "../../api";
import type {
  Alert,
  AttendanceRecord,
  CurrentUser,
  FacultyTodaySession,
  FacultyWeeklySlot,
  SubstitutionRecord,
} from "../../types";
import { requestNotificationPermission, sendBrowserNotification, scheduleReminder, cancelReminder } from "../../notifications";
import { SubstitutionInbox } from "./SubstitutionInbox";
import { FacultyBottomNav } from "../mobile/FacultyBottomNav";
import { OfflineBanner } from "../mobile/OfflineBanner";
import { useOfflineCache } from "../../hooks/useOfflineCache";

const STATUS_CYCLE: AttendanceRecord["status"][] = [
  "Present",
  "Late",
  "Excused",
  "Leave",
  "Out of Class",
  "Absent",
];

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const FULL_DAY_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

function statusClass(status: string) {
  return status.toLowerCase().replace(/\s+/g, "-");
}

function isLocked(record: AttendanceRecord): boolean {
  if (!record.marked_at) return false;
  const marked = new Date(record.marked_at).getTime();
  const cutoff = Date.now() - 48 * 60 * 60 * 1000;
  return marked < cutoff;
}

function getGradeColorClass(gradeLevel: string): string {
  if (gradeLevel.includes("8")) return "grade-8";
  if (gradeLevel.includes("9")) return "grade-9";
  if (gradeLevel.includes("10")) return "grade-10";
  return "";
}

function formatTimeLeft(minutes: number): string {
  if (minutes < 1) return "Starting now";
  if (minutes < 60) return `${Math.round(minutes)} mins left`;
  const h = Math.floor(minutes / 60);
  const m = Math.round(minutes % 60);
  return m > 0 ? `${h}h ${m}m left` : `${h}h left`;
}

function getPeriodDate(weekStart: string, dayOffset: number): string {
  const d = new Date(weekStart + "T00:00:00");
  d.setDate(d.getDate() + dayOffset);
  return d.toISOString().split("T")[0];
}

function getWeekStart(dateStr: string): string {
  const d = new Date(dateStr + "T00:00:00");
  const day = d.getDay();
  d.setDate(d.getDate() - day);
  return d.toISOString().split("T")[0];
}

function addDays(dateStr: string, days: number): string {
  const d = new Date(dateStr + "T00:00:00");
  d.setDate(d.getDate() + days);
  return d.toISOString().split("T")[0];
}

type AlertItem = {
  id: string;
  type: "next_class" | "room_change" | "cancellation" | "substitution_request" | "generic";
  message: string;
};

type FacultyAppProps = {
  user: CurrentUser;
  onLogout: () => void;
  weeklySlots: FacultyWeeklySlot[];
  substitutions: SubstitutionRecord[];
  pendingRequests: SubstitutionRecord[];
  onLoadWeeklySlots: () => Promise<void>;
  onLoadSubstitutions: () => Promise<void>;
  onAcceptSubstitution: (sessionId: number) => Promise<void>;
  onDeclineSubstitution: (sessionId: number, reason: string) => Promise<void>;
};

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
}: FacultyAppProps) {
  const [view, setView] = React.useState<"today" | "attendance" | "requests">("today");
  const [todaySessions, setTodaySessions] = React.useState<FacultyTodaySession[]>([]);
  const [activeSessionId, setActiveSessionId] = React.useState<number | null>(null);
  const [attendance, setAttendance] = React.useState<AttendanceRecord[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [saving, setSaving] = React.useState(false);
  const [notice, setNotice] = React.useState("");
  const [backendAlerts, setBackendAlerts] = React.useState<Alert[]>([]);
  const [dismissedBackendAlertIds, setDismissedBackendAlertIds] = React.useState<Set<string>>(new Set());
  const reminderTimers = React.useRef<number[]>([]);

  const [dayOffset, setDayOffset] = React.useState(0);
  const [showWeekView, setShowWeekView] = React.useState(false);
  const [expandedSlotId, setExpandedSlotId] = React.useState<string | null>(null);
  const [slotNotes, setSlotNotes] = React.useState<Record<string, string>>({});

  function slotKey(slot: FacultyWeeklySlot): string {
    return slot.id ? String(slot.id) : `${slot.school_id}-${slot.day_of_week}-${slot.period}-${slot.subject_name}`;
  }
  const [dismissedAlerts, setDismissedAlerts] = React.useState<Set<string>>(() => {
    try {
      const saved = localStorage.getItem("faculty:dismissedAlerts");
      return new Set(saved ? JSON.parse(saved) : []);
    } catch (e) {
      console.error("Failed to load dismissedAlerts:", e);
      return new Set();
    }
  });
  const [notifiedSubIds, setNotifiedSubIds] = React.useState<Set<number>>(() => {
    try {
      const saved = localStorage.getItem("faculty:notifiedSubIds");
      return new Set(saved ? JSON.parse(saved) : []);
    } catch (e) {
      console.error("Failed to load notifiedSubIds:", e);
      return new Set();
    }
  });
  const [lastUpdated, setLastUpdated] = React.useState<string>(() => new Date().toLocaleString());
  const [touchStartX, setTouchStartX] = React.useState<number | null>(null);
  const { isOnline, needsSync, cacheTimetable, getCachedTimetable, cacheSubstitutions, getCachedSubstitutions, clearSyncFlag } = useOfflineCache();
  const [attendanceLoading, setAttendanceLoading] = React.useState(false);
  const [attendanceError, setAttendanceError] = React.useState<string | null>(null);
  const [subLoading, setSubLoading] = React.useState(false);
  const [subError, setSubError] = React.useState<string | null>(null);

  const handleLoadSubstitutions = React.useCallback(async () => {
    setSubLoading(true);
    setSubError(null);
    try {
      await onLoadSubstitutions();
    } catch (e) {
      setSubError(String(e));
    } finally {
      setSubLoading(false);
    }
  }, [onLoadSubstitutions]);

  React.useEffect(() => {
    void loadToday();
    void onLoadWeeklySlots();
    void onLoadSubstitutions();
    void requestNotificationPermission();
  }, []);

  // Poll backend faculty alerts every 60 seconds
  React.useEffect(() => {
    async function loadFacultyAlerts() {
      try {
        const items = await api<Alert[]>("get_faculty_alerts");
        setBackendAlerts(items);
      } catch (e) {
        console.error("Failed to load faculty alerts:", e);
      }
    }
    void loadFacultyAlerts();
    const interval = setInterval(() => void loadFacultyAlerts(), 60_000);
    return () => clearInterval(interval);
  }, []);

  // Schedule browser reminders for next class and substitution requests
  React.useEffect(() => {
    // Clear old timers
    reminderTimers.current.forEach(cancelReminder);
    reminderTimers.current = [];

    const now = Date.now();

    // Next class reminder
    const upcoming = todaySessions
      .filter((s) => s.status !== "Cancelled")
      .map((s) => {
        const start = new Date(`${s.session_date}T${s.start_time || "00:00"}`).getTime();
        return { ...s, startTime: start, diffMs: start - now };
      })
      .filter((s) => s.diffMs > 0)
      .sort((a, b) => a.diffMs - b.diffMs)[0];

    if (upcoming && upcoming.diffMs > 15 * 60 * 1000) {
      const timer = scheduleReminder(
        upcoming.diffMs - 15 * 60 * 1000,
        `Next class in 15 minutes — ${upcoming.batch_id || `${upcoming.grade_level} ${upcoming.track}`}, Period ${upcoming.period}, ${upcoming.school_name}`
      );
      reminderTimers.current.push(timer);
    }

    // Substitution request notifications — deduplicated
    const newRequests = pendingRequests.filter((req) => !notifiedSubIds.has(req.session_id));
    newRequests.forEach((req) => {
      sendBrowserNotification(
        "SAATHI — Substitution Request",
        `${req.grade_level} ${req.subject_name} — accept or decline?`
      );
    });
    if (newRequests.length > 0) {
      setNotifiedSubIds((prev) => {
        const next = new Set(prev);
        newRequests.forEach((r) => next.add(r.session_id));
        return next;
      });
    }

    return () => {
      reminderTimers.current.forEach(cancelReminder);
    };
  }, [todaySessions, pendingRequests]);

  React.useEffect(() => {
    if (notice) {
      const t = setTimeout(() => setNotice(""), 3000);
      return () => clearTimeout(t);
    }
  }, [notice]);

  React.useEffect(() => {
    try {
      localStorage.setItem(
        "faculty:notifiedSubIds",
        JSON.stringify(Array.from(notifiedSubIds))
      );
    } catch (e) {
      console.error("Failed to save notifiedSubIds:", e);
    }
  }, [notifiedSubIds]);

  // Cache data for offline use
  React.useEffect(() => {
    if (weeklySlots.length > 0) cacheTimetable(weeklySlots);
  }, [weeklySlots, cacheTimetable]);

  React.useEffect(() => {
    if (substitutions.length > 0) cacheSubstitutions(substitutions);
  }, [substitutions, cacheSubstitutions]);

  React.useEffect(() => {
    setLastUpdated(new Date().toLocaleString());
  }, [todaySessions, pendingRequests, weeklySlots, substitutions]);

  React.useEffect(() => {
    try {
      localStorage.setItem(
        "faculty:dismissedAlerts",
        JSON.stringify(Array.from(dismissedAlerts))
      );
    } catch (e) {
      console.error("Failed to save dismissedAlerts:", e);
    }
  }, [dismissedAlerts]);

  async function loadToday() {
    setLoading(true);
    try {
      const sessions = await api<FacultyTodaySession[]>("faculty_today_sessions");
      setTodaySessions(sessions);
    } catch (e) {
      setNotice(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function openSession(sessionId: number) {
    setActiveSessionId(sessionId);
    setAttendanceLoading(true);
    setAttendanceError(null);
    try {
      const records = await api<AttendanceRecord[]>("faculty_session_attendance", {
        sessionId,
      });
      setAttendance(records);
    } catch (e) {
      setAttendanceError(String(e));
      setNotice(String(e));
    } finally {
      setAttendanceLoading(false);
    }
  }

  async function saveAttendance() {
    if (!activeSessionId) return;
    setSaving(true);
    try {
      const records = attendance.map((a) => ({
        student_id: a.student_id,
        status: a.status,
      }));
      await api("mark_attendance", {
        sessionId: activeSessionId,
        input: { records },
      });
      setNotice("Attendance saved");
      await loadToday();
      setActiveSessionId(null);
    } catch (e) {
      setNotice(String(e));
    } finally {
      setSaving(false);
    }
  }

  function cycleStatus(studentId: number) {
    setAttendance((prev) =>
      prev.map((a) => {
        if (a.student_id !== studentId) return a;
        const idx = STATUS_CYCLE.indexOf(a.status);
        const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length];
        return { ...a, status: next };
      })
    );
  }

  function dismissAlert(id: string) {
    setDismissedAlerts((prev) => new Set(prev).add(id));
  }

  // ── Alert generation ──
  const localAlerts: AlertItem[] = React.useMemo(() => {
    const items: AlertItem[] = [];
    const now = Date.now();

    // Next class reminder
    const upcoming = todaySessions
      .filter((s) => s.status !== "Cancelled")
      .map((s) => {
        const start = new Date(`${s.session_date}T${s.start_time || "00:00"}`).getTime();
        return { ...s, startTime: start, diffMin: (start - now) / 60000 };
      })
      .filter((s) => s.diffMin > -60 && s.diffMin <= 30)
      .sort((a, b) => a.diffMin - b.diffMin)[0];

    if (upcoming && upcoming.diffMin > 0 && upcoming.diffMin <= 30) {
      items.push({
        id: `next-${upcoming.session_id}`,
        type: "next_class",
        message: `Next class in ${Math.round(upcoming.diffMin)} minutes — ${upcoming.batch_id || `${upcoming.grade_level} ${upcoming.track}`}, ${upcoming.school_name}`,
      });
    }

    // Cancelled sessions
    todaySessions
      .filter((s) => s.status === "Cancelled")
      .forEach((s) => {
        items.push({
          id: `cancel-${s.session_id}`,
          type: "cancellation",
          message: `Your ${s.grade_level} class is cancelled today — ${s.subject_name}`,
        });
      });

    // Pending substitution requests
    pendingRequests.forEach((req) => {
      items.push({
        id: `sub-req-${req.session_id}`,
        type: "substitution_request",
        message: `Substitution request: ${req.grade_level} ${req.subject_name} — accept?`,
      });
    });

    return items;
  }, [todaySessions, pendingRequests]);

  const visibleLocalAlerts = localAlerts.filter((a) => !dismissedAlerts.has(a.id));
  const visibleBackendAlerts = backendAlerts.filter((a) => !dismissedBackendAlertIds.has(a.id));

  // ── Day / week view data ──
  const todayStr = new Date().toISOString().split("T")[0];
  const currentWeekStart = getWeekStart(todayStr);
  const selectedDate = addDays(todayStr, dayOffset);
  const selectedDayOfWeek = (new Date(selectedDate + "T00:00:00").getDay() + 6) % 7;

  const daySlots = React.useMemo(() => {
    return weeklySlots
      .filter((s) => s.day_of_week === selectedDayOfWeek)
      .sort((a, b) => a.period - b.period);
  }, [weeklySlots, selectedDayOfWeek]);

  const maxPeriod = React.useMemo(() => {
    if (weeklySlots.length === 0) return 8;
    return Math.max(...weeklySlots.map((s) => s.period), 8);
  }, [weeklySlots]);

  const weekGrid = React.useMemo(() => {
    const grid: Record<number, Record<number, FacultyWeeklySlot[]>> = {};
    for (let p = 1; p <= maxPeriod; p++) {
      grid[p] = {};
      for (let d = 0; d <= 6; d++) {
        grid[p][d] = [];
      }
    }
    weeklySlots.forEach((s) => {
      if (!grid[s.period]) grid[s.period] = {};
      if (!grid[s.period][s.day_of_week]) grid[s.period][s.day_of_week] = [];
      grid[s.period][s.day_of_week].push(s);
    });
    return grid;
  }, [weeklySlots, maxPeriod]);

  // ── Next-class banner text ──
  const nextBanner = React.useMemo(() => {
    const now = Date.now();
    const upcoming = todaySessions
      .filter((s) => s.status !== "Cancelled")
      .map((s) => {
        const start = new Date(`${s.session_date}T${s.start_time || "00:00"}`).getTime();
        const end = new Date(`${s.session_date}T${s.end_time || "23:59"}`).getTime();
        return { ...s, startTime: start, endTime: end, diffMin: (start - now) / 60000 };
      })
      .filter((s) => s.endTime > now)
      .sort((a, b) => a.startTime - b.startTime)[0];

    if (!upcoming) return null;
    const diffMin = upcoming.diffMin;
    const timeText = diffMin > 0 ? formatTimeLeft(diffMin) : "In progress";
    return `Next: ${upcoming.batch_id || `${upcoming.grade_level} ${upcoming.track}`}, Period ${upcoming.period}, ${upcoming.school_name} — ${timeText}`;
  }, [todaySessions]);

  // ── Touch handlers for swipe ──
  function onTouchStart(e: React.TouchEvent) {
    setTouchStartX(e.changedTouches[0].screenX);
  }
  function onTouchEnd(e: React.TouchEvent) {
    if (touchStartX == null) return;
    const endX = e.changedTouches[0].screenX;
    const diff = touchStartX - endX;
    if (diff > 50) {
      setDayOffset((o) => o + 1);
    } else if (diff < -50) {
      setDayOffset((o) => o - 1);
    }
    setTouchStartX(null);
  }

  function handlePrint() {
    window.print();
  }

  // ── Attendance view ──
  if (activeSessionId) {
    const session = todaySessions.find((s) => s.session_id === activeSessionId);
    return (
      <div className="faculty-app">
        <header className="faculty-header">
          <button type="button" onClick={() => setActiveSessionId(null)}>
            ← Back
          </button>
          <h2>{session?.subject_name}</h2>
        </header>
        <div className="faculty-session-meta">
          {session?.batch_id || `${session?.grade_level} ${session?.track}`} · Period {session?.period} ·{" "}
          {session?.school_name}
        </div>

        {attendanceLoading ? (
          <p className="empty-state">Loading students…</p>
        ) : attendanceError ? (
          <p className="empty-state">Failed to load. Pull to refresh or try again.</p>
        ) : attendance.length === 0 ? (
          <p className="empty-state">No data available</p>
        ) : (
          <>
            <div className="attendance-progress">
              Attending:{" "}
              {
                attendance.filter((a) => a.status === "Present" || a.status === "Late")
                  .length
              }{" "}
              / {attendance.length}
            </div>
            <div className="attendance-list">
              {attendance.map((a) => {
                const locked = isLocked(a);
                return (
                  <button
                    key={a.student_id}
                    type="button"
                    className={`attendance-row ${statusClass(a.status)} ${
                      locked ? "locked" : ""
                    }`}
                    onClick={() => !locked && cycleStatus(a.student_id)}
                    disabled={locked}
                    title={locked ? "Locked — contact admin to change" : undefined}
                  >
                    <span className="student-name">{a.student_name}</span>
                    <span className="status-badge">
                      {a.status}
                      {locked ? <span aria-hidden="true"> (Locked)</span> : null}
                    </span>
                  </button>
                );
              })}
            </div>
            <button
              type="button"
              className="primary-action save-attendance"
              onClick={saveAttendance}
              disabled={saving}
            >
              {saving ? "Saving…" : "Save Attendance"}
            </button>
          </>
        )}

        {notice && <div className="notice-bar">{notice}</div>}
      </div>
    );
  }

  return (
    <div className="faculty-app">
      <header className="faculty-header">
        <h1>SAATHI</h1>
        <div className="faculty-header-right">
          <span className="faculty-name">{user.display_name}</span>
          <button type="button" className="faculty-logout" onClick={onLogout}>
            Sign Out
          </button>
        </div>
      </header>

      <OfflineBanner isOnline={isOnline} needsSync={needsSync} onSync={clearSyncFlag} />

      <div className="last-updated" aria-live="polite">
        Last updated: {lastUpdated}
      </div>

      {/* Alerts */}
      {(visibleLocalAlerts.length > 0 || visibleBackendAlerts.length > 0) && (
        <div className="faculty-alerts">
          {visibleLocalAlerts.map((alert, idx) => (
            <div key={`local-${idx}-${alert.id || "no-id"}-${alert.message?.slice(0, 20) || "no-msg"}`} className={`faculty-alert alert-${alert.type}`}>
              <span>{alert.message}</span>
              <button
                type="button"
                className="alert-dismiss"
                onClick={() => dismissAlert(alert.id)}
                aria-label="Dismiss alert"
              >
                <span aria-hidden="true">×</span>
              </button>
            </div>
          ))}
          {visibleBackendAlerts.map((alert, idx) => (
            <div key={`backend-${idx}-${alert.id || "no-id"}-${alert.category || "no-cat"}-${alert.message?.slice(0, 20) || "no-msg"}`} className={`faculty-alert alert-${alert.severity}`}>
              <span>{alert.message}</span>
              <button
                type="button"
                className="alert-dismiss"
                onClick={() => setDismissedBackendAlertIds((prev) => new Set(prev).add(alert.id))}
                aria-label="Dismiss alert"
              >
                <span aria-hidden="true">×</span>
              </button>
            </div>
          ))}
        </div>
      )}

      {view === "today" && (
        <div className="faculty-today">
          {/* Next class banner */}
          {nextBanner && (
            <div className="next-class-banner">
              <span className="next-class-icon" aria-hidden="true">⏰</span>
              <span className="next-class-text">{nextBanner}</span>
            </div>
          )}

          {/* Quick action strip */}
          <div className="faculty-quick-actions">
            <button
              type="button"
              className="quick-action-card"
              onClick={() => setView("attendance")}
            >
              <span>✓</span>
              <strong>Mark Attendance</strong>
              <small>{todaySessions.filter((s) => s.status !== "Cancelled").length} sessions today</small>
            </button>
            <button
              type="button"
              className="quick-action-card"
              onClick={() => setView("requests")}
            >
              <span>🔄</span>
              <strong>Requests</strong>
              <small>{pendingRequests.length} pending</small>
            </button>
          </div>

          {/* Day navigator */}
          <div
            className="day-navigator"
            onTouchStart={onTouchStart}
            onTouchEnd={onTouchEnd}
          >
            <button
              type="button"
              className="day-nav-btn"
              onClick={() => setDayOffset((o) => o - 1)}
              aria-label="Previous day"
            >
              ‹
            </button>
            <div className="day-nav-center">
              <strong>
                {dayOffset === 0
                  ? "Today"
                  : dayOffset === -1
                  ? "Yesterday"
                  : dayOffset === 1
                  ? "Tomorrow"
                  : FULL_DAY_NAMES[selectedDayOfWeek]}
              </strong>
              <small>{selectedDate}</small>
            </div>
            <button
              type="button"
              className="day-nav-btn"
              onClick={() => setDayOffset((o) => o + 1)}
              aria-label="Next day"
            >
              ›
            </button>
          </div>

          {/* View toggle */}
          <div className="view-toggle">
            <button
              type="button"
              className={!showWeekView ? "active" : ""}
              onClick={() => setShowWeekView(false)}
            >
              Day
            </button>
            <button
              type="button"
              className={showWeekView ? "active" : ""}
              onClick={() => setShowWeekView(true)}
            >
              Week
            </button>
            <button
              type="button"
              className="print-btn"
              onClick={handlePrint}
              title="Print timetable"
            >
              <span aria-hidden="true">🖨️</span>
            </button>
          </div>

          {/* Day cards */}
          {!showWeekView && (
            <div
              className="day-card"
              onTouchStart={onTouchStart}
              onTouchEnd={onTouchEnd}
            >
              {loading ? (
                <p className="empty-state">Loading…</p>
              ) : daySlots.length === 0 ? (
                <p className="empty-state compact">No classes scheduled.</p>
              ) : (
                <div className="slot-list">
                  {daySlots.map((slot) => {
                    const isPrep = !slot.subject_name || slot.subject_name === "PREP";
                    const gradeClass = getGradeColorClass(slot.grade_level);
                    const key = slotKey(slot);
                    const isExpanded = expandedSlotId === key;
                    const note = slotNotes[key] ?? slot.notes ?? "";

                    return (
                      <div
                        key={key}
                        className={`slot-card ${gradeClass} ${
                          isPrep ? "prep-slot" : ""
                        } ${(slot.is_substitution ?? false) ? "substitution-slot" : ""}`}
                      >
                        <button
                          type="button"
                          className="slot-card-main"
                          onClick={() =>
                            setExpandedSlotId(isExpanded ? null : key)
                          }
                        >
                          <div className="slot-top">
                            <strong>
                              {isPrep ? "PREP" : `Period ${slot.period}`}
                            </strong>
                            <span className="slot-time">
                              {slot.start_time || "--:--"} – {slot.end_time || "--:--"}
                            </span>
                            {(slot.is_substitution ?? false) && (
                              <span className="sub-dot" title="Substitution" aria-label="Substitution" />
                            )}
                          </div>
                          <div className="slot-subject">
                            {isPrep ? "Prep Period" : slot.subject_name}
                          </div>
                          <div className="slot-meta">
                            {slot.grade_level} {slot.track} · {slot.school_name}
                            {slot.room && ` · Room ${slot.room}`}
                          </div>
                          {(slot.is_substitution ?? false) && (slot.original_faculty_name ?? null) && (
                            <div className="slot-origin">
                              Sub for {slot.original_faculty_name}
                            </div>
                          )}
                        </button>

                        {isExpanded && (
                          <div className="slot-expand">
                            <label>
                              Notes / Planned topic
                              <textarea
                                rows={3}
                                value={note}
                                onChange={(e) =>
                                  setSlotNotes((prev) => ({
                                    ...prev,
                                    [key]: e.target.value,
                                  }))
                                }
                                placeholder="Enter planned topic or notes…"
                              />
                            </label>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}

          {/* Week grid */}
          {showWeekView && (
            <div className="week-grid-wrapper">
              <table className="week-grid">
                <thead>
                  <tr>
                    <th>Period</th>
                    {DAY_NAMES.map((d, i) => (
                      <th key={i} className={i === (new Date().getDay() + 6) % 7 ? "today-col" : ""}>
                        {d}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {Array.from({ length: maxPeriod }, (_, pIdx) => {
                    const period = pIdx + 1;
                    return (
                      <tr key={period}>
                        <td className="period-label">{period}</td>
                        {Array.from({ length: 7 }, (_, dIdx) => {
                          const slots = weekGrid[period]?.[dIdx] ?? [];
                          return (
                            <td key={dIdx} className={dIdx === (new Date().getDay() + 6) % 7 ? "today-col" : ""}>
                              {slots.map((s) => {
                                const isPrep = !s.subject_name || s.subject_name === "PREP";
                                const key = slotKey(s);
                                return (
                                  <div
                                    key={key}
                                    className={`week-cell ${getGradeColorClass(
                                      s.grade_level
                                    )} ${isPrep ? "prep-cell" : ""} ${
                                      (s.is_substitution ?? false) ? "sub-cell" : ""
                                    }`}
                                  >
                                    <strong>{isPrep ? "PREP" : s.subject_name}</strong>
                                    {!isPrep && (
                                      <>
                                        <span>{s.grade_level} {s.track}</span>
                                        {s.room && <span>Rm {s.room}</span>}
                                      </>
                                    )}
                                  </div>
                                );
                              })}
                            </td>
                          );
                        })}
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {view === "attendance" && (
        <div className="faculty-attendance">
          <h2>Attendance</h2>
          {todaySessions.length === 0 ? (
            <p className="empty-state compact">No sessions today.</p>
          ) : (
            <div className="session-list">
              {todaySessions.map((s) => (
                <button
                  key={s.session_id}
                  type="button"
                  className={`session-card ${
                    s.status === "Cancelled" ? "cancelled" : ""
                  }`}
                  onClick={() =>
                    s.status !== "Cancelled" && openSession(s.session_id)
                  }
                  disabled={s.status === "Cancelled"}
                >
                  <div className="session-top">
                    <strong>Period {s.period}</strong>
                    <span className="session-time">
                      {s.start_time || "--:--"}
                    </span>
                  </div>
                  <div className="session-subject">{s.subject_name}</div>
                  <div className="session-meta">
                    {s.batch_id || `${s.grade_level} ${s.track}`} · {s.school_name}
                  </div>
                  <div className="session-stats">
                    {s.total_students} students ·{" "}
                    {s.present_count + s.late_count} attending · {s.absent_count}{" "}
                    absent
                  </div>
                  {s.status === "Completed" && (
                    <span className="badge completed">Completed</span>
                  )}
                  {s.status === "Substituted" && (
                    <span className="badge substituted">Substituted</span>
                  )}
                  {s.status === "Cancelled" && (
                    <span className="badge cancelled">Cancelled</span>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {view === "requests" && (
        <div className="faculty-requests">
          <h2>Requests</h2>
          {subLoading && <p className="empty-state">Loading…</p>}
          {subError && <p className="empty-state">Failed to load. Pull to refresh or try again.</p>}
          {!subLoading && !subError && (
            <SubstitutionInbox
              substitutions={substitutions}
              pendingRequests={pendingRequests}
              currentUser={user}
              onAccept={onAcceptSubstitution}
              onDecline={onDeclineSubstitution}
              onLoad={handleLoadSubstitutions}
            />
          )}
        </div>
      )}

      {notice && <div className="notice-bar">{notice}</div>}

      <FacultyBottomNav
        activeTab={view}
        onChange={(tab) => setView(tab)}
        pendingSubstitutionCount={pendingRequests.length}
      />
    </div>
  );
}
