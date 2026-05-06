import React from "react";
import type { SubjectCoverageCell } from "../../types";

type Props = {
  data: SubjectCoverageCell[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function SubjectCoverageHeatmap({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const regions = React.useMemo(() => Array.from(new Set(data.map((d) => d.region_name))).sort(), [data]);
  const subjects = React.useMemo(() => Array.from(new Set(data.map((d) => d.subject_name))).sort(), [data]);

  function cellColor(pct: number) {
    if (pct >= 90) return "#dcfce7";
    if (pct >= 75) return "#fef9c3";
    return "#fee2e2";
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Subject Coverage Heatmap</h2>
            <p>Region × Subject adherence %</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 24px", overflowX: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Region</th>
                {subjects.map((s) => <th key={s}>{s}</th>)}
              </tr>
            </thead>
            <tbody>
              {regions.map((region) => (
                <tr key={region}>
                  <td style={{ fontWeight: 600 }}>{region}</td>
                  {subjects.map((subject) => {
                    const cell = data.find((d) => d.region_name === region && d.subject_name === subject);
                    const pct = cell?.adherence_pct ?? 0;
                    return (
                      <td key={subject} style={{ background: cellColor(pct), textAlign: "center", fontWeight: 600, fontSize: 12 }}>
                        {pct.toFixed(0)}%
                      </td>
                    );
                  })}
                </tr>
              ))}
              {regions.length === 0 && <tr><td colSpan={subjects.length + 1} className="empty-state">No data.</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
