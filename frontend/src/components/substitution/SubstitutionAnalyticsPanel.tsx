import React from "react";
import { api } from "../../api";
import type { SubstitutionBalance, SubstitutionReportRow } from "../../types";

export function SubstitutionAnalyticsPanel({
  faculty,
}: {
  faculty: Array<{ id: number; display_name: string }>;
}) {
  const [selectedFacultyId, setSelectedFacultyId] = React.useState(faculty[0]?.id ?? 0);
  const [balance, setBalance] = React.useState<SubstitutionBalance | null>(null);
  const [month, setMonth] = React.useState(() => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  });
  const [reports, setReports] = React.useState<SubstitutionReportRow[]>([]);
  const [notice, setNotice] = React.useState("");

  React.useEffect(() => {
    if (!selectedFacultyId) return;
    void (async () => {
      try {
        const b = await api<SubstitutionBalance>("substitution_balance", { facultyUserId: selectedFacultyId });
        setBalance(b);
      } catch (e) {
        console.error("Failed to load substitution balance:", e);
        setBalance(null);
      }
    })();
  }, [selectedFacultyId]);

  React.useEffect(() => {
    void loadReports();
  }, [month]);

  async function loadReports() {
    try {
      const r = await api<SubstitutionReportRow[]>("substitution_reports", { month });
      setReports(r);
    } catch (e) {
      console.error("Failed to load substitution reports:", e);
      setReports([]);
    }
  }

  return (
    <section className="ticket-modal" aria-label="Substitution Analytics">
      <header>
        <div>
          <h2>Substitution Analytics</h2>
          <p>Balance tracker and monthly coverage reports</p>
        </div>
      </header>

      {notice ? <div className="notice-banner">{notice}</div> : null}

      <div className="analytics-grid">
        <div className="analytics-card">
          <h3>Substitution Balance</h3>
          <label>
            Faculty
            <select value={selectedFacultyId} onChange={(e) => setSelectedFacultyId(Number(e.target.value))}>
              {faculty.map((f) => (
                <option key={f.id} value={f.id}>{f.display_name}</option>
              ))}
            </select>
          </label>

          {balance ? (
            <div className="balance-display">
              <div className="balance-metric">
                <span className="balance-number">{balance.given_count}</span>
                <span className="balance-label">Given</span>
              </div>
              <div className="balance-metric">
                <span className="balance-number">{balance.received_count}</span>
                <span className="balance-label">Received</span>
              </div>
            </div>
          ) : (
            <p className="empty-state compact">Select a faculty to view balance.</p>
          )}
        </div>

        <div className="analytics-card">
          <h3>Coverage Report</h3>
          <label>
            Month
            <input type="month" value={month} onChange={(e) => setMonth(e.target.value)} />
          </label>

          <table className="data-table compact">
            <thead>
              <tr>
                <th>School</th>
                <th>Requests</th>
                <th>Filled</th>
                <th>Rate</th>
                <th>Top Absentee</th>
              </tr>
            </thead>
            <tbody>
              {reports.map((r) => (
                <tr key={`${r.school_id}-${r.month}`}>
                  <td>{r.school_name}</td>
                  <td>{r.request_count}</td>
                  <td>{r.filled_count}</td>
                  <td>{r.acceptance_rate_pct}%</td>
                  <td>{r.top_absentee_name} ({r.top_absentee_count})</td>
                </tr>
              ))}
              {reports.length === 0 && (
                <tr><td colSpan={5} className="empty-state compact">No data for {month}</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
