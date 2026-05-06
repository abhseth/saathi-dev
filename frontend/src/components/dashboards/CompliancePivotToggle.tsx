import React from "react";
import type { CompliancePivotRow } from "../../types";

type Props = {
  data: CompliancePivotRow[];
  pivot: "subject" | "school" | "region";
  onClose: () => void;
  onLoad: (pivot: string) => Promise<void>;
};

export function CompliancePivotToggle({ data, pivot, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad(pivot);
  }, [pivot, onLoad]);

  const [sortKey, setSortKey] = React.useState<"adherence" | "deviation">("deviation");

  const sorted = React.useMemo(() => {
    return [...data].sort((a, b) => {
      if (sortKey === "adherence") return b.adherence_pct - a.adherence_pct;
      return b.deviation - a.deviation;
    });
  }, [data, sortKey]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Compliance Pivot</h2>
            <p>{data.length} rows · {pivot} view</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 16px", display: "flex", gap: 8, alignItems: "center" }}>
          <span style={{ fontSize: 13, fontWeight: 600 }}>Pivot:</span>
          {(["subject", "school", "region"] as const).map((p) => (
            <button key={p} className={`secondary-button ${pivot === p ? "primary-action" : ""}`} onClick={() => onLoad(p)}>
              {p.charAt(0).toUpperCase() + p.slice(1)}
            </button>
          ))}
          <span style={{ marginLeft: "auto", fontSize: 13 }}>Sort by:</span>
          <select value={sortKey} onChange={(e) => setSortKey(e.target.value as "adherence" | "deviation")}>
            <option value="deviation">Deviation</option>
            <option value="adherence">Adherence %</option>
          </select>
        </div>
        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>{pivot === "subject" ? "Subject" : pivot === "school" ? "School" : "Region"}</th>
                <th>Planned</th>
                <th>Actual</th>
                <th>Deviation</th>
                <th>Adherence %</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((row, i) => (
                <tr key={i}>
                  <td style={{ fontWeight: 600 }}>{row.dimension_value}</td>
                  <td>{row.planned_periods}</td>
                  <td>{row.actual_periods}</td>
                  <td style={{ color: row.deviation > 0 ? "#dc2626" : "#16a34a", fontWeight: 600 }}>{row.deviation}</td>
                  <td>{row.adherence_pct.toFixed(1)}%</td>
                </tr>
              ))}
              {sorted.length === 0 && <tr><td colSpan={5} className="empty-state">No data.</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
