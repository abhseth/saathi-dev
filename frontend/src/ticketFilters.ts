import type { Filter, ProgramScopeFilters, Ticket } from "./types";
import { getSlaState } from "./formatters";

export function filterTickets(
  tickets: Ticket[],
  activeFilter: Filter,
  search: string,
  scopeFilters: ProgramScopeFilters = {
    school_name: "",
    grade_level: "",
    program_track: "",
    issue_category: "",
    queue: "",
  },
  currentUserDisplayName?: string,
  dateFrom?: string,
  dateTo?: string,
) {
  const normalizedSearch = search.toLowerCase();

  return tickets.filter((ticket) => {
    const matchesSearch = [
      ticket.title,
      ticket.description,
      ticket.requester,
      ticket.assignee,
      ticket.status,
      ticket.priority,
      ticket.queue,
      ticket.school_name,
      ticket.student_name,
      ticket.grade_level,
      ticket.program_track,
      ticket.issue_category,
      ticket.sla_due_at,
      ticket.escalation_status,
      getSlaState(ticket),
    ]
      .join(" ")
      .toLowerCase()
      .includes(normalizedSearch);

    const matchesFilter =
      activeFilter === "Inbox"
        ? ticket.status !== "Closed"
        : activeFilter === "My Tickets"
          ? currentUserDisplayName
            ? ticket.assignee === currentUserDisplayName && ticket.status !== "Closed"
            : ticket.assignee !== "Unassigned" && ticket.status !== "Closed"
          : activeFilter === "Unassigned"
            ? ticket.assignee === "Unassigned" && ticket.status !== "Closed"
            : activeFilter === "Pending SLA"
              ? ["Breached", "At Risk"].includes(getSlaState(ticket)) &&
                !["Resolved", "Closed"].includes(ticket.status)
              : activeFilter === "Escalated"
                ? ticket.escalation_status === "Escalated" &&
                  !["Resolved", "Closed"].includes(ticket.status)
                : ["Resolved", "Closed"].includes(ticket.status);

    const matchesScope =
      (!scopeFilters.school_name || ticket.school_name === scopeFilters.school_name) &&
      (!scopeFilters.grade_level || ticket.grade_level === scopeFilters.grade_level) &&
      (!scopeFilters.program_track || ticket.program_track === scopeFilters.program_track) &&
      (!scopeFilters.issue_category || ticket.issue_category === scopeFilters.issue_category) &&
      (!scopeFilters.queue || ticket.queue === scopeFilters.queue);

    const ticketDate = ticket.created_at.slice(0, 10);
    const matchesDate =
      (!dateFrom || ticketDate >= dateFrom) &&
      (!dateTo || ticketDate <= dateTo);

    return matchesSearch && matchesFilter && matchesScope && matchesDate;
  });
}
