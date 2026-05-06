import React from "react";
import { api, download } from "../api";
import { filterTickets } from "../ticketFilters";
import { getSlaState } from "../formatters";
import type {
  CommunicationTemplate,
  CreateTicketDraft,
  CurrentUser,
  Filter,
  LectureSession,
  Paginated,
  ProgramScopeFilters,
  ReplyDraft,
  School,
  Student,
  Ticket,
  TicketAttachment,
  TicketChanges,
  TicketComment,
  TicketEditDraft,
  TicketHistory,
  WeeklyTimetableSlot,
} from "../types";

// ── Empty draft constants ───────────────────────────────────────────────────

const emptyTicketDraft: CreateTicketDraft = {
  title: "",
  description: "",
  requester: "",
  priority: "Medium",
  school_id: null,
  school_name: "",
  student_name: "",
  grade_level: "Grade 10",
  program_track: "Integrated STEM",
  issue_category: "Academic Support",
};

const emptyTicketEditDraft: TicketEditDraft = {
  title: "",
  description: "",
  requester: "",
  school_id: null,
  school_name: "",
  student_name: "",
  grade_level: "",
  program_track: "",
  issue_category: "",
};

const emptyProgramScopeFilters: ProgramScopeFilters = {
  school_name: "",
  grade_level: "",
  program_track: "",
  issue_category: "",
  queue: "",
};

// ── Utility ─────────────────────────────────────────────────────────────────

function uniqueTicketValues(tickets: Ticket[], field: keyof Ticket): string[] {
  return Array.from(new Set(tickets.map((t) => t[field]).filter(Boolean))) as string[];
}

// ── Interfaces ──────────────────────────────────────────────────────────────

export interface UseTicketStateOptions {
  currentUser: CurrentUser | null;
  schools: School[];
  students: Student[];
  communicationTemplates: CommunicationTemplate[];
  weeklySlots: WeeklyTimetableSlot[];
  lectureSessions: LectureSession[];
  isMobile: boolean;
  onError: (msg: string) => void;
  onNotice: (msg: string) => void;
  onLoadAuditLog: () => Promise<void>;
  onSetMobileView?: (view: "home" | "work" | "detail") => void;
  onSetSectionTool?: (tool: string | null) => void;
}

export interface UseTicketStateReturn {
  // Core state
  tickets: Ticket[];
  ticketsLoading: boolean;
  comments: TicketComment[];
  allComments: TicketComment[];
  history: TicketHistory[];
  attachments: TicketAttachment[];
  selectedId: number | null;
  selected: Ticket | null;

  // Filter / search state
  activeFilter: Filter;
  programScopeFilters: ProgramScopeFilters;
  search: string;
  dateFrom: string;
  dateTo: string;

  // UI modal / form state
  newBreachCount: number;
  isCreating: boolean;
  isEditing: boolean;
  isConfirmingDelete: boolean;
  draft: CreateTicketDraft;
  reply: ReplyDraft;
  assigneeDraft: string;
  editDraft: TicketEditDraft;
  // Derived
  visibleTickets: Ticket[];
  assigneeWorkload: Record<string, number>;
  latestUpdate: string;
  filterCounts: Record<Filter, number>;
  openCount: number;
  unassignedCount: number;
  pendingSlaCount: number;
  activeSchoolCount: number;
  activeQueueCount: number;
  escalatedCount: number;
  schoolOptions: string[];

  // Setters
  setActiveFilter: (f: Filter) => void;
  setProgramScopeFilters: React.Dispatch<React.SetStateAction<ProgramScopeFilters>>;
  setSearch: (s: string) => void;
  setDateFrom: (d: string) => void;
  setDateTo: (d: string) => void;
  setIsCreating: (v: boolean) => void;
  setDraft: React.Dispatch<React.SetStateAction<CreateTicketDraft>>;
  setReply: React.Dispatch<React.SetStateAction<ReplyDraft>>;
  setAssigneeDraft: (v: string) => void;
  setEditDraft: React.Dispatch<React.SetStateAction<TicketEditDraft>>;
  setNewBreachCount: (v: number) => void;
  setIsEditing: (v: boolean) => void;
  setIsConfirmingDelete: (v: boolean) => void;
  setSelectedId: (id: number | null) => void;

