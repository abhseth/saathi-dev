import React from "react";
import type { SessionTypeBreakdown } from "../../types";

type Props = {
  data: SessionTypeBreakdown[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function SessionTypeBreakdownPanel({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Session-Type Adherence</h2>
            <p>{data.length} session types</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 24px" }}>
          {data.map((row) => (
            <div key={row.session_type} style={{ marginBottom: 12, padding: 12, border: "1px solid #e2e8f0", borderRadius: 8 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
                <span style={{ fontWeight: 700 }}>{row.session_type}</span>
                <span style={{ fontWeight: 600, color: row.adherence_pct >= 90 ? "#16a34a" : row.adherence_pct >= 75 ? "#eab308" : "#dc2626" }}>
                  {row.adherence_pct.toFixed(1)}%
                </span>
              </div>
              <div style={{ height: 8, background: "#f1f5f9", borderRadius: 4, overflow: "hidden" }}>
                <div style={{ width: `${Math.min(100, row.adherence_pct)}%`, height: "100%", background: row.adherence_pct >= 90 ? "#22c55e" : row.adherence_pct >= 75 ? "#eab308" : "#ef4444", borderRadius: 4 }} />
              </div>
              <div style={{ fontSize: 12, color: "#64748b", marginTop: 6 }}>
                {row.actual_periods} / {row.planned_periods} periods
              </div>
            </div>
          ))}
          {data.length === 0 && <p className="empty-state">No data.</p>}
        </div>
      </section>
    </div>
  );
}
