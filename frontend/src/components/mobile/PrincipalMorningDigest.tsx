import React from "react";
import type { Ticket, FacultyTodaySession } from "../../types";

type PrincipalMorningDigestProps = {
  tickets: Ticket[];
  sessions: FacultyTodaySession[];
  onMarkAllReviewed: () => void;
  onOpenTicket?: (ticketId: number) => void;
  onFindSubstitutes?: () => void;
};

export function PrincipalMorningDigest({ tickets, sessions, onMarkAllReviewed, onOpenTicket, onFindSubstitutes }: PrincipalMorningDigestProps) {
  const urgentTickets = tickets.filter(
    (t) => t.priority === "High" || t.priority === "Critical" || t.escalation_status === "Escalated"
  );
  const cancelledSessions = sessions.filter((s) => s.status === "Cancelled");
  const substitutedSessions = sessions.filter((s) => s.status === "Substituted");
  const periodsNeedingSubs = cancelledSessions.length + substitutedSessions.length;
  const scheduledSessions = sessions.filter((s) => s.status === "Scheduled").length;

  return (
    <div className="mobile-digest">
      <div className="digest-header">
        <h2>Start Your Day</h2>
        <button type="button" className="text-btn" onClick={onMarkAllReviewed}>
          Mark reviewed
        </button>
      </div>

      <div className="digest-cards">
        <div className="digest-card alert">
          <span className="digest-number">{periodsNeedingSubs}</span>
          <span className="digest-label">Needing Substitutes</span>
        </div>
        <div className="digest-card warning">
          <span className="digest-number">{urgentTickets.length}</span>
          <span className="digest-label">Urgent Tickets</span>
        </div>
        <div className="digest-card">
          <span className="digest-number">{scheduledSessions}</span>
          <span className="digest-label">Scheduled Sessions</span>
        </div>
      </div>

      {periodsNeedingSubs > 0 && onFindSubstitutes && (
        <div className="digest-actions">
          <button type="button" className="primary-action" onClick={onFindSubstitutes}>
            Find Substitutes
          </button>
        </div>
      )}

      {urgentTickets.length > 0 && (
        <div className="digest-section">
          <h3>Urgent Tickets</h3>
          {urgentTickets.slice(0, 5).map((t) => (
            <button
              key={t.id}
              type="button"
              className="digest-row"
              onClick={() => onOpenTicket?.(t.id)}
            >
              <div className="digest-row-content">
                <span className="digest-row-title">{t.title}</span>
                <span className="digest-row-meta">{t.assignee || "Unassigned"} · {t.school_name}</span>
              </div>
              <span className={`badge ${t.priority.toLowerCase()}`}>{t.priority}</span>
            </button>
          ))}
        </div>
      )}

      {periodsNeedingSubs > 0 && (
        <div className="digest-section">
          <h3>Schedule Changes</h3>
          {cancelledSessions.slice(0, 3).map((s) => (
            <div key={s.session_id} className="digest-row">
              <span className="digest-row-title">{s.subject_name} — {s.batch_id || s.grade_level}</span>
              <span className="badge cancelled">Cancelled</span>
            </div>
          ))}
          {substitutedSessions.slice(0, 3).map((s) => (
            <div key={s.session_id} className="digest-row">
              <span className="digest-row-title">{s.subject_name} — {s.batch_id || s.grade_level}</span>
              <span className="badge substituted">Substituted</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
