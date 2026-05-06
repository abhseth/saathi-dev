import React from "react";
import { api } from "../../api";
import type { AppUser, School, ReassignFacultyResult } from "../../types";

export function ReassignWizard({ users, schools, onClose }: { users: AppUser[]; schools: School[]; onClose: () => void }) {
  const [facultyId, setFacultyId] = React.useState<number>(0);
  const [sourceId, setSourceId] = React.useState<number>(0);
  const [targetId, setTargetId] = React.useState<number>(0);
  const [weekStart, setWeekStart] = React.useState(() => {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    return new Date(d.getFullYear(), d.getMonth(), diff).toISOString().slice(0, 10);
  });
  const [result, setResult] = React.useState<ReassignFacultyResult | null>(null);
  const [busy, setBusy] = React.useState(false);

  async function handleSubmit() {
    if (!facultyId || !sourceId || !targetId || !weekStart) return;
    setBusy(true);
    try {
      const res = await api<ReassignFacultyResult>("reassign_faculty", { input: { faculty_user_id: facultyId, source_school_id: sourceId, target_school_id: targetId, effective_week_start: weekStart } });
      setResult(res);
    } finally { setBusy(false); }
  }

  return (
    <section className="ticket-modal" aria-label="Reassign faculty wizard">
      <header>
        <div><h2>Reassign Faculty</h2><p>Clone template & weekly slots for linked faculty accounts with conflict warnings</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <p className="read-only-notice">Reassignment currently requires a linked faculty login because timetable slots still store faculty user IDs.</p>
      <div className="form-stack">
        <label>Linked Faculty
          <select value={facultyId} onChange={(e) => setFacultyId(Number(e.target.value))}>
            <option value={0}>Select…</option>
            {users.filter((u) => u.role === "faculty").map((u) => <option key={u.id} value={u.id}>{u.display_name}</option>)}
          </select>
        </label>
        <label>Source School
          <select value={sourceId} onChange={(e) => setSourceId(Number(e.target.value))}>
            <option value={0}>Select…</option>
            {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </label>
        <label>Target School
          <select value={targetId} onChange={(e) => setTargetId(Number(e.target.value))}>
            <option value={0}>Select…</option>
            {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </label>
        <label>Effective Week Start<input type="date" value={weekStart} onChange={(e) => setWeekStart(e.target.value)} /></label>
        <button type="button" className="primary-action" disabled={busy} onClick={handleSubmit}>Reassign</button>
      </div>
      {result && (
        <div style={{ marginTop: 12, padding: 12, background: "#f6f7f9", borderRadius: 8 }}>
          <p>Cloned <strong>{result.cloned_slots}</strong> slots.</p>
          {result.conflicts.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <strong>Conflicts:</strong>
              <ul>{result.conflicts.map((c, i) => <li key={i}>{c}</li>)}</ul>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
