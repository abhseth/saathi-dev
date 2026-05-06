use crate::models::{
    AddCommentInput, AssignmentRule, CommunicationTemplate, CreateTicketInput, EscalationPolicy,
    Paginated, SlaPolicy, StudentAttendanceSummary, StudentTimeline, Ticket, TicketAttachment,
    TicketComment, TicketHistory, UpdateAssignmentRuleInput, UpdateCommentStatusInput,
    UpdateCommunicationTemplateInput, UpdateEscalationPolicyInput, UpdateSlaPolicyInput,
    UpdateTicketInput,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

const ALLOWED_PRIORITIES: &[&str] = &["Low", "Medium", "High", "Critical"];
const ALLOWED_STATUSES: &[&str] = &["Open", "In Progress", "Pending", "Resolved", "Closed"];
const ALLOWED_QUEUES: &[&str] = &[
    "Academic Support",
    "Learning Platform",
    "IT / Device",
    "Operations",
    "Parent Communication",
];
use super::audit::*;
use super::common::*;
use crate::repo::schools::get_student;

pub fn list_tickets(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Ticket>, String> {
    let safe_limit = limit.clamp(1, 500);
    let safe_offset = offset.max(0);

    let mut count_sql = String::from("SELECT COUNT(*) FROM tickets");
    let mut sql = String::from(
        "SELECT id, title, description, requester, assignee, status, priority,
                queue, school_id,
                school_name, student_name, grade_level, program_track, issue_category,
                sla_due_at, escalation_status, escalated_at,
                created_at, updated_at,
                linked_grade_level, linked_subject
         FROM tickets",
    );
    let mut p: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let filter = format!(" WHERE school_id IN ({placeholders})");
            count_sql.push_str(&filter);
            sql.push_str(&filter);
            for id in ids {
                p.push(id);
            }
        }
    }

    let total_count: i64 = conn
        .query_row(&count_sql, rusqlite::params_from_iter(p.iter()), |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    sql.push_str(" ORDER BY datetime(updated_at) DESC, id DESC LIMIT ? OFFSET ?");
    p.push(&safe_limit);
    p.push(&safe_offset);

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), ticket_from_row)
        .map_err(|error| error.to_string())?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(Paginated {
        items,
        total_count,
        page: (safe_offset / safe_limit) + 1,
        page_size: safe_limit,
    })
}

