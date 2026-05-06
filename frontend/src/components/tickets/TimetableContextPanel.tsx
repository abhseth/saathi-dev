import React from "react";
import type { Ticket, School, WeeklyTimetableSlot, LectureSession } from "../../types";

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

const SCHEDULE_KEYWORDS = [
  "timetable", "schedule", "class", "period", "faculty", "teacher", "absent", "substitution",
  "कक्षा", "अनुपस्थित", "शिक्षक", "समय सारणी", // Hindi
  "کلاس", "استاد", "غیرحاضر", // Urdu
  "வகுப்பு", "ஆசிரியர்", // Tamil
];

function isScheduleRelated(ticket: Ticket): boolean {
  const text = `${ticket.title} ${ticket.description}`.toLowerCase();
  const hasKeyword = SCHEDULE_KEYWORDS.some((kw) => text.includes(kw));
  const hasSchool = ticket.school_id !== null;
  const hasGrade = Boolean(ticket.grade_level && ticket.grade_level !== "");
  return hasKeyword && hasSchool && hasGrade;
}

type TimetableContextPanelProps = {
  ticket: Ticket;
  schools: School[];
  slots: WeeklyTimetableSlot[];
  sessions: LectureSession[];
  onViewFullTimetable: (schoolId: number) => void;
};

export function TimetableContextPanel({
  ticket,
  schools,
  slots,
  sessions,
  onViewFullTimetable,
}: TimetableContextPanelProps) {
  if (!isScheduleRelated(ticket)) return null;

  const school = schools.find((s) => s.id === ticket.school_id);
  if (!school) return null;

  const today = new Date();
  const past7 = new Date(today);
  past7.setDate(today.getDate() - 7);
  const future7 = new Date(today);
  future7.setDate(today.getDate() + 7);

  const relevantSlots = slots.filter(
    (s) =>
      s.school_id === ticket.school_id &&
      s.grade_level === ticket.grade_level &&
      (ticket.program_track ? s.track === ticket.program_track : true),
  );

  const relevantSessions = sessions.filter(
    (s) =>
      s.school_id === ticket.school_id &&
      s.grade_level === ticket.grade_level &&
      s.session_date >= formatDate(past7) &&
      s.session_date <= formatDate(future7),
  );

  // Faculty overload check
  const facultyCounts = new Map<number, number>();
  for (const s of relevantSlots) {
    if (s.faculty_user_id) {
      facultyCounts.set(s.faculty_user_id, (facultyCounts.get(s.faculty_user_id) || 0) + 1);
    }
  }
  const overloadedFaculty = Array.from(facultyCounts.entries()).filter(([, count]) => count > 24);

  // Gaps check
  const gaps = relevantSlots.filter((s) => !s.faculty_user_id);
  const cancelledWithoutSub = relevantSessions.filter(
    (s) => s.status === "Cancelled" && s.actual_faculty_user_id === null,
  );

  return (
    <div className="timetable-context-panel">
      <div className="timetable-context-header">
        <h4><span aria-hidden="true">📅</span> Timetable Context</h4>
        <button
          type="button"
          className="secondary-button"
          onClick={() => onViewFullTimetable(ticket.school_id!)}
        >
          View Full Timetable
        </button>
      </div>

      <div className="timetable-context-stats">
        <span>{relevantSlots.length} slots</span>
        <span>{relevantSessions.length} sessions (last 7d + next 7d)</span>
        {gaps.length > 0 && (
          <span className="alert-stat critical">{gaps.length} gaps</span>
        )}
        {cancelledWithoutSub.length > 0 && (
          <span className="alert-stat critical">{cancelledWithoutSub.length} cancelled without sub</span>
        )}
        {overloadedFaculty.length > 0 && (
          <span className="alert-stat warning">{overloadedFaculty.length} faculty overloaded</span>
        )}
      </div>

      {gaps.length > 0 && (
        <details className="timetable-context-section">
          <summary>Missing faculty assignments ({gaps.length})</summary>
          <ul className="gap-list">
            {gaps.slice(0, 10).map((g, i) => (
              <li key={i}>
                {DAY_NAMES[g.day_of_week]} Period {g.period} · {g.subject_name}
                {g.track ? ` (${g.track})` : ""} · {g.batch_pattern}
              </li>
            ))}
            {gaps.length > 10 && <li>… and {gaps.length - 10} more</li>}
          </ul>
        </details>
      )}

      {cancelledWithoutSub.length > 0 && (
        <details className="timetable-context-section">
          <summary>Cancelled without substitute ({cancelledWithoutSub.length})</summary>
          <ul className="gap-list">
            {cancelledWithoutSub.slice(0, 10).map((s, i) => (
              <li key={i}>
                {s.session_date} · {s.start_time ?? "--:--"} · {s.grade_level} {s.track ?? ""}
              </li>
            ))}
            {cancelledWithoutSub.length > 10 && <li>… and {cancelledWithoutSub.length - 10} more</li>}
          </ul>
        </details>
      )}

      {overloadedFaculty.length > 0 && (
        <details className="timetable-context-section">
          <summary>Faculty overload (&gt;24 periods)</summary>
          <ul className="gap-list">
            {overloadedFaculty.map(([id, count]) => {
              const name = slots.find((s) => s.faculty_user_id === id)?.faculty_display_name ?? `Faculty ${id}`;
              return (
                <li key={id}>
                  {name} — {count} periods
                </li>
              );
            })}
          </ul>
        </details>
      )}

      {relevantSessions.length > 0 && (
        <details className="timetable-context-section">
          <summary>Recent & upcoming sessions</summary>
          <ul className="gap-list">
            {relevantSessions.slice(0, 14).map((s) => (
              <li key={s.id} className={s.status === "Cancelled" ? "cancelled" : ""}>
                {s.session_date} · {s.start_time ?? "--:--"} · {s.status}
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}

function formatDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}
