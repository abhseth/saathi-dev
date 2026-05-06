import React from "react";
import type { FacultyStabilityRow } from "../../types";

type Props = {
  data: FacultyStabilityRow[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function FacultyStabilityReport({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const sorted = React.useMemo(() => {
    return [...data].sort((a, b) => b.substitution_rate_pct - a.substitution_rate_pct);
  }, [data]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Faculty Stability Report</h2>
            <p>{sorted.length} faculty members</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Faculty</th>
                <th>School</th>
                <th>Sub Rate %</th>
                <th>Cancel Rate %</th>
                <th>Planned vs Actual</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((row) => (
                <tr key={`${row.faculty_user_id}-${row.school_name}`}>
                  <td>{row.faculty_name}</td>
                  <td>{row.school_name}</td>
                  <td style={{ color: row.substitution_rate_pct > 20 ? "#dc2626" : "#166534", fontWeight: 600 }}>
                    {row.substitution_rate_pct.toFixed(1)}%
                  </td>
                  <td style={{ color: row.cancellation_rate_pct > 10 ? "#eab308" : "#166534", fontWeight: 600 }}>
                    {row.cancellation_rate_pct.toFixed(1)}%
                  </td>
                  <td>{row.planned_vs_actual_variance}</td>
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
