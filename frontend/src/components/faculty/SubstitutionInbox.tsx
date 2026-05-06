import React from "react";
import type { SubstitutionRecord } from "../../types";

type SubstitutionInboxProps = {
  substitutions: SubstitutionRecord[];
  pendingRequests: SubstitutionRecord[];
  currentUser: { id: number } | null;
  onAccept: (sessionId: number) => Promise<void>;
  onDecline: (sessionId: number, reason: string) => Promise<void>;
  onLoad: () => Promise<void>;
};

export function SubstitutionInbox({
  substitutions,
  pendingRequests,
  currentUser,
  onAccept,
  onDecline,
  onLoad,
}: SubstitutionInboxProps) {
  const [tab, setTab] = React.useState<"pending" | "my" | "coverage">("pending");
  const [decliningId, setDecliningId] = React.useState<number | null>(null);
  const [declineReason, setDeclineReason] = React.useState("");
  const [loadingId, setLoadingId] = React.useState<number | null>(null);

  React.useEffect(() => {
    void onLoad();
  }, []);

  const mySubstitutions = substitutions.filter(
    (s) => s.substitute_faculty_user_id === currentUser?.id
  );
  const coverageReceived = substitutions.filter(
    (s) => s.original_faculty_user_id === currentUser?.id
  );

  async function handleAccept(sessionId: number) {
    setLoadingId(sessionId);
    try {
      await onAccept(sessionId);
      await onLoad();
    } catch (e) {
      alert(String(e));
    } finally {
      setLoadingId(null);
    }
  }

  async function handleDecline(sessionId: number) {
    if (!declineReason.trim()) return;
    setLoadingId(sessionId);
    try {
      await onDecline(sessionId, declineReason.trim());
      setDecliningId(null);
      setDeclineReason("");
      await onLoad();
    } catch (e) {
      alert(String(e));
    } finally {
      setLoadingId(null);
    }
  }

  return (
    <div className="substitution-inbox">
      <div className="sub-tabs">
        <button
          type="button"
          className={tab === "pending" ? "active" : ""}
          onClick={() => setTab("pending")}
        >
          Pending
          {pendingRequests.length > 0 && (
            <span className="sub-tab-badge">{pendingRequests.length}</span>
          )}
        </button>
        <button
          type="button"
          className={tab === "my" ? "active" : ""}
          onClick={() => setTab("my")}
        >
          My Substitutions
        </button>
        <button
          type="button"
          className={tab === "coverage" ? "active" : ""}
          onClick={() => setTab("coverage")}
        >
          Coverage Received
        </button>
      </div>

      {tab === "pending" && (
        <div className="sub-list">
          {pendingRequests.length === 0 ? (
            <p className="empty-state compact">No pending substitution requests.</p>
          ) : (
            pendingRequests.map((req) => (
              <div key={req.session_id} className="sub-card pending">
                <div className="sub-card-header">
                  <strong>
                    {req.grade_level} {req.subject_name}
                  </strong>
                </div>
                <div className="sub-card-meta">
                  {req.session_date}
                </div>
                <div className="sub-card-origin">
                  Original teacher: {req.original_faculty_name}
                </div>
                {decliningId === req.session_id ? (
                  <div className="sub-decline-form">
                    <input
                      type="text"
                      placeholder="Reason for declining…"
                      value={declineReason}
                      onChange={(e) => setDeclineReason(e.target.value)}
                      autoFocus
                    />
                    <div className="sub-decline-actions">
                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => {
                          setDecliningId(null);
                          setDeclineReason("");
                        }}
                      >
                        Cancel
                      </button>
                      <button
                        type="button"
                        className="primary-action"
                        onClick={() => handleDecline(req.session_id)}
                        disabled={!declineReason.trim() || loadingId === req.session_id}
                      >
                        {loadingId === req.session_id ? "Saving…" : "Decline"}
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="sub-actions">
                    <button
                      type="button"
                      className="primary-action"
                      onClick={() => handleAccept(req.session_id)}
                      disabled={loadingId === req.session_id}
                    >
                      {loadingId === req.session_id ? "…" : "Accept"}
                    </button>
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => setDecliningId(req.session_id)}
                      disabled={loadingId === req.session_id}
                    >
                      Decline
                    </button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      {tab === "my" && (
        <div className="sub-list">
          {mySubstitutions.length === 0 ? (
            <p className="empty-state compact">No substitutions covered this month.</p>
          ) : (
            mySubstitutions.map((s) => (
              <div key={s.session_id} className="sub-card history">
                <div className="sub-card-header">
                  <strong>
                    {s.grade_level} {s.subject_name}
                  </strong>
                  <span className="sub-status accepted">Accepted</span>
                </div>
                <div className="sub-card-meta">
                  {s.session_date}
                </div>
                <div className="sub-card-origin">
                  For {s.original_faculty_name}
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {tab === "coverage" && (
        <div className="sub-list">
          {coverageReceived.length === 0 ? (
            <p className="empty-state compact">No colleagues have covered your classes yet.</p>
          ) : (
            coverageReceived.map((s) => (
              <div key={s.session_id} className="sub-card history">
                <div className="sub-card-header">
                  <strong>
                    {s.grade_level} {s.subject_name}
                  </strong>
                  <span className="sub-status covered">Covered</span>
                </div>
                <div className="sub-card-meta">
                  {s.session_date}
                </div>
                <div className="sub-card-origin">
                  Covered by: {s.substitute_faculty_name}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
