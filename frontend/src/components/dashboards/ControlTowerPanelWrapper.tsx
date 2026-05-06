import React from "react";
import { api } from "../../api";
import type { ControlTowerCard } from "../../types";
import { ControlTowerPanel } from "./ControlTowerPanel";

export function ControlTowerPanelWrapper({ onClose }: { onClose: () => void }) {
  const [data, setData] = React.useState<ControlTowerCard[]>([]);
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      const result = await api<ControlTowerCard[]>("control_tower");
      setData(result);
    } catch (e) {
      console.error("Failed to load control tower:", e);
    }
    setLoading(false);
  }, []);

  if (loading && data.length === 0) {
    return (
      <section className="ticket-modal directory-modal" aria-label="Control Tower">
        <header><h2>Control Tower</h2></header>
        <p style={{ margin: 24 }}>Loading…</p>
      </section>
    );
  }

  return <ControlTowerPanel data={data} onClose={onClose} onLoad={load} />;
}
