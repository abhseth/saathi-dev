import React from "react";
import type { DeviationScoreboardRow } from "../../types";

type Props = {
  data: DeviationScoreboardRow[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function DeviationScoreboardPanel({ data, onClose, onLoad }: Props) {
  const [expandedId, setExpandedId] = React.useState<number | null>(null);

  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  function scoreColor(score: number) {
    if (score >= 20) return "#dc2626";
    if (score >= 10) return "#eab308";
    return "#16a34a";
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Deviation Scoreboard</h2>
            <p>{data.length} schools ranked by deviation</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Rank</th>
                <th>School</th>
                <th>Region</th>
                <th>Deviation Score</th>
                <th>Top Gaps</th>
              </tr>
            </thead>
            <tbody>
              {data.map((row, idx) => (
                <React.Fragment key={row.school_id}>
                  <tr style={{ cursor: "pointer" }} onClick={() => setExpandedId(expandedId === row.school_id ? null : row.school_id)}>
                    <td>{idx + 1}</td>
                    <td>{row.school_name}</td>
                    <td>{row.region_name || "—"}</td>
                    <td style={{ color: scoreColor(row.overall_deviation_score), fontWeight: 700 }}>
                      {row.overall_deviation_score.toFixed(1)}%
                    </td>
                    <td>{row.top_gaps.length} gaps</td>
                  </tr>
                  {expandedId === row.school_id && (
                    <tr>
                      <td colSpan={5} style={{ background: "#f8fafc", padding: "12px 24px" }}>
                        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13 }}>
                          {row.top_gaps.map((g, i) => (
                            <li key={i}>
                              {g.subject_name} — {g.grade_level} {g.track && `(${g.track})`}: {g.actual}/{g.planned} planned
                            </li>
                          ))}
                        </ul>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              ))}
              {data.length === 0 && <tr><td colSpan={5} className="empty-state">No data.</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
