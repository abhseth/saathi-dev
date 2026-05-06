import React from "react";
import { SchoolContactsBar } from "../../components/SchoolContactsBar";
import { TimetableContextPanel } from "./TimetableContextPanel";
import { formatTimestamp, getSlaState, formatSlaCountdown, formatBytes, formatField } from "../../formatters";
import { statuses, priorities, queues, gradeLevels, programTracks, issueCategories } from "../../constants";
import type {
  Ticket,
  School,
  Student,
  CommunicationTemplate,
  TicketAttachment,
  TicketComment,
  TicketHistory,
  ReplyDraft,
  TicketEditDraft,
  TicketChanges,
  Status,
  Queue,
  Priority,
  WeeklyTimetableSlot,
  LectureSession,
} from "../../types";

type TicketDetailProps = {
  assigneeDraft: string;
  assigneeWorkload: Record<string, number>;
  attachments: TicketAttachment[];
  comments: TicketComment[];
  editDraft: TicketEditDraft;
  history: TicketHistory[];
  isConfirmingDelete: boolean;
  isEditing: boolean;
  reply: ReplyDraft;
  selected: Ticket | null;
  schools: School[];
  students: Student[];
  templates: CommunicationTemplate[];
  weeklySlots: WeeklyTimetableSlot[];
  lectureSessions: LectureSession[];
  onAddComment: (isInternal: boolean) => void;
  onAssigneeDraftChange: (value: string) => void;
  onCancelDelete: () => void;
  onCancelEdit: () => void;
  onConfirmDelete: () => void;
  onEditDraftChange: React.Dispatch<React.SetStateAction<TicketEditDraft>>;
  onReplyChange: React.Dispatch<React.SetStateAction<ReplyDraft>>;
  onRequestDelete: () => void;
  onRequestEdit: () => void;
  onSaveEdits: (event: React.FormEvent<HTMLFormElement>) => void;
  onUpdateCommentStatus: (id: number, deliveryStatus: string, nextFollowUpDue?: string) => void;
  onUpdateTicket: (changes: TicketChanges) => void;
  onViewFullTimetable: (schoolId: number) => void;
};

