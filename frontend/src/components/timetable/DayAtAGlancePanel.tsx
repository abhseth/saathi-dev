import React from "react";
import type { AppUser, School, WeeklyTimetableSlot } from "../../types";
import { useModalFocus } from "./useModalFocus";

type DayAtAGlancePanelProps = {
  schools: School[];
  users: AppUser[];
  slots: WeeklyTimetableSlot[];
  onClose: () => void;
  onLoad: (schoolId: number, weekStart: string) => Promise<void>;
};

const DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];



export function DayAtAGlancePanel({ schools, users, slots, onClose, onLoad }: DayAtAGlancePanelProps) {
  const modalRef = useModalFocus(onClose);
  const [selectedSchoolId, setSelectedSchoolId] = React.useState<number>(schools[0]?.id ?? 0);
  const [filterGrade, setFilterGrade] = React.useState("");
  const [filterBatch, setFilterBatch] = React.useState("");
  const [filterFaculty, setFilterFaculty] = React.useState("");
  const [filterSubject, setFilterSubject] = React.useState("");
  const [filterRoom, setFilterRoom] = React.useState("");
  const [facultySearch, setFacultySearch] = React.useState("");

  React.useEffect(() => {
    if (selectedSchoolId) {
      const today = new Date();
      const day = today.getDay();
      const diff = today.getDate() - day + (day === 0 ? -6 : 1);
      const monday = new Date(today.setDate(diff));
      const weekStart = monday.toISOString().split("T")[0];
      void onLoad(selectedSchoolId, weekStart);
    }
  }, [selectedSchoolId, onLoad]);

  const todayDayIndex = new Date().getDay();
  const todayDayOfWeek = todayDayIndex === 0 ? 6 : todayDayIndex - 1;

  const todaySlots = React.useMemo(() => {
    return slots.filter((s) => s.school_id === selectedSchoolId && s.day_of_week === todayDayOfWeek);
  }, [slots, selectedSchoolId, todayDayOfWeek]);

  const grades = React.useMemo(() => Array.from(new Set(todaySlots.map((s) => s.grade_level))).sort(), [todaySlots]);
  const batches = React.useMemo(() => Array.from(new Set(todaySlots.map((s) => s.batch_pattern))).sort(), [todaySlots]);
  const subjects = React.useMemo(() => Array.from(new Set(todaySlots.map((s) => s.subject_name))).sort(), [todaySlots]);
  const rooms = React.useMemo(() => Array.from(new Set(todaySlots.map((s) => s.room ?? ""))).filter(Boolean).sort(), [todaySlots]);

  const facultyList = React.useMemo(() => {
    const q = facultySearch.trim().toLowerCase();
    const facultyUsers = users.filter((u) => u.role === "faculty");
    return q ? facultyUsers.filter((u) => u.display_name.toLowerCase().includes(q)).sort((a, b) => a.display_name.localeCompare(b.display_name)) : facultyUsers.sort((a, b) => a.display_name.localeCompare(b.display_name));
  }, [users, facultySearch]);

  const filteredSlots = React.useMemo(() => {
    return todaySlots.filter((s) => {
      if (filterGrade && s.grade_level !== filterGrade) return false;
      if (filterBatch && s.batch_pattern !== filterBatch) return false;
      if (filterFaculty && String(s.faculty_user_id) !== filterFaculty) return false;
      if (filterSubject && s.subject_name !== filterSubject) return false;
      if (filterRoom && s.room !== filterRoom) return false;
      return true;
    });
  }, [todaySlots, filterGrade, filterBatch, filterFaculty, filterSubject, filterRoom]);

  const periods = React.useMemo(() => {
    return Array.from(new Set(filteredSlots.map((s) => s.period))).sort((a, b) => a - b);
  }, [filteredSlots]);

  const columns = React.useMemo(() => {
    const combos = Array.from(new Set(filteredSlots.map((s) => `${s.grade_level}|${s.batch_pattern}`)));
    return combos.sort().map((c) => {
      const [grade, batch] = c.split("|");
      return { grade, batch, key: c };
    });
  }, [filteredSlots]);

  const cellMap = React.useMemo(() => {
    const map = new Map<string, WeeklyTimetableSlot[]>();
    for (const s of filteredSlots) {
      const key = `${s.period}-${s.grade_level}|${s.batch_pattern}`;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(s);
    }
    return map;
  }, [filteredSlots]);

  return (
    <div className="modal-backdrop" role="presentation" ref={modalRef} tabIndex={-1}>
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Day at a Glance</h2>
            <p>{DAYS[todayDayOfWeek]} — {schools.find((s) => s.id === selectedSchoolId)?.name ?? "Select school"}</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>

        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            School
            <select value={selectedSchoolId} onChange={(e) => setSelectedSchoolId(Number(e.target.value))}>
              {schools.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </label>
          <label>
            Grade
            <select value={filterGrade} onChange={(e) => setFilterGrade(e.target.value)}>
              <option value="">All</option>
              {grades.map((g) => (
                <option key={g} value={g}>{g}</option>
              ))}
            </select>
          </label>
          <label>
            Batch
            <select value={filterBatch} onChange={(e) => setFilterBatch(e.target.value)}>
              <option value="">All</option>
              {batches.map((b) => (
                <option key={b} value={b}>{b}</option>
              ))}
            </select>
          </label>
          <label>
            Subject
            <select value={filterSubject} onChange={(e) => setFilterSubject(e.target.value)}>
              <option value="">All</option>
              {subjects.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
          </label>
          <label>
            Room
            <select value={filterRoom} onChange={(e) => setFilterRoom(e.target.value)}>
              <option value="">All</option>
              {rooms.map((r) => (
                <option key={r} value={r}>{r}</option>
              ))}
            </select>
          </label>
        </div>

        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            Search Linked Faculty
            <input
              type="search"
              placeholder="Type linked faculty name..."
              value={facultySearch}
              onChange={(e) => setFacultySearch(e.target.value)}
            />
          </label>
          <label>
            Filter by Linked Faculty
            <select value={filterFaculty} onChange={(e) => setFilterFaculty(e.target.value)}>
              <option value="">All</option>
              {facultyList.map((u) => (
                <option key={u.id} value={u.id}>{u.display_name}</option>
              ))}
            </select>
          </label>
        </div>
        <p className="read-only-notice" style={{ margin: "0 24px 16px" }}>This operational view filters by login-linked faculty only. No-login faculty can be planned in Faculty Assignments but do not appear in attendance/session views.</p>

        {periods.length === 0 || columns.length === 0 ? (
          <p className="empty-state">No slots match the selected filters.</p>
        ) : (
          <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
            <div style={{ overflowX: "auto" }}>
              <table className="data-table">
                <thead>
                  <tr>
                    <th style={{ minWidth: 80 }}>Period</th>
                    {columns.map((col) => (
                      <th key={col.key} style={{ minWidth: 140 }}>
                        {col.grade}
                        <br />
                        <small>{col.batch}</small>
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {periods.map((period) => (
                    <tr key={period}>
                      <td><strong>P{period}</strong></td>
                      {columns.map((col) => {
                        const key = `${period}-${col.key}`;
                        const cellSlots = cellMap.get(key) ?? [];
                        return (
                          <td key={col.key + period} style={{ verticalAlign: "top" }}>
                            {cellSlots.map((s, idx) => (
                              <div key={idx} style={{ marginBottom: 4, fontSize: 12 }}>
                                <div><strong>{s.subject_name}</strong></div>
                                <div className="text-muted">{s.faculty_display_name ?? "—"}</div>
                                <div className="text-muted">{s.room ?? "—"}</div>
                                {(s.is_substitution ?? false) ? (
                                  <span className="sub-badge" title="Substitution" aria-label="Substitution">SUB</span>
                                ) : null}
                              </div>
                            ))}
                            {cellSlots.length === 0 && <span className="text-muted">—</span>}
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}
