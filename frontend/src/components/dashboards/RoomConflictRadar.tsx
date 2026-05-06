import React from "react";
import type { RoomConflictRadarCell } from "../../types";

type Props = {
  data: RoomConflictRadarCell[];
  onClose: () => void;
  onLoad: () => Promise<void>;
};

export function RoomConflictRadar({ data, onClose, onLoad }: Props) {
  React.useEffect(() => {
    void onLoad();
  }, [onLoad]);

  const rooms = React.useMemo(() => Array.from(new Set(data.map((d) => d.room))).sort(), [data]);
  const periods = React.useMemo(() => {
    const set = new Set<number>();
    data.forEach((d) => set.add(d.period));
    return Array.from(set).sort((a, b) => a - b);
  }, [data]);
  const days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="ticket-modal directory-modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <header>
          <div>
            <h2 id="modal-title">Room Conflict Radar</h2>
            <p>{data.length} conflicts</p>
          </div>
          <button type="button" aria-label="Close" onClick={onClose}>Close</button>
        </header>
        <div style={{ margin: "0 24px 24px", overflowX: "auto" }}>
          {rooms.map((room) => (
            <div key={room} style={{ marginBottom: 24 }}>
              <div style={{ fontWeight: 700, marginBottom: 8 }}>{room}</div>
              <table className="data-table" style={{ fontSize: 12 }}>
                <thead>
                  <tr>
                    <th>Day</th>
                    {periods.map((p) => <th key={p}>P{p}</th>)}
                  </tr>
                </thead>
                <tbody>
                  {days.map((day, dIdx) => (
                    <tr key={day}>
                      <td>{day}</td>
                      {periods.map((p) => {
                        const cell = data.find((c) => c.room === room && c.day_of_week === dIdx && c.period === p);
                        return (
                          <td key={p} style={{ background: cell ? "#fee2e2" : "#dcfce7", textAlign: "center" }}>
                            {cell ? (
                              <span style={{ color: "#991b1b", fontWeight: 700 }} title={cell.schools.join(", ")}>
                                {cell.conflict_count}
                              </span>
                            ) : (
                              <span style={{ color: "#166534" }}>✓</span>
                            )}
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ))}
          {rooms.length === 0 && <p className="empty-state">No room conflicts detected.</p>}
        </div>
      </section>
    </div>
  );
}
