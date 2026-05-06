import React from "react";
import { api } from "../../api";
import type { School, CloneWeekResult } from "../../types";

export function WeekClonePanel({ schools, onClose }: { schools: School[]; onClose: () => void }) {
  const [schoolId, setSchoolId] = React.useState<number>(0);
  const [fromWeek, setFromWeek] = React.useState(() => {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    return new Date(d.getFullYear(), d.getMonth(), diff).toISOString().slice(0, 10);
  });
  const [toWeek, setToWeek] = React.useState(() => {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1) + 7;
    return new Date(d.getFullYear(), d.getMonth(), diff).toISOString().slice(0, 10);
  });
  const [result, setResult] = React.useState<CloneWeekResult | null>(null);
  const [busy, setBusy] = React.useState(false);

  async function handleClone() {
    if (!schoolId || !fromWeek || !toWeek) return;
    setBusy(true);
    try {
      const res = await api<CloneWeekResult>("clone_week_with_check", { input: { from_week: fromWeek, to_week: toWeek, school_id: schoolId } });
      setResult(res);
    } finally { setBusy(false); }
  }

  return (
    <section className="ticket-modal" aria-label="Clone week">
      <header>
        <div><h2>Clone Week with Conflict Check</h2><p>Auto-validate faculty overlaps and room double-bookings</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div className="form-stack">
        <label>School
          <select value={schoolId} onChange={(e) => setSchoolId(Number(e.target.value))}>
            <option value={0}>Select…</option>
            {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </label>
        <label>Source Week<input type="date" value={fromWeek} onChange={(e) => setFromWeek(e.target.value)} /></label>
        <label>Target Week<input type="date" value={toWeek} onChange={(e) => setToWeek(e.target.value)} /></label>
        <button type="button" className="primary-action" disabled={busy} onClick={handleClone}>Clone & Check</button>
      </div>
      {result && (
        <div style={{ marginTop: 12, padding: 12, background: result.conflicts.length === 0 ? "#ecfdf5" : "#fef2f2", borderRadius: 8 }}>
          <p>Cloned <strong>{result.cloned_slots}</strong> slots.</p>
          {result.conflicts.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <strong>Conflicts detected:</strong>
              <ul>{result.conflicts.map((c, i) => <li key={i}>{c}</li>)}</ul>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
