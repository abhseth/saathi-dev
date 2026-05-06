import React from "react";
import { api } from "../../api";
import type { CentralPolicy } from "../../types";

export function PolicyConfigPanel({ onClose }: { onClose: () => void }) {
  const [policies, setPolicies] = React.useState<CentralPolicy[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [draft, setDraft] = React.useState<Record<string, string>>({});

  const loadPolicies = React.useCallback(async () => {
    setLoading(true);
    try {
      const data = await api<CentralPolicy[]>("list_policies");
      setPolicies(data);
      const map: Record<string, string> = {};
      for (const p of data) map[p.key] = p.value;
      setDraft(map);
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    void loadPolicies();
  }, [loadPolicies]);

  async function savePolicy(key: string) {
    await api<CentralPolicy>("update_policy", { key, value: draft[key] || "" });
    void loadPolicies();
  }

  const knownKeys = [
    { key: "max_periods_per_faculty", label: "Max Periods per Faculty", default: "24" },
    { key: "mandatory_subjects", label: "Mandatory Subjects (comma-separated)", default: "" },
    { key: "attendance_marking_deadline", label: "Attendance Marking Deadline (hours)", default: "24" },
  ];

  return (
    <section className="ticket-modal" aria-label="Policy configuration">
      <header>
        <div><h2>Central Policies</h2><p>Configure system-wide thresholds and mandates</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      {loading ? <p>Loading…</p> : (
        <div className="policy-list" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          {knownKeys.map((k) => (
            <div key={k.key} className="policy-row" style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <label style={{ minWidth: 280 }}>{k.label}</label>
              <input
                type="text"
                value={draft[k.key] ?? k.default}
                onChange={(e) => setDraft((prev) => ({ ...prev, [k.key]: e.target.value }))}
                style={{ flex: 1 }}
              />
              <button type="button" className="primary-action" onClick={() => savePolicy(k.key)}>Save</button>
            </div>
          ))}
          {policies.filter((p) => !knownKeys.find((k) => k.key === p.key)).map((p) => (
            <div key={p.key} className="policy-row" style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <label style={{ minWidth: 280 }}>{p.key}</label>
              <input
                type="text"
                value={draft[p.key] ?? p.value}
                onChange={(e) => setDraft((prev) => ({ ...prev, [p.key]: e.target.value }))}
                style={{ flex: 1 }}
              />
              <button type="button" className="primary-action" onClick={() => savePolicy(p.key)}>Save</button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
