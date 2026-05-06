import React from "react";
import { api } from "../../api";
import type { TodaySubstitutions, TodaySubstitutionLane, SubstituteCandidate, SubstitutionDetail } from "../../types";

export function SubstitutionCommandCenter({ schools }: { schools: Array<{ id: number; name: string }> }) {
  const [data, setData] = React.useState<TodaySubstitutions | null>(null);
  const [selectedSessionId, setSelectedSessionId] = React.useState<number | null>(null);
  const [candidates, setCandidates] = React.useState<SubstituteCandidate[]>([]);
  const [detail, setDetail] = React.useState<SubstitutionDetail | null>(null);
  const [loading, setLoading] = React.useState(false);
  const [notice, setNotice] = React.useState("");

  const load = React.useCallback(async () => {
    setLoading(true);
    try {
      const result = await api<TodaySubstitutions>("today_substitutions");
      setData(result);
    } catch (e) {
      console.error("Failed to load substitutions:", e);
    }
    setLoading(false);
  }, []);

  React.useEffect(() => {
    void load();
    const interval = setInterval(() => void load(), 30000);
    return () => clearInterval(interval);
  }, [load]);

  React.useEffect(() => {
    if (!selectedSessionId) {
      setCandidates([]);
      setDetail(null);
      return;
    }
    void (async () => {
      try {
        const d = await api<SubstitutionDetail>("substitution_detail", { sessionId: selectedSessionId });
        setDetail(d);
      } catch (e) {
        console.error("Failed to load substitution detail:", e);
        setDetail(null);
      }
      try {
        const c = await api<SubstituteCandidate[]>("suggest_substitutes", { input: { session_id: selectedSessionId } });
        setCandidates(c);
      } catch (e) {
        console.error("Failed to load substitute candidates:", e);
        setCandidates([]);
      }
    })();
  }, [selectedSessionId]);

  async function handleAssign(sessionId: number, facultyUserId: number) {
    try {
      await api("assign_substitute", { sessionId, input: { substitute_faculty_user_id: facultyUserId } });
      setNotice("Substitute assigned.");
      void load();
    } catch (e) {
      setNotice(String(e));
    }
  }

  function LaneCard({ lane }: { lane: TodaySubstitutionLane }) {
    const isSelected = selectedSessionId === lane.session_id;
    return (
      <button
        type="button"
        className={`substitution-lane-card ${isSelected ? "selected" : ""}`}
        onClick={() => setSelectedSessionId(lane.session_id)}
      >
        <div className="lane-card-header">
          <strong>{lane.subject_name}</strong>
          <span className="lane-badge">P{lane.period}</span>
        </div>
        <div className="lane-card-meta">
          {lane.school_name} · {lane.grade_level} · {lane.batch_pattern}
        </div>
        <div className="lane-card-faculty">
          Absent: {lane.original_faculty_name}
          {lane.substitute_faculty_name ? (
            <span className="lane-sub"> → {lane.substitute_faculty_name}</span>
          ) : null}
        </div>
        {lane.room ? <div className="lane-card-room">Room: {lane.room}</div> : null}
      </button>
    );
  }

  return (
    <section className="ticket-modal" aria-label="Substitution Command Center">
      <header>
        <div>
          <h2>Today's Substitutions</h2>
          <p>Three-lane view: Unfilled, Assigned, Completed</p>
        </div>
        <button type="button" className="ghost-button" onClick={load}>
          Refresh
        </button>
      </header>

      {notice ? <div className="notice-banner">{notice}</div> : null}

      <div className="substitution-lanes">
        <div className="substitution-lane">
          <h3>Unfilled Absences ({data?.unfilled.length ?? 0})</h3>
          <div className="lane-scroll">
            {data?.unfilled.map((lane) => <LaneCard key={lane.session_id} lane={lane} />)}
            {!data?.unfilled.length && <p className="empty-state compact">No unfilled absences</p>}
          </div>
        </div>
        <div className="substitution-lane">
          <h3>Substitutes Assigned ({data?.assigned.length ?? 0})</h3>
          <div className="lane-scroll">
            {data?.assigned.map((lane) => <LaneCard key={lane.session_id} lane={lane} />)}
            {!data?.assigned.length && <p className="empty-state compact">No assigned substitutes</p>}
          </div>
        </div>
        <div className="substitution-lane">
          <h3>Completed Today ({data?.completed.length ?? 0})</h3>
          <div className="lane-scroll">
            {data?.completed.map((lane) => <LaneCard key={lane.session_id} lane={lane} />)}
            {!data?.completed.length && <p className="empty-state compact">No completed sessions</p>}
          </div>
        </div>
      </div>

      {selectedSessionId && detail && (
        <div className="substitution-detail-panel">
          <h3>Request Detail</h3>
          <dl className="detail-grid">
            <div><dt>School</dt><dd>{detail.school_name}</dd></div>
            <div><dt>Grade</dt><dd>{detail.grade_level}</dd></div>
            <div><dt>Track</dt><dd>{detail.track}</dd></div>
            <div><dt>Batch</dt><dd>{detail.batch_pattern}</dd></div>
            <div><dt>Subject</dt><dd>{detail.subject_name}</dd></div>
            <div><dt>Room</dt><dd>{detail.room || "—"}</dd></div>
            <div><dt>Roster</dt><dd>{detail.roster_count}</dd></div>
            <div><dt>Present</dt><dd>{detail.present_count}</dd></div>
            <div><dt>Absent</dt><dd>{detail.absent_count}</dd></div>
            <div><dt>Last Topics</dt><dd>{detail.last_covered_topics}</dd></div>
          </dl>

          <h4>Suggested Substitutes</h4>
          {candidates.length === 0 ? (
            <p className="empty-state compact">No candidates found.</p>
          ) : (
            <table className="data-table compact">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Subject</th>
                  <th>Free</th>
                  <th>School</th>
                  <th>Workload</th>
                  <th>Score</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {candidates.map((c) => (
                  <tr key={c.faculty_user_id}>
                    <td>{c.faculty_name}</td>
                    <td>{c.subject_match ? "✓" : "—"}</td>
                    <td>{c.free_period ? "✓" : "✗"}</td>
                    <td>{c.same_school ? "✓" : "—"}</td>
                    <td>{c.workload_score}</td>
                    <td><strong>{c.overall_score}</strong></td>
                    <td>
                      <button
                        type="button"
                        className="primary-action small"
                        onClick={() => handleAssign(selectedSessionId, c.faculty_user_id)}
                      >
                        Assign
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {loading && <p className="empty-state compact">Loading…</p>}
    </section>
  );
}
