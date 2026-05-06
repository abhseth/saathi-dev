import React from "react";
import type { ControlTowerCard } from "../../types";

type Props = {
  data: ControlTowerCard[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function ControlTowerPanel({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Control Tower</h2>
            <p>{data.length} schools</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div className="session-manager-table-wrapper" style={{ margin: "0 24px 24px" }}>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: 16, padding: 8 }}>
            {data.map((card) => (
              <div key={card.school_id} style={{ border: "1px solid #e2e8f0", borderRadius: 8, padding: 16, background: "#fff" }}>
                <div style={{ fontWeight: 700, fontSize: 16, marginBottom: 4 }}>{card.school_name}</div>
                <div style={{ fontSize: 12, color: "#64748b", marginBottom: 12 }}>{card.region_name || "—"}</div>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, fontSize: 13 }}>
                  <div style={{ background: "#f1f5f9", padding: "8px 10px", borderRadius: 6 }}>
                    <div style={{ fontWeight: 700 }}>{card.filled_periods}/{card.total_periods}</div>
                    <div style={{ fontSize: 11, color: "#64748b" }}>Periods</div>
                  </div>
                  <div style={{ background: "#fef9c3", padding: "8px 10px", borderRadius: 6 }}>
                    <div style={{ fontWeight: 700, color: "#854d0e" }}>{card.alert_count}</div>
                    <div style={{ fontSize: 11, color: "#854d0e" }}>Alerts</div>
                  </div>
                  <div style={{ background: "#dcfce7", padding: "8px 10px", borderRadius: 6 }}>
                    <div style={{ fontWeight: 700, color: "#166534" }}>{card.attendance_percent}%</div>
                    <div style={{ fontSize: 11, color: "#166534" }}>Attendance</div>
                  </div>
                  <div style={{ background: "#fee2e2", padding: "8px 10px", borderRadius: 6 }}>
                    <div style={{ fontWeight: 700, color: "#991b1b" }}>{card.active_substitutions}</div>
                    <div style={{ fontSize: 11, color: "#991b1b" }}>Substitutions</div>
                  </div>
                </div>
              </div>
            ))}
          </div>
          {data.length === 0 && <p className="empty-state">No data.</p>}
        </div>
      </section>
    </div>
  );
}