export function TicketDetail(props: TicketDetailProps) {
  const {
    assigneeDraft,
    assigneeWorkload,
    attachments,
    comments,
    editDraft,
    history,
    isConfirmingDelete,
    isEditing,
    reply,
    selected,
    schools,
    students,
    templates,
    onAddComment,
    onAssigneeDraftChange,
    onCancelDelete,
    onCancelEdit,
    onConfirmDelete,
    onEditDraftChange,
    onReplyChange,
    onRequestDelete,
    onRequestEdit,
    onSaveEdits,
    onUpdateCommentStatus,
    onUpdateTicket,
    weeklySlots,
    lectureSessions,
    onViewFullTimetable,
  } = props;

  const [detailTab, setDetailTab] = React.useState<"details" | "work">("details");
  const [showContext, setShowContext] = React.useState(true);

  const slaState = selected ? getSlaState(selected) : null;

  function statusActions(status: Status): Array<{ label: string; next: Status; variant: "primary" | "secondary" }> {
    switch (status) {
      case "Open":
        return [
          { label: "Start work", next: "In Progress", variant: "primary" },
          { label: "Resolve", next: "Resolved", variant: "secondary" },
          { label: "Close", next: "Closed", variant: "secondary" },
        ];
      case "In Progress":
        return [
          { label: "Resolve", next: "Resolved", variant: "primary" },
          { label: "Close", next: "Closed", variant: "secondary" },
          { label: "Set pending", next: "Pending", variant: "secondary" },
        ];
      case "Pending":
        return [
          { label: "Resume", next: "In Progress", variant: "primary" },
          { label: "Resolve", next: "Resolved", variant: "secondary" },
          { label: "Close", next: "Closed", variant: "secondary" },
        ];
      case "Resolved":
        return [
          { label: "Reopen", next: "Open", variant: "secondary" },
          { label: "Close", next: "Closed", variant: "secondary" },
        ];
      case "Closed":
        return [{ label: "Reopen", next: "Open", variant: "secondary" }];
      default:
        return [];
    }
  }

  return (
    <section className="ticket-detail" aria-label="Ticket detail">
      {selected ? (
        <>
          {/* ── Header ── */}
          <div className="detail-header">
            <div className="detail-header-title">
              <span className="ticket-id">#{selected.id}</span>
              <h2>{selected.title}</h2>
            </div>
          </div>

          {/* ── Summary strip ── */}
          <div className="detail-summary">
            <div className="detail-summary-item">
              <span className="detail-summary-label">Status</span>
              <span className={`status-chip status-${selected.status.toLowerCase().replace(/\s/g, "-")}`}>
                {selected.status}
              </span>
            </div>
            <div className="detail-summary-item">
              <span className="detail-summary-label">Priority</span>
              <span className={`priority-badge priority-${selected.priority.toLowerCase()}`}>{selected.priority}</span>
            </div>
            <div className="detail-summary-item">
              <span className="detail-summary-label">Assignee</span>
              <span className="detail-summary-value">{selected.assignee || "Unassigned"}</span>
            </div>
            <div className="detail-summary-item">
              <span className="detail-summary-label">SLA</span>
              <span className={`sla-state sla-state-${slaState?.toLowerCase().replace(" ", "-") ?? "ok"}`}>
                {slaState}
              </span>
              {selected.sla_due_at ? (
                <span className="sla-countdown">{formatSlaCountdown(selected.sla_due_at)}</span>
              ) : null}
            </div>
          </div>

          {/* ── Quick actions ── */}
          <div className="detail-quick-actions">
            {statusActions(selected.status).map((action) => (
              <button
                key={action.next}
                type="button"
                className={`quick-action quick-action--${action.variant}`}
                onClick={() => onUpdateTicket({ status: action.next })}
              >
                {action.label}
              </button>
            ))}
            <button
              type="button"
              className="quick-action quick-action--secondary"
              onClick={() => onUpdateTicket({ assignee: assigneeDraft.trim() || "Service Desk" })}
            >
              Assign to me
            </button>
            <button type="button" className="quick-action quick-action--ghost" onClick={onRequestEdit}>
              Edit details
            </button>
          </div>

          {/* ── Mobile tabs ── */}
          <div className="detail-tabs">
            <button className={`detail-tab${detailTab === "details" ? " active" : ""}`} onClick={() => setDetailTab("details")}>
              Details
            </button>
            <button className={`detail-tab${detailTab === "work" ? " active" : ""}`} onClick={() => setDetailTab("work")}>
              Work
            </button>
          </div>

          {/* ── Details panel ── */}
          <div className={`detail-tab-panel detail-tab-details${detailTab === "details" ? " visible" : ""}`}>
            {/* Description */}
            {isEditing ? (
              <TicketEditForm
                draft={editDraft}
                schools={schools}
                students={students}
                onCancel={onCancelEdit}
                onDraftChange={onEditDraftChange}
                onSubmit={onSaveEdits}
              />
            ) : (
              <div className="detail-description">
                <div className="detail-description-header">
                  <strong>Description</strong>
                  <button type="button" className="ghost-button" onClick={onRequestEdit}>
                    Edit
                  </button>
                </div>
                <p>{selected.description}</p>
                <small className="detail-meta">
                  Requested by {selected.requester} · Created {formatTimestamp(selected.created_at)}
                </small>
              </div>
            )}

            {/* Context metadata */}
            <details className="detail-context" open={showContext} onToggle={(e) => setShowContext(e.currentTarget.open)}>
              <summary>Context</summary>
              <dl className="metadata metadata-compact">
                <div className="metadata-row">
                  <dt>School</dt>
                  <dd>{selected.school_name}</dd>
                </div>
                <div className="metadata-row">
                  <dt>Student</dt>
                  <dd>
                    <span>{selected.student_name}</span>
                  </dd>
                </div>
                <div className="metadata-row">
                  <dt>Grade</dt>
                  <dd>{selected.grade_level}</dd>
                </div>
                <div className="metadata-row">
                  <dt>Program</dt>
                  <dd>{selected.program_track}</dd>
                </div>
                <div className="metadata-row">
                  <dt>Category</dt>
                  <dd>{selected.issue_category}</dd>
                </div>
                <div className="metadata-row">
                  <dt>Queue</dt>
                  <dd>
                    <select
                      value={selected.queue}
                      onChange={(e) => onUpdateTicket({ queue: e.target.value as Queue })}
                    >
                      {queues.map((q) => (
                        <option key={q}>{q}</option>
                      ))}
                    </select>
                  </dd>
                </div>
                <div className="metadata-row">
                  <dt>Assignee</dt>
                  <dd>
                    <input
                      list="assignee-options"
                      value={assigneeDraft}
                      onBlur={() => onUpdateTicket({ assignee: assigneeDraft.trim() || "Unassigned" })}
                      onChange={(e) => onAssigneeDraftChange(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") e.currentTarget.blur();
                      }}
                    />
                    <datalist id="assignee-options">
                      {Object.entries(assigneeWorkload)
                        .sort((a, b) => a[1] - b[1])
                        .map(([name, count]) => (
                          <option key={name} value={name}>
                            {count} open
                          </option>
                        ))}
                    </datalist>
                  </dd>
                </div>
                <div className="metadata-row">
                  <dt>Priority</dt>
                  <dd>
                    <select
                      value={selected.priority}
                      onChange={(e) => onUpdateTicket({ priority: e.target.value as Priority })}
                    >
                      {priorities.map((p) => (
                        <option key={p}>{p}</option>
                      ))}
                    </select>
                  </dd>
                </div>
                <div className="metadata-row">
                  <dt>Escalation</dt>
                  <dd>
                    {selected.escalation_status}
                    {selected.escalated_at ? ` since ${formatTimestamp(selected.escalated_at)}` : ""}
                  </dd>
                </div>
              </dl>
              <SchoolContactsBar school={schools.find((s) => s.id === selected.school_id) ?? null} />
              <TimetableContextPanel
                ticket={selected}
                schools={schools}
                slots={weeklySlots}
                sessions={lectureSessions}
                onViewFullTimetable={onViewFullTimetable}
              />
            </details>

            <AttachmentsPanel attachments={attachments} />

            <HistoryPanel history={history} />
          </div>

          {/* ── Work panel ── */}
          <div className={`detail-tab-panel detail-tab-work${detailTab === "work" ? " visible" : ""}`}>
            <div className="conversation">
              {comments.map((comment) => (
                <article className={comment.is_internal ? "internal-note" : ""} key={comment.id}>
                  <div className="comment-header">
                    <strong>{comment.author}</strong>
                    {comment.is_internal ? <span className="internal-badge">internal</span> : null}
                    <span className="comment-time">{formatTimestamp(comment.created_at)}</span>
                  </div>
                  <div className="comment-meta">
                    <span>{comment.channel}</span>
                    <span>{comment.audience}</span>
                    {comment.recipient_name ? <span>To: {comment.recipient_name}</span> : null}
                    <span className={`delivery-badge delivery-${comment.delivery_status.toLowerCase().replace(/\s/g, "-")}`}>
                      {comment.delivery_status}
                    </span>
                    {comment.last_contacted_at ? <span>Last contact: {formatTimestamp(comment.last_contacted_at)}</span> : null}
                    {comment.next_follow_up_due ? <span>Next follow-up: {formatTimestamp(comment.next_follow_up_due)}</span> : null}
                  </div>
                  {comment.recipient_contact ? <small className="comment-contact">{comment.recipient_contact}</small> : null}
                  <p className="comment-body">{comment.body}</p>
                  {!comment.is_internal ? (
                    <div className="comment-actions">
                      {["Sent", "Failed", "Acknowledged", "Follow-up Due"].map((status) => (
                        <button
                          type="button"
                          className={comment.delivery_status === status ? "secondary-button active-chip" : "secondary-button"}
                          key={status}
                          onClick={() => onUpdateCommentStatus(comment.id, status, comment.next_follow_up_due)}
                        >
                          {status}
                        </button>
                      ))}
                      <label className="follow-up-input">
                        <span>Next follow-up</span>
                        <input
                          type="datetime-local"
                          value={comment.next_follow_up_due ? comment.next_follow_up_due.replace(" ", "T") : ""}
                          onChange={(e) =>
                            onUpdateCommentStatus(
                              comment.id,
                              e.target.value ? "Follow-up Due" : comment.delivery_status,
                              e.target.value,
                            )
                          }
                        />
                      </label>
                    </div>
                  ) : null}
                </article>
              ))}

              <ReplyBox draft={reply} templates={templates} onAddComment={onAddComment} onDraftChange={onReplyChange} />
            </div>
          </div>

          {/* ── Footer / Delete ── */}
          {isConfirmingDelete ? (
            <div className="confirm-delete">
              <span>This permanently deletes the ticket and its notes.</span>
              <div className="confirm-delete-actions">
                <button type="button" className="ghost-button" onClick={onCancelDelete}>
                  Cancel
                </button>
                <button type="button" className="danger-action" onClick={onConfirmDelete}>
                  Confirm Delete
                </button>
              </div>
            </div>
          ) : (
            <div className="detail-footer">
              <button type="button" className="delete-link" onClick={onRequestDelete}>
                Delete ticket
              </button>
            </div>
          )}
        </>
      ) : (
        <p className="empty-state">Create or select a ticket to begin.</p>
      )}
    </section>
  );
}

