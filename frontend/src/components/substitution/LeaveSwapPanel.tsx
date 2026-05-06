import React from "react";
import { api } from "../../api";
import type { LeaveRequest, SwapRequest, CreateLeaveRequestInput, CreateSwapRequestInput, LeaveImpactPreview } from "../../types";

export function LeaveSwapPanel({
  schools,
  faculty,
  currentUser,
}: {
  schools: Array<{ id: number; name: string }>;
  faculty: Array<{ id: number; display_name: string }>;
  currentUser: { id: number; role: string } | null;
}) {
  const [leaveRequests, setLeaveRequests] = React.useState<LeaveRequest[]>([]);
  const [swapRequests, setSwapRequests] = React.useState<SwapRequest[]>([]);
  const [showLeaveForm, setShowLeaveForm] = React.useState(false);
  const [showSwapForm, setShowSwapForm] = React.useState(false);
  const [notice, setNotice] = React.useState("");
  const [approverTab, setApproverTab] = React.useState<"Pending" | "Approved" | "Rejected">("Pending");
  const [impactPreview, setImpactPreview] = React.useState<LeaveImpactPreview | null>(null);
  const [impactForId, setImpactForId] = React.useState<number | null>(null);
  const [rejectingId, setRejectingId] = React.useState<number | null>(null);
  const [rejectReason, setRejectReason] = React.useState("");

  const loadLeaves = React.useCallback(async () => {
    try {
      setLeaveRequests(await api<LeaveRequest[]>("list_leave_requests"));
    } catch (e) {
      console.error("Failed to load leave requests:", e);
    }
  }, []);

  const loadSwaps = React.useCallback(async () => {
    try {
      setSwapRequests(await api<SwapRequest[]>("list_swap_requests"));
    } catch (e) {
      console.error("Failed to load swap requests:", e);
    }
  }, []);

  React.useEffect(() => {
    void loadLeaves();
    void loadSwaps();
  }, [loadLeaves, loadSwaps]);

  async function handleCreateLeave(input: CreateLeaveRequestInput) {
    try {
      await api("create_leave_request", { input });
      setNotice("Leave request created.");
      setShowLeaveForm(false);
      void loadLeaves();
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function handleApproveLeave(id: number) {
    try {
      await api("approve_leave_request", { id });
      setNotice("Leave approved and substitutions triggered.");
      setImpactPreview(null);
      setImpactForId(null);
      void loadLeaves();
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function handleRejectLeave(id: number) {
    if (!rejectReason.trim()) {
      setNotice("Rejection reason is required.");
      return;
    }
    try {
      await api("reject_leave_request", { id, input: { reason: rejectReason.trim() } });
      setNotice("Leave request rejected.");
      setRejectingId(null);
      setRejectReason("");
      void loadLeaves();
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function showImpact(id: number) {
    try {
      const preview = await api<LeaveImpactPreview>("get_leave_impact", { id });
      setImpactPreview(preview);
      setImpactForId(id);
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function handleCreateSwap(input: CreateSwapRequestInput) {
    try {
      await api("create_swap_request", { input });
      setNotice("Swap request sent.");
      setShowSwapForm(false);
      void loadSwaps();
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function handleAcceptSwap(id: number) {
    try {
      await api("accept_swap_request", { id });
      setNotice("Swap accepted and timetable updated.");
      void loadSwaps();
    } catch (e) {
      setNotice(String(e));
    }
  }

  const isFaculty = currentUser?.role === "faculty";
  const canApproveLeave = currentUser?.role === "admin" || currentUser?.role === "aom" || currentUser?.role === "head";

  const myLeaves = leaveRequests.filter((lr) => lr.faculty_user_id === currentUser?.id);
  const pendingLeaves = leaveRequests.filter((lr) => lr.status === "Pending");
  const approvedLeaves = leaveRequests.filter((lr) => lr.status === "Approved");
  const rejectedLeaves = leaveRequests.filter((lr) => lr.status === "Rejected");

  function renderLeaveTable(rows: LeaveRequest[], showActions: boolean) {
    return (
      <table className="data-table compact">
        <thead>
          <tr>
            <th>Faculty</th>
            <th>Dates</th>
            <th>Reason</th>
            <th>Status</th>
            {showActions ? <th>Action</th> : null}
          </tr>
        </thead>
        <tbody>
          {rows.map((lr) => (
            <tr key={lr.id}>
              <td>{lr.faculty_name}</td>
              <td>{lr.start_date} → {lr.end_date}</td>
              <td>{lr.reason}</td>
              <td>
                <span className={`status-pill ${lr.status.toLowerCase()}`}>{lr.status}</span>
                {lr.status === "Rejected" && lr.rejection_reason ? (
                  <div className="meta-note" style={{ fontSize: 11, color: "#666", marginTop: 2 }}>{lr.rejection_reason}</div>
                ) : null}
              </td>
              {showActions ? (
                <td>
                  {lr.status === "Pending" ? (
                    rejectingId === lr.id ? (
                      <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
                        <input
                          type="text"
                          placeholder="Reason for rejection…"
                          value={rejectReason}
                          onChange={(e) => setRejectReason(e.target.value)}
                          autoFocus
                          style={{ padding: "6px 10px", borderRadius: 6, border: "1px solid #d0d7e0", fontSize: 13, minWidth: 180 }}
                        />
                        <button
                          type="button"
                          className="secondary-button small"
                          onClick={() => handleRejectLeave(lr.id)}
                          disabled={!rejectReason.trim()}
                        >
                          Submit
                        </button>
                        <button
                          type="button"
                          className="ghost-button small"
                          onClick={() => { setRejectingId(null); setRejectReason(""); }}
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                        <button type="button" className="primary-action small" onClick={() => showImpact(lr.id)}>
                          Preview
                        </button>
                        <button type="button" className="primary-action small" onClick={() => handleApproveLeave(lr.id)}>
                          Approve
                        </button>
                        <button type="button" className="secondary-button small" onClick={() => setRejectingId(lr.id)}>
                          Reject
                        </button>
                      </div>
                    )
                  ) : (
                    "—"
                  )}
                </td>
              ) : null}
            </tr>
          ))}
          {rows.length === 0 && (
            <tr><td colSpan={showActions ? 5 : 4} className="empty-state compact">No leave requests</td></tr>
          )}
        </tbody>
      </table>
    );
  }

  return (
    <section className="ticket-modal" aria-label="Leave and Swap Requests">
      <header>
        <div>
          <h2>Leave &amp; Swap</h2>
          <p>Manage leave requests and peer period swaps</p>
        </div>
      </header>

      {notice ? <div className="notice-banner">{notice}</div> : null}
      <p className="read-only-notice">Leave and swap workflows require linked faculty login accounts. No-login faculty can be planned in Faculty Assignments but cannot submit or receive leave/swap requests.</p>

      {impactPreview && impactForId !== null ? (
        <div className="notice-banner" style={{ background: "#fff8e1", borderColor: "#ffc107" }}>
          <strong>Impact Preview</strong>
          <div style={{ fontSize: 13, marginTop: 4 }}>
            {impactPreview.faculty_name} — {impactPreview.school_name}<br />
            Dates: {impactPreview.date_range_start} → {impactPreview.date_range_end}<br />
            Affected sessions: <strong>{impactPreview.affected_session_count}</strong>
          </div>
          <div style={{ marginTop: 8, display: "flex", gap: 8 }}>
            <button type="button" className="primary-action small" onClick={() => handleApproveLeave(impactForId)}>
              Confirm Approve
            </button>
            <button type="button" className="secondary-button small" onClick={() => { setImpactPreview(null); setImpactForId(null); }}>
              Cancel
            </button>
          </div>
        </div>
      ) : null}

      <div className="leave-swap-grid">
        <div>
          {isFaculty ? (
            <>
              <div className="panel-toolbar">
                <h3>My Leave Requests</h3>
                <button type="button" className="secondary-button" onClick={() => setShowLeaveForm((s) => !s)}>
                  {showLeaveForm ? "Cancel" : "Request Leave"}
                </button>
              </div>

              {showLeaveForm && (
                <LeaveRequestForm
                  schools={schools}
                  faculty={faculty}
                  currentUser={currentUser}
                  onSubmit={handleCreateLeave}
                />
              )}

              {renderLeaveTable(myLeaves, false)}
            </>
          ) : (
            <>
              <div className="panel-toolbar">
                <h3>Leave Requests</h3>
                {canApproveLeave ? (
                  <div style={{ display: "flex", gap: 6 }}>
                    {(["Pending", "Approved", "Rejected"] as const).map((tab) => (
                      <button
                        key={tab}
                        type="button"
                        className={approverTab === tab ? "primary-action small" : "secondary-button small"}
                        onClick={() => setApproverTab(tab)}
                      >
                        {tab}
                      </button>
                    ))}
                  </div>
                ) : null}
              </div>

              {approverTab === "Pending" && renderLeaveTable(pendingLeaves, canApproveLeave)}
              {approverTab === "Approved" && renderLeaveTable(approvedLeaves, false)}
              {approverTab === "Rejected" && renderLeaveTable(rejectedLeaves, false)}
            </>
          )}
        </div>

        <div>
          <div className="panel-toolbar">
            <h3>Swap Requests</h3>
            <button type="button" className="secondary-button" onClick={() => setShowSwapForm((s) => !s)}>
              {showSwapForm ? "Cancel" : "Propose Swap"}
            </button>
          </div>

          {showSwapForm && (
            <SwapRequestForm faculty={faculty} currentUser={currentUser} onSubmit={handleCreateSwap} />
          )}

          <table className="data-table compact">
            <thead>
              <tr>
                <th>Requester</th>
                <th>Recipient</th>
                <th>Status</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {swapRequests.map((sr) => (
                <tr key={sr.id}>
                  <td>{sr.requester_name}</td>
                  <td>{sr.recipient_name}</td>
                  <td>
                    <span className={`status-pill ${sr.status.toLowerCase()}`}>{sr.status}</span>
                  </td>
                  <td>
                    {sr.status === "Pending" && currentUser?.id === sr.recipient_faculty_id ? (
                      <button type="button" className="primary-action small" onClick={() => handleAcceptSwap(sr.id)}>
                        Accept
                      </button>
                    ) : (
                      "—"
                    )}
                  </td>
                </tr>
              ))}
              {swapRequests.length === 0 && (
                <tr><td colSpan={4} className="empty-state compact">No swap requests</td></tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}

function LeaveRequestForm({
  schools,
  faculty,
  currentUser,
  onSubmit,
}: {
  schools: Array<{ id: number; name: string }>;
  faculty: Array<{ id: number; display_name: string }>;
  currentUser: { id: number; role: string } | null;
  onSubmit: (input: CreateLeaveRequestInput) => void;
}) {
  const [facultyId, setFacultyId] = React.useState(currentUser?.id ?? 0);
  const [schoolId, setSchoolId] = React.useState(schools[0]?.id ?? 0);
  const [startDate, setStartDate] = React.useState(new Date().toISOString().split("T")[0]);
  const [endDate, setEndDate] = React.useState(new Date().toISOString().split("T")[0]);
  const [reason, setReason] = React.useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onSubmit({ faculty_user_id: facultyId, school_id: schoolId, start_date: startDate, end_date: endDate, reason });
  }

  const isAdminOrAom = currentUser?.role === "admin" || currentUser?.role === "aom";

  return (
    <form onSubmit={handleSubmit} className="form-stack compact">
      <label>
        Faculty
        <select value={facultyId} onChange={(e) => setFacultyId(Number(e.target.value))} disabled={!isAdminOrAom}>
          {faculty.map((f) => (
            <option key={f.id} value={f.id}>{f.display_name}</option>
          ))}
        </select>
      </label>
      <label>
        School
        <select value={schoolId} onChange={(e) => setSchoolId(Number(e.target.value))}>
          {schools.map((s) => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>
      </label>
      <div className="form-row">
        <label>Start <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} /></label>
        <label>End <input type="date" value={endDate} onChange={(e) => setEndDate(e.target.value)} /></label>
      </div>
      <label>
        Reason
        <input value={reason} onChange={(e) => setReason(e.target.value)} placeholder="Optional" />
      </label>
      <button type="submit" className="primary-action">Submit Leave Request</button>
    </form>
  );
}

function SwapRequestForm({
  faculty,
  currentUser,
  onSubmit,
}: {
  faculty: Array<{ id: number; display_name: string }>;
  currentUser: { id: number; role: string } | null;
  onSubmit: (input: CreateSwapRequestInput) => void;
}) {
  const [requesterId, setRequesterId] = React.useState(currentUser?.id ?? 0);
  const [recipientId, setRecipientId] = React.useState(0);
  const [slotA, setSlotA] = React.useState(0);
  const [slotB, setSlotB] = React.useState(0);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onSubmit({ requester_faculty_id: requesterId, recipient_faculty_id: recipientId, slot_a_id: slotA, slot_b_id: slotB });
  }

  return (
    <form onSubmit={handleSubmit} className="form-stack compact">
      <label>
        Your slot ID
        <input type="number" value={slotA} onChange={(e) => setSlotA(Number(e.target.value))} />
      </label>
      <label>
        Recipient faculty
        <select value={recipientId} onChange={(e) => setRecipientId(Number(e.target.value))}>
          <option value={0}>Select…</option>
          {faculty.filter((f) => f.id !== requesterId).map((f) => (
            <option key={f.id} value={f.id}>{f.display_name}</option>
          ))}
        </select>
      </label>
      <label>
        Recipient slot ID
        <input type="number" value={slotB} onChange={(e) => setSlotB(Number(e.target.value))} />
      </label>
      <button type="submit" className="primary-action">Propose Swap</button>
    </form>
  );
}