/// Add `hours` business hours to `start`, skipping weekends and holidays.
fn add_business_hours(conn: &Connection, start: &str, hours: i64) -> Result<String, String> {
    let mut remaining = hours;
    let mut current = start.to_string();
    while remaining > 0 {
        let next: String = conn
            .query_row("SELECT datetime(?1, '+1 hour')", params![&current], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?;
        let dow: String = conn
            .query_row("SELECT strftime('%w', ?1)", params![&next], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?;
        let is_holiday: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM holidays WHERE date = date(?1))",
                params![&next],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?
            != 0;
        if dow != "0" && dow != "6" && !is_holiday {
            remaining -= 1;
        }
        current = next;
    }
    Ok(current)
}

pub fn refresh_escalations(conn: &Connection) -> Result<usize, String> {
    let policy = get_escalation_policy(conn)?;
    let at_risk_modifier = format!("+{} hours", policy.at_risk_hours);
    let mut stmt = conn
        .prepare(
            "
            SELECT id, escalation_status, assignee, queue, priority, created_at,
                   CASE
                       WHEN status IN ('Resolved', 'Closed') THEN 'None'
                       WHEN datetime(sla_due_at) <= datetime('now', 'localtime') THEN 'Escalated'
                       WHEN datetime(sla_due_at) <= datetime('now', ?1, 'localtime') THEN 'At Risk'
                       ELSE 'None'
                   END AS next_escalation_status
            FROM tickets
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![&at_risk_modifier], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|error| error.to_string())?;

    let changes = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut updated_count = 0;

    for (ticket_id, current_status, current_assignee, queue, priority, created_at, next_status) in
        changes
    {
        let hours_open = if let Ok(created_dt) =
            chrono::NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")
        {
            chrono::Local::now()
                .naive_local()
                .signed_duration_since(created_dt)
                .num_hours()
        } else {
            0
        };

        let rule_assignee = if next_status == "Escalated" || next_status == "At Risk" {
            crate::escalation::evaluate_rules_for_ticket(conn, &queue, &priority, hours_open)?
        } else {
            None
        };

        let escalation_assignee =
            rule_assignee.unwrap_or_else(|| policy.escalation_assignee.clone());

        let should_assign_escalation_owner = next_status == "Escalated"
            && policy.auto_assign_on_breach
            && current_assignee != escalation_assignee;

        if current_status == next_status && !should_assign_escalation_owner {
            continue;
        }

        let escalated_at_sql = if next_status == "None" {
            "''"
        } else {
            "datetime('now', 'localtime')"
        };

        if should_assign_escalation_owner {
            conn.execute(
                &format!(
                    "
                    UPDATE tickets
                    SET escalation_status = ?1,
                        escalated_at = {escalated_at_sql},
                        assignee = ?2,
                        updated_at = datetime('now', 'localtime')
                    WHERE id = ?3
                    "
                ),
                params![next_status, &escalation_assignee, ticket_id],
            )
            .map_err(|error| error.to_string())?;
        } else {
            conn.execute(
                &format!(
                    "
                    UPDATE tickets
                    SET escalation_status = ?1,
                        escalated_at = {escalated_at_sql},
                        updated_at = datetime('now', 'localtime')
                    WHERE id = ?2
                    "
                ),
                params![next_status, ticket_id],
            )
            .map_err(|error| error.to_string())?;
        }

        record_history(
            conn,
            ticket_id,
            "System",
            "escalation_status",
            &current_status,
            &next_status,
        )?;
        if should_assign_escalation_owner {
            record_history(
                conn,
                ticket_id,
                "System",
                "assignee",
                &current_assignee,
                &escalation_assignee,
            )?;
        }
        updated_count += 1;
    }

    Ok(updated_count)
}

pub fn get_ticket(conn: &Connection, id: i64) -> Result<Ticket, String> {
    conn.query_row(
        "
        SELECT id, title, description, requester, assignee, status, priority,
               queue, school_id,
               school_name, student_name, grade_level, program_track, issue_category,
               sla_due_at, escalation_status, escalated_at,
               created_at, updated_at,
               linked_grade_level, linked_subject
        FROM tickets
        WHERE id = ?1
        ",
        params![id],
        ticket_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Ticket {id} was not found"))
}

#[allow(dead_code)]
pub fn get_student_timeline(conn: &Connection, student_id: i64) -> Result<StudentTimeline, String> {
    let student = get_student(conn, student_id)?;

    let tickets = {
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.title, t.description, t.requester, t.assignee,
                    t.status, t.priority, t.queue, t.school_id, s.name,
                    t.student_name, t.grade_level, t.program_track, t.issue_category,
                    t.sla_due_at, t.escalation_status, t.escalated_at,
                    t.created_at, t.updated_at,
                    t.linked_grade_level, t.linked_subject
             FROM tickets t
             JOIN schools s ON s.id = t.school_id
             WHERE t.school_id = ?1 AND t.student_name = ?2
             ORDER BY datetime(t.updated_at) DESC
             LIMIT 1000",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![student.school_id, student.name], ticket_from_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let ticket_ids = tickets
        .iter()
        .map(|ticket| ticket.id)
        .collect::<HashSet<_>>();

    let attendance = {
        let mut stmt = conn
            .prepare(
                "SELECT ls.session_date, sub.name, ar.status, ar.marked_at
             FROM attendance_records ar
             JOIN lecture_sessions ls ON ls.id = ar.lecture_session_id
             JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
             JOIN subjects sub ON sub.id = ts.subject_id
             WHERE ar.student_id = ?1
             ORDER BY ls.session_date DESC, ts.period ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![student_id], |row| {
                Ok(StudentAttendanceSummary {
                    session_date: row.get(0)?,
                    subject_name: row.get(1)?,
                    status: row.get(2)?,
                    marked_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(StudentTimeline {
        student,
        comments: {
            if ticket_ids.is_empty() {
                vec![]
            } else {
                let placeholders = ticket_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT c.id, c.ticket_id, c.author, c.body, c.is_internal, c.channel, c.audience,
                            c.recipient_name, c.recipient_contact, c.delivery_status, c.last_contacted_at,
                            c.next_follow_up_due, c.created_at
                     FROM ticket_comments c
                     WHERE c.ticket_id IN ({placeholders})
                     ORDER BY c.ticket_id ASC, datetime(c.created_at) ASC, c.id ASC
                     LIMIT 1000"
                );
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let ids: Vec<&dyn rusqlite::ToSql> = ticket_ids
                    .iter()
                    .map(|id| id as &dyn rusqlite::ToSql)
                    .collect();
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(ids.iter()), comment_from_row)
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            }
        },
        history: {
            if ticket_ids.is_empty() {
                vec![]
            } else {
                let placeholders = ticket_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT id, ticket_id, actor, field, old_value, new_value, created_at
                     FROM ticket_history
                     WHERE ticket_id IN ({placeholders})
                     ORDER BY datetime(created_at) DESC, id DESC
                     LIMIT 1000"
                );
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let ids: Vec<&dyn rusqlite::ToSql> = ticket_ids
                    .iter()
                    .map(|id| id as &dyn rusqlite::ToSql)
                    .collect();
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(ids.iter()), history_from_row)
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            }
        },
        attachments: {
            if ticket_ids.is_empty() {
                vec![]
            } else {
                let placeholders = ticket_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT id, ticket_id, original_filename, stored_path, size_bytes, uploaded_by, created_at
                     FROM ticket_attachments
                     WHERE ticket_id IN ({placeholders})
                     ORDER BY created_at DESC
                     LIMIT 1000"
                );
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let ids: Vec<&dyn rusqlite::ToSql> = ticket_ids
                    .iter()
                    .map(|id| id as &dyn rusqlite::ToSql)
                    .collect();
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(ids.iter()), attachment_from_row)
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            }
        },
        tickets,
        attendance,
    })
}

fn extract_linked_metadata(
    conn: &Connection,
    title: &str,
    description: &str,
) -> Result<(String, String), String> {
    let text = format!("{} {}", title, description).to_lowercase();
    let mut linked_grade = String::new();
    let mut linked_subject = String::new();

    // Extract grade level (English + Hindi/Urdu variants)
    for grade in [
        "grade 6",
        "grade 7",
        "grade 8",
        "grade 9",
        "grade 10",
        "grade 11",
        "grade 12",
        "dropper",
        "कक्षा 6",
        "कक्षा 7",
        "कक्षा 8",
        "कक्षा 9",
        "कक्षा 10",
        "कक्षा 11",
        "कक्षा 12",
        "کلاس 6",
        "کلاس 7",
        "کلاس 8",
        "کلاس 9",
        "کلاس 10",
        "کلاس 11",
        "کلاس 12",
    ] {
        if text.contains(grade) {
            linked_grade = grade.to_string();
            break;
        }
    }

    // Extract subject from known subjects
    let mut stmt = conn
        .prepare("SELECT name FROM subjects ORDER BY LENGTH(name) DESC")
        .map_err(|e| e.to_string())?;
    let subjects: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for subject in subjects {
        if text.contains(&subject.to_lowercase()) {
            linked_subject = subject;
            break;
        }
    }

    Ok((linked_grade, linked_subject))
}

pub fn create_ticket(
    conn: &Connection,
    input: &CreateTicketInput,
    actor: &str,
) -> Result<Ticket, String> {
    validate_nonempty("Title", &input.title)?;
    validate_nonempty("Description", &input.description)?;
    validate_nonempty("Requester", &input.requester)?;
    validate_nonempty("School", &input.school_name)?;
    validate_nonempty("Student", &input.student_name)?;
    validate_nonempty("Grade", &input.grade_level)?;
    validate_nonempty("Program track", &input.program_track)?;
    validate_nonempty("Issue category", &input.issue_category)?;
    validate_priority(&input.priority)?;
    let (school_id, school_name) =
        resolve_ticket_school(conn, input.school_id, &input.school_name)?;
    let sla_hours = get_sla_policy_hours(conn, input.issue_category.trim())?;
    let queue = queue_for_category(input.issue_category.trim());
    let assignee =
        active_assignment_for_queue(conn, queue)?.unwrap_or_else(|| "Unassigned".to_string());
    let (linked_grade, linked_subject) =
        extract_linked_metadata(conn, &input.title, &input.description)?;
    let now: String = conn
        .query_row("SELECT datetime('now', 'localtime')", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let sla_due_at = add_business_hours(conn, &now, sla_hours)?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let result = (|| -> Result<Ticket, String> {
        conn.execute(
            "
            INSERT INTO tickets (
                title, description, requester, assignee, priority, queue,
                school_id, school_name, student_name, grade_level, program_track, issue_category,
                sla_due_at, linked_grade_level, linked_subject
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15
            )
            ",
            params![
                input.title.trim(),
                input.description.trim(),
                input.requester.trim(),
                assignee,
                input.priority.trim(),
                queue,
                school_id,
                school_name,
                input.student_name.trim(),
                input.grade_level.trim(),
                input.program_track.trim(),
                input.issue_category.trim(),
                sla_due_at,
                linked_grade,
                linked_subject,
            ],
        )
        .map_err(|error| error.to_string())?;

        let ticket_id = conn.last_insert_rowid();
        record_history(conn, ticket_id, actor, "ticket", "", "Created")?;
        record_audit(
            conn,
            "ticket",
            ticket_id,
            "created",
            actor,
            &format!("Created ticket for {school_name}"),
        )?;
        get_ticket(conn, ticket_id)
    })();
    match result {
        Ok(ticket) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(ticket)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn update_ticket(
    conn: &Connection,
    input: &UpdateTicketInput,
    actor: &str,
) -> Result<Ticket, String> {
    validate_nonempty("Title", &input.title)?;
    validate_nonempty("Description", &input.description)?;
    validate_nonempty("Requester", &input.requester)?;
    validate_nonempty("Status", &input.status)?;
    validate_nonempty("Priority", &input.priority)?;
    validate_nonempty("Assignee", &input.assignee)?;
    validate_nonempty("Queue", &input.queue)?;
    validate_nonempty("School", &input.school_name)?;
    validate_nonempty("Student", &input.student_name)?;
    validate_nonempty("Grade", &input.grade_level)?;
    validate_nonempty("Program track", &input.program_track)?;
    validate_nonempty("Issue category", &input.issue_category)?;
    validate_status(&input.status)?;
    validate_priority(&input.priority)?;
    validate_queue(&input.queue)?;
    let (school_id, school_name) =
        resolve_ticket_school(conn, input.school_id, &input.school_name)?;
    let (linked_grade, linked_subject) =
        extract_linked_metadata(conn, &input.title, &input.description)?;

    let before = get_ticket(conn, input.id)?;
    validate_status_transition(&before.status, &input.status)?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let result = (|| -> Result<Ticket, String> {
        conn.execute(
            "
            UPDATE tickets
            SET title = ?1,
                description = ?2,
                requester = ?3,
                status = ?4,
                priority = ?5,
                assignee = ?6,
                queue = ?7,
                school_id = ?8,
                school_name = ?9,
                student_name = ?10,
                grade_level = ?11,
                program_track = ?12,
                issue_category = ?13,
                linked_grade_level = ?14,
                linked_subject = ?15,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?16
            ",
            params![
                input.title.trim(),
                input.description.trim(),
                input.requester.trim(),
                input.status.trim(),
                input.priority.trim(),
                input.assignee.trim(),
                input.queue.trim(),
                school_id,
                school_name,
                input.student_name.trim(),
                input.grade_level.trim(),
                input.program_track.trim(),
                input.issue_category.trim(),
                linked_grade,
                linked_subject,
                input.id
            ],
        )
        .map_err(|error| error.to_string())?;

        record_history(
            conn,
            input.id,
            actor,
            "title",
            &before.title,
            input.title.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "description",
            &before.description,
            input.description.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "requester",
            &before.requester,
            input.requester.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "status",
            &before.status,
            input.status.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "priority",
            &before.priority,
            input.priority.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "assignee",
            &before.assignee,
            input.assignee.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "queue",
            &before.queue,
            input.queue.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "school_name",
            &before.school_name,
            &school_name,
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "student_name",
            &before.student_name,
            input.student_name.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "grade_level",
            &before.grade_level,
            input.grade_level.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "program_track",
            &before.program_track,
            input.program_track.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "issue_category",
            &before.issue_category,
            input.issue_category.trim(),
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "linked_grade_level",
            &before.linked_grade_level,
            &linked_grade,
        )?;
        record_history(
            conn,
            input.id,
            actor,
            "linked_subject",
            &before.linked_subject,
            &linked_subject,
        )?;
        record_audit(
            conn,
            "ticket",
            input.id,
            "updated",
            actor,
            &format!("Updated ticket {}", input.id),
        )?;

        // Fast-path: clear escalation flags when ticket is resolved/closed
        // (full re-scan is handled by the background worker every 60s)
        if input.status == "Resolved" || input.status == "Closed" {
            conn.execute(
                "UPDATE tickets SET escalation_status = 'None', escalated_at = '' WHERE id = ?1",
                params![input.id],
            )
            .map_err(|e| e.to_string())?;
        }

        get_ticket(conn, input.id)
    })();
    match result {
        Ok(ticket) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(ticket)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn delete_ticket(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM tickets WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_comments(conn: &Connection, ticket_id: i64) -> Result<Vec<TicketComment>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, author, body, is_internal, channel, audience,
                   recipient_name, recipient_contact, delivery_status, last_contacted_at,
                   next_follow_up_due, created_at
            FROM ticket_comments
            WHERE ticket_id = ?1
            ORDER BY datetime(created_at) ASC, id ASC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![ticket_id], comment_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_all_comments(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<TicketComment>, String> {
    let safe_limit = limit.clamp(1, 500);
    let safe_offset = offset.max(0);

    let school_filter = match scope_school_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!("WHERE t.school_id IN ({placeholders})")
        }
        _ => String::new(),
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM ticket_comments c JOIN tickets t ON t.id = c.ticket_id {school_filter}"
    );
    let sql = format!(
        "SELECT c.id, c.ticket_id, c.author, c.body, c.is_internal, c.channel, c.audience,
               c.recipient_name, c.recipient_contact, c.delivery_status, c.last_contacted_at,
               c.next_follow_up_due, c.created_at
        FROM ticket_comments c
        JOIN tickets t ON t.id = c.ticket_id
        {school_filter}
        ORDER BY c.ticket_id ASC, datetime(c.created_at) ASC, c.id ASC
        LIMIT ? OFFSET ?"
    );

    let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];
    if let Some(ids) = scope_school_ids {
        for id in ids {
            params_vec.push(id);
        }
    }

    let total_count: i64 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(params_vec.iter()),
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    params_vec.push(&safe_limit);
    params_vec.push(&safe_offset);

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter()),
            comment_from_row,
        )
        .map_err(|error| error.to_string())?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(Paginated {
        items,
        total_count,
        page: (safe_offset / safe_limit) + 1,
        page_size: safe_limit,
    })
}

pub fn add_comment(conn: &Connection, input: &AddCommentInput) -> Result<TicketComment, String> {
    validate_nonempty("Author", &input.author)?;
    validate_nonempty("Comment", &input.body)?;
    if !input.is_internal {
        validate_nonempty("Audience", &input.audience)?;
        validate_nonempty("Recipient", &input.recipient_name)?;
    }
    if !input.recipient_contact.trim().is_empty() {
        validate_email_or_mobile("Recipient contact", &input.recipient_contact)?;
    }
    let next_follow_up_due = normalize_follow_up_due(input.next_follow_up_due.as_deref())?;
    let channel = if input.is_internal {
        "Internal Note".to_string()
    } else if input.channel.trim().is_empty() {
        "Local".to_string()
    } else {
        input.channel.trim().to_string()
    };
    let audience = if input.is_internal {
        "Internal".to_string()
    } else {
        input.audience.trim().to_string()
    };
    let delivery_status = if input.is_internal {
        "Logged".to_string()
    } else {
        "Prepared".to_string()
    };

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;
    let result = (|| -> Result<TicketComment, String> {
        conn.execute(
            "
            INSERT INTO ticket_comments (
                ticket_id, author, body, is_internal, channel, audience,
                recipient_name, recipient_contact, delivery_status, last_contacted_at, next_follow_up_due
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ",
            params![
                input.ticket_id,
                input.author.trim(),
                input.body.trim(),
                if input.is_internal { 1 } else { 0 },
                channel,
                audience,
                input.recipient_name.trim(),
                input.recipient_contact.trim(),
                delivery_status,
                if input.is_internal {
                    "".to_string()
                } else {
                    current_local_timestamp(conn)?
                },
                next_follow_up_due
            ],
        )
        .map_err(|error| error.to_string())?;

        conn.execute(
            "UPDATE tickets SET updated_at = datetime('now', 'localtime') WHERE id = ?1",
            params![input.ticket_id],
        )
        .map_err(|error| error.to_string())?;

        let comment = conn
            .query_row(
                "
            SELECT id, ticket_id, author, body, is_internal, channel, audience,
                   recipient_name, recipient_contact, delivery_status, last_contacted_at,
                   next_follow_up_due, created_at
            FROM ticket_comments
            WHERE id = ?1
            ",
                params![conn.last_insert_rowid()],
                comment_from_row,
            )
            .map_err(|error| error.to_string())?;

        record_audit(
            conn,
            "ticket",
            input.ticket_id,
            if input.is_internal {
                "internal_note_added"
            } else {
                "communication_logged"
            },
            input.author.trim(),
            &format!(
                "{} via {} to {}",
                if input.is_internal {
                    "Internal note"
                } else {
                    "Reply logged"
                },
                channel_for_audit(&comment.channel),
                if comment.recipient_name.is_empty() {
                    audience_for_audit(&comment.audience)
                } else {
                    comment.recipient_name.as_str()
                }
            ),
        )?;
        Ok(comment)
    })();
    match result {
        Ok(comment) => {
            conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            Ok(comment)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn update_comment_status(
    conn: &Connection,
    input: &UpdateCommentStatusInput,
    actor: &str,
) -> Result<TicketComment, String> {
    validate_comment_status(&input.delivery_status)?;
    let comment = get_comment(conn, input.id)?;
    if comment.is_internal {
        return Err("Internal notes do not support delivery status changes".to_string());
    }

    let next_status = input.delivery_status.trim();
    let next_follow_up_due = normalize_follow_up_due(input.next_follow_up_due.as_deref())?;
    let last_contacted_at = if matches!(next_status, "Sent" | "Acknowledged" | "Failed") {
        current_local_timestamp(conn)?
    } else {
        comment.last_contacted_at.clone()
    };
    conn.execute(
        "
        UPDATE ticket_comments
        SET delivery_status = ?1,
            last_contacted_at = ?2,
            next_follow_up_due = ?3
        WHERE id = ?4
        ",
        params![next_status, last_contacted_at, next_follow_up_due, input.id],
    )
    .map_err(|error| error.to_string())?;

    conn.execute(
        "UPDATE tickets SET updated_at = datetime('now', 'localtime') WHERE id = ?1",
        params![comment.ticket_id],
    )
    .map_err(|error| error.to_string())?;

    let updated = get_comment(conn, input.id)?;
    record_audit(
        conn,
        "ticket",
        comment.ticket_id,
        "communication_status_updated",
        actor,
        &format!(
            "Communication to {} marked {}",
            if updated.recipient_name.is_empty() {
                audience_for_audit(&updated.audience)
            } else {
                updated.recipient_name.as_str()
            },
            updated.delivery_status
        ),
    )?;
    Ok(updated)
}

pub fn list_history(conn: &Connection, ticket_id: i64) -> Result<Vec<TicketHistory>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, actor, field, old_value, new_value, created_at
            FROM ticket_history
            WHERE ticket_id = ?1
            ORDER BY datetime(created_at) DESC, id DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![ticket_id], history_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub fn list_all_history(conn: &Connection) -> Result<Vec<TicketHistory>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, actor, field, old_value, new_value, created_at
            FROM ticket_history
            ORDER BY ticket_id ASC, datetime(created_at) ASC, id ASC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], history_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub fn list_attachments(
    conn: &Connection,
    ticket_id: i64,
) -> Result<Vec<TicketAttachment>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, original_filename, stored_path, size_bytes, uploaded_by, created_at
            FROM ticket_attachments
            WHERE ticket_id = ?1
            ORDER BY datetime(created_at) DESC, id DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![ticket_id], attachment_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub fn list_all_attachments(conn: &Connection) -> Result<Vec<TicketAttachment>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, original_filename, stored_path, size_bytes, uploaded_by, created_at
            FROM ticket_attachments
            ORDER BY ticket_id ASC, datetime(created_at) ASC, id ASC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], attachment_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub fn insert_attachment(
    conn: &Connection,
    ticket_id: i64,
    original_filename: &str,
    stored_path: &str,
    size_bytes: i64,
    uploaded_by: &str,
) -> Result<TicketAttachment, String> {
    conn.execute(
        "
        INSERT INTO ticket_attachments
            (ticket_id, original_filename, stored_path, size_bytes, uploaded_by)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            ticket_id,
            original_filename,
            stored_path,
            size_bytes,
            uploaded_by.trim()
        ],
    )
    .map_err(|error| error.to_string())?;

    let attachment_id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE tickets SET updated_at = datetime('now', 'localtime') WHERE id = ?1",
        params![ticket_id],
    )
    .map_err(|error| error.to_string())?;

    record_history(
        conn,
        ticket_id,
        uploaded_by.trim(),
        "attachment",
        "",
        original_filename,
    )?;

    conn.query_row(
        "
        SELECT id, ticket_id, original_filename, stored_path, size_bytes, uploaded_by, created_at
        FROM ticket_attachments
        WHERE id = ?1
        ",
        params![attachment_id],
        attachment_from_row,
    )
    .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub fn list_migrations(conn: &Connection) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_sla_policies(conn: &Connection) -> Result<Vec<SlaPolicy>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT issue_category, hours
            FROM sla_policies
            ORDER BY issue_category
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SlaPolicy {
                issue_category: row.get(0)?,
                hours: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn update_sla_policy(
    conn: &Connection,
    input: &UpdateSlaPolicyInput,
) -> Result<SlaPolicy, String> {
    validate_nonempty("Issue category", &input.issue_category)?;
    if input.hours < 1 || input.hours > 720 {
        return Err("SLA hours must be between 1 and 720".to_string());
    }

    conn.execute(
        "
        INSERT INTO sla_policies (issue_category, hours, updated_at)
        VALUES (?1, ?2, datetime('now', 'localtime'))
        ON CONFLICT(issue_category) DO UPDATE SET
            hours = excluded.hours,
            updated_at = excluded.updated_at
        ",
        params![input.issue_category.trim(), input.hours],
    )
    .map_err(|error| error.to_string())?;

    get_sla_policy(conn, input.issue_category.trim())
}

pub fn list_assignment_rules(conn: &Connection) -> Result<Vec<AssignmentRule>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT queue, assignee, is_active, updated_at
            FROM assignment_rules
            ORDER BY queue
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], assignment_rule_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn update_assignment_rule(
    conn: &Connection,
    input: &UpdateAssignmentRuleInput,
) -> Result<AssignmentRule, String> {
    validate_nonempty("Queue", &input.queue)?;
    validate_nonempty("Assignee", &input.assignee)?;
    validate_queue(&input.queue)?;

    conn.execute(
        "
        INSERT INTO assignment_rules (queue, assignee, is_active, updated_at)
        VALUES (?1, ?2, ?3, datetime('now', 'localtime'))
        ON CONFLICT(queue) DO UPDATE SET
            assignee = excluded.assignee,
            is_active = excluded.is_active,
            updated_at = excluded.updated_at
        ",
        params![
            input.queue.trim(),
            input.assignee.trim(),
            if input.is_active { 1 } else { 0 }
        ],
    )
    .map_err(|error| error.to_string())?;

    get_assignment_rule(conn, input.queue.trim())
}

pub fn get_escalation_policy(conn: &Connection) -> Result<EscalationPolicy, String> {
    conn.query_row(
        "
        SELECT at_risk_hours, escalation_assignee, auto_assign_on_breach, updated_at
        FROM escalation_policy
        WHERE id = 1
        ",
        [],
        escalation_policy_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Escalation policy was not found".to_string())
}

pub fn update_escalation_policy(
    conn: &Connection,
    input: &UpdateEscalationPolicyInput,
) -> Result<EscalationPolicy, String> {
    validate_nonempty("Escalation assignee", &input.escalation_assignee)?;
    if input.at_risk_hours < 1 || input.at_risk_hours > 720 {
        return Err("At-risk hours must be between 1 and 720".to_string());
    }

    conn.execute(
        "
        INSERT INTO escalation_policy
            (id, at_risk_hours, escalation_assignee, auto_assign_on_breach, updated_at)
        VALUES (1, ?1, ?2, ?3, datetime('now', 'localtime'))
        ON CONFLICT(id) DO UPDATE SET
            at_risk_hours = excluded.at_risk_hours,
            escalation_assignee = excluded.escalation_assignee,
            auto_assign_on_breach = excluded.auto_assign_on_breach,
            updated_at = excluded.updated_at
        ",
        params![
            input.at_risk_hours,
            input.escalation_assignee.trim(),
            if input.auto_assign_on_breach { 1 } else { 0 }
        ],
    )
    .map_err(|error| error.to_string())?;

    refresh_escalations(conn)?;
    get_escalation_policy(conn)
}

pub fn list_communication_templates(
    conn: &Connection,
) -> Result<Vec<CommunicationTemplate>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, name, audience, body, is_active, updated_at
            FROM communication_templates
            ORDER BY is_active DESC, audience, name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], communication_template_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn update_communication_template(
    conn: &Connection,
    input: &UpdateCommunicationTemplateInput,
) -> Result<CommunicationTemplate, String> {
    validate_nonempty("Template name", &input.name)?;
    validate_nonempty("Audience", &input.audience)?;
    validate_nonempty("Template body", &input.body)?;

    if let Some(id) = input.id {
        conn.execute(
            "
            UPDATE communication_templates
            SET name = ?1,
                audience = ?2,
                body = ?3,
                is_active = ?4,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?5
            ",
            params![
                input.name.trim(),
                input.audience.trim(),
                input.body.trim(),
                if input.is_active { 1 } else { 0 },
                id
            ],
        )
        .map_err(|error| error.to_string())?;

        get_communication_template(conn, id)
    } else {
        conn.execute(
            "
            INSERT INTO communication_templates (name, audience, body, is_active)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                input.name.trim(),
                input.audience.trim(),
                input.body.trim(),
                if input.is_active { 1 } else { 0 }
            ],
        )
        .map_err(|error| error.to_string())?;

        get_communication_template(conn, conn.last_insert_rowid())
    }
}

pub fn resolve_ticket_school(
    conn: &Connection,
    school_id: Option<i64>,
    school_name: &str,
) -> Result<(Option<i64>, String), String> {
    if let Some(id) = school_id {
        let school = get_school(conn, id)?;
        if school.is_dropped {
            return Err(format!(
                "School {} is dropped and cannot be used on active tickets",
                school.name
            ));
        }
        return Ok((Some(school.id), school.name));
    }

    let school_name = school_name.trim();
    validate_nonempty("School", school_name)?;
    let school = get_school_by_name(conn, school_name)
        .map_err(|_| format!("School must be selected from master data: {school_name}"))?;
    if school.is_dropped {
        return Err(format!(
            "School {} is dropped and cannot be used on active tickets",
            school.name
        ));
    }
    Ok((Some(school.id), school.name))
}

pub fn get_comment(conn: &Connection, id: i64) -> Result<TicketComment, String> {
    conn.query_row(
        "
        SELECT id, ticket_id, author, body, is_internal, channel, audience,
               recipient_name, recipient_contact, delivery_status, last_contacted_at,
               next_follow_up_due, created_at
        FROM ticket_comments
        WHERE id = ?1
        ",
        params![id],
        comment_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Comment {id} was not found"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        db::initialize_db(&conn).expect("initialize schema");
        conn
    }

    const TEST_ACTOR: &str = "Test User";

    fn create_input() -> CreateTicketInput {
        CreateTicketInput {
            title: "Payroll issue".to_string(),
            description: "Cannot submit payroll".to_string(),
            requester: "Priya".to_string(),
            priority: "Critical".to_string(),
            school_id: None,
            school_name: "Green Valley Public School".to_string(),
            student_name: "Aarav Shah".to_string(),
            grade_level: "Grade 11".to_string(),
            program_track: "JEE Foundation".to_string(),
            issue_category: "Academic Support".to_string(),
        }
    }

    fn update_input(ticket: &Ticket) -> UpdateTicketInput {
        UpdateTicketInput {
            id: ticket.id,
            title: ticket.title.clone(),
            description: ticket.description.clone(),
            requester: ticket.requester.clone(),
            status: ticket.status.clone(),
            priority: ticket.priority.clone(),
            assignee: ticket.assignee.clone(),
            queue: ticket.queue.clone(),
            school_id: ticket.school_id,
            school_name: ticket.school_name.clone(),
            student_name: ticket.student_name.clone(),
            grade_level: ticket.grade_level.clone(),
            program_track: ticket.program_track.clone(),
            issue_category: ticket.issue_category.clone(),
        }
    }

    #[test]
    fn create_ticket_trims_required_fields_and_records_history() {
        let conn = test_db();

        let ticket = create_ticket(
            &conn,
            &CreateTicketInput {
                title: "  Payroll issue  ".to_string(),
                description: "  Cannot submit payroll  ".to_string(),
                requester: "  Priya  ".to_string(),
                priority: "Critical".to_string(),
                school_id: None,
                school_name: "  Green Valley Public School  ".to_string(),
                student_name: "  Aarav Shah  ".to_string(),
                grade_level: "  Grade 11  ".to_string(),
                program_track: "  JEE Foundation  ".to_string(),
                issue_category: "  Academic Support  ".to_string(),
            },
            TEST_ACTOR,
        )
        .expect("create ticket");

        assert_eq!(ticket.title, "Payroll issue");
        assert_eq!(ticket.description, "Cannot submit payroll");
        assert_eq!(ticket.requester, "Priya");
        assert_eq!(ticket.assignee, "Academic Coordinator");
        assert_eq!(ticket.status, "Open");
        assert_eq!(ticket.priority, "Critical");
        assert_eq!(ticket.queue, "Academic Support");
        assert_eq!(ticket.school_name, "Green Valley Public School");
        assert_eq!(ticket.student_name, "Aarav Shah");
        assert_eq!(ticket.grade_level, "Grade 11");
        assert_eq!(ticket.program_track, "JEE Foundation");
        assert_eq!(ticket.issue_category, "Academic Support");
        assert!(!ticket.sla_due_at.is_empty());
        assert_eq!(ticket.escalation_status, "None");
        assert_eq!(ticket.escalated_at, "");

        let history = list_history(&conn, ticket.id).expect("list history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].field, "ticket");
        assert_eq!(history[0].new_value, "Created");
    }
    #[test]
    fn create_ticket_rejects_blank_required_fields() {
        let conn = test_db();

        let error = create_ticket(
            &conn,
            &CreateTicketInput {
                title: " ".to_string(),
                ..create_input()
            },
            TEST_ACTOR,
        )
        .expect_err("blank title should fail");

        assert_eq!(error, "Title is required");
    }
    #[test]
    fn create_ticket_rejects_invalid_priority() {
        let conn = test_db();

        let error = create_ticket(
            &conn,
            &CreateTicketInput {
                priority: "Emergency".to_string(),
                ..create_input()
            },
            TEST_ACTOR,
        )
        .expect_err("invalid priority should fail");

        assert_eq!(
            error,
            "Priority must be one of: Low, Medium, High, Critical"
        );
    }
    #[test]
    fn sla_policies_can_be_updated_and_validated() {
        let conn = test_db();

        let updated = update_sla_policy(
            &conn,
            &UpdateSlaPolicyInput {
                issue_category: "Assessment".to_string(),
                hours: 6,
            },
        )
        .expect("update SLA policy");

        let error = update_sla_policy(
            &conn,
            &UpdateSlaPolicyInput {
                issue_category: "Assessment".to_string(),
                hours: 0,
            },
        )
        .expect_err("zero-hour SLA should fail");

        assert_eq!(updated.issue_category, "Assessment");
        assert_eq!(updated.hours, 6);
        assert_eq!(get_sla_policy_hours(&conn, "Assessment"), Ok(6));
        assert_eq!(error, "SLA hours must be between 1 and 720");
    }
    #[test]
    fn create_ticket_requires_a_matching_sla_policy() {
        let conn = test_db();

        let error = create_ticket(
            &conn,
            &CreateTicketInput {
                issue_category: "Unknown Category".to_string(),
                ..create_input()
            },
            TEST_ACTOR,
        )
        .expect_err("unknown policy should fail");

        assert_eq!(error, "SLA policy was not found for Unknown Category");
    }
    #[test]
    fn refresh_escalations_marks_breached_tickets_and_clears_closed_tickets() {
        let conn = test_db();
        let ticket = get_ticket(&conn, 1).expect("seed ticket");

        conn.execute(
            "
            UPDATE tickets
            SET sla_due_at = datetime('now', '+7 days', 'localtime'),
                escalation_status = 'None',
                escalated_at = ''
            ",
            [],
        )
        .expect("reset seed escalations");

        conn.execute(
            "
            UPDATE tickets
            SET sla_due_at = datetime('now', '-1 hour', 'localtime'),
                escalation_status = 'None',
                escalated_at = ''
            WHERE id = ?1
            ",
            params![ticket.id],
        )
        .expect("force breached SLA");

        let changed = refresh_escalations(&conn).expect("refresh escalations");
        let escalated = get_ticket(&conn, ticket.id).expect("get escalated ticket");

        assert_eq!(changed, 1);
        assert_eq!(escalated.escalation_status, "Escalated");
        assert_eq!(escalated.assignee, "Program Supervisor");
        assert!(!escalated.escalated_at.is_empty());

        let closed = update_ticket(
            &conn,
            &UpdateTicketInput {
                status: "Closed".to_string(),
                ..update_input(&escalated)
            },
            TEST_ACTOR,
        )
        .expect("close ticket");

        assert_eq!(closed.escalation_status, "None");
        assert_eq!(closed.escalated_at, "");
    }
    #[test]
    fn escalation_policy_can_be_updated_and_controls_refresh() {
        let conn = test_db();
        let ticket = get_ticket(&conn, 1).expect("seed ticket");

        let policy = update_escalation_policy(
            &conn,
            &UpdateEscalationPolicyInput {
                at_risk_hours: 12,
                escalation_assignee: "Academic Director".to_string(),
                auto_assign_on_breach: true,
            },
        )
        .expect("update escalation policy");

        conn.execute(
            "
            UPDATE tickets
            SET sla_due_at = datetime('now', '+18 hours', 'localtime'),
                escalation_status = 'None',
                escalated_at = ''
            WHERE id = ?1
            ",
            params![ticket.id],
        )
        .expect("force future SLA");

        refresh_escalations(&conn).expect("refresh escalations");
        let on_track = get_ticket(&conn, ticket.id).expect("get ticket");

        conn.execute(
            "
            UPDATE tickets
            SET sla_due_at = datetime('now', '-1 hour', 'localtime'),
                escalation_status = 'None',
                escalated_at = ''
            WHERE id = ?1
            ",
            params![ticket.id],
        )
        .expect("force breached SLA");

        refresh_escalations(&conn).expect("refresh escalations");
        let escalated = get_ticket(&conn, ticket.id).expect("get escalated ticket");

        assert_eq!(policy.at_risk_hours, 12);
        assert_eq!(policy.escalation_assignee, "Academic Director");
        assert_eq!(on_track.escalation_status, "None");
        assert_eq!(escalated.escalation_status, "Escalated");
        assert_eq!(escalated.assignee, "Academic Director");
    }
    #[test]
    fn escalation_policy_rejects_invalid_values() {
        let conn = test_db();

        let invalid_hours = update_escalation_policy(
            &conn,
            &UpdateEscalationPolicyInput {
                at_risk_hours: 0,
                escalation_assignee: "Academic Director".to_string(),
                auto_assign_on_breach: true,
            },
        )
        .expect_err("zero-hour threshold should fail");

        let blank_assignee = update_escalation_policy(
            &conn,
            &UpdateEscalationPolicyInput {
                at_risk_hours: 12,
                escalation_assignee: " ".to_string(),
                auto_assign_on_breach: true,
            },
        )
        .expect_err("blank assignee should fail");

        assert_eq!(invalid_hours, "At-risk hours must be between 1 and 720");
        assert_eq!(blank_assignee, "Escalation assignee is required");
    }
    #[test]
    fn communication_templates_can_be_created_and_updated() {
        let conn = test_db();

        let created = update_communication_template(
            &conn,
            &UpdateCommunicationTemplateInput {
                id: None,
                name: "Custom parent note".to_string(),
                audience: "Parent".to_string(),
                body: "We will follow up after review.".to_string(),
                is_active: true,
            },
        )
        .expect("create template");

        let updated = update_communication_template(
            &conn,
            &UpdateCommunicationTemplateInput {
                id: Some(created.id),
                name: "Custom parent note".to_string(),
                audience: "Parent".to_string(),
                body: "We will follow up after academic review.".to_string(),
                is_active: false,
            },
        )
        .expect("update template");

        assert_eq!(created.name, "Custom parent note");
        assert_eq!(updated.body, "We will follow up after academic review.");
        assert!(!updated.is_active);
    }
    #[test]
    fn communication_template_rejects_blank_body() {
        let conn = test_db();

        let error = update_communication_template(
            &conn,
            &UpdateCommunicationTemplateInput {
                id: None,
                name: "Blank".to_string(),
                audience: "Parent".to_string(),
                body: " ".to_string(),
                is_active: true,
            },
        )
        .expect_err("blank body should fail");

        assert_eq!(error, "Template body is required");
    }
    #[test]
    fn assignment_rules_can_be_updated_and_used_for_new_tickets() {
        let conn = test_db();

        let rule = update_assignment_rule(
            &conn,
            &UpdateAssignmentRuleInput {
                queue: "Academic Support".to_string(),
                assignee: "Program Owner".to_string(),
                is_active: true,
            },
        )
        .expect("update assignment rule");
        let ticket = create_ticket(&conn, &create_input(), TEST_ACTOR).expect("create ticket");

        assert_eq!(rule.queue, "Academic Support");
        assert_eq!(rule.assignee, "Program Owner");
        assert!(rule.is_active);
        assert_eq!(ticket.assignee, "Program Owner");
    }
    #[test]
    fn inactive_assignment_rule_leaves_new_ticket_unassigned() {
        let conn = test_db();

        update_assignment_rule(
            &conn,
            &UpdateAssignmentRuleInput {
                queue: "Academic Support".to_string(),
                assignee: "Program Owner".to_string(),
                is_active: false,
            },
        )
        .expect("disable assignment rule");
        let ticket = create_ticket(&conn, &create_input(), TEST_ACTOR).expect("create ticket");

        assert_eq!(ticket.assignee, "Unassigned");
    }
    #[test]
    fn assignment_rule_rejects_invalid_queue() {
        let conn = test_db();

        let error = update_assignment_rule(
            &conn,
            &UpdateAssignmentRuleInput {
                queue: "Random Queue".to_string(),
                assignee: "Program Owner".to_string(),
                is_active: true,
            },
        )
        .expect_err("invalid queue should fail");

        assert_eq!(
            error,
            "Queue must be one of: Academic Support, Learning Platform, IT / Device, Operations, Parent Communication"
        );
    }
    #[test]
    fn student_timeline_collects_related_ticket_activity() {
        let conn = test_db();
        let ticket = create_ticket(&conn, &create_input(), TEST_ACTOR).expect("create ticket");
        add_comment(
            &conn,
            &AddCommentInput {
                ticket_id: ticket.id,
                author: "Service Desk".to_string(),
                body: "Follow up shared".to_string(),
                is_internal: false,
                channel: "Email".to_string(),
                audience: "School".to_string(),
                recipient_name: "School SPOC".to_string(),
                recipient_contact: "spoc@example.com".to_string(),
                next_follow_up_due: Some("2026-04-30 10:00".to_string()),
            },
        )
        .expect("add comment");
        insert_attachment(
            &conn,
            ticket.id,
            "plan.pdf",
            "/tmp/plan.pdf",
            256,
            "Service Desk",
        )
        .expect("insert attachment");

        let timeline = get_student_timeline(&conn, 1).expect("get student timeline");

        assert_eq!(timeline.student.name, "Aarav Shah");
        assert_eq!(timeline.student.school_name, "Green Valley Public School");
        assert_eq!(timeline.tickets.len(), 2);
        assert_eq!(timeline.comments.len(), 1);
        assert_eq!(timeline.attachments.len(), 1);
        assert!(timeline.history.iter().any(|item| item.field == "ticket"));
    }
    #[test]
    fn update_ticket_records_only_changed_fields() {
        let conn = test_db();
        conn.execute(
            "UPDATE tickets SET sla_due_at = datetime('now', '+7 days', 'localtime') WHERE id = 1",
            [],
        )
        .expect("keep seed ticket out of escalation path");
        let ticket = get_ticket(&conn, 1).expect("seed ticket");

        let updated = update_ticket(
            &conn,
            &UpdateTicketInput {
                description: "Updated description".to_string(),
                status: "Pending".to_string(),
                assignee: "Asha".to_string(),
                queue: "Parent Communication".to_string(),
                school_id: None,
                school_name: "North City Senior Secondary".to_string(),
                issue_category: "Attendance".to_string(),
                ..update_input(&ticket)
            },
            TEST_ACTOR,
        )
        .expect("update ticket");

        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.status, "Pending");
        assert_eq!(updated.assignee, "Asha");
        assert_eq!(updated.queue, "Parent Communication");
        assert_eq!(updated.school_name, "North City Senior Secondary");
        assert_eq!(updated.issue_category, "Attendance");

        let fields = list_history(&conn, ticket.id)
            .expect("list history")
            .into_iter()
            .map(|item| item.field)
            .collect::<Vec<_>>();

        assert_eq!(
            fields,
            vec![
                "issue_category",
                "school_name",
                "queue",
                "assignee",
                "status",
                "description"
            ]
        );
    }
    #[test]
    fn update_ticket_rejects_invalid_status_and_priority() {
        let conn = test_db();
        let ticket = get_ticket(&conn, 1).expect("seed ticket");

        let invalid_status = update_ticket(
            &conn,
            &UpdateTicketInput {
                status: "Escalated".to_string(),
                ..update_input(&ticket)
            },
            TEST_ACTOR,
        )
        .expect_err("invalid status should fail");

        let invalid_priority = update_ticket(
            &conn,
            &UpdateTicketInput {
                priority: "Urgent".to_string(),
                ..update_input(&ticket)
            },
            TEST_ACTOR,
        )
        .expect_err("invalid priority should fail");

        assert_eq!(
            invalid_status,
            "Status must be one of: Open, In Progress, Pending, Resolved, Closed"
        );
        assert_eq!(
            invalid_priority,
            "Priority must be one of: Low, Medium, High, Critical"
        );
    }
    #[test]
    fn update_ticket_rejects_invalid_queue() {
        let conn = test_db();
        let ticket = get_ticket(&conn, 1).expect("seed ticket");

        let error = update_ticket(
            &conn,
            &UpdateTicketInput {
                queue: "Random Queue".to_string(),
                ..update_input(&ticket)
            },
            TEST_ACTOR,
        )
        .expect_err("invalid queue should fail");

        assert_eq!(
            error,
            "Queue must be one of: Academic Support, Learning Platform, IT / Device, Operations, Parent Communication"
        );
    }
    #[test]
    fn add_comment_trims_values_and_preserves_internal_flag() {
        let conn = test_db();

        let comment = add_comment(
            &conn,
            &AddCommentInput {
                ticket_id: 1,
                author: "  Service Desk  ".to_string(),
                body: "  Checking logs  ".to_string(),
                is_internal: true,
                channel: "Internal Note".to_string(),
                audience: "Internal".to_string(),
                recipient_name: "".to_string(),
                recipient_contact: "".to_string(),
                next_follow_up_due: None,
            },
        )
        .expect("add comment");

        assert_eq!(comment.author, "Service Desk");
        assert_eq!(comment.body, "Checking logs");
        assert!(comment.is_internal);

        let comments = list_comments(&conn, 1).expect("list comments");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].id, comment.id);
    }
    #[test]
    fn delete_ticket_cascades_related_records() {
        let conn = test_db();

        add_comment(
            &conn,
            &AddCommentInput {
                ticket_id: 1,
                author: "Service Desk".to_string(),
                body: "Reply".to_string(),
                is_internal: false,
                channel: "Email".to_string(),
                audience: "School".to_string(),
                recipient_name: "School SPOC".to_string(),
                recipient_contact: "spoc@example.com".to_string(),
                next_follow_up_due: None,
            },
        )
        .expect("add comment");
        insert_attachment(
            &conn,
            1,
            "example.pdf",
            "/tmp/example.pdf",
            128,
            "Service Desk",
        )
        .expect("insert attachment");

        delete_ticket(&conn, 1).expect("delete ticket");

        assert!(get_ticket(&conn, 1).is_err());
        assert!(list_comments(&conn, 1).expect("list comments").is_empty());
        assert!(list_history(&conn, 1).expect("list history").is_empty());
        assert!(list_attachments(&conn, 1)
            .expect("list attachments")
            .is_empty());
    }
    #[test]
    fn update_comment_status_updates_outbound_communications_only() {
        let conn = test_db();
        let comment = add_comment(
            &conn,
            &AddCommentInput {
                ticket_id: 1,
                author: "Service Desk".to_string(),
                body: "Reply".to_string(),
                is_internal: false,
                channel: "Email".to_string(),
                audience: "School".to_string(),
                recipient_name: "School SPOC".to_string(),
                recipient_contact: "spoc@example.com".to_string(),
                next_follow_up_due: Some("2026-04-30 10:00".to_string()),
            },
        )
        .expect("add outbound comment");

        let updated = update_comment_status(
            &conn,
            &UpdateCommentStatusInput {
                id: comment.id,
                delivery_status: "Sent".to_string(),
                next_follow_up_due: None,
            },
            TEST_ACTOR,
        )
        .expect("update comment status");

        assert_eq!(updated.delivery_status, "Sent");

        let internal = add_comment(
            &conn,
            &AddCommentInput {
                ticket_id: 1,
                author: "Service Desk".to_string(),
                body: "Internal note".to_string(),
                is_internal: true,
                channel: "Internal Note".to_string(),
                audience: "Internal".to_string(),
                recipient_name: "".to_string(),
                recipient_contact: "".to_string(),
                next_follow_up_due: None,
            },
        )
        .expect("add internal note");

        let error = update_comment_status(
            &conn,
            &UpdateCommentStatusInput {
                id: internal.id,
                delivery_status: "Sent".to_string(),
                next_follow_up_due: None,
            },
            TEST_ACTOR,
        )
        .expect_err("internal notes should reject status updates");

        assert_eq!(
            error,
            "Internal notes do not support delivery status changes"
        );
    }
    #[test]
    fn list_tickets_respects_school_scope() {
        let conn = test_db();
        // All 3 seed tickets are visible without scope
        let all = list_tickets(&conn, None, 1000, 0).unwrap().items;
        assert_eq!(all.len(), 3);

        // Scope to school 1 (Green Valley Public School) should only return its tickets
        let scoped = list_tickets(&conn, Some(&[1]), 1000, 0).unwrap().items;
        assert!(!scoped.is_empty());
        assert!(scoped.iter().all(|t| t.school_id == Some(1)));

        // Empty scope (represented as [-1] by scope_filter) should return nothing
        let empty = list_tickets(&conn, Some(&[-1]), 1000, 0).unwrap().items;
        assert!(empty.is_empty());
    }
}

pub fn validate_priority(value: &str) -> Result<(), String> {
    let value = value.trim();
    if ALLOWED_PRIORITIES.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Priority must be one of: {}",
            ALLOWED_PRIORITIES.join(", ")
        ))
    }
}

