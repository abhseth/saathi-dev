import React from "react";
import { formatTimestamp, getSlaState } from "../../formatters";
import type { Filter, Ticket, CurrentUser } from "../../types";

type TicketListProps = {
  activeFilter: Filter;
  currentUser: CurrentUser | null;
  selectedId: number | null;
  tickets: Ticket[];
  loading?: boolean;
  onSelectTicket: (id: number) => void;
  onQuickResolve: (id: number) => void;
  onQuickAssign: (id: number) => void;
};

const emptyStateMessages: Record<Filter, string> = {
  Inbox: "No open tickets — everything is resolved.",
  "My Tickets": "No tickets assigned to you right now.",
  Unassigned: "All tickets are assigned.",
  "Pending SLA": "No SLA issues — all tickets are within policy.",
  Escalated: "No escalated tickets.",
  Resolved: "No resolved or closed tickets yet.",
};

function ticketAgeClass(ticket: Ticket): string {
  if (["Resolved", "Closed"].includes(ticket.status)) return "";
  const created = new Date(ticket.created_at.replace(" ", "T"));
  const ageHours = (Date.now() - created.getTime()) / 3_600_000;
  if (ageHours > 72) return "age-old";
  if (ageHours > 24) return "age-aging";
  return "";
}

function SlaBadge({ ticket }: { ticket: Ticket }) {
  const state = getSlaState(ticket);
  if (["Resolved", "Closed"].includes(ticket.status)) return null;
  if (state === "Breached") {
    return (
      <>
        <span className="meta-dot">·</span>
        <span className="sla-breached">Overdue</span>
      </>
    );
  }
  if (state === "At Risk") {
    return (
      <>
        <span className="meta-dot">·</span>
        <span className="sla-at-risk">At Risk</span>
      </>
    );
  }
  return null;
}

export function TicketList({
  activeFilter,
  currentUser,
  selectedId,
  tickets,
  loading,
  onSelectTicket,
  onQuickResolve,
  onQuickAssign,
}: TicketListProps) {
  return (
    <section className="ticket-list" aria-label="Ticket list">
      <div className="section-heading">
        <h1>{activeFilter}</h1>
        <span>{tickets.length} tickets</span>
      </div>
      {loading ? (
        <p className="empty-state">Loading tickets…</p>
      ) : tickets.length > 0 ? (
        tickets.map((ticket) => {
          const isResolved = ["Resolved", "Closed"].includes(ticket.status);
          const isAssignedToMe = currentUser && ticket.assignee === currentUser.display_name;
          return (
            <div
              className={`ticket-row ${ticket.id === selectedId ? "selected" : ""} ${ticketAgeClass(ticket)}`}
              key={ticket.id}
            >
              <button
                type="button"
                className="ticket-row-select"
                onClick={() => onSelectTicket(ticket.id)}
                aria-label={`Open ticket ${ticket.title}`}
              >
                <span className={`priority priority-${ticket.priority.toLowerCase()}`} />
                <span className="ticket-row-info">
                  <span className="ticket-row-title">{ticket.title}</span>
                  <span className="ticket-row-meta">
                    #{ticket.id}
                    <span className="meta-dot">·</span>
                    {ticket.requester}
                    <span className="meta-dot">·</span>
                    {formatTimestamp(ticket.updated_at)}
                  </span>
                  <span className="ticket-row-meta">
                    {ticket.school_name}
                    <span className="meta-dot">·</span>
                    {ticket.queue}
                    <SlaBadge ticket={ticket} />
                  </span>
                </span>
                <span className={`ticket-status ticket-status-${ticket.status.toLowerCase().replace(/\s+/g, "-")}`}>
                  {ticket.status}
                </span>
              </button>
              {!isResolved && (
                <span className="ticket-row-actions">
                  {!isAssignedToMe && currentUser && (
                    <button
                      type="button"
                      className="quick-action"
                      title="Assign to me"
                      onClick={() => onQuickAssign(ticket.id)}
                    >
                      Assign to me
                    </button>
                  )}
                  <button
                    type="button"
                    className="quick-action quick-action-resolve"
                    title="Resolve ticket"
                    onClick={() => onQuickResolve(ticket.id)}
                  >
                    Resolve
                  </button>
                </span>
              )}
            </div>
          );
        })
      ) : (
        <p className="empty-state">{emptyStateMessages[activeFilter] ?? "No tickets match this view."}</p>
      )}
    </section>
  );
}
