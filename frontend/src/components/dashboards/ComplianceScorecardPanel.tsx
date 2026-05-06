import React from "react";
import type { ActionableComplianceItem } from "../../types";

type Props = {
  items: ActionableComplianceItem[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function ComplianceScorecardPanel({ items, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const sorted = React.useMemo(() => {
    const severityOrder = { critical: 0, warning: 1, info: 2 };
    return [...items].sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);
  }, [items]);

  function severityStyle(sev: string) {
    if (sev === "critical") return { background: "#fee2e2", color: "#991b1b", borderLeft: "4px solid #dc2626" };
    if (sev === "warning") return { background: "#fef9c3", color: "#854d0e", borderLeft: "4px solid #eab308" };
    return { background: "#dcfce7", color: "#166534", borderLeft: "4px solid #22c55e" };
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Compliance Scorecard</h2>
            <p>{sorted.length} actionable items</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 24px" }}>
          {sorted.map((item, i) => (
            <div key={i} style={{ ...severityStyle(item.severity), padding: "12px 16px", borderRadius: 6, marginBottom: 8, fontSize: 14 }}>
              <div style={{ fontWeight: 600 }}>{item.message}</div>
              <div style={{ fontSize: 12, marginTop: 4, opacity: 0.9 }}>
                {item.school_name} · {item.grade_level} {item.track && `(${item.track})`} · Deviation: {item.deviation}
              </div>
            </div>
          ))}
          {sorted.length === 0 && <p className="empty-state">All clear — no actionable gaps.</p>}
        </div>
      </section>
    </div>
  );
}
