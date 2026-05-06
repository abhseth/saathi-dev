import React from "react";
import type { AppUser, School, WeeklyTimetableSlot } from "../../types";
import { useModalFocus } from "./useModalFocus";

type CrossSchoolFacultyPanelProps = {
  schools: School[];
  users: AppUser[];
  onClose: () => void;
  onLoad: (facultyUserId: number, weekStart: string) => Promise<WeeklyTimetableSlot[]>;
};

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function CrossSchoolFacultyPanel({ schools, users, onClose, onLoad }: CrossSchoolFacultyPanelProps) {
  const [selectedFacultyId, setSelectedFacultyId] = React.useState<number>(0);
  const [weekStart, setWeekStart] = React.useState<string>(() => {
    const today = new Date();
    const day = today.getDay();
    const diff = today.getDate() - day + (day === 0 ? -6 : 1);
    const monday = new Date(today.setDate(diff));
    return monday.toISOString().split("T")[0];
  });
  const [slots, setSlots] = React.useState<WeeklyTimetableSlot[]>([]);
  const [loading, setLoading] = React.useState(false);
  const modalRef = useModalFocus(onClose);

  const facultyUsers = React.useMemo(() => users.filter((u) => u.role === "faculty").sort((a, b) => a.display_name.localeCompare(b.display_name)), [users]);

  React.useEffect(() => {
    if (selectedFacultyId && weekStart) {
      setLoading(true);
      onLoad(selectedFacultyId, weekStart)
        .then((data) => setSlots(data))
        .catch((e) => { console.error("Failed to load cross-school faculty slots:", e); setSlots([]); })
        .finally(() => setLoading(false));
    }
  }, [selectedFacultyId, weekStart, onLoad]);

  const periods = React.useMemo(() => {
    const maxPeriod = Math.max(0, ...slots.map((s) => s.period));
    return Array.from({ length: maxPeriod }, (_, i) => i + 1);
  }, [slots]);

  const dayMap = React.useMemo(() => {
    const map = new Map<string, WeeklyTimetableSlot[]>();
    for (const s of slots) {
      const key = `${s.day_of_week}-${s.period}`;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(s);
    }
    return map;
  }, [slots]);

  const conflicts = React.useMemo(() => {
    const conflictKeys = new Set<string>();
    for (const [key, list] of dayMap.entries()) {
      const uniqueSchools = new Set(list.map((s) => s.school_id));
      if (uniqueSchools.size > 1) {
        conflictKeys.add(key);
      }
    }
    return conflictKeys;
  }, [dayMap]);

  const totalPeriods = slots.length;
  const schoolsServed = new Set(slots.map((s) => s.school_id)).size;
  const maxPossible = periods.length * 7;
  const freePeriods = Math.max(0, maxPossible - totalPeriods);

  return (
    <div className="modal-backdrop" role="presentation" ref={modalRef} tabIndex={-1}>
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Cross-School Faculty View</h2>
            <p>One login-linked faculty member across all assigned schools</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>

        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            Linked Faculty
            <select value={selectedFacultyId} onChange={(e) => setSelectedFacultyId(Number(e.target.value))}>
              <option value={0}>Select linked faculty…</option>
              {facultyUsers.map((u) => (
                <option key={u.id} value={u.id}>{u.display_name}</option>
              ))}
            </select>
          </label>
          <label>
            Week Start
            <input
              type="date"
              value={weekStart}
              onChange={(e) => setWeekStart(e.target.value)}
            />
          </label>
        </div>
        <p className="read-only-notice" style={{ margin: "0 24px 16px" }}>This schedule view requires a linked faculty user account because it loads by faculty login ID.</p>

        {selectedFacultyId === 0 ? (
          <p className="empty-state">Select a faculty member to view their schedule.</p>
        ) : loading ? (
          <p className="empty-state">Loading…</p>
        ) : (
          <>
            <div style={{ margin: "0 24px 16px", display: "flex", gap: 16, flexWrap: "wrap" }}>
              <div style={{ padding: "10px 16px", background: "#f1f5f9", borderRadius: 6 }}>
                <strong>{totalPeriods}</strong> <span className="text-muted">periods</span>
              </div>
              <div style={{ padding: "10px 16px", background: "#f1f5f9", borderRadius: 6 }}>
                <strong>{schoolsServed}</strong> <span className="text-muted">schools</span>
              </div>
              <div style={{ padding: "10px 16px", background: "#f1f5f9", borderRadius: 6 }}>
                <strong>{freePeriods}</strong> <span className="text-muted">free periods</span>
              </div>
            </div>

            {slots.length === 0 ? (
              <p className="empty-state">No scheduled slots for this week.</p>
            ) : (
              <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
                <div style={{ overflowX: "auto" }}>
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th style={{ minWidth: 60 }}>Period</th>
                        {DAYS.map((d, i) => (
                          <th key={d} style={{ minWidth: 140 }}>{d}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {periods.map((period) => (
                        <tr key={period}>
                          <td><strong>P{period}</strong></td>
                          {DAYS.map((_, dayIdx) => {
                            const key = `${dayIdx}-${period}`;
                            const cellSlots = dayMap.get(key) ?? [];
                            const isConflict = conflicts.has(key);
                            return (
                              <td
                                key={dayIdx}
                                style={{
                                  verticalAlign: "top",
                                  background: isConflict ? "#fee2e2" : undefined,
                                }}
                              >
                                {cellSlots.map((s, idx) => (
                                  <div key={idx} style={{ marginBottom: 4, fontSize: 12 }}>
                                    <div><strong>{s.school_name}</strong></div>
                                    <div>{s.subject_name}</div>
                                    <div className="text-muted">{s.room || "—"}</div>
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
          </>
        )}
      </section>
    </div>
  );
}