pub fn validate_status(value: &str) -> Result<(), String> {
    let value = value.trim();
    if ALLOWED_STATUSES.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Status must be one of: {}",
            ALLOWED_STATUSES.join(", ")
        ))
    }
}

pub fn validate_status_transition(before: &str, after: &str) -> Result<(), String> {
    let after = after.trim();
    let before = before.trim();
    if after == before {
        return Ok(());
    }
    let allowed = match before {
        "Open" => &["In Progress", "Pending", "Resolved", "Closed"][..],
        "In Progress" => &["Open", "Pending", "Resolved", "Closed"][..],
        "Pending" => &["Open", "In Progress", "Resolved", "Closed"][..],
        "Resolved" => &["Open", "In Progress", "Pending", "Closed"][..],
        "Closed" => &["Open", "In Progress", "Pending", "Resolved"][..],
        _ => &[][..],
    };
    if allowed.contains(&after) {
        Ok(())
    } else {
        Err(format!("Cannot transition from {before} to {after}"))
    }
}

pub fn validate_queue(value: &str) -> Result<(), String> {
    let value = value.trim();
    if ALLOWED_QUEUES.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Queue must be one of: {}",
            ALLOWED_QUEUES.join(", ")
        ))
    }
}

pub fn validate_comment_status(value: &str) -> Result<(), String> {
    let value = value.trim();
    if matches!(value, "Draft" | "Sent" | "Delivered" | "Failed" | "Bounced") {
        Ok(())
    } else {
        Err(format!(
            "Delivery status must be one of: Draft, Sent, Delivered, Failed, Bounced"
        ))
    }
}

