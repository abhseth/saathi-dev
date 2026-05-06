import React from "react";
import { api } from "../../api";
import type { InterventionDigest, SipBrief } from "../../types";

export function DigestPanel({ onClose }: { onClose: () => void }) {
  const [tab, setTab] = React.useState<"intervention" | "sip">("intervention");
  const [intervention, setIntervention] = React.useState<InterventionDigest | null>(null);
  const [sip, setSip] = React.useState<SipBrief | null>(null);
  const [loading, setLoading] = React.useState(false);

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      if (tab === "intervention") {
        const data = await api<InterventionDigest>("intervention_digest");
        setIntervention(data);
      } else {
        const data = await api<SipBrief>("sip_brief");
        setSip(data);
      }
    } finally { setLoading(false); }
  }, [tab]);

  React.useEffect(() => {
    void load();
  }, [load]);

  return (
    <section className="ticket-modal" aria-label="Weekly digests">
      <header>
        <div><h2>Weekly Digests</h2><p>Monday 8 AM briefing content</p></div>
        <button type="button" className="ghost-button" aria-label="Close" onClick={onClose}>Close</button>
      </header>
      <div className="tab-bar" style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <button className={tab === "intervention" ? "active" : ""} onClick={() => setTab("intervention")}>Intervention</button>
        <button className={tab === "sip" ? "active" : ""} onClick={() => setTab("sip")}>SIP Brief</button>
      </div>
      {loading ? <p>Loading…</p> : tab === "intervention" && intervention ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div>
            <h4>Top Schools by Deviation</h4>
            <ol>{intervention.top_schools_by_deviation.map((s) => <li key={s.school_id}>{s.school_name}: {s.deviation_score.toFixed(1)}</li>)}</ol>
          </div>
          <div>
            <h4>SLA Breaches</h4>
            <ul>{intervention.sla_breaches.map((b) => <li key={b.ticket_id}>{b.title} ({b.school_name}) — {b.hours_overdue}h overdue</li>)}</ul>
          </div>
          <div>
            <h4>Low Attendance Regions</h4>
            <ul>{intervention.low_attendance_regions.map((r, i) => <li key={i}>{r.region_name}: {r.avg_attendance_pct.toFixed(1)}%</li>)}</ul>
          </div>
        </div>
      ) : tab === "sip" && sip ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div>
            <h4>Status Flips (Red/Amber)</h4>
            <ul>{sip.status_flips.map((f, i) => <li key={i}>{f.school_name}: {f.previous_status} → {f.current_status}</li>)}</ul>
          </div>
          <div>
            <h4>High Deviation Subjects (&gt;10%)</h4>
            <ul>{sip.high_deviation_subjects.map((s, i) => <li key={i}>{s.school_name} — {s.subject_name}: {s.deviation_pct.toFixed(1)}%</li>)}</ul>
          </div>
          <div>
            <h4>High Substitution Faculty (&gt;2)</h4>
            <ul>{sip.high_substitution_faculty.map((f, i) => <li key={i}>{f.faculty_name}: {f.substitution_count} substitutions</li>)}</ul>
          </div>
          <div>
            <h4>Stale Tickets (&gt;14 days)</h4>
            <ul>{sip.stale_tickets.map((t) => <li key={t.ticket_id}>{t.title} — {t.days_open} days open</li>)}</ul>
          </div>
        </div>
      ) : <p>No data.</p>}
    </section>
  );
}
