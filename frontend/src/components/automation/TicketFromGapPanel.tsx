import React from "react";
import { api } from "../../api";
import type { School, Ticket } from "../../types";

export function TicketFromGapPanel({ schools, onClose }: { schools: School[]; onClose: () => void }) {
  const [schoolId, setSchoolId] = React.useState<number>(0);
  const [gradeLevel, setGradeLevel] = React.useState("");
  const [track, setTrack] = React.useState("");
  const [subjectName, setSubjectName] = React.useState("");
  const [gapDescription, setGapDescription] = React.useState("");
  const [ticket, setTicket] = React.useState<Ticket | null>(null);
  const [busy, setBusy] = React.useState(false);

  async function handleCreate() {
    if (!schoolId || !gradeLevel || !subjectName) return;
    setBusy(true);
    try {
      const res = await api<Ticket>("ticket_from_gap", { input: { school_id: schoolId, grade_level: gradeLevel, track, subject_name: subjectName, gap_description: gapDescription } });
      setTicket(res);
    } finally { setBusy(false); }
  }

  return (
    <section className="ticket-modal" aria-label="Raise ticket from gap">
      <header>
        <div><h2>Raise Ticket from Gap</h2><p>One-click ticket creation pre-filled with context</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div className="form-stack">
        <label>School
          <select value={schoolId} onChange={(e) => setSchoolId(Number(e.target.value))}>
            <option value={0}>Select…</option>
            {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </label>
        <label>Grade Level<input value={gradeLevel} onChange={(e) => setGradeLevel(e.target.value)} placeholder="e.g. Grade 10" /></label>
        <label>Track<input value={track} onChange={(e) => setTrack(e.target.value)} placeholder="e.g. JEE Foundation" /></label>
        <label>Subject<input value={subjectName} onChange={(e) => setSubjectName(e.target.value)} placeholder="e.g. Physics" /></label>
        <label>Gap Description<textarea rows={4} value={gapDescription} onChange={(e) => setGapDescription(e.target.value)} placeholder="Describe the compliance gap…" /></label>
        <button type="button" className="primary-action" disabled={busy} onClick={handleCreate}>Raise Ticket</button>
      </div>
      {ticket && (
        <div style={{ marginTop: 12, padding: 12, background: "#ecfdf5", borderRadius: 8 }}>
          <p>Ticket <strong>#{ticket.id}</strong> created: {ticket.title}</p>
        </div>
      )}
    </section>
  );
}
