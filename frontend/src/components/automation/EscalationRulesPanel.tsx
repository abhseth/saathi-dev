import React from "react";
import { api } from "../../api";
import type { EscalationRule } from "../../types";

export function EscalationRulesPanel({ onClose }: { onClose: () => void }) {
  const [rules, setRules] = React.useState<EscalationRule[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [editing, setEditing] = React.useState<EscalationRule | null>(null);

  const loadRules = React.useCallback(async () => {
    setLoading(true);
    try {
      const data = await api<EscalationRule[]>("list_escalation_rules");
      setRules(data);
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    void loadRules();
  }, [loadRules]);

  async function saveRule() {
    if (!editing) return;
    if (editing.id) {
      await api<EscalationRule>("update_escalation_rule", { id: editing.id, input: editing });
    } else {
      await api<EscalationRule>("create_escalation_rule", { input: editing });
    }
    setEditing(null);
    void loadRules();
  }

  return (
    <section className="ticket-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <header>
        <div><h2 id="modal-title">Escalation Rules</h2><p>Smart escalation: queue + priority → assignee after N hours</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 12 }}>
        <button type="button" className="primary-action" onClick={() => setEditing({ id: 0, name: "", conditions_json: "{}", action: "escalate", assignee_role: "", hours_threshold: 24, is_active: true, created_at: "", updated_at: "" })}>
          + Add Rule
        </button>
      </div>
      {loading ? <p>Loading…</p> : (
        <table className="data-table">
          <thead><tr><th>Name</th><th>Conditions</th><th>Action</th><th>Assignee</th><th>Hours</th><th>Active</th><th /></tr></thead>
          <tbody>
            {rules.map((r) => (
              <tr key={r.id}>
                <td>{r.name}</td>
                <td><code>{r.conditions_json}</code></td>
                <td>{r.action}</td>
                <td>{r.assignee_role}</td>
                <td>{r.hours_threshold}</td>
                <td>{r.is_active ? "Yes" : "No"}</td>
                <td><button type="button" onClick={() => setEditing(r)}>Edit</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {editing && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>{editing.id ? "Edit Rule" : "New Rule"}</h3>
            <div className="form-stack">
              <label>Name<input value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })} /></label>
              <label>Conditions JSON<textarea rows={3} value={editing.conditions_json} onChange={(e) => setEditing({ ...editing, conditions_json: e.target.value })} /></label>
              <label>Action<input value={editing.action} onChange={(e) => setEditing({ ...editing, action: e.target.value })} /></label>
              <label>Assignee Role<input value={editing.assignee_role} onChange={(e) => setEditing({ ...editing, assignee_role: e.target.value })} /></label>
              <label>Hours Threshold<input type="number" value={editing.hours_threshold} onChange={(e) => setEditing({ ...editing, hours_threshold: Number(e.target.value) })} /></label>
              <label><input type="checkbox" checked={editing.is_active} onChange={(e) => setEditing({ ...editing, is_active: e.target.checked })} /> Active</label>
            </div>
            <div className="modal-actions">
              <button type="button" className="primary-action" onClick={saveRule}>Save</button>
              <button type="button" onClick={() => setEditing(null)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
