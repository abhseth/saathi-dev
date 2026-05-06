import React from "react";
import { api } from "../../api";
import type { DeviationScoreboardRow } from "../../types";
import { DeviationScoreboardPanel } from "./DeviationScoreboardPanel";

export function DeviationScoreboardPanelWrapper({ onClose }: { onClose: () => void }) {
  const [data, setData] = React.useState<DeviationScoreboardRow[]>([]);
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      const result = await api<DeviationScoreboardRow[]>("deviation_scoreboard");
      setData(result);
    } catch (e) {
      console.error("Failed to load deviation scoreboard:", e);
    }
    setLoading(false);
  }, []);

  if (loading && data.length === 0) {
    return (
      <section className="ticket-modal directory-modal" aria-label="Deviation Scoreboard">
        <header><h2>Deviation Scoreboard</h2></header>
        <p style={{ margin: 24 }}>Loading…</p>
      </section>
    );
  }

  return <DeviationScoreboardPanel data={data} onClose={onClose} onLoad={load} />;
}
