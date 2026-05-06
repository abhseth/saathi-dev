import React from "react";
import { api } from "../../api";
import type { ActionableComplianceItem } from "../../types";
import { ComplianceScorecardPanel } from "./ComplianceScorecardPanel";

export function ComplianceScorecardPanelWrapper({ onClose }: { onClose: () => void }) {
  const [items, setItems] = React.useState<ActionableComplianceItem[]>([]);
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      const result = await api<ActionableComplianceItem[]>("compliance_scorecard");
      setItems(result);
    } catch (e) {
      console.error("Failed to load compliance scorecard:", e);
    }
    setLoading(false);
  }, []);

  if (loading && items.length === 0) {
    return (
      <section className="ticket-modal directory-modal" aria-label="Compliance Scorecard">
        <header><h2>Compliance Scorecard</h2></header>
        <p style={{ margin: 24 }}>Loading…</p>
      </section>
    );
  }

  return <ComplianceScorecardPanel items={items} onClose={onClose} onLoad={load} />;
}
