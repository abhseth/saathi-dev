import React from "react";
import type { FacultyUtilizationTrend, HealthTrendWeek, SubstitutionTrendWeek } from "../../types";

type Props = {
  facultyUtilization: FacultyUtilizationTrend[];
  healthTrends: HealthTrendWeek[];
  substitutionTrends: SubstitutionTrendWeek[];
  onClose: () => void;
  onLoadFaculty: () => Promise<void>;
  onLoadHealth: () => Promise<void>;
  onLoadSubstitutions: () => Promise<void>;
};

function SimpleLineChart({ labels, datasets }: { labels: string[]; datasets: { label: string; color: string; values: number[] }[] }) {
  const max = Math.max(1, ...datasets.flatMap((d) => d.values));
  const width = 600;
  const height = 200;
  const pad = { top: 20, right: 20, bottom: 30, left: 40 };
  const chartW = width - pad.left - pad.right;
  const chartH = height - pad.top - pad.bottom;
  const n = labels.length || 1;

  return (
    <svg width="100%" viewBox={`0 0 ${width} ${height}`} style={{ background: "#fff" }}>
      {datasets.map((ds, di) => (
        <g key={di}>
          <polyline
            fill="none"
            stroke={ds.color}
            strokeWidth={2}
            points={ds.values.map((v, i) => {
              const x = pad.left + (i / (n - 1)) * chartW;
              const y = pad.top + chartH - (v / max) * chartH;
              return `${x},${y}`;
            }).join(" ")}
          />
          {ds.values.map((v, i) => {
            const x = pad.left + (i / (n - 1)) * chartW;
            const y = pad.top + chartH - (v / max) * chartH;
            return <circle key={i} cx={x} cy={y} r={3} fill={ds.color} />;
          })}
        </g>
      ))}
      {labels.map((lbl, i) => {
        const x = pad.left + (i / (n - 1)) * chartW;
        return <text key={i} x={x} y={height - 5} fontSize={10} textAnchor="middle" fill="#64748b">{lbl.slice(5)}</text>;
      })}
      <text x={pad.left - 10} y={pad.top} fontSize={10} textAnchor="end" fill="#64748b">{max}</text>
      <text x={pad.left - 10} y={pad.top + chartH} fontSize={10} textAnchor="end" fill="#64748b">0</text>
    </svg>
  );
}

export function TrendChartsPanel({
  facultyUtilization,
  healthTrends,
  substitutionTrends,
  onClose,
  onLoadFaculty,
  onLoadHealth,
  onLoadSubstitutions,
}: Props) {
  const [tab, setTab] = React.useState<"faculty" | "health" | "substitutions">("health");

  React.useEffect(() => {
    void onLoadFaculty();
    void onLoadHealth();
    void onLoadSubstitutions();
  }, [onLoadFaculty, onLoadHealth, onLoadSubstitutions]);

  const healthLabels = healthTrends.map((w) => w.week_start_date);
  const healthDatasets = [
    { label: "Green", color: "#16a34a", values: healthTrends.map((w) => w.green_count) },
    { label: "Amber", color: "#eab308", values: healthTrends.map((w) => w.amber_count) },
    { label: "Red", color: "#dc2626", values: healthTrends.map((w) => w.red_count) },
  ];

  const subLabels = substitutionTrends.map((w) => w.week_start_date);
  const subDatasets = [
    { label: "Absences", color: "#dc2626", values: substitutionTrends.map((w) => w.faculty_absences) },
    { label: "Short-staffed", color: "#eab308", values: substitutionTrends.map((w) => w.short_staffed_periods) },
    { label: "Over-utilized", color: "#2563eb", values: substitutionTrends.map((w) => w.over_utilized_substitutes) },
  ];

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Trend Charts</h2>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 16px" }}>
          <div style={{ display: "flex", gap: 8 }}>
            <button className={`secondary-button ${tab === "health" ? "primary-action" : ""}`} onClick={() => setTab("health")}>Health Trends</button>
            <button className={`secondary-button ${tab === "substitutions" ? "primary-action" : ""}`} onClick={() => setTab("substitutions")}>Substitutions</button>
            <button className={`secondary-button ${tab === "faculty" ? "primary-action" : ""}`} onClick={() => setTab("faculty")}>Faculty Utilization</button>
          </div>
        </div>
        <div style={{ margin: "0 24px 24px" }}>
          {tab === "health" && (
            <>
              <h3 className="section-heading">8-Week Health Trajectory</h3>
              <SimpleLineChart labels={healthLabels} datasets={healthDatasets} />
              <div style={{ marginTop: 12, fontSize: 13, color: "#64748b" }}>
                Network adherence: {healthTrends[healthTrends.length - 1]?.network_adherence_pct.toFixed(1) ?? "—"}%
              </div>
            </>
          )}
          {tab === "substitutions" && (
            <>
              <h3 className="section-heading">4-Week Substitution Patterns</h3>
              <SimpleLineChart labels={subLabels} datasets={subDatasets} />
            </>
          )}
          {tab === "faculty" && (
            <>
              <h3 className="section-heading">Faculty Utilization (4-week)</h3>
              <div style={{ display: "grid", gap: 12 }}>
                {facultyUtilization.slice(0, 10).map((f) => (
                  <div key={f.faculty_user_id} style={{ fontSize: 13 }}>
                    <div style={{ fontWeight: 600 }}>{f.faculty_name}</div>
                    <div style={{ display: "flex", gap: 4, alignItems: "flex-end", height: 40 }}>
                      {f.weeks.map((w, i) => (
                        <div key={i} style={{ width: 20, background: "#3b82f6", height: `${(w.period_count / 20) * 100}%`, minHeight: 2, borderRadius: 2 }} title={`${w.week_start_date}: ${w.period_count}`} />
                      ))}
                    </div>
                  </div>
                ))}
                {facultyUtilization.length === 0 && <p className="empty-state">No data.</p>}
              </div>
            </>
          )}
        </div>
      </section>
    </div>
  );
}