pub fn normalize_follow_up_due(value: Option<&str>) -> Result<String, String> {
    match value {
        None | Some("") => Ok(String::new()),
        Some(v) => {
            let v = v.trim();
            if v.len() == 16 && v.contains("T") {
                Ok(v.to_string())
            } else if v.len() == 10 {
                Ok(format!("{v}T09:00"))
            } else {
                Ok(v.to_string())
            }
        }
    }
}

pub fn channel_for_audit(channel: &str) -> &str {
    match channel {
        "Email" => "Email",
        "Phone" => "Phone",
        "WhatsApp" => "WhatsApp",
        "In Person" => "In Person",
        _ => "Other",
    }
}

pub fn audience_for_audit(audience: &str) -> &str {
    match audience {
        "School" => "School",
        "Parent" => "Parent",
        "Internal" => "Internal",
        _ => "Other",
    }
}

pub fn current_local_timestamp(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT datetime('now', 'localtime')", [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

pub fn record_history(
    conn: &Connection,
    ticket_id: i64,
    actor: &str,
    field: &str,
    old_value: &str,
    new_value: &str,
) -> Result<(), String> {
    if old_value == new_value {
        return Ok(());
    }

    conn.execute(
        "
        INSERT INTO ticket_history (ticket_id, actor, field, old_value, new_value)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![ticket_id, actor, field, old_value, new_value],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}
