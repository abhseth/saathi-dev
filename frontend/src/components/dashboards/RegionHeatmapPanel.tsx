import React from "react";
import type { RegionHeatmapCell } from "../../types";

type Props = {
  data: RegionHeatmapCell[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

const DAY_LABELS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function RegionHeatmapPanel({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const schools = React.useMemo(() => {
    const map = new Map<number, string>();
    for (const d of data) map.set(d.school_id, d.school_name);
    return Array.from(map.entries());
  }, [data]);

  const maxIssue = React.useMemo(() => Math.max(1, ...data.map((d) => d.issue_count)), [data]);

  function cellColor(count: number) {
    if (count === 0) return "#dcfce7";
    const intensity = Math.min(1, count / maxIssue);
    const r = Math.round(220 + (254 - 220) * intensity);
    const g = Math.round(252 + (226 - 252) * intensity);
    const b = Math.round(231 + (226 - 231) * intensity);
    return `rgb(${r},${g},${b})`;
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Region Heat Map</h2>
            <p>Schools × Days issue count</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 24px", overflowX: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>School</th>
                {DAY_LABELS.map((d) => <th key={d}>{d}</th>)}
              </tr>
            </thead>
            <tbody>
              {schools.map(([sid, sname]) => (
                <tr key={sid}>
                  <td>{sname}</td>
                  {DAY_LABELS.map((_, day) => {
                    const cell = data.find((d) => d.school_id === sid && d.day_of_week === day);
                    const count = cell?.issue_count ?? 0;
                    return (
                      <td key={day} style={{ background: cellColor(count), textAlign: "center", fontWeight: 600, fontSize: 12 }}>
                        {count}
                      </td>
                    );
                  })}
                </tr>
              ))}
              {schools.length === 0 && <tr><td colSpan={8} className="empty-state">No data.</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
