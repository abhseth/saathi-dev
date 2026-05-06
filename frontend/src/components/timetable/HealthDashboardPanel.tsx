import React from "react";
import type { AppUser, Region, School, TimetableHealthStatus } from "../../types";
import { useModalFocus } from "./useModalFocus";

type HealthDashboardPanelProps = {
  schools: School[];
  regions: Region[];
  users: AppUser[];
  healthData: TimetableHealthStatus[];
  onClose: () => void;
  onLoad: () => Promise<void>;
  onViewSchoolTimetable: (schoolId: number) => void;
};

function normalizeStatus(status: string): string {
  const s = status.toLowerCase();
  if (s === "green") return "Green";
  if (s === "amber") return "Amber";
  if (s === "red") return "Red";
  return status;
}

function statusLabel(status: string): string {
  const s = normalizeStatus(status);
  if (s === "Green") return "Healthy";
  if (s === "Amber") return "Needs Attention";
  return "Critical";
}

export function HealthDashboardPanel({
  schools,
  regions,
  users,
  healthData,
  onClose,
  onLoad,
  onViewSchoolTimetable,
}: HealthDashboardPanelProps) {
  const [filterRegion, setFilterRegion] = React.useState<string>("");
  const [searchSchool, setSearchSchool] = React.useState("");
  const [expandedSchoolId, setExpandedSchoolId] = React.useState<number | null>(null);
  const modalRef = useModalFocus(onClose);

  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const aomMap = React.useMemo(() => {
    const map = new Map<number, string>();
    for (const s of schools) {
      map.set(s.id, s.aom_name || "—");
    }
    return map;
  }, [schools]);

  const filteredData = React.useMemo(() => {
    let data = healthData;
    if (filterRegion) {
      data = data.filter((h) => h.region_name === filterRegion);
    }
    if (searchSchool.trim()) {
      const q = searchSchool.trim().toLowerCase();
      data = data.filter((h) => h.school_name.toLowerCase().includes(q));
    }
    // Sort by severity: Red first, then Amber, then Green
    const severityOrder: Record<string, number> = { red: 0, amber: 1, green: 2 };
    data = [...data].sort((a, b) => {
      const sa = severityOrder[a.status.toLowerCase()] ?? 99;
      const sb = severityOrder[b.status.toLowerCase()] ?? 99;
      return sa - sb;
    });
    return data;
  }, [healthData, filterRegion, searchSchool]);

  const total = filteredData.length;
  const greenCount = filteredData.filter((h) => normalizeStatus(h.status) === "Green").length;
  const amberCount = filteredData.filter((h) => normalizeStatus(h.status) === "Amber").length;
  const redCount = filteredData.filter((h) => normalizeStatus(h.status) === "Red").length;

  function statusStyle(status: string): React.CSSProperties {
    const s = normalizeStatus(status);
    if (s === "Green") return { background: "#dcfce7", color: "#166534", fontWeight: 600 };
    if (s === "Amber") return { background: "#fef9c3", color: "#854d0e", fontWeight: 600 };
    return { background: "#fee2e2", color: "#991b1b", fontWeight: 600 };
  }

  return (
    <div className="modal-backdrop" role="presentation" ref={modalRef} tabIndex={-1}>
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Timetable Health Dashboard</h2>
            <p>{total} schools · {greenCount} green · {amberCount} amber · {redCount} red</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>

        <div style={{ margin: "0 24px 16px", display: "flex", gap: 16, flexWrap: "wrap" }}>
          <div style={{ padding: "12px 20px", borderRadius: 8, background: "#f1f5f9", minWidth: 100, textAlign: "center" }}>
            <div style={{ fontSize: 20, fontWeight: 700 }}>{total}</div>
            <div style={{ fontSize: 12, color: "#64748b" }}>Total</div>
          </div>
          <div style={{ padding: "12px 20px", borderRadius: 8, background: "#dcfce7", minWidth: 100, textAlign: "center" }}>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#166534" }}>{greenCount}</div>
            <div style={{ fontSize: 12, color: "#166534" }}>Green</div>
          </div>
          <div style={{ padding: "12px 20px", borderRadius: 8, background: "#fef9c3", minWidth: 100, textAlign: "center" }}>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#854d0e" }}>{amberCount}</div>
            <div style={{ fontSize: 12, color: "#854d0e" }}>Amber</div>
          </div>
          <div style={{ padding: "12px 20px", borderRadius: 8, background: "#fee2e2", minWidth: 100, textAlign: "center" }}>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#991b1b" }}>{redCount}</div>
            <div style={{ fontSize: 12, color: "#991b1b" }}>Red</div>
          </div>
        </div>

        <div className="master-data-form" style={{ margin: "0 24px 16px" }}>
          <label>
            Region
            <select value={filterRegion} onChange={(e) => setFilterRegion(e.target.value)}>
              <option value="">All Regions</option>
              {regions.map((r) => (
                <option key={r.id} value={r.name}>{r.name}</option>
              ))}
            </select>
          </label>
          <label>
            Search School
            <input
              type="search"
              placeholder="Type school name..."
              value={searchSchool}
              onChange={(e) => setSearchSchool(e.target.value)}
            />
          </label>
        </div>

        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>School</th>
                <th>Region</th>
                <th>AOM</th>
                <th>Class Offerings?</th>
                <th>Master Timetable?</th>
                <th>Sessions Generated?</th>
                <th>Gaps?</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {filteredData.map((h) => (
                <React.Fragment key={h.school_id}>
                  <tr
                    style={{ cursor: "pointer" }}
                    onClick={() => onViewSchoolTimetable(h.school_id)}
                  >
                    <td>{h.school_name}</td>
                    <td>{h.region_name || "—"}</td>
                    <td>{aomMap.get(h.school_id) ?? "—"}</td>
                    <td>{h.class_plans_configured ? "Yes" : "No"}</td>
                    <td>{h.master_timetable_complete ? "Yes" : "No"}</td>
                    <td>{h.sessions_generated ? "Yes" : "No"}</td>
                    <td>{h.gaps_count > 0 ? "Warning" : "OK"}</td>
                    <td>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          setExpandedSchoolId(expandedSchoolId === h.school_id ? null : h.school_id);
                        }}
                        style={{ ...statusStyle(h.status), border: "none", borderRadius: 4, padding: "4px 12px", cursor: "pointer" }}
                        aria-label={`Status: ${statusLabel(h.status)}`}
                      >
                        {statusLabel(h.status)}
                      </button>
                    </td>
                  </tr>
                  {expandedSchoolId === h.school_id && (
                    <tr>
                      <td colSpan={8} style={{ background: "#f8fafc", padding: "12px 24px" }}>
                        <div style={{ fontSize: 13 }}>
                          <strong>Breakdown:</strong>
                          {h.gap_details?.length === 0 ? (
                            <p style={{ margin: "4px 0 0" }} className="text-muted">No gaps reported.</p>
                          ) : (
                            <ul style={{ margin: "4px 0 0", paddingLeft: 18 }}>
                              {h.gap_details?.map((d, i) => (
                                <li key={i}>{d}</li>
                              ))}
                            </ul>
                          )}
                          <p className="text-muted" style={{ marginTop: 8 }}>Last updated: {h.last_updated || "—"}</p>
                        </div>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              ))}
              {filteredData.length === 0 && (
                <tr>
                  <td colSpan={8} className="empty-state">No health data available.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
