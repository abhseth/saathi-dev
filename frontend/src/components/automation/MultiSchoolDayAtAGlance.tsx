import React from "react";
import { api } from "../../api";
import type { FacultyWeeklySlot, School } from "../../types";

export function MultiSchoolDayAtAGlance({ schools, onClose }: { schools: School[]; onClose: () => void }) {
  const [slots, setSlots] = React.useState<FacultyWeeklySlot[]>([]);
  const [date, setDate] = React.useState(() => new Date().toISOString().slice(0, 10));
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      // Build week_start from date
      const d = new Date(date);
      const day = d.getDay();
      const diff = d.getDate() - day + (day === 0 ? -6 : 1);
      const mon = new Date(d.getFullYear(), d.getMonth(), diff);
      const weekStart = mon.toISOString().slice(0, 10);
      const dow = (d.getDay() + 6) % 7; // Monday=0
      // Fetch weekly timetable for all schools and filter by day
      const all = await api<FacultyWeeklySlot[]>("list_weekly_timetable", { schoolId: 0, weekStart });
      setSlots(all.filter((s) => s.day_of_week === dow));
    } finally {
      setLoading(false);
    }
  }, [date]);

  React.useEffect(() => {
    void load();
  }, [load]);

  const bySchool = React.useMemo(() => {
    const map: Record<string, FacultyWeeklySlot[]> = {};
    for (const s of slots) {
      const key = s.school_name;
      if (!map[key]) map[key] = [];
      map[key].push(s);
    }
    return map;
  }, [slots]);

  return (
    <section className="ticket-modal" aria-label="Multi-school day at a glance">
      <header>
        <div><h2>Multi-School Day at a Glance</h2><p>Consolidated operations view</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <label>Date <input type="date" value={date} onChange={(e) => setDate(e.target.value)} /></label>
      {loading ? <p>Loading…</p> : (
        <div style={{ display: "flex", flexDirection: "column", gap: 16, marginTop: 12 }}>
          {Object.entries(bySchool).map(([schoolName, list]) => (
            <div key={schoolName} style={{ border: "1px solid #e5e7eb", borderRadius: 8, padding: 12 }}>
              <h4 style={{ marginBottom: 8 }}>{schoolName}</h4>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 8 }}>
                {list.sort((a, b) => a.period - b.period).map((s, i) => (
                  <div key={i} style={{ background: "#f9fafb", padding: 8, borderRadius: 6 }}>
                    <div><strong>P{s.period}</strong> {s.subject_name}</div>
                    <div style={{ fontSize: 12, color: "#6b7280" }}>{s.grade_level} {s.track} — {s.faculty_name ?? "Unassigned"}</div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
