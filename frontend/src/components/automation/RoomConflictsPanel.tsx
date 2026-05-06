import React from "react";
import { api } from "../../api";
import type { CrossSchoolRoomConflict } from "../../types";

export function RoomConflictsPanel({ onClose }: { onClose: () => void }) {
  const [conflicts, setConflicts] = React.useState<CrossSchoolRoomConflict[]>([]);
  const [weekStart, setWeekStart] = React.useState(() => {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    return new Date(d.getFullYear(), d.getMonth(), diff).toISOString().slice(0, 10);
  });
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      const data = await api<CrossSchoolRoomConflict[]>("cross_school_room_conflicts", { weekStart });
      setConflicts(data);
    } finally { setLoading(false); }
  }, [weekStart]);

  React.useEffect(() => {
    void load();
  }, [load]);

  return (
    <section className="ticket-modal" aria-label="Cross-school room conflicts">
      <header>
        <div><h2>Room Conflicts (Cross-School)</h2><p>Double-booked shared facilities</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <label>Week Start <input type="date" value={weekStart} onChange={(e) => setWeekStart(e.target.value)} /></label>
      {loading ? <p>Loading…</p> : conflicts.length === 0 ? <p>No conflicts found.</p> : (
        <div style={{ display: "flex", flexDirection: "column", gap: 12, marginTop: 12 }}>
          {conflicts.map((c, idx) => (
            <div key={idx} style={{ border: "1px solid #ef4444", background: "#fef2f2", padding: 12, borderRadius: 8 }}>
              <strong>Room {c.room}</strong> — Day {c.day_of_week + 1}, Period {c.period}
              <ul style={{ marginTop: 6 }}>
                {c.slots.map((s, i) => (
                  <li key={i}>{s.school_name}: {s.grade_level} {s.track} — {s.subject_name} ({s.faculty_name})</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