  // Actions
  loadTickets: (preferredId?: number) => Promise<void>;
  loadAllComments: () => Promise<void>;
  loadHistory: (ticketId: number) => Promise<void>;
  quickUpdateTicket: (id: number, changes: TicketChanges) => Promise<void>;
  createTicket: (event: React.FormEvent<HTMLFormElement>) => Promise<void>;
  updateTicket: (changes: TicketChanges) => Promise<void>;
  saveTicketEdits: (event: React.FormEvent<HTMLFormElement>) => Promise<void>;
  deleteSelectedTicket: () => Promise<void>;
  addComment: (isInternal: boolean) => Promise<void>;
  updateCommentStatus: (id: number, deliveryStatus: string, nextFollowUpDue?: string) => Promise<void>;
  selectTicket: (ticketId: number) => void;
  openTicketFromCommunication: (ticketId: number) => void;
  exportTicketCsvBundle: () => Promise<void>;
  exportCommunicationCsv: () => Promise<void>;
  refreshSlaStatus: () => Promise<void>;
}

// ── Hook ────────────────────────────────────────────────────────────────────

export function useTicketState(options: UseTicketStateOptions): UseTicketStateReturn {
  const {
    currentUser,
    schools,
    onError,
    onNotice,
    onLoadAuditLog,
    onSetMobileView = () => {},
    onSetSectionTool = () => {},
    isMobile,
  } = options;

  // ── State ─────────────────────────────────────────────────────────────────
  const [tickets, setTickets] = React.useState<Ticket[]>([]);
  const [ticketsLoading, setTicketsLoading] = React.useState(false);
  const [comments, setComments] = React.useState<TicketComment[]>([]);
  const [allComments, setAllComments] = React.useState<TicketComment[]>([]);
  const [history, setHistory] = React.useState<TicketHistory[]>([]);
  const [attachments, setAttachments] = React.useState<TicketAttachment[]>([]);
  const [selectedId, setSelectedId] = React.useState<number | null>(null);
  const [activeFilter, setActiveFilter] = React.useState<Filter>(() => {
    try { return (localStorage.getItem("td:activeFilter") as Filter) || "Inbox"; } catch { return "Inbox"; }
  });
  const [programScopeFilters, setProgramScopeFilters] = React.useState<ProgramScopeFilters>(() => {
    try { return JSON.parse(localStorage.getItem("td:scopeFilters") || "null") ?? emptyProgramScopeFilters; } catch { return emptyProgramScopeFilters; }
  });
  const [search, setSearch] = React.useState(() => {
    try { return localStorage.getItem("td:search") || ""; } catch { return ""; }
  });
  const [dateFrom, setDateFrom] = React.useState(() => {
    try { return localStorage.getItem("td:dateFrom") || ""; } catch { return ""; }
  });
  const [dateTo, setDateTo] = React.useState(() => {
    try { return localStorage.getItem("td:dateTo") || ""; } catch { return ""; }
  });
  const [newBreachCount, setNewBreachCount] = React.useState(0);
  const knownBreachedIds = React.useRef(new Set<number>());
  const [isCreating, setIsCreating] = React.useState(false);
  const [isEditing, setIsEditing] = React.useState(false);
  const [isConfirmingDelete, setIsConfirmingDelete] = React.useState(false);
  const [draft, setDraft] = React.useState<CreateTicketDraft>(emptyTicketDraft);
  const [reply, setReply] = React.useState<ReplyDraft>({
    author: "Service Desk",
    body: "",
    is_internal: false,
    channel: "Local",
    audience: "School",
    recipient_name: "",
    recipient_contact: "",
    next_follow_up_due: "",
  });
  const [assigneeDraft, setAssigneeDraft] = React.useState("");
  const [editDraft, setEditDraft] = React.useState<TicketEditDraft>(emptyTicketEditDraft);
  // ── Derived values ────────────────────────────────────────────────────────
  const selected = React.useMemo(
    () => tickets.find((ticket) => ticket.id === selectedId) ?? tickets[0] ?? null,
    [tickets, selectedId]
  );

  const visibleTickets = React.useMemo(() => {
    return filterTickets(tickets, activeFilter, search, programScopeFilters, currentUser?.display_name, dateFrom, dateTo);
  }, [tickets, activeFilter, search, programScopeFilters, currentUser?.display_name, dateFrom, dateTo]);

  const assigneeWorkload = React.useMemo(() => {
    return tickets
      .filter((t) => !["Resolved", "Closed"].includes(t.status) && t.assignee !== "Unassigned")
      .reduce<Record<string, number>>((acc, t) => {
        acc[t.assignee] = (acc[t.assignee] ?? 0) + 1;
        return acc;
      }, {});
  }, [tickets]);

  const latestUpdate = React.useMemo(
    () => tickets.reduce((latest, t) => (t.updated_at > latest ? t.updated_at : latest), ""),
    [tickets]
  );

  const filterCounts = React.useMemo(() => ({
    Inbox: tickets.filter((t) => t.status !== "Closed").length,
    "My Tickets": tickets.filter(
      (t) =>
        t.status !== "Closed" &&
        (currentUser ? t.assignee === currentUser.display_name : t.assignee !== "Unassigned"),
    ).length,
    Unassigned: tickets.filter((t) => t.assignee === "Unassigned" && t.status !== "Closed").length,
    "Pending SLA": tickets.filter(
      (t) =>
        ["Breached", "At Risk"].includes(getSlaState(t)) &&
        !["Resolved", "Closed"].includes(t.status),
    ).length,
    Escalated: tickets.filter(
      (t) => t.escalation_status === "Escalated" && !["Resolved", "Closed"].includes(t.status),
    ).length,
    Resolved: tickets.filter((t) => ["Resolved", "Closed"].includes(t.status)).length,
  } satisfies Record<Filter, number>), [tickets, currentUser]);

  const openCount = React.useMemo(
    () => tickets.filter((ticket) => ticket.status !== "Closed").length,
    [tickets]
  );
  const unassignedCount = React.useMemo(
    () => tickets.filter((ticket) => ticket.assignee === "Unassigned").length,
    [tickets]
  );
  const pendingSlaCount = React.useMemo(
    () =>
      tickets.filter(
        (ticket) =>
          ["Breached", "At Risk"].includes(getSlaState(ticket)) &&
          !["Resolved", "Closed"].includes(ticket.status),
      ).length,
    [tickets]
  );
  const activeSchoolCount = React.useMemo(
    () =>
      new Set(
        tickets
          .filter((ticket) => ticket.status !== "Closed")
          .map((ticket) => ticket.school_name)
          .filter(Boolean),
      ).size,
    [tickets]
  );
  const activeQueueCount = React.useMemo(
    () =>
      new Set(
        tickets
          .filter((ticket) => ticket.status !== "Closed")
          .map((ticket) => ticket.queue)
          .filter(Boolean),
      ).size,
    [tickets]
  );
  const escalatedCount = React.useMemo(
    () =>
      tickets.filter(
        (ticket) => ticket.escalation_status === "Escalated" && ticket.status !== "Closed",
      ).length,
    [tickets]
  );
  const schoolOptions = React.useMemo(
    () => uniqueTicketValues(tickets, "school_name"),
    [tickets]
  );

  // ── Effects ───────────────────────────────────────────────────────────────

  // Sync reply author with current user
  React.useEffect(() => {
    setReply((current) => ({
      ...current,
      author: currentUser?.display_name ?? "Service Desk",
    }));
  }, [currentUser?.display_name]);

  // SLA breach detection
  React.useEffect(() => {
    const currentBreached = tickets
      .filter(
        (t) =>
          getSlaState(t) === "Breached" && !["Resolved", "Closed"].includes(t.status),
      )
      .map((t) => t.id);
    const newIds = currentBreached.filter((id) => !knownBreachedIds.current.has(id));
    if (newIds.length > 0) {
      setNewBreachCount((c) => c + newIds.length);
      newIds.forEach((id) => knownBreachedIds.current.add(id));
    }
  }, [tickets]);

  // Document title sync
  React.useEffect(() => {
    const open = tickets.filter((t) => t.status !== "Closed").length;
    const breached = tickets.filter(
      (t) => getSlaState(t) === "Breached" && !["Resolved", "Closed"].includes(t.status),
    ).length;
    document.title = open > 0
      ? `Ticketing Desktop (${open} open${breached > 0 ? `, ${breached} breached` : ""})`
      : "Ticketing Desktop";
  }, [tickets]);

  // Filter persistence
  React.useEffect(() => {
    try {
      localStorage.setItem("td:activeFilter", activeFilter);
      localStorage.setItem("td:scopeFilters", JSON.stringify(programScopeFilters));
      localStorage.setItem("td:search", search);
      localStorage.setItem("td:dateFrom", dateFrom);
      localStorage.setItem("td:dateTo", dateTo);
    } catch {}
  }, [activeFilter, programScopeFilters, search, dateFrom, dateTo]);

  // Reply body persistence
  React.useEffect(() => {
    if (selectedId) {
      try { localStorage.setItem(`td:reply:${selectedId}`, reply.body); } catch {}
    }
  }, [selectedId, reply.body]);

  // Selected-ticket side-effects
  React.useEffect(() => {
    if (selected?.id) {
      void loadComments(selected.id);
      void loadHistory(selected.id);
      loadAttachments(selected.id);
      setAssigneeDraft(selected.assignee);
      setEditDraft({
        title: selected.title,
        description: selected.description,
        requester: selected.requester,
        school_id: selected.school_id,
        school_name: selected.school_name,
        student_name: selected.student_name,
        grade_level: selected.grade_level,
        program_track: selected.program_track,
        issue_category: selected.issue_category,
      });
      const school = schools.find((item) => item.id === selected.school_id);
      const savedBody = (() => { try { return localStorage.getItem(`td:reply:${selected.id}`) ?? ""; } catch { return ""; } })();
      setReply((current) => ({
        ...current,
        body: savedBody,
        audience: "School",
        channel: current.channel === "Internal Note" ? "Local" : current.channel,
        recipient_name: school?.school_spoc_name || school?.principal_name || selected.requester,
        recipient_contact:
          school?.school_spoc_email ||
          school?.school_spoc_mobile ||
          school?.principal_email ||
          school?.principal_mobile ||
          "",
        next_follow_up_due: "",
      }));
    } else {
      setComments([]);
      setHistory([]);
      setAttachments([]);
      setAssigneeDraft("");
      setEditDraft(emptyTicketEditDraft);
      setReply((current) => ({
        ...current,
        body: "",
        audience: "School",
        channel: "Local",
        recipient_name: "",
        recipient_contact: "",
        next_follow_up_due: "",
      }));
    }
    setIsEditing(false);
    setIsConfirmingDelete(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    selected?.id,
    selected?.assignee,
    selected?.description,
    selected?.requester,
    selected?.title,
    selected?.school_id,
    schools,
  ]);

  // ── Data loaders ──────────────────────────────────────────────────────────

  const loadTickets = React.useCallback(async (preferredId?: number) => {
    setTicketsLoading(true);
    try {
      const result = await api<Paginated<Ticket>>("refresh_escalations");
      const items = result.items;
      setTickets(items);
      setSelectedId((currentId) => {
        if (preferredId && items.some((ticket) => ticket.id === preferredId)) {
          return preferredId;
        }
        if (currentId && items.some((ticket) => ticket.id === currentId)) {
          return currentId;
        }
        return items[0]?.id ?? null;
      });
      onError("");
    } catch (caught) {
      onError(String(caught));
    } finally {
      setTicketsLoading(false);
    }
  }, [onError]);

  const loadComments = React.useCallback(async (ticketId: number) => {
    try {
      setComments(await api<TicketComment[]>("list_comments", { ticketId }));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAllComments = React.useCallback(async () => {
    try {
      const commentResult = await api<Paginated<TicketComment>>("list_all_comments");
      setAllComments(commentResult.items);
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadHistory = React.useCallback(async (ticketId: number) => {
    try {
      setHistory(await api<TicketHistory[]>("list_history", { ticketId }));
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const loadAttachments = React.useCallback((_ticketId: number) => {
    setAttachments([]);
  }, []);

  // ── Mutations ─────────────────────────────────────────────────────────────

  const quickUpdateTicket = React.useCallback(async (id: number, changes: TicketChanges) => {
    const ticket = tickets.find((t) => t.id === id);
    if (!ticket) return;
    const input = {
      id: ticket.id,
      title: ticket.title,
      description: ticket.description,
      requester: ticket.requester,
      status: changes.status ?? ticket.status,
      priority: ticket.priority,
      assignee: changes.assignee ?? ticket.assignee,
      queue: ticket.queue,
      school_id: ticket.school_id,
      school_name: ticket.school_name,
      student_name: ticket.student_name,
      grade_level: ticket.grade_level,
      program_track: ticket.program_track,
      issue_category: ticket.issue_category,
    };
    try {
      const updated = await api<Ticket>("update_ticket", { input });
      setTickets((current) => current.map((t) => (t.id === updated.id ? updated : t)));
      if (selected?.id === id) await loadHistory(id);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [tickets, selected?.id, loadHistory, onError]);

  const createTicket = React.useCallback(async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    try {
      const ticket = await api<Ticket>("create_ticket", { input: draft });
      setDraft(emptyTicketDraft);
      setSearch("");
      setActiveFilter("Inbox");
      setIsCreating(false);
      await loadTickets(ticket.id);
      await onLoadAuditLog();
    } catch (caught) {
      onError(String(caught));
    }
  }, [draft, loadTickets, onLoadAuditLog, onError]);

  const updateTicket = React.useCallback(async (changes: TicketChanges) => {
    if (!selected) return;
    const input = {
      id: selected.id,
      title: changes.title ?? selected.title,
      description: changes.description ?? selected.description,
      requester: changes.requester ?? selected.requester,
      status: changes.status ?? selected.status,
      priority: changes.priority ?? selected.priority,
      assignee: changes.assignee ?? selected.assignee,
      queue: changes.queue ?? selected.queue,
      school_id: changes.school_id ?? selected.school_id,
      school_name: changes.school_name ?? selected.school_name,
      student_name: changes.student_name ?? selected.student_name,
      grade_level: changes.grade_level ?? selected.grade_level,
      program_track: changes.program_track ?? selected.program_track,
      issue_category: changes.issue_category ?? selected.issue_category,
    };
    try {
      const updated = await api<Ticket>("update_ticket", { input });
      setTickets((currentTickets) =>
        currentTickets.map((ticket) => (ticket.id === updated.id ? updated : ticket)),
      );
      await loadHistory(updated.id);
      await onLoadAuditLog();
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [selected, loadHistory, onLoadAuditLog, onError]);

  const saveTicketEdits = React.useCallback(async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await updateTicket({
      title: editDraft.title.trim(),
      description: editDraft.description.trim(),
      requester: editDraft.requester.trim(),
      school_id: editDraft.school_id,
      school_name: editDraft.school_name.trim(),
      student_name: editDraft.student_name.trim(),
      grade_level: editDraft.grade_level.trim(),
      program_track: editDraft.program_track.trim(),
      issue_category: editDraft.issue_category.trim(),
    });
    setIsEditing(false);
  }, [updateTicket, editDraft]);

  const deleteSelectedTicket = React.useCallback(async () => {
    if (!selected) return;
    try {
      await api("delete_ticket", { id: selected.id });
      setIsConfirmingDelete(false);
      await loadTickets();
    } catch (caught) {
      onError(String(caught));
    }
  }, [selected, loadTickets, onError]);

  const addComment = React.useCallback(async (isInternal: boolean) => {
    if (!selected || !reply.body.trim()) return;
    try {
      await api<TicketComment>("add_comment", {
        input: {
          ticket_id: selected.id,
          author: reply.author,
          body: reply.body,
          is_internal: isInternal,
          channel: isInternal ? "Internal Note" : reply.channel,
          audience: isInternal ? "Internal" : reply.audience,
          recipient_name: isInternal ? "" : reply.recipient_name,
          recipient_contact: isInternal ? "" : reply.recipient_contact,
          next_follow_up_due: isInternal ? null : reply.next_follow_up_due || null,
        },
      });
      try { localStorage.removeItem(`td:reply:${selected.id}`); } catch {}
      setReply((current) => ({
        ...current,
        body: "",
        is_internal: isInternal,
      }));
      await Promise.all([loadComments(selected.id), loadAllComments(), loadTickets(selected.id)]);
    } catch (caught) {
      onError(String(caught));
    }
  }, [selected, reply, loadComments, loadAllComments, loadTickets, onError]);

  const updateCommentStatus = React.useCallback(async (id: number, deliveryStatus: string, nextFollowUpDue = "") => {
    if (!selected) return;
    try {
      await api<TicketComment>("update_comment_status", {
        input: {
          id,
          delivery_status: deliveryStatus,
          next_follow_up_due: nextFollowUpDue || null,
        },
      });
      await Promise.all([loadComments(selected.id), loadAllComments(), loadTickets(selected.id)]);
      onNotice(`Communication marked ${deliveryStatus}.`);
      onError("");
    } catch (caught) {
      onError(String(caught));
    }
  }, [selected, loadComments, loadAllComments, loadTickets, onNotice, onError]);

  // ── Selection & navigation ────────────────────────────────────────────────

  const selectTicket = React.useCallback((ticketId: number) => {
    setSelectedId(ticketId);
    if (isMobile) {
      onSetMobileView("detail");
    }
  }, [isMobile, onSetMobileView]);

  const openTicketFromCommunication = React.useCallback((ticketId: number) => {
    setSelectedId(ticketId);
    onSetSectionTool(null);
    if (isMobile) {
      onSetMobileView("detail");
    }
  }, [isMobile, onSetMobileView, onSetSectionTool]);

  // ── Export / attachment stubs ─────────────────────────────────────────────

  const exportTicketCsvBundle = React.useCallback(async () => {
    try {
      await download("/export/tickets.csv", "tickets-export.csv");
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  const refreshSlaStatus = React.useCallback(async () => {
    try {
      await api("refresh_sla_status");
      await loadTickets();
      onNotice("SLA status refreshed");
    } catch (caught) {
      onError(String(caught));
    }
  }, [loadTickets, onError, onNotice]);

  const exportCommunicationCsv = React.useCallback(async () => {
    try {
      const result = await api<{ path: string }>("export_communications_csv");
      if (result.path) {
        await download(result.path, "communications-export.csv");
      }
    } catch (caught) {
      onError(String(caught));
    }
  }, [onError]);

  // ── Return ────────────────────────────────────────────────────────────────

  return {
    // Core state
    tickets,
    ticketsLoading,
    comments,
    allComments,
    history,
    attachments,
    selectedId,
    selected,

    // Filter / search state
    activeFilter,
    programScopeFilters,
    search,
    dateFrom,
    dateTo,

    // UI modal / form state
    newBreachCount,
    isCreating,
    isEditing,
    isConfirmingDelete,
    draft,
    reply,
    assigneeDraft,
    editDraft,

    // Derived
    visibleTickets,
    assigneeWorkload,
    latestUpdate,
    filterCounts,
    openCount,
    unassignedCount,
    pendingSlaCount,
    activeSchoolCount,
    activeQueueCount,
    escalatedCount,
    schoolOptions,

    // Setters
    setActiveFilter,
    setProgramScopeFilters,
    setSearch,
    setDateFrom,
    setDateTo,
    setIsCreating,
    setDraft,
    setReply,
    setAssigneeDraft,
    setEditDraft,
    setNewBreachCount,
    setIsEditing,
    setIsConfirmingDelete,
    setSelectedId,

    // Actions
    loadTickets,
    loadAllComments,
    loadHistory,
    quickUpdateTicket,
    createTicket,
    updateTicket,
    saveTicketEdits,
    deleteSelectedTicket,
    addComment,
    updateCommentStatus,
    selectTicket,
    openTicketFromCommunication,
    exportTicketCsvBundle,
    exportCommunicationCsv,
    refreshSlaStatus,
  };
}
