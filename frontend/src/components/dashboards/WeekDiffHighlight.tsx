import React from "react";
import type { WeekDiffSlot } from "../../types";

type Props = {
  slots: WeekDiffSlot[];
  weekA: string;
  weekB: string;
  onClose: () => void;
  onLoad: (schoolId: number, weekA: string, weekB: string) => Promise<void>;
};

export function WeekDiffHighlight({ slots, weekA, weekB, onClose, onLoad }: Props) {
  const [schoolId, setSchoolId] = React.useState<number | "">("");
  const [wA, setWA] = React.useState(weekA);
  const [wB, setWB] = React.useState(weekB);

  const handleLoad = React.useCallback(() => {
    if (typeof schoolId === "number" && wA && wB) {
      void onLoad(schoolId, wA, wB);
    }
  }, [schoolId, wA, wB, onLoad]);

  const changeColor = (type: string) => {
    if (type === "added") return { background: "#dcfce7", color: "#166534" };
    if (type === "removed") return { background: "#fee2e2", color: "#991b1b" };
    return { background: "#fef9c3", color: "#854d0e" };
  };

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Week-over-Week Diff</h2>
            <p>Highlight changed slots vs last week</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            School ID
            <input type="number" value={schoolId} onChange={(e) => setSchoolId(e.target.value === "" ? "" : Number(e.target.value))} />
          </label>
          <label>
            Week A
            <input type="date" value={wA} onChange={(e) => setWA(e.target.value)} />
          </label>
          <label>
            Week B
            <input type="date" value={wB} onChange={(e) => setWB(e.target.value)} />
          </label>
          <button type="button" className="primary-action" onClick={handleLoad}>Compare</button>
        </div>
        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Change</th>
                <th>Grade</th>
                <th>Day</th>
                <th>Period</th>
                <th>Subject</th>
                <th>Faculty</th>
                <th>Room</th>
                <th>Type</th>
              </tr>
            </thead>
            <tbody>
              {slots.map((s) => (
                <tr key={s.id} style={changeColor(s.change_type)}>
                  <td style={{ fontWeight: 700, textTransform: "capitalize" }}>{s.change_type}</td>
                  <td>{s.grade_level}</td>
                  <td>{["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][s.day_of_week]}</td>
                  <td>{s.period}</td>
                  <td>{s.subject_name}</td>
                  <td>{s.faculty_display_name || "—"}</td>
                  <td>{s.room}</td>
                  <td>{s.session_type}</td>
                </tr>
              ))}
              {slots.length === 0 && <tr><td colSpan={8} className="empty-state">No differences.</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
