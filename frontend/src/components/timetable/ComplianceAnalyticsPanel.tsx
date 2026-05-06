import React from "react";
import type { ComplianceMetrics, School } from "../../types";

type ComplianceAnalyticsPanelProps = {
  schools: School[];
  complianceData: ComplianceMetrics[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function ComplianceAnalyticsPanel({
  schools,
  complianceData,
  onClose,
  onLoad,
}: ComplianceAnalyticsPanelProps) {
  const [filterSchool, setFilterSchool] = React.useState<number | "">("");
  const [filterGrade, setFilterGrade] = React.useState("");
  const [filterTrack, setFilterTrack] = React.useState("");
  const [filterSubject, setFilterSubject] = React.useState("");

  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const grades = React.useMemo(() => Array.from(new Set(complianceData.map((g) => g.grade_level))).sort(), [complianceData]);
  const tracks = React.useMemo(() => Array.from(new Set(complianceData.map((s) => s.track))).filter(Boolean).sort(), [complianceData]);
  const subjects = React.useMemo(() => Array.from(new Set(complianceData.map((g) => g.subject_name))).sort(), [complianceData]);

  const filteredData = React.useMemo(() => {
    return complianceData.filter((d) => {
      if (filterSchool && d.school_id !== filterSchool) return false;
      if (filterGrade && d.grade_level !== filterGrade) return false;
      if (filterTrack && d.track !== filterTrack) return false;
      if (filterSubject && d.subject_name !== filterSubject) return false;
      return true;
    });
  }, [complianceData, filterSchool, filterGrade, filterTrack, filterSubject]);

  function exportCsv() {
    const rows: string[][] = [];
    rows.push(["School", "Grade", "Track", "Subject", "Planned", "Actual", "Deviation", "Adherence %"]);
    for (const g of filteredData) {
      rows.push([g.school_name, g.grade_level, g.track, g.subject_name, String(g.planned_periods), String(g.actual_periods), String(g.deviation), String(Math.round(g.lecture_model_adherence_pct))]);
    }
    const csv = rows.map((r) => r.map((c) => `"${c.replace(/"/g, '""')}"`).join(",")).join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "compliance-analytics.csv";
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Compliance Analytics</h2>
            <p>{filteredData.length} rows</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>

        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            School
            <select value={filterSchool} onChange={(e) => setFilterSchool(e.target.value === "" ? "" : Number(e.target.value))}>
              <option value="">All</option>
              {schools.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </label>
          <label>
            Grade
            <select value={filterGrade} onChange={(e) => setFilterGrade(e.target.value)}>
              <option value="">All</option>
              {grades.map((g) => (
                <option key={g} value={g}>{g}</option>
              ))}
            </select>
          </label>
          <label>
            Track
            <select value={filterTrack} onChange={(e) => setFilterTrack(e.target.value)}>
              <option value="">All</option>
              {tracks.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </label>
          <label>
            Subject
            <select value={filterSubject} onChange={(e) => setFilterSubject(e.target.value)}>
              <option value="">All</option>
              {subjects.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
          </label>
        </div>

        <div style={{ margin: "0 24px 16px" }}>
          <button type="button" className="secondary-button" onClick={exportCsv}>
            Export CSV
          </button>
        </div>

        <div style={{ margin: "0 24px 24px" }}>
          <h3 className="section-heading">Subject Compliance Rows</h3>
          <div className="session-manager-table-wrapper">
            <table className="data-table">
              <thead>
                <tr>
                  <th>School</th>
                  <th>Grade</th>
                  <th>Track</th>
                  <th>Subject</th>
                  <th>Planned</th>
                  <th>Actual</th>
                  <th>Deviation</th>
                  <th>Adherence %</th>
                </tr>
              </thead>
              <tbody>
                {filteredData.map((d, i) => (
                  <tr key={`${d.school_id}-${d.grade_level}-${d.track}-${d.subject_name}-${i}`}>
                    <td>{d.school_name}</td>
                    <td>{d.grade_level}</td>
                    <td>{d.track}</td>
                    <td>{d.subject_name}</td>
                    <td>{d.planned_periods}</td>
                    <td>{d.actual_periods}</td>
                    <td style={{ color: d.deviation > 0 ? "#dc2626" : "#16a34a", fontWeight: 600 }}>{d.deviation}</td>
                    <td>{Math.round(d.lecture_model_adherence_pct)}%</td>
                  </tr>
                ))}
                {filteredData.length === 0 && (
                  <tr><td colSpan={8} className="empty-state">No data.</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </div>
  );
}
