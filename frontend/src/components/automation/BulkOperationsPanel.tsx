import React from "react";
import { api } from "../../api";
import type { School, AppUser, BulkOperationLog } from "../../types";

export function BulkOperationsPanel({ schools, users, onClose }: { schools: School[]; users: AppUser[]; onClose: () => void }) {
  const [tab, setTab] = React.useState<"assign" | "subjects" | "publish">("assign");
  const [selectedUsers, setSelectedUsers] = React.useState<number[]>([]);
  const [selectedSchools, setSelectedSchools] = React.useState<number[]>([]);
  const [role, setRole] = React.useState("faculty");
  const [csvText, setCsvText] = React.useState("");
  const [targetSchoolId, setTargetSchoolId] = React.useState<number>(0);
  const [weekStart, setWeekStart] = React.useState(() => {
    const d = new Date();
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    const mon = new Date(d.getFullYear(), d.getMonth(), diff);
    return mon.toISOString().slice(0, 10);
  });
  const [result, setResult] = React.useState<BulkOperationLog | null>(null);
  const [busy, setBusy] = React.useState(false);

  async function handleAssign() {
    if (selectedUsers.length === 0 || selectedSchools.length === 0) return;
    setBusy(true);
    try {
      const res = await api<BulkOperationLog>("bulk_assign_users", { input: { user_ids: selectedUsers, school_ids: selectedSchools, role } });
      setResult(res);
    } finally { setBusy(false); }
  }

  async function handleImportSubjects() {
    if (!targetSchoolId || !csvText.trim()) return;
    setBusy(true);
    try {
      const res = await api<BulkOperationLog>("bulk_import_subjects", { input: { school_id: targetSchoolId, csv_data: csvText } });
      setResult(res);
    } finally { setBusy(false); }
  }

  async function handlePublish() {
    if (selectedSchools.length === 0 || !weekStart) return;
    setBusy(true);
    try {
      const res = await api<BulkOperationLog>("bulk_publish_timetables", { input: { school_ids: selectedSchools, week_start_date: weekStart, region_id: null } });
      setResult(res);
    } finally { setBusy(false); }
  }

  return (
    <section className="ticket-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <header>
        <div><h2 id="modal-title">Bulk Operations</h2><p>Assign users, import subjects, publish timetables</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div className="tab-bar" style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <button className={tab === "assign" ? "active" : ""} onClick={() => setTab("assign")}>Assign Users</button>
        <button className={tab === "subjects" ? "active" : ""} onClick={() => setTab("subjects")}>Import Subjects</button>
        <button className={tab === "publish" ? "active" : ""} onClick={() => setTab("publish")}>Publish Timetables</button>
      </div>
      {tab === "assign" && (
        <div className="form-stack">
          <label>Role
            <select value={role} onChange={(e) => setRole(e.target.value)}>
              <option value="aom">AOM</option>
              <option value="faculty">Faculty</option>
            </select>
          </label>
          <label>Users (multi-select)
            <select multiple value={selectedUsers.map(String)} onChange={(e) => setSelectedUsers(Array.from(e.target.selectedOptions).map((o) => Number(o.value)))} style={{ minHeight: 120 }}>
              {users.filter((u) => u.role === role || role === "aom").map((u) => <option key={u.id} value={u.id}>{u.display_name} ({u.username})</option>)}
            </select>
          </label>
          <label>Schools (multi-select)
            <select multiple value={selectedSchools.map(String)} onChange={(e) => setSelectedSchools(Array.from(e.target.selectedOptions).map((o) => Number(o.value)))} style={{ minHeight: 120 }}>
              {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          </label>
          <button type="button" className="primary-action" disabled={busy} onClick={handleAssign}>Assign</button>
        </div>
      )}
      {tab === "subjects" && (
        <div className="form-stack">
          <label>School
            <select value={targetSchoolId} onChange={(e) => setTargetSchoolId(Number(e.target.value))}>
              <option value={0}>Select…</option>
              {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          </label>
          <label>CSV (name,track per line)<textarea rows={6} value={csvText} onChange={(e) => setCsvText(e.target.value)} placeholder="Physics,Foundation&#10;Chemistry,Foundation" /></label>
          <button type="button" className="primary-action" disabled={busy} onClick={handleImportSubjects}>Import</button>
        </div>
      )}
      {tab === "publish" && (
        <div className="form-stack">
          <label>Week Start<input type="date" value={weekStart} onChange={(e) => setWeekStart(e.target.value)} /></label>
          <label>Schools (multi-select)
            <select multiple value={selectedSchools.map(String)} onChange={(e) => setSelectedSchools(Array.from(e.target.selectedOptions).map((o) => Number(o.value)))} style={{ minHeight: 120 }}>
              {schools.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          </label>
          <button type="button" className="primary-action" disabled={busy} onClick={handlePublish}>Publish</button>
        </div>
      )}
      {result && (
        <div className="result-card" style={{ marginTop: 12, padding: 12, background: "#f6f7f9", borderRadius: 8 }}>
          <strong>Result:</strong> {result.op_type} — {result.status}
          <pre style={{ marginTop: 8, fontSize: 12 }}>{result.result_json}</pre>
        </div>
      )}
    </section>
  );
}
