import React from "react";
import { api } from "../../api";
import type { Alert } from "../../types";

export function AlertInboxPanel({ onClose }: { onClose: () => void }) {
  const [alerts, setAlerts] = React.useState<Alert[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [selected, setSelected] = React.useState<Set<string>>(new Set());

  const loadAlerts = React.useCallback(async () => {
    setLoading(true);
    try {
      const data = await api<Alert[]>("alert_inbox", {});
      setAlerts(data);
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    void loadAlerts();
  }, [loadAlerts]);

  function toggleSelect(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function bulkAction(action: string) {
    if (selected.size === 0) return;
    await api("bulk_alert_action", { input: { ids: Array.from(selected), action, snooze_hours: 24 } });
    setSelected(new Set());
    void loadAlerts();
  }

  async function dismissOne(id: string) {
    await api("dismiss_alert", { hash: id });
    void loadAlerts();
  }

  const priorityOrder: Record<string, number> = { critical: 0, warning: 1, info: 2 };
  const sorted = [...alerts].sort((a, b) => (priorityOrder[a.severity] ?? 3) - (priorityOrder[b.severity] ?? 3));

  return (
    <section className="ticket-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <header>
        <div><h2 id="modal-title">Alert Inbox</h2><p>Prioritized alerts with bulk actions</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <button type="button" className="primary-action" disabled={selected.size === 0} onClick={() => bulkAction("dismiss")}>Dismiss Selected</button>
        <button type="button" disabled={selected.size === 0} onClick={() => bulkAction("snooze")}>Snooze 24h</button>
        <button type="button" disabled={selected.size === 0} onClick={() => bulkAction("ticket")}>Convert to Ticket</button>
      </div>
      {loading ? <p>Loading…</p> : (
        <table className="data-table">
          <thead>
            <tr>
              <th><input type="checkbox" checked={selected.size > 0 && selected.size === alerts.length} onChange={() => {
                if (selected.size === alerts.length) setSelected(new Set());
                else setSelected(new Set(alerts.map((a) => a.id)));
              }} /></th>
              <th>Severity</th><th>Category</th><th>Message</th><th>School</th><th /></tr>
          </thead>
          <tbody>
            {sorted.map((a) => (
              <tr key={a.id}>
                <td><input type="checkbox" checked={selected.has(a.id)} onChange={() => toggleSelect(a.id)} /></td>
                <td><span className={`badge badge-${a.severity}`}>{a.severity}</span></td>
                <td>{a.category}</td>
                <td>{a.message}</td>
                <td>{a.school_name ?? "—"}</td>
                <td><button type="button" onClick={() => dismissOne(a.id)}>Dismiss</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
