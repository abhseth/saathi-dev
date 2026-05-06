import React from "react";
import type { Ticket, TimetableHealthStatus } from "../../types";

type SpocExecutiveSummaryProps = {
  tickets: Ticket[];
  healthData: TimetableHealthStatus[];
  onOpenTicket?: (ticketId: number) => void;
  onViewSchool?: (schoolId: number) => void;
};

export function SpocExecutiveSummary({ tickets, healthData, onOpenTicket, onViewSchool }: SpocExecutiveSummaryProps) {
  const openTickets = tickets.filter((t) => t.status !== "Closed" && t.status !== "Resolved").length;
  const breached = tickets.filter(
    (t) =>
      t.escalation_status === "Escalated" &&
      t.status !== "Closed" &&
      t.status !== "Resolved"
  ).length;
  const atRisk = tickets.filter(
    (t) =>
      t.escalation_status === "At Risk" &&
      t.status !== "Closed" &&
      t.status !== "Resolved"
  ).length;
  const redSchools = healthData.filter((h) => h.status === "Red");
  const amberSchools = healthData.filter((h) => h.status === "Amber");

  const kpis = [
    { label: "Open Tickets", value: openTickets, alert: openTickets > 10 },
    { label: "Breached SLA", value: breached, alert: breached > 0 },
    { label: "At Risk", value: atRisk, alert: atRisk > 0 },
    { label: "Red Schools", value: redSchools.length, alert: redSchools.length > 0 },
  ];

  return (
    <div className="mobile-digest">
      <div className="digest-header">
        <h2>Executive Summary</h2>
      </div>
      <div className="kpi-grid">
        {kpis.map((kpi) => (
          <div key={kpi.label} className={`kpi-card ${kpi.alert ? "alert" : ""}`}>
            <span className="kpi-value">{kpi.value}</span>
            <span className="kpi-label">{kpi.label}</span>
          </div>
        ))}
      </div>

      {(breached > 0 || atRisk > 0) && (
        <div className="digest-section">
          <h3>SLA Concerns</h3>
          {tickets
            .filter((t) => t.escalation_status === "Escalated" && t.status !== "Closed" && t.status !== "Resolved")
            .slice(0, 5)
            .map((t) => (
              <button
                key={t.id}
                type="button"
                className="digest-row"
                onClick={() => onOpenTicket?.(t.id)}
              >
                <span className="digest-row-title">{t.title}</span>
                <span className="badge critical">{t.escalation_status}</span>
              </button>
            ))}
        </div>
      )}

      {(redSchools.length > 0 || amberSchools.length > 0) && (
        <div className="digest-section">
          <h3>School Health</h3>
          {redSchools.slice(0, 5).map((h) => (
            <button
              key={h.school_id}
              type="button"
              className="digest-row"
              onClick={() => onViewSchool?.(h.school_id)}
            >
              <span className="digest-row-title">{h.school_name}</span>
              <span className="badge critical">{h.gaps_count} gaps</span>
            </button>
          ))}
          {amberSchools.slice(0, 3).map((h) => (
            <button
              key={h.school_id}
              type="button"
              className="digest-row"
              onClick={() => onViewSchool?.(h.school_id)}
            >
              <span className="digest-row-title">{h.school_name}</span>
              <span className="badge warning">{h.gaps_count} gaps</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