/* ── Sub-components ── */

type TicketEditFormProps = {
  draft: TicketEditDraft;
  schools: School[];
  students: Student[];
  onCancel: () => void;
  onDraftChange: React.Dispatch<React.SetStateAction<TicketEditDraft>>;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
};

function TicketEditForm({ draft, schools, students, onCancel, onDraftChange, onSubmit }: TicketEditFormProps) {
  const selectedSchool = schools.find((school) => school.id === draft.school_id);
  const studentOptions = selectedSchool
    ? students.filter((student) => student.school_id === selectedSchool.id)
    : students;

  return (
    <form className="edit-panel" onSubmit={onSubmit}>
      <label>
        Title
        <input
          required
          value={draft.title}
          onChange={(e) => onDraftChange((c) => ({ ...c, title: e.target.value }))}
        />
      </label>
      <label>
        Requester
        <input
          required
          value={draft.requester}
          onChange={(e) => onDraftChange((c) => ({ ...c, requester: e.target.value }))}
        />
      </label>
      <label>
        School
        <select
          required
          value={draft.school_id ?? ""}
          onChange={(e) => {
            const school = schools.find((item) => item.id === Number(e.target.value));
            onDraftChange((c) => ({
              ...c,
              school_id: school?.id ?? null,
              school_name: school?.name ?? "",
              student_name: "",
            }));
          }}
        >
          <option value="">Select school</option>
          {schools.map((school) => (
            <option key={school.id} value={school.id}>
              {school.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        Student
        <select
          required
          value={draft.student_name}
          onChange={(e) => {
            const student = studentOptions.find((item) => item.name === e.target.value);
            onDraftChange((c) => ({
              ...c,
              student_name: e.target.value,
              grade_level: student?.grade_level ?? c.grade_level,
              program_track: student?.program_track ?? c.program_track,
            }));
          }}
        >
          <option value="">Select student</option>
          {studentOptions.map((student) => (
            <option key={student.id}>{student.name}</option>
          ))}
        </select>
      </label>
      <label>
        Grade
        <select
          value={draft.grade_level}
          onChange={(e) => onDraftChange((c) => ({ ...c, grade_level: e.target.value }))}
        >
          {gradeLevels.map((grade) => (
            <option key={grade}>{grade}</option>
          ))}
        </select>
      </label>
      <label>
        Program
        <select
          value={draft.program_track}
          onChange={(e) => onDraftChange((c) => ({ ...c, program_track: e.target.value }))}
        >
          {programTracks.map((track) => (
            <option key={track}>{track}</option>
          ))}
        </select>
      </label>
      <label>
        Category
        <select
          value={draft.issue_category}
          onChange={(e) => onDraftChange((c) => ({ ...c, issue_category: e.target.value }))}
        >
          {issueCategories.map((category) => (
            <option key={category}>{category}</option>
          ))}
        </select>
      </label>
      <label>
        Description
        <textarea
          required
          value={draft.description}
          onChange={(e) => onDraftChange((c) => ({ ...c, description: e.target.value }))}
        />
      </label>
      <div className="actions">
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
        <button className="primary-action" type="submit">
          Save Changes
        </button>
      </div>
    </form>
  );
}

type ReplyBoxProps = {
  draft: ReplyDraft;
  templates: CommunicationTemplate[];
  onAddComment: (isInternal: boolean) => void;
  onDraftChange: React.Dispatch<React.SetStateAction<ReplyDraft>>;
};

function ReplyBox({ draft, templates, onAddComment, onDraftChange }: ReplyBoxProps) {
  const activeTemplates = templates.filter((template) => template.is_active);

  return (
    <>
      <div className="reply-box">
        <input
          aria-label="Author"
          value={draft.author}
          onChange={(e) => onDraftChange((c) => ({ ...c, author: e.target.value }))}
        />
        <textarea
          placeholder="Write a reply or internal note"
          value={draft.body}
          onChange={(e) => onDraftChange((c) => ({ ...c, body: e.target.value }))}
        />
        <div className="reply-routing">
          <label>
            Channel
            <select
              value={draft.channel}
              onChange={(e) => onDraftChange((c) => ({ ...c, channel: e.target.value }))}
            >
              <option>Local</option>
              <option>Email</option>
              <option>Phone</option>
              <option>WhatsApp</option>
            </select>
          </label>
          <label>
            Audience
            <select
              value={draft.audience}
              onChange={(e) => onDraftChange((c) => ({ ...c, audience: e.target.value }))}
            >
              <option>School</option>
              <option>Parent</option>
              <option>Student</option>
              <option>Internal</option>
            </select>
          </label>
          <label>
            Recipient
            <input
              placeholder="Name"
              value={draft.recipient_name}
              onChange={(e) => onDraftChange((c) => ({ ...c, recipient_name: e.target.value }))}
            />
          </label>
          <label>
            Contact
            <input
              placeholder="Email or mobile"
              value={draft.recipient_contact}
              onChange={(e) => onDraftChange((c) => ({ ...c, recipient_contact: e.target.value }))}
            />
          </label>
          <label>
            Next follow-up
            <input
              type="datetime-local"
              value={draft.next_follow_up_due}
              onChange={(e) => onDraftChange((c) => ({ ...c, next_follow_up_due: e.target.value }))}
            />
          </label>
        </div>
        <label>
          Use template
          <select
            value=""
            onChange={(e) => {
              const template = activeTemplates.find((item) => item.id === Number(e.target.value));
              if (template) {
                onDraftChange((c) => ({
                  ...c,
                  body: template.body,
                  audience: template.audience || c.audience,
                }));
              }
            }}
          >
            <option value="">Choose a template</option>
            {activeTemplates.map((template) => (
              <option key={template.id} value={template.id}>
                {template.name} ({template.audience})
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="actions">
        <button
          onClick={() => onDraftChange((c) => ({ ...c, audience: "Internal", channel: "Internal Note" }))}
        >
          Mark Internal
        </button>
        <button onClick={() => onAddComment(true)}>Internal Note</button>
        <button className="primary-action" onClick={() => onAddComment(false)}>
          Send Reply
        </button>
      </div>
    </>
  );
}

type AttachmentsPanelProps = {
  attachments: TicketAttachment[];
};

function AttachmentsPanel({
  attachments,
}: AttachmentsPanelProps) {
  return (
    <details className="attachments-panel">
      <summary>Attachments ({attachments.length})</summary>
      {attachments.length > 0 ? (
        attachments.map((attachment) => (
          <div className="attachment-row" key={attachment.id}>
            <div className="attachment-title">
              <strong>{attachment.original_filename}</strong>
            </div>
            <span>{formatBytes(attachment.size_bytes)}</span>
            <small>
              {attachment.uploaded_by} - {formatTimestamp(attachment.created_at)}
            </small>
            <code>{attachment.stored_path}</code>
          </div>
        ))
      ) : (
        <p className="empty-state compact">No attachments yet.</p>
      )}
    </details>
  );
}

type HistoryPanelProps = {
  history: TicketHistory[];
};

function HistoryPanel({ history }: HistoryPanelProps) {
  return (
    <details className="history-panel">
      <summary>History ({history.length})</summary>
      {history.length > 0 ? (
        history.map((item) => (
          <div className="history-row" key={item.id}>
            <strong>{formatField(item.field)}</strong>
            <span>
              {item.old_value || "Blank"} {"->"} {item.new_value || "Blank"}
            </span>
            <small>
              {item.actor} - {formatTimestamp(item.created_at)}
            </small>
          </div>
        ))
      ) : (
        <p className="empty-state compact">No history recorded yet.</p>
      )}
    </details>
  );
}
