import React from "react";
import { gradeLevels, programTracks, issueCategories, priorities } from "../../constants";
import type { Priority, CreateTicketDraft, School, Student } from "../../types";

type CreateTicketModalProps = {
  draft: CreateTicketDraft;
  schools: School[];
  students: Student[];
  onCancel: () => void;
  onDraftChange: React.Dispatch<React.SetStateAction<CreateTicketDraft>>;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
};

const TICKET_TEMPLATES: Array<{
  label: string;
  issue_category: string;
  description: string;
}> = [
  {
    label: "Academic support request",
    issue_category: "Academic Support",
    description: "Student is experiencing difficulties with academic content. Please describe the specific subject or topic, the nature of the challenge, and any previous support provided.",
  },
  {
    label: "Attendance concern",
    issue_category: "Attendance",
    description: "Reporting an attendance concern for this student. Please include the dates missed, reason if known, and steps taken to follow up with the family.",
  },
  {
    label: "Assessment issue",
    issue_category: "Assessment",
    description: "Issue related to an assessment or evaluation. Please describe the assessment affected, the problem observed, and any impact on the student's grade or progress.",
  },
  {
    label: "Device / hardware problem",
    issue_category: "Device",
    description: "Student's device is not functioning as expected. Please describe the device type, the fault observed, and whether a replacement or repair is needed.",
  },
  {
    label: "Learning platform issue",
    issue_category: "Learning Platform",
    description: "Student is unable to access or use the learning platform. Please describe the error or failure, the steps already attempted, and the impact on learning.",
  },
  {
    label: "Operations request",
    issue_category: "Operations",
    description: "Operational matter requiring coordination. Please describe the request, the expected outcome, and any deadline or urgency.",
  },
  {
    label: "Parent communication needed",
    issue_category: "Parent Communication",
    description: "A communication with the parent or guardian is required. Please describe the topic to be discussed, preferred communication channel, and any prior contact made.",
  },
];

export function CreateTicketModal({
  draft,
  schools,
  students,
  onCancel,
  onDraftChange,
  onSubmit,
}: CreateTicketModalProps) {
  const selectedSchool = schools.find((school) => school.id === draft.school_id);
  const studentOptions = selectedSchool
    ? students.filter((student) => student.school_id === selectedSchool.id)
    : students;

  function applyTemplate(label: string) {
    const template = TICKET_TEMPLATES.find((t) => t.label === label);
    if (!template) return;
    onDraftChange((current) => ({
      ...current,
      issue_category: template.issue_category,
      description: template.description,
    }));
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <form className="ticket-modal" onSubmit={onSubmit}>
        <header>
          <div>
            <h2>New Ticket</h2>
            <p>Create a school-program support request.</p>
          </div>
          <button type="button" onClick={onCancel}>
            Close
          </button>
        </header>

        <label>
          Template (optional)
          <select
            value=""
            onChange={(e) => applyTemplate(e.target.value)}
          >
            <option value="">— Select a template —</option>
            {TICKET_TEMPLATES.map((t) => (
              <option key={t.label} value={t.label}>{t.label}</option>
            ))}
          </select>
        </label>

        <label>
          Title
          <input
            autoFocus
            required
            value={draft.title}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, title: event.target.value }))
            }
          />
        </label>

        <label>
          Requester
          <input
            required
            value={draft.requester}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, requester: event.target.value }))
            }
          />
        </label>
        <label>
          School
          <select
            required
            value={draft.school_id ?? ""}
            onChange={(event) => {
              const school = schools.find((item) => item.id === Number(event.target.value));
              onDraftChange((current) => ({
                ...current,
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
            onChange={(event) => {
              const student = studentOptions.find((item) => item.name === event.target.value);
              onDraftChange((current) => ({
                ...current,
                student_name: event.target.value,
                grade_level: student?.grade_level ?? current.grade_level,
                program_track: student?.program_track ?? current.program_track,
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
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                grade_level: event.target.value,
              }))
            }
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
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                program_track: event.target.value,
              }))
            }
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
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                issue_category: event.target.value,
              }))
            }
          >
            {issueCategories.map((category) => (
              <option key={category}>{category}</option>
            ))}
          </select>
        </label>

        <label>
          Priority
          <select
            value={draft.priority}
            onChange={(event) =>
              onDraftChange((current) => ({
                ...current,
                priority: event.target.value as Priority,
              }))
            }
          >
            {priorities.map((priority) => (
              <option key={priority}>{priority}</option>
            ))}
          </select>
        </label>

        <label>
          Description
          <textarea
            required
            value={draft.description}
            onChange={(event) =>
              onDraftChange((current) => ({ ...current, description: event.target.value }))
            }
          />
        </label>

        <div className="actions">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button className="primary-action" type="submit">
            Create Ticket
          </button>
        </div>
      </form>
    </div>
  );
}
