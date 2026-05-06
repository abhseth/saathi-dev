import React from "react";
import { api } from "../../api";
import type { FacultyTodaySession, AttendanceRecord, BulkAttendanceInput } from "../../types";

export function BulkQuickAttendancePanel({
  faculty,
  schools,
}: {
  faculty: Array<{ id: number; display_name: string }>;
  schools: Array<{ id: number; name: string }>;
}) {
  const [selectedFacultyIds, setSelectedFacultyIds] = React.useState<number[]>([]);
  const [date, setDate] = React.useState(new Date().toISOString().split("T")[0]);
  const [reason, setReason] = React.useState("");
  const [notice, setNotice] = React.useState("");
  const [quickSessionId, setQuickSessionId] = React.useState(0);
  const [students, setStudents] = React.useState<AttendanceRecord[]>([]);
  const [quickStatus, setQuickStatus] = React.useState<"Present" | "Absent">("Present");

  async function handleBulkAbsent() {
    if (selectedFacultyIds.length === 0) {
      setNotice("Select at least one faculty member.");
      return;
    }
    try {
      const input: BulkAttendanceInput = {
        faculty_user_ids: selectedFacultyIds,
        date,
        reason,
      };
      await api<number[]>("bulk_attendance", { input });
      setNotice(`Marked ${selectedFacultyIds.length} faculty absent and generated tickets.`);
      setSelectedFacultyIds([]);
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function loadSessionStudents(sessionId: number) {
    if (!sessionId) return;
    try {
      const records = await api<AttendanceRecord[]>("faculty_session_attendance", { sessionId });
      setStudents(records);
    } catch (e) {
      console.error("Failed to load session students:", e);
      setStudents([]);
    }
  }

  async function handleQuickMark(studentId: number) {
    if (!quickSessionId) return;
    try {
      await api("mark_attendance_quick", {
        input: { session_id: quickSessionId, student_id: studentId, status: quickStatus },
      });
      setNotice("Attendance marked.");
      void loadSessionStudents(quickSessionId);
    } catch (e) {
      setNotice(String(e));
    }
  }

  return (
    <section className="ticket-modal" aria-label="Bulk & Quick Attendance">
      <header>
        <div>
          <h2>Bulk & Quick Attendance</h2>
          <p>Mark linked faculty absent or tap attendance per student</p>
        </div>
      </header>

      {notice ? <div className="notice-banner">{notice}</div> : null}
      <p className="read-only-notice">Attendance workflows require faculty login accounts. No-login faculty are available for planning but cannot be marked absent here.</p>

      <div className="attendance-panels">
        <div className="attendance-card">
          <h3>Bulk Absence + Auto-Ticket</h3>
          <label>
            Date
            <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
          </label>
          <label>
            Reason
            <input value={reason} onChange={(e) => setReason(e.target.value)} placeholder="e.g. Sick leave" />
          </label>
          <div className="bulk-faculty-list">
            <strong>Select linked faculty</strong>
            {faculty.map((f) => (
              <label key={f.id} className="bulk-faculty-row">
                <input
                  type="checkbox"
                  checked={selectedFacultyIds.includes(f.id)}
                  onChange={() => {
                    setSelectedFacultyIds((prev) =>
                      prev.includes(f.id) ? prev.filter((id) => id !== f.id) : [...prev, f.id]
                    );
                  }}
                />
                <span>{f.display_name}</span>
              </label>
            ))}
          </div>
          <button type="button" className="primary-action" onClick={handleBulkAbsent}>
            Mark Absent & Generate Tickets
          </button>
        </div>

        <div className="attendance-card">
          <h3>One-Tap Attendance</h3>
          <label>
            Session ID
            <input
              type="number"
              value={quickSessionId || ""}
              onChange={(e) => {
                const id = Number(e.target.value);
                setQuickSessionId(id);
                void loadSessionStudents(id);
              }}
            />
          </label>
          <label>
            Default status
            <select value={quickStatus} onChange={(e) => setQuickStatus(e.target.value as "Present" | "Absent")}>
              <option value="Present">Present</option>
              <option value="Absent">Absent</option>
            </select>
          </label>

          <div className="quick-attendance-list">
            {students.length === 0 ? (
              <p className="empty-state compact">Enter a session ID to load roster.</p>
            ) : (
              students.map((s) => (
                <div key={s.id} className="quick-attendance-row">
                  <span>{s.student_name}</span>
                  <span className={`status-dot ${s.status.toLowerCase()}`}>{s.status}</span>
                  <button
                    type="button"
                    className="secondary-button small"
                    onClick={() => handleQuickMark(s.student_id)}
                  >
                    Mark {quickStatus}
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
