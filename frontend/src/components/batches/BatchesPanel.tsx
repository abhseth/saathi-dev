import React from "react";
import { api } from "../../api";
import type { BatchAnalytics, BatchDetail, Student } from "../../types";

export function BatchesPanel() {
  const [analytics, setAnalytics] = React.useState<BatchAnalytics | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [selectedBatch, setSelectedBatch] = React.useState<BatchDetail | null>(null);
  const [students, setStudents] = React.useState<Student[]>([]);
  const [studentsLoading, setStudentsLoading] = React.useState(false);

  React.useEffect(() => {
    setLoading(true);
    api<BatchAnalytics>("batch_analytics")
      .then(setAnalytics)
      .catch(() => setAnalytics(null))
      .finally(() => setLoading(false));
  }, []);

  async function handleSelectBatch(batch: BatchDetail) {
    setSelectedBatch(batch);
    setStudentsLoading(true);
    try {
      const data = await api<Student[]>("get_batch_students", { id: batch.id });
      setStudents(data);
    } catch {
      setStudents([]);
    } finally {
      setStudentsLoading(false);
    }
  }

  if (selectedBatch) {
    return (
      <BatchDetailPanel
        batch={selectedBatch}
        students={students}
        loading={studentsLoading}
        onClose={() => setSelectedBatch(null)}
      />
    );
  }

  return (
    <BatchesList
      analytics={analytics}
      loading={loading}
      onSelectBatch={handleSelectBatch}
    />
  );
}

function BatchesList({
  analytics,
  loading,
  onSelectBatch,
}: {
  analytics: BatchAnalytics | null;
  loading: boolean;
  onSelectBatch: (batch: BatchDetail) => void;
}) {
  if (loading) return <p className="empty-state">Loading batch data…</p>;
  if (!analytics || analytics.batches.length === 0) {
    return <p className="empty-state">No batches found.</p>;
  }

  const utilColor =
    analytics.overall_utilization >= 90
      ? "#d93025"
      : analytics.overall_utilization >= 75
        ? "#f9ab00"
        : "#1E7A6F";

  return (
    <section className="ticket-modal" aria-label="Batches">
      <header>
        <h2>Batches</h2>
      </header>

      <div className="metrics" style={{ marginBottom: "1rem" }}>
        <div className="metric-box">
          <strong>{analytics.batches.length}</strong>
          <span>Batches</span>
        </div>
        <div className="metric-box">
          <strong>{analytics.total_students}</strong>
          <span>Students</span>
        </div>
        <div className="metric-box">
          <strong>{analytics.total_capacity}</strong>
          <span>Capacity</span>
        </div>
        <div className="metric-box" style={{ color: utilColor }}>
          <strong>{analytics.overall_utilization.toFixed(1)}%</strong>
          <span>Utilization</span>
        </div>
      </div>

      <table className="data-table">
        <thead>
          <tr>
            <th>School</th>
            <th>Batch</th>
            <th>Class</th>
            <th>Track</th>
            <th>Pattern</th>
            <th>Capacity</th>
            <th>Students</th>
            <th>Faculty</th>
            <th>Tickets</th>
            <th>Sessions</th>
          </tr>
        </thead>
        <tbody>
          {analytics.batches.map((b) => {
            const utilization = b.capacity > 0 ? (b.student_count / b.capacity) * 100 : 0;
            return (
              <tr
                key={b.id}
                style={{ cursor: "pointer" }}
                onClick={() => onSelectBatch(b)}
                title="Click to view student roster"
              >
                <td>{b.school_name}</td>
                <td>
                  <strong>{b.batch_id}</strong>
                </td>
                <td>{b.grade_level}</td>
                <td>{b.track || "Foundation"}</td>
                <td>{b.batch_pattern}</td>
                <td>{b.capacity || "—"}</td>
                <td>
                  {b.student_count}
                  {b.capacity > 0 && (
                    <small style={{ color: utilization >= 90 ? "#d93025" : "#5f6368", marginLeft: 4 }}>
                      ({utilization.toFixed(0)}%)
                    </small>
                  )}
                </td>
                <td>{b.faculty_count}</td>
                <td>{b.active_ticket_count}</td>
                <td>{b.upcoming_session_count}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </section>
  );
}

function BatchDetailPanel({
  batch,
  students,
  loading,
  onClose,
}: {
  batch: BatchDetail;
  students: Student[];
  loading: boolean;
  onClose: () => void;
}) {
  const utilization = batch.capacity > 0 ? (batch.student_count / batch.capacity) * 100 : 0;

  return (
    <section className="ticket-modal" aria-label={`Batch ${batch.batch_id}`}>
      <header>
        <h2>
          {batch.batch_id} — {batch.school_name}
        </h2>
        <button className="ghost-button" onClick={onClose}>
          Back to Batches
        </button>
      </header>

      <div className="metrics" style={{ marginBottom: "1rem" }}>
        <div className="metric-box">
          <strong>{batch.grade_level}</strong>
          <span>Class</span>
        </div>
        <div className="metric-box">
          <strong>{batch.track || "Foundation"}</strong>
          <span>Track</span>
        </div>
        <div className="metric-box">
          <strong>{batch.batch_pattern}</strong>
          <span>Pattern</span>
        </div>
        <div className="metric-box">
          <strong>{batch.capacity || "—"}</strong>
          <span>Capacity</span>
        </div>
        <div className="metric-box">
          <strong>{batch.student_count}</strong>
          <span>Students</span>
        </div>
        <div className="metric-box">
          <strong>{batch.faculty_count}</strong>
          <span>Faculty</span>
        </div>
        <div className="metric-box">
          <strong>{batch.active_ticket_count}</strong>
          <span>Active Tickets</span>
        </div>
        <div className="metric-box">
          <strong>{batch.upcoming_session_count}</strong>
          <span>Upcoming Sessions</span>
        </div>
        <div
          className="metric-box"
          style={{
            color:
              utilization >= 90 ? "#d93025" : utilization >= 75 ? "#f9ab00" : "#1E7A6F",
          }}
        >
          <strong>{utilization.toFixed(1)}%</strong>
          <span>Utilization</span>
        </div>
      </div>

      <h3 style={{ margin: "1rem 0 0.5rem" }}>Student Roster</h3>
      {loading ? (
        <p className="empty-state">Loading students…</p>
      ) : students.length === 0 ? (
        <p className="empty-state">No students in this batch.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Reg. No</th>
              <th>Grade</th>
              <th>Student Mobile</th>
              <th>Student Email</th>
              <th>Father</th>
              <th>Mother</th>
            </tr>
          </thead>
          <tbody>
            {students.map((s) => (
              <tr key={s.id}>
                <td>{s.name}</td>
                <td>{s.registration_number}</td>
                <td>{s.grade_level}</td>
                <td>{s.student_mobile}</td>
                <td>{s.student_email}</td>
                <td>
                  {s.father_name}
                  {s.father_mobile && <small> ({s.father_mobile})</small>}
                </td>
                <td>
                  {s.mother_name}
                  {s.mother_mobile && <small> ({s.mother_mobile})</small>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
