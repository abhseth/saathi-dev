use crate::models::{
    AddCommentInput, AppUser, AssignmentRule, AuditLogEntry, ChangePasswordInput,
    CommunicationTemplate, CreateLectureModelInput, CreateSchoolInput, CreateStudentInput,
    CreateFacultyAssignmentInput, CreateSubjectInput, CreateTicketInput, CreateUserInput,
    EffectiveSubject, EscalationPolicy, FacultyAssignment,
    LectureModel, Region, School, SchoolClassPlan, SchoolProgramDashboard, SchoolRegionHistory,
    SessionUser, SlaPolicy, Student, StudentTimeline, Subject, Ticket, TicketAttachment,
    TicketComment, TicketHistory, UpdateAssignmentRuleInput, UpdateCommentStatusInput,
    UpdateCommunicationTemplateInput, UpdateEscalationPolicyInput, UpdateSlaPolicyInput,
    TimetableSlot, UpdateSubjectInput, UpdateTicketInput, UpdateUserInput, UpsertRegionInput,
    UpsertSchoolClassPlanInput, UpsertTimetableSlotInput,
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
const ALLOWED_BATCH_PATTERNS: &[&str] = &["Weekday", "Weekend", "Both"];

pub fn ticket_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Ticket> {
    Ok(Ticket {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        requester: row.get(3)?,
        assignee: row.get(4)?,
        status: row.get(5)?,
        priority: row.get(6)?,
        queue: row.get(7)?,
        school_id: row.get(8)?,
        school_name: row.get(9)?,
        student_name: row.get(10)?,
        grade_level: row.get(11)?,
        program_track: row.get(12)?,
        issue_category: row.get(13)?,
        sla_due_at: row.get(14)?,
        escalation_status: row.get(15)?,
        escalated_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

pub fn comment_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TicketComment> {
    let is_internal: i64 = row.get(4)?;

    Ok(TicketComment {
        id: row.get(0)?,
        ticket_id: row.get(1)?,
        author: row.get(2)?,
        body: row.get(3)?,
        is_internal: is_internal == 1,
        channel: row.get(5)?,
        audience: row.get(6)?,
        recipient_name: row.get(7)?,
        recipient_contact: row.get(8)?,
        delivery_status: row.get(9)?,
        last_contacted_at: row.get(10)?,
        next_follow_up_due: row.get(11)?,
        created_at: row.get(12)?,
    })
}

pub fn history_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TicketHistory> {
    Ok(TicketHistory {
        id: row.get(0)?,
        ticket_id: row.get(1)?,
        actor: row.get(2)?,
        field: row.get(3)?,
        old_value: row.get(4)?,
        new_value: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn attachment_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TicketAttachment> {
    Ok(TicketAttachment {
        id: row.get(0)?,
        ticket_id: row.get(1)?,
        original_filename: row.get(2)?,
        stored_path: row.get(3)?,
        size_bytes: row.get(4)?,
        uploaded_by: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn list_tickets(conn: &Connection) -> Result<Vec<Ticket>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, title, description, requester, assignee, status, priority,
                   queue, school_id,
                   school_name, student_name, grade_level, program_track, issue_category,
                   sla_due_at, escalation_status, escalated_at,
                   created_at, updated_at
            FROM tickets
            ORDER BY datetime(updated_at) DESC, id DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], ticket_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn refresh_escalations(conn: &Connection) -> Result<usize, String> {
    let policy = get_escalation_policy(conn)?;
    let at_risk_modifier = format!("+{} hours", policy.at_risk_hours);
    let mut stmt = conn
        .prepare(
            "
            SELECT id, escalation_status, assignee,
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
        .query_map(params![at_risk_modifier], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?;

    let changes = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut updated_count = 0;

    for (ticket_id, current_status, current_assignee, next_status) in changes {
        let should_assign_escalation_owner = next_status == "Escalated"
            && policy.auto_assign_on_breach
            && current_assignee != policy.escalation_assignee;

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
                params![next_status, policy.escalation_assignee, ticket_id],
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
                &policy.escalation_assignee,
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
               created_at, updated_at
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

pub fn get_student_timeline(
    conn: &Connection,
    school_name: &str,
    student_name: &str,
) -> Result<StudentTimeline, String> {
    validate_nonempty("School", school_name)?;
    validate_nonempty("Student", student_name)?;

    let tickets = list_tickets(conn)?
        .into_iter()
        .filter(|ticket| {
            ticket.school_name == school_name.trim() && ticket.student_name == student_name.trim()
        })
        .collect::<Vec<_>>();
    let ticket_ids = tickets
        .iter()
        .map(|ticket| ticket.id)
        .collect::<HashSet<_>>();

    Ok(StudentTimeline {
        school_name: school_name.trim().to_string(),
        student_name: student_name.trim().to_string(),
        comments: list_all_comments(conn)?
            .into_iter()
            .filter(|comment| ticket_ids.contains(&comment.ticket_id))
            .collect(),
        history: list_all_history(conn)?
            .into_iter()
            .filter(|item| ticket_ids.contains(&item.ticket_id))
            .collect(),
        attachments: list_all_attachments(conn)?
            .into_iter()
            .filter(|attachment| ticket_ids.contains(&attachment.ticket_id))
            .collect(),
        tickets,
    })
}

pub fn create_ticket(conn: &Connection, input: &CreateTicketInput, actor: &str) -> Result<Ticket, String> {
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

    conn.execute(
        "
        INSERT INTO tickets (
            title, description, requester, assignee, priority, queue,
            school_id, school_name, student_name, grade_level, program_track, issue_category,
            sla_due_at
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            datetime('now', ?13, 'localtime')
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
            format!("+{sla_hours} hours")
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
    refresh_escalations(conn)?;
    get_ticket(conn, ticket_id)
}

pub fn update_ticket(conn: &Connection, input: &UpdateTicketInput, actor: &str) -> Result<Ticket, String> {
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

    let before = get_ticket(conn, input.id)?;
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
            updated_at = datetime('now', 'localtime')
        WHERE id = ?14
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
            input.id
        ],
    )
    .map_err(|error| error.to_string())?;

    record_history(conn, input.id, actor, "title", &before.title, input.title.trim())?;
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
    record_history(conn, input.id, actor, "queue", &before.queue, input.queue.trim())?;
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
    record_audit(
        conn,
        "ticket",
        input.id,
        "updated",
        actor,
        &format!("Updated ticket {}", input.id),
    )?;

    refresh_escalations(conn)?;
    get_ticket(conn, input.id)
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

pub fn list_all_comments(conn: &Connection) -> Result<Vec<TicketComment>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, ticket_id, author, body, is_internal, channel, audience,
                   recipient_name, recipient_contact, delivery_status, last_contacted_at,
                   next_follow_up_due, created_at
            FROM ticket_comments
            ORDER BY ticket_id ASC, datetime(created_at) ASC, id ASC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], comment_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
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

    conn.query_row(
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
    .map_err(|error| error.to_string())
    .and_then(|comment| {
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
    })
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

    record_history(conn, ticket_id, uploaded_by.trim(), "attachment", "", original_filename)?;

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

pub fn list_schools(conn: &Connection) -> Result<Vec<School>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
                   schools.program_model, schools.distance_classification,
                   sip_academic_owner_role, sip_academic_owner_name,
                   sip_academic_owner_mobile, sip_academic_owner_email,
                   center_head_name, center_head_mobile, center_head_email,
                   principal_name, principal_mobile, principal_email,
                   school_spoc_name, school_spoc_mobile, school_spoc_email,
                   central_academic_spoc_name, central_academic_spoc_mobile,
                   central_academic_spoc_email, central_business_spoc_name,
                   central_business_spoc_mobile, central_business_spoc_email,
                   bh_name, bh_mobile, bh_email, aom_name, aom_mobile, aom_email,
                   mapped_vp_center,
                   is_dropped, dropped_at, dropped_reason, schools.created_at
            FROM schools
            LEFT JOIN regions ON regions.id = schools.region_id
            WHERE is_dropped = 0
            ORDER BY schools.name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], school_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_dropped_schools(conn: &Connection) -> Result<Vec<School>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
                   schools.program_model, schools.distance_classification,
                   sip_academic_owner_role, sip_academic_owner_name,
                   sip_academic_owner_mobile, sip_academic_owner_email,
                   center_head_name, center_head_mobile, center_head_email,
                   principal_name, principal_mobile, principal_email,
                   school_spoc_name, school_spoc_mobile, school_spoc_email,
                   central_academic_spoc_name, central_academic_spoc_mobile,
                   central_academic_spoc_email, central_business_spoc_name,
                   central_business_spoc_mobile, central_business_spoc_email,
                   bh_name, bh_mobile, bh_email, aom_name, aom_mobile, aom_email,
                   mapped_vp_center,
                   is_dropped, dropped_at, dropped_reason, schools.created_at
            FROM schools
            LEFT JOIN regions ON regions.id = schools.region_id
            WHERE is_dropped = 1
            ORDER BY datetime(dropped_at) DESC, schools.name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], school_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_school(conn: &Connection, input: &CreateSchoolInput, actor: &str) -> Result<School, String> {
    validate_nonempty("School", &input.name)?;
    validate_school_model(&input.program_model)?;
    validate_distance_classification(&input.distance_classification)?;
    validate_school_contact_fields(input)?;
    if let Some(region_id) = input.region_id {
        get_region(conn, region_id)?;
    }
    let previous_region = get_school_region_by_name(conn, input.name.trim())?;

    conn.execute(
        "
        INSERT INTO schools (
            name, region_id, program_model, distance_classification,
            sip_academic_owner_role, sip_academic_owner_name,
            sip_academic_owner_mobile, sip_academic_owner_email,
            center_head_name, center_head_mobile, center_head_email,
            principal_name, principal_mobile, principal_email,
            school_spoc_name, school_spoc_mobile, school_spoc_email,
            central_academic_spoc_name, central_academic_spoc_mobile,
            central_academic_spoc_email, central_business_spoc_name,
            central_business_spoc_mobile, central_business_spoc_email,
            bh_name, bh_mobile, bh_email, aom_name, aom_mobile, aom_email,
            mapped_vp_center
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
            ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30
        )
        ON CONFLICT(name) DO UPDATE SET
            region_id = excluded.region_id,
            program_model = excluded.program_model,
            distance_classification = excluded.distance_classification,
            sip_academic_owner_role = excluded.sip_academic_owner_role,
            sip_academic_owner_name = excluded.sip_academic_owner_name,
            sip_academic_owner_mobile = excluded.sip_academic_owner_mobile,
            sip_academic_owner_email = excluded.sip_academic_owner_email,
            center_head_name = excluded.center_head_name,
            center_head_mobile = excluded.center_head_mobile,
            center_head_email = excluded.center_head_email,
            principal_name = excluded.principal_name,
            principal_mobile = excluded.principal_mobile,
            principal_email = excluded.principal_email,
            school_spoc_name = excluded.school_spoc_name,
            school_spoc_mobile = excluded.school_spoc_mobile,
            school_spoc_email = excluded.school_spoc_email,
            central_academic_spoc_name = excluded.central_academic_spoc_name,
            central_academic_spoc_mobile = excluded.central_academic_spoc_mobile,
            central_academic_spoc_email = excluded.central_academic_spoc_email,
            central_business_spoc_name = excluded.central_business_spoc_name,
            central_business_spoc_mobile = excluded.central_business_spoc_mobile,
            central_business_spoc_email = excluded.central_business_spoc_email,
            bh_name = excluded.bh_name,
            bh_mobile = excluded.bh_mobile,
            bh_email = excluded.bh_email,
            aom_name = excluded.aom_name,
            aom_mobile = excluded.aom_mobile,
            aom_email = excluded.aom_email,
            mapped_vp_center = excluded.mapped_vp_center
        ",
        params![
            input.name.trim(),
            input.region_id,
            input.program_model.trim(),
            input.distance_classification.trim(),
            input.sip_academic_owner_role.trim(),
            input.sip_academic_owner_name.trim(),
            input.sip_academic_owner_mobile.trim(),
            input.sip_academic_owner_email.trim(),
            input.center_head_name.trim(),
            input.center_head_mobile.trim(),
            input.center_head_email.trim(),
            input.principal_name.trim(),
            input.principal_mobile.trim(),
            input.principal_email.trim(),
            input.school_spoc_name.trim(),
            input.school_spoc_mobile.trim(),
            input.school_spoc_email.trim(),
            input.central_academic_spoc_name.trim(),
            input.central_academic_spoc_mobile.trim(),
            input.central_academic_spoc_email.trim(),
            input.central_business_spoc_name.trim(),
            input.central_business_spoc_mobile.trim(),
            input.central_business_spoc_email.trim(),
            input.bh_name.trim(),
            input.bh_mobile.trim(),
            input.bh_email.trim(),
            input.aom_name.trim(),
            input.aom_mobile.trim(),
            input.aom_email.trim(),
            input.mapped_vp_center.trim(),
        ],
    )
    .map_err(|error| error.to_string())?;

    let school = get_school_by_name(conn, input.name.trim())?;
    let action = if previous_region.is_some() {
        "updated"
    } else {
        "created"
    };
    if let Some((old_region_id, old_region_name)) = previous_region {
        if old_region_id != input.region_id {
            record_school_region_history(
                conn,
                school.id,
                old_region_id,
                &old_region_name,
                input.region_id,
                &school.region_name,
            )?;
        }
    }
    record_audit(
        conn,
        "school",
        school.id,
        action,
        actor,
        &format!("{action} school {}", school.name),
    )?;

    Ok(school)
}

pub fn list_students(conn: &Connection, school_id: Option<i64>) -> Result<Vec<Student>, String> {
    let (sql, params_value): (&str, Vec<i64>) = if let Some(id) = school_id {
        (
            "
            SELECT students.id, students.school_id, schools.name, students.name,
                   students.grade_level, students.program_track, students.track,
                   students.created_at
            FROM students
            JOIN schools ON schools.id = students.school_id
            WHERE students.school_id = ?1 AND schools.is_dropped = 0
            ORDER BY schools.name, students.name
            ",
            vec![id],
        )
    } else {
        (
            "
            SELECT students.id, students.school_id, schools.name, students.name,
                   students.grade_level, students.program_track, students.track,
                   students.created_at
            FROM students
            JOIN schools ON schools.id = students.school_id
            WHERE schools.is_dropped = 0
            ORDER BY schools.name, students.name
            ",
            vec![],
        )
    };

    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = if let Some(id) = params_value.first() {
        stmt.query_map(params![id], student_from_row)
            .map_err(|error| error.to_string())?
    } else {
        stmt.query_map([], student_from_row)
            .map_err(|error| error.to_string())?
    };

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_student(conn: &Connection, input: &CreateStudentInput) -> Result<Student, String> {
    validate_nonempty("Student", &input.name)?;
    validate_nonempty("Grade", &input.grade_level)?;
    validate_nonempty("Program track", &input.program_track)?;
    let _school = get_school(conn, input.school_id)?;

    conn.execute(
        "
        INSERT INTO students (school_id, name, grade_level, program_track, track)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(school_id, name) DO UPDATE SET
            grade_level = excluded.grade_level,
            program_track = excluded.program_track,
            track = excluded.track
        ",
        params![
            input.school_id,
            input.name.trim(),
            input.grade_level.trim(),
            input.program_track.trim(),
            input.track.trim()
        ],
    )
    .map_err(|error| error.to_string())?;

    get_student_by_school_and_name(conn, input.school_id, input.name.trim())
}

pub fn list_regions(conn: &Connection) -> Result<Vec<Region>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, name, regional_academic_head_name, regional_academic_head_mobile,
                   regional_academic_head_email, regional_business_head_name,
                   regional_business_head_mobile, regional_business_head_email, updated_at
            FROM regions
            ORDER BY name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], region_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn upsert_region(conn: &Connection, input: &UpsertRegionInput, actor: &str) -> Result<Region, String> {
    validate_nonempty("Region", &input.name)?;
    validate_email(
        "Regional Academic Head email",
        &input.regional_academic_head_email,
    )?;
    validate_email(
        "Regional Business Head email",
        &input.regional_business_head_email,
    )?;
    validate_mobile(
        "Regional Academic Head mobile",
        &input.regional_academic_head_mobile,
    )?;
    validate_mobile(
        "Regional Business Head mobile",
        &input.regional_business_head_mobile,
    )?;

    if let Some(id) = input.id {
        conn.execute(
            "
            UPDATE regions
            SET name = ?1,
                regional_academic_head_name = ?2,
                regional_academic_head_mobile = ?3,
                regional_academic_head_email = ?4,
                regional_business_head_name = ?5,
                regional_business_head_mobile = ?6,
                regional_business_head_email = ?7,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?8
            ",
            params![
                input.name.trim(),
                input.regional_academic_head_name.trim(),
                input.regional_academic_head_mobile.trim(),
                input.regional_academic_head_email.trim(),
                input.regional_business_head_name.trim(),
                input.regional_business_head_mobile.trim(),
                input.regional_business_head_email.trim(),
                id
            ],
        )
        .map_err(|error| error.to_string())?;

        let region = get_region(conn, id)?;
        record_audit(
            conn,
            "region",
            region.id,
            "updated",
            actor,
            &format!("Updated region {}", region.name),
        )?;
        Ok(region)
    } else {
        conn.execute(
            "
            INSERT INTO regions (
                name, regional_academic_head_name, regional_academic_head_mobile,
                regional_academic_head_email, regional_business_head_name,
                regional_business_head_mobile, regional_business_head_email
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(name) DO UPDATE SET
                regional_academic_head_name = excluded.regional_academic_head_name,
                regional_academic_head_mobile = excluded.regional_academic_head_mobile,
                regional_academic_head_email = excluded.regional_academic_head_email,
                regional_business_head_name = excluded.regional_business_head_name,
                regional_business_head_mobile = excluded.regional_business_head_mobile,
                regional_business_head_email = excluded.regional_business_head_email,
                updated_at = datetime('now', 'localtime')
            ",
            params![
                input.name.trim(),
                input.regional_academic_head_name.trim(),
                input.regional_academic_head_mobile.trim(),
                input.regional_academic_head_email.trim(),
                input.regional_business_head_name.trim(),
                input.regional_business_head_mobile.trim(),
                input.regional_business_head_email.trim()
            ],
        )
        .map_err(|error| error.to_string())?;

        let region = get_region_by_name(conn, input.name.trim())?;
        record_audit(
            conn,
            "region",
            region.id,
            "upserted",
            actor,
            &format!("Upserted region {}", region.name),
        )?;
        Ok(region)
    }
}

pub fn delete_region(conn: &Connection, id: i64) -> Result<(), String> {
    let linked_schools: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schools WHERE region_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if linked_schools > 0 {
        return Err(
            "Region is mapped to schools. Move those schools to another region before deleting."
                .to_string(),
        );
    }

    let deleted = conn
        .execute("DELETE FROM regions WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;

    if deleted == 0 {
        Err(format!("Region {id} was not found"))
    } else {
        Ok(())
    }
}

pub fn drop_school(conn: &Connection, id: i64, reason: &str, actor: &str) -> Result<School, String> {
    validate_nonempty("Drop reason", reason)?;
    let before = get_school(conn, id)?;
    conn.execute(
        "
        UPDATE schools
        SET is_dropped = 1,
            dropped_at = datetime('now', 'localtime'),
            dropped_reason = ?1
        WHERE id = ?2
        ",
        params![reason.trim(), id],
    )
    .map_err(|error| error.to_string())?;

    let school = get_school(conn, id)?;
    record_audit(
        conn,
        "school",
        id,
        "dropped",
        actor,
        &format!("Dropped school {}: {}", before.name, reason.trim()),
    )?;
    Ok(school)
}

pub fn delete_school(conn: &Connection, id: i64, actor: &str) -> Result<(), String> {
    let school = get_school(conn, id)?;
    conn.execute("DELETE FROM schools WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    record_audit(
        conn,
        "school",
        id,
        "deleted",
        actor,
        &format!("Deleted school {}", school.name),
    )?;
    Ok(())
}

pub fn restore_school(conn: &Connection, id: i64, actor: &str) -> Result<School, String> {
    let before = get_school(conn, id)?;
    conn.execute(
        "
        UPDATE schools
        SET is_dropped = 0,
            dropped_at = '',
            dropped_reason = ''
        WHERE id = ?1
        ",
        params![id],
    )
    .map_err(|error| error.to_string())?;

    let school = get_school(conn, id)?;
    record_audit(
        conn,
        "school",
        id,
        "restored",
        actor,
        &format!("Restored school {}", before.name),
    )?;
    Ok(school)
}

pub fn list_audit_log(conn: &Connection, limit: i64) -> Result<Vec<AuditLogEntry>, String> {
    let safe_limit = limit.clamp(1, 500);
    let mut stmt = conn
        .prepare(
            "
            SELECT id, entity_type, entity_id, action, actor, summary, created_at
            FROM audit_log
            ORDER BY datetime(created_at) DESC, id DESC
            LIMIT ?1
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![safe_limit], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                action: row.get(3)?,
                actor: row.get(4)?,
                summary: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_school_region_history(conn: &Connection) -> Result<Vec<SchoolRegionHistory>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT school_region_history.id, school_region_history.school_id, schools.name,
                   school_region_history.old_region_id, school_region_history.old_region_name,
                   school_region_history.new_region_id, school_region_history.new_region_name,
                   school_region_history.changed_at
            FROM school_region_history
            JOIN schools ON schools.id = school_region_history.school_id
            ORDER BY datetime(school_region_history.changed_at) DESC, school_region_history.id DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SchoolRegionHistory {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                old_region_id: row.get(3)?,
                old_region_name: row.get(4)?,
                new_region_id: row.get(5)?,
                new_region_name: row.get(6)?,
                changed_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_lecture_models(conn: &Connection) -> Result<Vec<LectureModel>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, name, days_per_week, lectures_per_day, created_at
            FROM lecture_models
            ORDER BY days_per_week, lectures_per_day, name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], lecture_model_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_lecture_model(
    conn: &Connection,
    input: &CreateLectureModelInput,
) -> Result<LectureModel, String> {
    validate_nonempty("Lecture model", &input.name)?;
    if input.days_per_week <= 0 || input.lectures_per_day <= 0 {
        return Err("Lecture model days and lectures must be greater than zero".to_string());
    }

    conn.execute(
        "
        INSERT INTO lecture_models (name, days_per_week, lectures_per_day)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(name) DO UPDATE SET
            days_per_week = excluded.days_per_week,
            lectures_per_day = excluded.lectures_per_day
        ",
        params![
            input.name.trim(),
            input.days_per_week,
            input.lectures_per_day
        ],
    )
    .map_err(|error| error.to_string())?;

    get_lecture_model_by_name(conn, input.name.trim())
}

pub fn list_school_class_plans(
    conn: &Connection,
    school_id: Option<i64>,
) -> Result<Vec<SchoolClassPlan>, String> {
    let (sql, params_value): (&str, Vec<i64>) = if let Some(id) = school_id {
        (
            "
            SELECT school_class_plans.id, school_class_plans.school_id, schools.name,
                   school_class_plans.grade_level, school_class_plans.track,
                   school_class_plans.lecture_model_id,
                   lecture_models.name, lecture_models.days_per_week, lecture_models.lectures_per_day,
                   school_class_plans.batch_pattern, school_class_plans.aop_admissions,
                   school_class_plans.registrations, school_class_plans.actual_admissions,
                   school_class_plans.updated_at
            FROM school_class_plans
            JOIN schools ON schools.id = school_class_plans.school_id
            JOIN lecture_models ON lecture_models.id = school_class_plans.lecture_model_id
            WHERE school_class_plans.school_id = ?1 AND schools.is_dropped = 0
            ORDER BY schools.name, school_class_plans.grade_level
            ",
            vec![id],
        )
    } else {
        (
            "
            SELECT school_class_plans.id, school_class_plans.school_id, schools.name,
                   school_class_plans.grade_level, school_class_plans.track,
                   school_class_plans.lecture_model_id,
                   lecture_models.name, lecture_models.days_per_week, lecture_models.lectures_per_day,
                   school_class_plans.batch_pattern, school_class_plans.aop_admissions,
                   school_class_plans.registrations, school_class_plans.actual_admissions,
                   school_class_plans.updated_at
            FROM school_class_plans
            JOIN schools ON schools.id = school_class_plans.school_id
            JOIN lecture_models ON lecture_models.id = school_class_plans.lecture_model_id
            WHERE schools.is_dropped = 0
            ORDER BY schools.name, school_class_plans.grade_level
            ",
            Vec::new(),
        )
    };

    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = if params_value.is_empty() {
        stmt.query_map([], school_class_plan_from_row)
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
    } else {
        stmt.query_map(params![params_value[0]], school_class_plan_from_row)
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
    };

    rows.map_err(|error| error.to_string())
}

pub fn list_all_school_class_plans(conn: &Connection) -> Result<Vec<SchoolClassPlan>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT school_class_plans.id, school_class_plans.school_id, schools.name,
                   school_class_plans.grade_level, school_class_plans.track,
                   school_class_plans.lecture_model_id,
                   lecture_models.name, lecture_models.days_per_week, lecture_models.lectures_per_day,
                   school_class_plans.batch_pattern, school_class_plans.aop_admissions,
                   school_class_plans.registrations, school_class_plans.actual_admissions,
                   school_class_plans.updated_at
            FROM school_class_plans
            JOIN schools ON schools.id = school_class_plans.school_id
            JOIN lecture_models ON lecture_models.id = school_class_plans.lecture_model_id
            ORDER BY schools.name, school_class_plans.grade_level
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], school_class_plan_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn upsert_school_class_plan(
    conn: &Connection,
    input: &UpsertSchoolClassPlanInput,
) -> Result<SchoolClassPlan, String> {
    get_school(conn, input.school_id)?;
    validate_nonempty("Grade", &input.grade_level)?;
    validate_batch_pattern(&input.batch_pattern)?;
    if input.aop_admissions < 0 || input.registrations < 0 || input.actual_admissions < 0 {
        return Err("Admission numbers cannot be negative".to_string());
    }
    get_lecture_model(conn, input.lecture_model_id)?;

    conn.execute(
        "
        INSERT INTO school_class_plans (
            school_id, grade_level, track, lecture_model_id, batch_pattern,
            aop_admissions, registrations, actual_admissions
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(school_id, grade_level, track) DO UPDATE SET
            lecture_model_id = excluded.lecture_model_id,
            batch_pattern = excluded.batch_pattern,
            aop_admissions = excluded.aop_admissions,
            registrations = excluded.registrations,
            actual_admissions = excluded.actual_admissions,
            updated_at = datetime('now', 'localtime')
        ",
        params![
            input.school_id,
            input.grade_level.trim(),
            input.track.trim(),
            input.lecture_model_id,
            input.batch_pattern.trim(),
            input.aop_admissions,
            input.registrations,
            input.actual_admissions
        ],
    )
    .map_err(|error| error.to_string())?;

    get_school_class_plan(conn, input.school_id, input.grade_level.trim(), input.track.trim())
}

pub fn get_school_program_dashboard(conn: &Connection) -> Result<SchoolProgramDashboard, String> {
    let plans = list_school_class_plans(conn, None)?;
    let total_schools = count_active_schools(conn)?;
    let schools_with_class_plans = plans
        .iter()
        .map(|plan| plan.school_id)
        .collect::<HashSet<_>>()
        .len() as i64;
    let total_aop_admissions = plans.iter().map(|plan| plan.aop_admissions).sum::<i64>();
    let total_actual_admissions = plans.iter().map(|plan| plan.actual_admissions).sum::<i64>();
    let admission_gap = total_aop_admissions - total_actual_admissions;

    Ok(SchoolProgramDashboard {
        total_schools,
        schools_with_class_plans,
        total_classes: plans.len() as i64,
        total_aop_admissions,
        total_actual_admissions,
        admission_gap,
        admission_attainment_percent: percent(total_actual_admissions, total_aop_admissions),
        remote_school_count: count_school_field(conn, "distance_classification", "Remote")?,
        near_proximity_school_count: count_school_field(
            conn,
            "distance_classification",
            "Near Proximity",
        )?,
        aspire_school_count: count_school_field(conn, "program_model", "Aspire")?,
        minimum_guarantee_school_count: count_school_field(
            conn,
            "program_model",
            "Minimum Guarantee",
        )?,
        class_plans: plans,
    })
}

pub fn validate_nonempty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label} is required"))
    } else {
        Ok(())
    }
}

fn get_school(conn: &Connection, id: i64) -> Result<School, String> {
    conn.query_row(
        "
        SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
               schools.program_model, schools.distance_classification,
               sip_academic_owner_role, sip_academic_owner_name,
               sip_academic_owner_mobile, sip_academic_owner_email,
               center_head_name, center_head_mobile, center_head_email,
               principal_name, principal_mobile, principal_email,
               school_spoc_name, school_spoc_mobile, school_spoc_email,
               central_academic_spoc_name, central_academic_spoc_mobile,
               central_academic_spoc_email, central_business_spoc_name,
               central_business_spoc_mobile, central_business_spoc_email,
               bh_name, bh_mobile, bh_email, aom_name, aom_mobile, aom_email,
               mapped_vp_center,
               is_dropped, dropped_at, dropped_reason, schools.created_at
        FROM schools
        LEFT JOIN regions ON regions.id = schools.region_id
        WHERE schools.id = ?1
        ",
        params![id],
        school_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("School {id} was not found"))
}

fn get_school_by_name(conn: &Connection, name: &str) -> Result<School, String> {
    conn.query_row(
        "
        SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
               schools.program_model, schools.distance_classification,
               sip_academic_owner_role, sip_academic_owner_name,
               sip_academic_owner_mobile, sip_academic_owner_email,
               center_head_name, center_head_mobile, center_head_email,
               principal_name, principal_mobile, principal_email,
               school_spoc_name, school_spoc_mobile, school_spoc_email,
               central_academic_spoc_name, central_academic_spoc_mobile,
               central_academic_spoc_email, central_business_spoc_name,
               central_business_spoc_mobile, central_business_spoc_email,
               bh_name, bh_mobile, bh_email, aom_name, aom_mobile, aom_email,
               mapped_vp_center,
               is_dropped, dropped_at, dropped_reason, schools.created_at
        FROM schools
        LEFT JOIN regions ON regions.id = schools.region_id
        WHERE schools.name = ?1
        ",
        params![name],
        school_from_row,
    )
    .map_err(|error| error.to_string())
}

fn resolve_ticket_school(
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

fn school_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<School> {
    Ok(School {
        id: row.get(0)?,
        name: row.get(1)?,
        region_id: row.get(2)?,
        region_name: row.get(3)?,
        program_model: row.get(4)?,
        distance_classification: row.get(5)?,
        sip_academic_owner_role: row.get(6)?,
        sip_academic_owner_name: row.get(7)?,
        sip_academic_owner_mobile: row.get(8)?,
        sip_academic_owner_email: row.get(9)?,
        center_head_name: row.get(10)?,
        center_head_mobile: row.get(11)?,
        center_head_email: row.get(12)?,
        principal_name: row.get(13)?,
        principal_mobile: row.get(14)?,
        principal_email: row.get(15)?,
        school_spoc_name: row.get(16)?,
        school_spoc_mobile: row.get(17)?,
        school_spoc_email: row.get(18)?,
        central_academic_spoc_name: row.get(19)?,
        central_academic_spoc_mobile: row.get(20)?,
        central_academic_spoc_email: row.get(21)?,
        central_business_spoc_name: row.get(22)?,
        central_business_spoc_mobile: row.get(23)?,
        central_business_spoc_email: row.get(24)?,
        bh_name: row.get(25)?,
        bh_mobile: row.get(26)?,
        bh_email: row.get(27)?,
        aom_name: row.get(28)?,
        aom_mobile: row.get(29)?,
        aom_email: row.get(30)?,
        mapped_vp_center: row.get(31)?,
        is_dropped: row.get::<_, i64>(32)? == 1,
        dropped_at: row.get(33)?,
        dropped_reason: row.get(34)?,
        created_at: row.get(35)?,
    })
}

fn get_school_region_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Option<(Option<i64>, String)>, String> {
    conn.query_row(
        "
        SELECT schools.region_id, COALESCE(regions.name, '')
        FROM schools
        LEFT JOIN regions ON regions.id = schools.region_id
        WHERE schools.name = ?1
        ",
        params![name],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn record_school_region_history(
    conn: &Connection,
    school_id: i64,
    old_region_id: Option<i64>,
    old_region_name: &str,
    new_region_id: Option<i64>,
    new_region_name: &str,
) -> Result<(), String> {
    conn.execute(
        "
        INSERT INTO school_region_history (
            school_id, old_region_id, old_region_name, new_region_id, new_region_name
        )
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            school_id,
            old_region_id,
            old_region_name,
            new_region_id,
            new_region_name
        ],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn region_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Region> {
    Ok(Region {
        id: row.get(0)?,
        name: row.get(1)?,
        regional_academic_head_name: row.get(2)?,
        regional_academic_head_mobile: row.get(3)?,
        regional_academic_head_email: row.get(4)?,
        regional_business_head_name: row.get(5)?,
        regional_business_head_mobile: row.get(6)?,
        regional_business_head_email: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn get_region(conn: &Connection, id: i64) -> Result<Region, String> {
    conn.query_row(
        "
        SELECT id, name, regional_academic_head_name, regional_academic_head_mobile,
               regional_academic_head_email, regional_business_head_name,
               regional_business_head_mobile, regional_business_head_email, updated_at
        FROM regions
        WHERE id = ?1
        ",
        params![id],
        region_from_row,
    )
    .map_err(|error| error.to_string())
}

fn get_region_by_name(conn: &Connection, name: &str) -> Result<Region, String> {
    conn.query_row(
        "
        SELECT id, name, regional_academic_head_name, regional_academic_head_mobile,
               regional_academic_head_email, regional_business_head_name,
               regional_business_head_mobile, regional_business_head_email, updated_at
        FROM regions
        WHERE name = ?1
        ",
        params![name],
        region_from_row,
    )
    .map_err(|error| error.to_string())
}

fn get_student_by_school_and_name(
    conn: &Connection,
    school_id: i64,
    name: &str,
) -> Result<Student, String> {
    conn.query_row(
        "
        SELECT students.id, students.school_id, schools.name, students.name,
               students.grade_level, students.program_track, students.track,
               students.created_at
        FROM students
        JOIN schools ON schools.id = students.school_id
        WHERE students.school_id = ?1 AND students.name = ?2
        ",
        params![school_id, name],
        student_from_row,
    )
    .map_err(|error| error.to_string())
}

fn student_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Student> {
    Ok(Student {
        id: row.get(0)?,
        school_id: row.get(1)?,
        school_name: row.get(2)?,
        name: row.get(3)?,
        grade_level: row.get(4)?,
        program_track: row.get(5)?,
        track: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn lecture_model_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LectureModel> {
    Ok(LectureModel {
        id: row.get(0)?,
        name: row.get(1)?,
        days_per_week: row.get(2)?,
        lectures_per_day: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn school_class_plan_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SchoolClassPlan> {
    let aop_admissions: i64 = row.get(10)?;
    let registrations: i64 = row.get(11)?;
    let actual_admissions: i64 = row.get(12)?;
    Ok(SchoolClassPlan {
        id: row.get(0)?,
        school_id: row.get(1)?,
        school_name: row.get(2)?,
        grade_level: row.get(3)?,
        track: row.get(4)?,
        lecture_model_id: row.get(5)?,
        lecture_model_name: row.get(6)?,
        days_per_week: row.get(7)?,
        lectures_per_day: row.get(8)?,
        batch_pattern: row.get(9)?,
        aop_admissions,
        registrations,
        actual_admissions,
        admission_gap: aop_admissions - actual_admissions,
        admission_attainment_percent: percent(actual_admissions, aop_admissions),
        updated_at: row.get(13)?,
    })
}

fn get_lecture_model(conn: &Connection, id: i64) -> Result<LectureModel, String> {
    conn.query_row(
        "
        SELECT id, name, days_per_week, lectures_per_day, created_at
        FROM lecture_models
        WHERE id = ?1
        ",
        params![id],
        lecture_model_from_row,
    )
    .map_err(|error| error.to_string())
}

fn get_lecture_model_by_name(conn: &Connection, name: &str) -> Result<LectureModel, String> {
    conn.query_row(
        "
        SELECT id, name, days_per_week, lectures_per_day, created_at
        FROM lecture_models
        WHERE name = ?1
        ",
        params![name],
        lecture_model_from_row,
    )
    .map_err(|error| error.to_string())
}

fn get_school_class_plan(
    conn: &Connection,
    school_id: i64,
    grade_level: &str,
    track: &str,
) -> Result<SchoolClassPlan, String> {
    conn.query_row(
        "
        SELECT school_class_plans.id, school_class_plans.school_id, schools.name,
               school_class_plans.grade_level, school_class_plans.track,
               school_class_plans.lecture_model_id,
               lecture_models.name, lecture_models.days_per_week, lecture_models.lectures_per_day,
               school_class_plans.batch_pattern, school_class_plans.aop_admissions,
               school_class_plans.registrations, school_class_plans.actual_admissions,
               school_class_plans.updated_at
        FROM school_class_plans
        JOIN schools ON schools.id = school_class_plans.school_id
        JOIN lecture_models ON lecture_models.id = school_class_plans.lecture_model_id
        WHERE school_class_plans.school_id = ?1
          AND school_class_plans.grade_level = ?2
          AND school_class_plans.track = ?3
        ",
        params![school_id, grade_level, track],
        school_class_plan_from_row,
    )
    .map_err(|error| error.to_string())
}

fn count_active_schools(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM schools WHERE is_dropped = 0",
        [],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn count_school_field(conn: &Connection, field_name: &str, value: &str) -> Result<i64, String> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM schools WHERE is_dropped = 0 AND {field_name} = ?1"),
        params![value],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn percent(numerator: i64, denominator: i64) -> i64 {
    if denominator <= 0 {
        0
    } else {
        ((numerator as f64 / denominator as f64) * 100.0).round() as i64
    }
}

fn validate_priority(value: &str) -> Result<(), String> {
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

fn validate_status(value: &str) -> Result<(), String> {
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

fn validate_queue(value: &str) -> Result<(), String> {
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

fn validate_batch_pattern(value: &str) -> Result<(), String> {
    let value = value.trim();
    if ALLOWED_BATCH_PATTERNS.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Batch pattern must be one of: {}",
            ALLOWED_BATCH_PATTERNS.join(", ")
        ))
    }
}

fn validate_school_model(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "Aspire" | "Minimum Guarantee") {
        Ok(())
    } else {
        Err("School model must be Aspire or Minimum Guarantee".to_string())
    }
}

fn validate_distance_classification(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "Remote" | "Near Proximity") {
        Ok(())
    } else {
        Err("Distance classification must be Remote or Near Proximity".to_string())
    }
}

fn validate_email(label: &str, value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || (value.contains('@') && value.contains('.') && !value.contains(' ')) {
        Ok(())
    } else {
        Err(format!("{label} must be a valid email address"))
    }
}

fn validate_mobile(label: &str, value: &str) -> Result<(), String> {
    let value = value.trim();
    let digit_count = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .count();
    let valid_symbols = value.chars().all(|character| {
        character.is_ascii_digit() || matches!(character, '+' | '-' | ' ' | '(' | ')')
    });

    if value.is_empty() || (valid_symbols && (8..=15).contains(&digit_count)) {
        Ok(())
    } else {
        Err(format!("{label} must contain 8 to 15 digits"))
    }
}

fn validate_email_or_mobile(label: &str, value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.contains('@') {
        validate_email(label, trimmed)
    } else {
        validate_mobile(label, trimmed)
    }
}

fn validate_comment_status(value: &str) -> Result<(), String> {
    let value = value.trim();
    if matches!(
        value,
        "Prepared" | "Sent" | "Failed" | "Acknowledged" | "Follow-up Due" | "Logged"
    ) {
        Ok(())
    } else {
        Err(
            "Communication status must be one of: Prepared, Sent, Failed, Acknowledged, Follow-up Due, Logged"
                .to_string(),
        )
    }
}

fn normalize_follow_up_due(value: Option<&str>) -> Result<String, String> {
    let trimmed = value.unwrap_or("").trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let normalized = trimmed.replace('T', " ");
    if normalized.len() != 16 || !normalized.chars().enumerate().all(|(index, character)| {
        matches!(index, 4 | 7 | 10 | 13 if character == '-' || character == ' ' || character == ':')
            || !matches!(index, 4 | 7 | 10 | 13) && character.is_ascii_digit()
    }) {
        return Err("Follow-up due must use YYYY-MM-DD HH:MM format".to_string());
    }

    Ok(normalized)
}

fn channel_for_audit(channel: &str) -> &str {
    if channel.trim().is_empty() {
        "Local"
    } else {
        channel
    }
}

fn audience_for_audit(audience: &str) -> &str {
    if audience.trim().is_empty() {
        "recipient"
    } else {
        audience
    }
}

fn get_comment(conn: &Connection, id: i64) -> Result<TicketComment, String> {
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

fn current_local_timestamp(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT datetime('now', 'localtime')", [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn validate_school_contact_fields(input: &CreateSchoolInput) -> Result<(), String> {
    for (label, value) in [
        (
            "SIP Academic Owner email",
            input.sip_academic_owner_email.as_str(),
        ),
        ("Center Head email", input.center_head_email.as_str()),
        ("Principal email", input.principal_email.as_str()),
        ("School SPOC email", input.school_spoc_email.as_str()),
        (
            "Central Academic SPOC email",
            input.central_academic_spoc_email.as_str(),
        ),
        (
            "Central Business SPOC email",
            input.central_business_spoc_email.as_str(),
        ),
        ("BH email", input.bh_email.as_str()),
        ("AOM email", input.aom_email.as_str()),
    ] {
        validate_email(label, value)?;
    }

    for (label, value) in [
        (
            "SIP Academic Owner mobile",
            input.sip_academic_owner_mobile.as_str(),
        ),
        ("Center Head mobile", input.center_head_mobile.as_str()),
        ("Principal mobile", input.principal_mobile.as_str()),
        ("School SPOC mobile", input.school_spoc_mobile.as_str()),
        (
            "Central Academic SPOC mobile",
            input.central_academic_spoc_mobile.as_str(),
        ),
        (
            "Central Business SPOC mobile",
            input.central_business_spoc_mobile.as_str(),
        ),
        ("BH mobile", input.bh_mobile.as_str()),
        ("AOM mobile", input.aom_mobile.as_str()),
    ] {
        validate_mobile(label, value)?;
    }

    Ok(())
}

fn assignment_rule_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssignmentRule> {
    let is_active: i64 = row.get(2)?;

    Ok(AssignmentRule {
        queue: row.get(0)?,
        assignee: row.get(1)?,
        is_active: is_active == 1,
        updated_at: row.get(3)?,
    })
}

fn escalation_policy_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EscalationPolicy> {
    let auto_assign_on_breach: i64 = row.get(2)?;

    Ok(EscalationPolicy {
        at_risk_hours: row.get(0)?,
        escalation_assignee: row.get(1)?,
        auto_assign_on_breach: auto_assign_on_breach == 1,
        updated_at: row.get(3)?,
    })
}

fn communication_template_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommunicationTemplate> {
    let is_active: i64 = row.get(4)?;

    Ok(CommunicationTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        audience: row.get(2)?,
        body: row.get(3)?,
        is_active: is_active == 1,
        updated_at: row.get(5)?,
    })
}

fn get_communication_template(conn: &Connection, id: i64) -> Result<CommunicationTemplate, String> {
    conn.query_row(
        "
        SELECT id, name, audience, body, is_active, updated_at
        FROM communication_templates
        WHERE id = ?1
        ",
        params![id],
        communication_template_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Communication template {id} was not found"))
}

fn queue_for_category(category: &str) -> &'static str {
    match category {
        "Assessment" | "Academic Support" => "Academic Support",
        "Learning Platform" => "Learning Platform",
        "Device" => "IT / Device",
        "Operations" => "Operations",
        "Attendance" | "Parent Communication" => "Parent Communication",
        _ => "Academic Support",
    }
}

fn get_assignment_rule(conn: &Connection, queue: &str) -> Result<AssignmentRule, String> {
    conn.query_row(
        "
        SELECT queue, assignee, is_active, updated_at
        FROM assignment_rules
        WHERE queue = ?1
        ",
        params![queue],
        assignment_rule_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Assignment rule was not found for {queue}"))
}

fn active_assignment_for_queue(conn: &Connection, queue: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "
        SELECT assignee
        FROM assignment_rules
        WHERE queue = ?1 AND is_active = 1
        ",
        params![queue],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn get_sla_policy(conn: &Connection, category: &str) -> Result<SlaPolicy, String> {
    conn.query_row(
        "
        SELECT issue_category, hours
        FROM sla_policies
        WHERE issue_category = ?1
        ",
        params![category],
        |row| {
            Ok(SlaPolicy {
                issue_category: row.get(0)?,
                hours: row.get(1)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("SLA policy was not found for {category}"))
}

fn get_sla_policy_hours(conn: &Connection, category: &str) -> Result<i64, String> {
    get_sla_policy(conn, category).map(|policy| policy.hours)
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

fn record_audit(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    action: &str,
    actor: &str,
    summary: &str,
) -> Result<(), String> {
    conn.execute(
        "
        INSERT INTO audit_log (entity_type, entity_id, action, actor, summary)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![entity_type, entity_id, action, actor, summary],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub fn list_users(conn: &Connection) -> Result<Vec<AppUser>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, username, display_name, role, is_active, created_at, last_login_at
            FROM users
            ORDER BY display_name
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let is_active: i64 = row.get(4)?;
            Ok(AppUser {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                role: row.get(3)?,
                is_active: is_active != 0,
                created_at: row.get(5)?,
                last_login_at: row.get(6)?,
                school_ids: Vec::new(),
            })
        })
        .map_err(|error| error.to_string())?;

    let mut users = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    for u in &mut users {
        u.school_ids = list_user_schools(conn, u.id)?;
    }
    Ok(users)
}

// ── user_schools (M:M scope between users and schools) ───────────────────────

pub fn list_user_schools(conn: &Connection, user_id: i64) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare("SELECT school_id FROM user_schools WHERE user_id = ?1 ORDER BY school_id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![user_id], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn set_user_schools(conn: &Connection, user_id: i64, school_ids: &[i64]) -> Result<(), String> {
    conn.execute("DELETE FROM user_schools WHERE user_id = ?1", params![user_id])
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("INSERT OR IGNORE INTO user_schools (user_id, school_id) VALUES (?1, ?2)")
        .map_err(|e| e.to_string())?;
    for &sid in school_ids {
        stmt.execute(params![user_id, sid])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn authenticate_user(
    conn: &Connection,
    username: &str,
    password: &str,
) -> Result<SessionUser, String> {
    let result = conn.query_row(
        "
        SELECT id, username, display_name, role, password_hash, is_active
        FROM users
        WHERE username = ?1
        ",
        params![username.trim()],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        },
    );

    match result {
        Err(_) => Err("Invalid username or password".to_string()),
        Ok((id, uname, display_name, role, hash, is_active)) => {
            if is_active == 0 {
                return Err("Account is disabled. Contact an administrator.".to_string());
            }
            let valid = bcrypt::verify(password, &hash)
                .map_err(|error| error.to_string())?;
            if !valid {
                return Err("Invalid username or password".to_string());
            }
            conn.execute(
                "UPDATE users SET last_login_at = datetime('now', 'localtime') WHERE id = ?1",
                params![id],
            )
            .map_err(|error| error.to_string())?;
            Ok(SessionUser { id, username: uname, display_name, role })
        }
    }
}

pub fn create_user(conn: &Connection, input: &CreateUserInput, actor: &str) -> Result<AppUser, String> {
    validate_nonempty("Username", &input.username)?;
    validate_nonempty("Display name", &input.display_name)?;
    validate_nonempty("Password", &input.password)?;
    validate_user_role(&input.role)?;
    if input.password.len() < 6 {
        return Err("Password must be at least 6 characters".to_string());
    }

    let hash = bcrypt::hash(&input.password, bcrypt::DEFAULT_COST)
        .map_err(|error| error.to_string())?;

    conn.execute(
        "
        INSERT INTO users (username, display_name, role, password_hash)
        VALUES (?1, ?2, ?3, ?4)
        ",
        params![
            input.username.trim().to_lowercase(),
            input.display_name.trim(),
            input.role.trim(),
            hash
        ],
    )
    .map_err(|error| {
        if error.to_string().contains("UNIQUE") {
            "Username already exists".to_string()
        } else {
            error.to_string()
        }
    })?;

    let id = conn.last_insert_rowid();
    set_user_schools(conn, id, &input.school_ids)?;
    let user = get_user(conn, id)?;
    record_audit(conn, "user", id, "created", actor,
        &format!("Created user '{}' with role '{}'", user.display_name, user.role))?;
    Ok(user)
}

pub fn update_user(conn: &Connection, input: &UpdateUserInput, actor: &str) -> Result<AppUser, String> {
    validate_nonempty("Username", &input.username)?;
    validate_nonempty("Display name", &input.display_name)?;
    validate_user_role(&input.role)?;

    conn.execute(
        "
        UPDATE users
        SET username = ?1, display_name = ?2, role = ?3, is_active = ?4
        WHERE id = ?5
        ",
        params![
            input.username.trim().to_lowercase(),
            input.display_name.trim(),
            input.role.trim(),
            if input.is_active { 1 } else { 0 },
            input.id
        ],
    )
    .map_err(|error| error.to_string())?;

    set_user_schools(conn, input.id, &input.school_ids)?;
    let user = get_user(conn, input.id)?;
    let status = if user.is_active { "active" } else { "disabled" };
    record_audit(conn, "user", input.id, "updated", actor,
        &format!("Updated user '{}': role='{}' status='{}'", user.display_name, user.role, status))?;
    Ok(user)
}

pub fn delete_user(conn: &Connection, id: i64, current_user_id: i64, actor: &str) -> Result<(), String> {
    if id == current_user_id {
        return Err("Cannot delete your own account".to_string());
    }
    let display_name: String = conn
        .query_row("SELECT display_name FROM users WHERE id = ?1", params![id], |row| row.get(0))
        .unwrap_or_else(|_| format!("id={id}"));
    let deleted = conn
        .execute("DELETE FROM users WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    if deleted == 0 {
        return Err(format!("User {id} was not found"));
    }
    record_audit(conn, "user", id, "deleted", actor,
        &format!("Deleted user '{display_name}'"))?;
    Ok(())
}

pub fn change_password(
    conn: &Connection,
    user_id: i64,
    input: &ChangePasswordInput,
    actor: &str,
) -> Result<(), String> {
    if input.new_password.len() < 6 {
        return Err("New password must be at least 6 characters".to_string());
    }

    let current_hash: String = conn
        .query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|_| "User not found".to_string())?;

    let valid = bcrypt::verify(&input.current_password, &current_hash)
        .map_err(|error| error.to_string())?;
    if !valid {
        return Err("Current password is incorrect".to_string());
    }

    let new_hash = bcrypt::hash(&input.new_password, bcrypt::DEFAULT_COST)
        .map_err(|error| error.to_string())?;

    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        params![new_hash, user_id],
    )
    .map_err(|error| error.to_string())?;

    record_audit(conn, "user", user_id, "password_changed", actor, "Password changed")?;
    Ok(())
}

fn get_user(conn: &Connection, id: i64) -> Result<AppUser, String> {
    let mut user = conn
        .query_row(
            "
        SELECT id, username, display_name, role, is_active, created_at, last_login_at
        FROM users WHERE id = ?1
        ",
            params![id],
            |row| {
                let is_active: i64 = row.get(4)?;
                Ok(AppUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    role: row.get(3)?,
                    is_active: is_active != 0,
                    created_at: row.get(5)?,
                    last_login_at: row.get(6)?,
                    school_ids: Vec::new(),
                })
            },
        )
        .map_err(|_| format!("User {id} was not found"))?;
    user.school_ids = list_user_schools(conn, id)?;
    Ok(user)
}

fn validate_user_role(role: &str) -> Result<(), String> {
    match role.trim() {
        "admin" | "agent" | "viewer" | "aom" | "faculty" => Ok(()),
        _ => Err(format!(
            "Invalid role '{role}'. Must be admin, agent, viewer, aom, or faculty"
        )),
    }
}

// ── Subjects ─────────────────────────────────────────────────────────────────

fn subject_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Subject> {
    Ok(Subject {
        id: row.get(0)?,
        name: row.get(1)?,
        track: row.get(2)?,
        is_default: row.get::<_, i64>(3)? == 1,
        sort_order: row.get(4)?,
    })
}

pub fn list_subjects(conn: &Connection) -> Result<Vec<Subject>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, track, is_default, sort_order FROM subjects ORDER BY track, sort_order, name")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], subject_from_row).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_subject(conn: &Connection, input: &CreateSubjectInput) -> Result<Subject, String> {
    validate_nonempty("Subject name", &input.name)?;
    validate_track(&input.track)?;
    conn.execute(
        "INSERT INTO subjects (name, track, is_default, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![
            input.name.trim(),
            input.track.trim(),
            if input.is_default { 1 } else { 0 },
            input.sort_order
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    get_subject(conn, id)
}

pub fn update_subject(conn: &Connection, input: &UpdateSubjectInput) -> Result<Subject, String> {
    validate_nonempty("Subject name", &input.name)?;
    validate_track(&input.track)?;
    conn.execute(
        "UPDATE subjects SET name = ?1, track = ?2, is_default = ?3, sort_order = ?4 WHERE id = ?5",
        params![
            input.name.trim(),
            input.track.trim(),
            if input.is_default { 1 } else { 0 },
            input.sort_order,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;
    get_subject(conn, input.id)
}

pub fn delete_subject(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM subjects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_subject(conn: &Connection, id: i64) -> Result<Subject, String> {
    conn.query_row(
        "SELECT id, name, track, is_default, sort_order FROM subjects WHERE id = ?1",
        params![id],
        subject_from_row,
    )
    .map_err(|e| e.to_string())
}

fn validate_track(track: &str) -> Result<(), String> {
    match track.trim() {
        "JEE" | "NEET" | "Foundation" => Ok(()),
        _ => Err(format!(
            "Invalid track '{track}'. Must be JEE, NEET, or Foundation"
        )),
    }
}

// Effective subjects available at a school for a track:
// - Track defaults always included
// - Foundation optional subjects only if school has opted in via school_optional_subjects
pub fn list_effective_subjects(
    conn: &Connection,
    school_id: i64,
    track: &str,
) -> Result<Vec<EffectiveSubject>, String> {
    validate_track(track)?;
    let mut stmt = conn
        .prepare(
            "
            SELECT s.id, s.name, s.track, s.is_default,
                   CASE
                       WHEN s.is_default = 1 THEN 1
                       WHEN sos.subject_id IS NOT NULL THEN 1
                       ELSE 0
                   END AS is_offered
            FROM subjects s
            LEFT JOIN school_optional_subjects sos
              ON sos.subject_id = s.id AND sos.school_id = ?1
            WHERE s.track = ?2
            ORDER BY s.sort_order, s.name
            ",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![school_id, track], |row| {
            Ok(EffectiveSubject {
                id: row.get(0)?,
                name: row.get(1)?,
                track: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
                is_offered: row.get::<_, i64>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ── Faculty assignments ──────────────────────────────────────────────────────

pub fn list_faculty_assignments(
    conn: &Connection,
    school_id: Option<i64>,
    faculty_user_id: Option<i64>,
) -> Result<Vec<FacultyAssignment>, String> {
    let base = "
        SELECT fa.id, fa.faculty_user_id, u.display_name,
               fa.school_id, s.name,
               fa.grade_level, fa.track,
               fa.subject_id, sub.name,
               fa.created_at
        FROM faculty_assignments fa
        JOIN users u    ON u.id   = fa.faculty_user_id
        JOIN schools s  ON s.id   = fa.school_id
        JOIN subjects sub ON sub.id = fa.subject_id
        WHERE 1=1";
    let mut sql = base.to_string();
    if school_id.is_some() {
        sql.push_str(" AND fa.school_id = ?1");
    }
    if faculty_user_id.is_some() {
        if school_id.is_some() {
            sql.push_str(" AND fa.faculty_user_id = ?2");
        } else {
            sql.push_str(" AND fa.faculty_user_id = ?1");
        }
    }
    sql.push_str(" ORDER BY u.display_name, s.name, fa.grade_level, fa.track, sub.name");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<FacultyAssignment> {
        Ok(FacultyAssignment {
            id: row.get(0)?,
            faculty_user_id: row.get(1)?,
            faculty_display_name: row.get(2)?,
            school_id: row.get(3)?,
            school_name: row.get(4)?,
            grade_level: row.get(5)?,
            track: row.get(6)?,
            subject_id: row.get(7)?,
            subject_name: row.get(8)?,
            created_at: row.get(9)?,
        })
    };

    let rows = match (school_id, faculty_user_id) {
        (Some(sid), Some(fid)) => stmt.query_map(params![sid, fid], map_row),
        (Some(sid), None) => stmt.query_map(params![sid], map_row),
        (None, Some(fid)) => stmt.query_map(params![fid], map_row),
        (None, None) => stmt.query_map([], map_row),
    }
    .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_faculty_assignment(
    conn: &Connection,
    input: &CreateFacultyAssignmentInput,
) -> Result<FacultyAssignment, String> {
    validate_nonempty("Grade level", &input.grade_level)?;
    if !input.track.is_empty() {
        validate_track(&input.track)?;
    }
    // Subject's track must match the assignment's track. For Foundation
    // grades the assignment's track is empty but the subject's track is
    // "Foundation" — accept that as a match.
    let subject = get_subject(conn, input.subject_id)?;
    let assignment_track = if input.track.is_empty() {
        "Foundation"
    } else {
        input.track.as_str()
    };
    if subject.track != assignment_track {
        return Err(format!(
            "Subject '{}' belongs to track '{}', does not match assignment track '{}'",
            subject.name, subject.track, assignment_track
        ));
    }
    // Faculty user must exist and have role faculty (allow any role for
    // flexibility — admin can be assigned for testing).
    let _ = get_user(conn, input.faculty_user_id)?;
    let _ = get_school(conn, input.school_id)?;

    conn.execute(
        "
        INSERT INTO faculty_assignments
            (faculty_user_id, school_id, grade_level, track, subject_id)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            input.faculty_user_id,
            input.school_id,
            input.grade_level.trim(),
            input.track.trim(),
            input.subject_id,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "This faculty already has this exact assignment".to_string()
        } else {
            e.to_string()
        }
    })?;

    let id = conn.last_insert_rowid();
    list_faculty_assignments(conn, None, None)?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| "Failed to read back the new assignment".to_string())
}

// ── Timetable slots ──────────────────────────────────────────────────────────

fn timetable_slot_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimetableSlot> {
    Ok(TimetableSlot {
        id: row.get(0)?,
        school_id: row.get(1)?,
        school_name: row.get(2)?,
        grade_level: row.get(3)?,
        track: row.get(4)?,
        batch_pattern: row.get(5)?,
        day_of_week: row.get(6)?,
        period: row.get(7)?,
        subject_id: row.get(8)?,
        subject_name: row.get(9)?,
        faculty_user_id: row.get(10)?,
        faculty_display_name: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
        start_time: row.get(12)?,
        end_time: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

pub fn list_timetable_slots(
    conn: &Connection,
    school_id: Option<i64>,
    grade_level: Option<&str>,
    track: Option<&str>,
    batch_pattern: Option<&str>,
) -> Result<Vec<TimetableSlot>, String> {
    let mut sql = String::from(
        "
        SELECT ts.id, ts.school_id, s.name, ts.grade_level, ts.track, ts.batch_pattern,
               ts.day_of_week, ts.period,
               ts.subject_id, sub.name,
               ts.faculty_user_id, u.display_name,
               ts.start_time, ts.end_time, ts.updated_at
        FROM timetable_slots ts
        JOIN schools  s   ON s.id   = ts.school_id
        JOIN subjects sub ON sub.id = ts.subject_id
        LEFT JOIN users u ON u.id   = ts.faculty_user_id
        WHERE 1=1",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(v) = school_id {
        sql.push_str(" AND ts.school_id = ?");
        p.push(v.into());
    }
    if let Some(v) = grade_level {
        sql.push_str(" AND ts.grade_level = ?");
        p.push(v.to_string().into());
    }
    if let Some(v) = track {
        sql.push_str(" AND ts.track = ?");
        p.push(v.to_string().into());
    }
    if let Some(v) = batch_pattern {
        sql.push_str(" AND ts.batch_pattern = ?");
        p.push(v.to_string().into());
    }
    sql.push_str(" ORDER BY ts.day_of_week, ts.period");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), timetable_slot_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn upsert_timetable_slot(
    conn: &Connection,
    input: &UpsertTimetableSlotInput,
) -> Result<TimetableSlot, String> {
    validate_nonempty("Grade level", &input.grade_level)?;
    validate_nonempty("Batch pattern", &input.batch_pattern)?;
    if !input.track.is_empty() {
        validate_track(&input.track)?;
    }
    if input.day_of_week < 0 || input.day_of_week > 6 {
        return Err("day_of_week must be in 0..6".to_string());
    }
    if input.period < 1 {
        return Err("period must be >= 1".to_string());
    }
    let _ = get_school(conn, input.school_id)?;
    let subject = get_subject(conn, input.subject_id)?;
    let assignment_track = if input.track.is_empty() { "Foundation" } else { input.track.as_str() };
    if subject.track != assignment_track {
        return Err(format!(
            "Subject '{}' belongs to track '{}', does not match slot track '{}'",
            subject.name, subject.track, assignment_track
        ));
    }
    if let Some(fid) = input.faculty_user_id {
        let _ = get_user(conn, fid)?;
    }

    conn.execute(
        "
        INSERT INTO timetable_slots
            (school_id, grade_level, track, batch_pattern,
             day_of_week, period, subject_id, faculty_user_id,
             start_time, end_time)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(school_id, grade_level, track, batch_pattern, day_of_week, period)
        DO UPDATE SET
            subject_id      = excluded.subject_id,
            faculty_user_id = excluded.faculty_user_id,
            start_time      = excluded.start_time,
            end_time        = excluded.end_time,
            updated_at      = datetime('now', 'localtime')
        ",
        params![
            input.school_id,
            input.grade_level.trim(),
            input.track.trim(),
            input.batch_pattern.trim(),
            input.day_of_week,
            input.period,
            input.subject_id,
            input.faculty_user_id,
            input.start_time.trim(),
            input.end_time.trim(),
        ],
    )
    .map_err(|e| e.to_string())?;

    let slots = list_timetable_slots(
        conn,
        Some(input.school_id),
        Some(input.grade_level.trim()),
        Some(input.track.trim()),
        Some(input.batch_pattern.trim()),
    )?;
    slots
        .into_iter()
        .find(|s| s.day_of_week == input.day_of_week && s.period == input.period)
        .ok_or_else(|| "Failed to read back the timetable slot".to_string())
}

pub fn delete_timetable_slot(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM timetable_slots WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!("Timetable slot {id} was not found"));
    }
    Ok(())
}

pub fn delete_faculty_assignment(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = conn
        .execute("DELETE FROM faculty_assignments WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        return Err(format!("Assignment {id} was not found"));
    }
    Ok(())
}

pub fn set_school_optional_subject(
    conn: &Connection,
    school_id: i64,
    subject_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let subj = get_subject(conn, subject_id)?;
    if subj.is_default {
        return Err(format!(
            "{} is a default subject for {}; cannot toggle per school",
            subj.name, subj.track
        ));
    }
    if enabled {
        conn.execute(
            "INSERT OR IGNORE INTO school_optional_subjects (school_id, subject_id) VALUES (?1, ?2)",
            params![school_id, subject_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "DELETE FROM school_optional_subjects WHERE school_id = ?1 AND subject_id = ?2",
            params![school_id, subject_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
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
    fn initialize_db_seeds_tickets_and_runs_migrations() {
        let conn = test_db();

        let tickets = list_tickets(&conn).expect("list tickets");
        let migrations = list_migrations(&conn).expect("list migrations");

        assert_eq!(tickets.len(), 3);
        assert_eq!(
            migrations,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
        );
        assert_eq!(tickets[0].school_name, "Sunrise International School");
        assert_eq!(tickets[0].queue, "IT / Device");
        assert!(!tickets[0].sla_due_at.is_empty());

        let policies = list_sla_policies(&conn).expect("list SLA policies");
        assert_eq!(policies.len(), 7);
        assert_eq!(get_sla_policy_hours(&conn, "Assessment"), Ok(24));

        let schools = list_schools(&conn).expect("list schools");
        let students = list_students(&conn, None).expect("list students");
        assert_eq!(schools.len(), 3);
        assert_eq!(students.len(), 3);

        let assignment_rules = list_assignment_rules(&conn).expect("list assignment rules");
        assert_eq!(assignment_rules.len(), 5);
        assert!(list_regions(&conn).expect("list regions").is_empty());

        let escalation_policy = get_escalation_policy(&conn).expect("get escalation policy");
        assert_eq!(escalation_policy.at_risk_hours, 24);
        assert_eq!(escalation_policy.escalation_assignee, "Program Supervisor");
        assert!(escalation_policy.auto_assign_on_breach);

        let templates = list_communication_templates(&conn).expect("list templates");
        assert_eq!(templates.len(), 5);

        let lecture_models = list_lecture_models(&conn).expect("list lecture models");
        assert_eq!(lecture_models.len(), 5);
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
    fn school_and_student_master_data_can_be_created_and_listed() {
        let conn = test_db();
        let region = upsert_region(
            &conn,
            &UpsertRegionInput {
                id: None,
                name: "North Region".to_string(),
                regional_academic_head_name: "Regional Academic Head".to_string(),
                regional_academic_head_mobile: "6666666666".to_string(),
                regional_academic_head_email: "rah@example.com".to_string(),
                regional_business_head_name: "Regional Business Head".to_string(),
                regional_business_head_mobile: "5555555555".to_string(),
                regional_business_head_email: "rbh@example.com".to_string(),
            },
            TEST_ACTOR,
        )
        .expect("create region");

        let school = create_school(
            &conn,
            &CreateSchoolInput {
                name: "West End Public School".to_string(),
                region_id: Some(region.id),
                program_model: "Aspire".to_string(),
                distance_classification: "Remote".to_string(),
                sip_academic_owner_role: "SIP Academic Head".to_string(),
                sip_academic_owner_name: "Asha Mehta".to_string(),
                sip_academic_owner_mobile: "9999999999".to_string(),
                sip_academic_owner_email: "asha@example.com".to_string(),
                center_head_name: "Rajiv Menon".to_string(),
                center_head_mobile: "".to_string(),
                center_head_email: "".to_string(),
                principal_name: "".to_string(),
                principal_mobile: "".to_string(),
                principal_email: "".to_string(),
                school_spoc_name: "".to_string(),
                school_spoc_mobile: "".to_string(),
                school_spoc_email: "".to_string(),
                central_academic_spoc_name: "".to_string(),
                central_academic_spoc_mobile: "".to_string(),
                central_academic_spoc_email: "".to_string(),
                central_business_spoc_name: "".to_string(),
                central_business_spoc_mobile: "".to_string(),
                central_business_spoc_email: "".to_string(),
                bh_name: "Business Head".to_string(),
                bh_mobile: "7777777777".to_string(),
                bh_email: "bh@example.com".to_string(),
                aom_name: "Academic Operations Manager".to_string(),
                aom_mobile: "9999900000".to_string(),
                aom_email: "aom@example.com".to_string(),
                mapped_vp_center: "".to_string(),
            },
            TEST_ACTOR,
        )
        .expect("create school");
        let student = create_student(
            &conn,
            &CreateStudentInput {
                school_id: school.id,
                name: "Priya Nair".to_string(),
                grade_level: "Grade 12".to_string(),
                program_track: "Board Excellence".to_string(),
                track: "JEE".to_string(),
            },
        )
        .expect("create student");

        let students = list_students(&conn, Some(school.id)).expect("list students");

        assert_eq!(school.name, "West End Public School");
        assert_eq!(school.region_name, "North Region");
        assert_eq!(school.program_model, "Aspire");
        assert_eq!(school.distance_classification, "Remote");
        assert_eq!(school.sip_academic_owner_name, "Asha Mehta");
        assert_eq!(school.bh_name, "Business Head");
        assert_eq!(school.aom_name, "Academic Operations Manager");
        assert_eq!(student.school_name, school.name);
        assert_eq!(student.grade_level, "Grade 12");
        assert_eq!(students.len(), 1);

        let south_region = upsert_region(
            &conn,
            &UpsertRegionInput {
                id: None,
                name: "South Region".to_string(),
                regional_academic_head_name: "".to_string(),
                regional_academic_head_mobile: "".to_string(),
                regional_academic_head_email: "".to_string(),
                regional_business_head_name: "".to_string(),
                regional_business_head_mobile: "".to_string(),
                regional_business_head_email: "".to_string(),
            },
            TEST_ACTOR,
        )
        .expect("create south region");
        let remapped_school = create_school(
            &conn,
            &CreateSchoolInput {
                name: school.name.clone(),
                region_id: Some(south_region.id),
                program_model: school.program_model.clone(),
                distance_classification: school.distance_classification.clone(),
                sip_academic_owner_role: school.sip_academic_owner_role.clone(),
                sip_academic_owner_name: school.sip_academic_owner_name.clone(),
                sip_academic_owner_mobile: school.sip_academic_owner_mobile.clone(),
                sip_academic_owner_email: school.sip_academic_owner_email.clone(),
                center_head_name: school.center_head_name.clone(),
                center_head_mobile: school.center_head_mobile.clone(),
                center_head_email: school.center_head_email.clone(),
                principal_name: school.principal_name.clone(),
                principal_mobile: school.principal_mobile.clone(),
                principal_email: school.principal_email.clone(),
                school_spoc_name: school.school_spoc_name.clone(),
                school_spoc_mobile: school.school_spoc_mobile.clone(),
                school_spoc_email: school.school_spoc_email.clone(),
                central_academic_spoc_name: school.central_academic_spoc_name.clone(),
                central_academic_spoc_mobile: school.central_academic_spoc_mobile.clone(),
                central_academic_spoc_email: school.central_academic_spoc_email.clone(),
                central_business_spoc_name: school.central_business_spoc_name.clone(),
                central_business_spoc_mobile: school.central_business_spoc_mobile.clone(),
                central_business_spoc_email: school.central_business_spoc_email.clone(),
                bh_name: school.bh_name.clone(),
                bh_mobile: school.bh_mobile.clone(),
                bh_email: school.bh_email.clone(),
                aom_name: school.aom_name.clone(),
                aom_mobile: school.aom_mobile.clone(),
                aom_email: school.aom_email.clone(),
                mapped_vp_center: school.mapped_vp_center.clone(),
            },
            TEST_ACTOR,
        )
        .expect("remap school");
        let region_history = list_school_region_history(&conn).expect("list region history");

        assert_eq!(remapped_school.region_name, "South Region");
        assert_eq!(region_history.len(), 1);
        assert_eq!(region_history[0].school_name, school.name);
        assert_eq!(region_history[0].old_region_name, "North Region");
        assert_eq!(region_history[0].new_region_name, "South Region");
        assert!(!region_history[0].changed_at.is_empty());

        let dropped_school =
            drop_school(&conn, school.id, "Program discontinued", TEST_ACTOR).expect("drop school");
        assert!(dropped_school.is_dropped);
        assert_eq!(dropped_school.dropped_reason, "Program discontinued");
        assert!(list_schools(&conn)
            .expect("list active schools")
            .iter()
            .all(|item| item.id != school.id));
        assert_eq!(list_dropped_schools(&conn).expect("list dropped").len(), 1);
        assert!(list_students(&conn, Some(school.id))
            .expect("list dropped school students")
            .is_empty());

        let restored_school = restore_school(&conn, school.id, TEST_ACTOR).expect("restore school");
        assert!(!restored_school.is_dropped);
        assert!(restored_school.dropped_at.is_empty());
        assert!(restored_school.dropped_reason.is_empty());
        assert!(list_schools(&conn)
            .expect("list active schools after restore")
            .iter()
            .any(|item| item.id == school.id));

        let linked_delete_error =
            delete_region(&conn, south_region.id).expect_err("linked region should not delete");
        assert_eq!(
            linked_delete_error,
            "Region is mapped to schools. Move those schools to another region before deleting."
        );

        delete_school(&conn, school.id, TEST_ACTOR).expect("delete dummy school");
        assert!(list_schools(&conn)
            .expect("list schools after delete")
            .iter()
            .all(|item| item.id != school.id));
        assert!(list_students(&conn, Some(school.id))
            .expect("list deleted school students")
            .is_empty());
        assert!(list_school_region_history(&conn)
            .expect("list history after school delete")
            .iter()
            .all(|item| item.school_id != school.id));

        let unused_region = upsert_region(
            &conn,
            &UpsertRegionInput {
                id: None,
                name: "Unused Region".to_string(),
                regional_academic_head_name: "".to_string(),
                regional_academic_head_mobile: "".to_string(),
                regional_academic_head_email: "".to_string(),
                regional_business_head_name: "".to_string(),
                regional_business_head_mobile: "".to_string(),
                regional_business_head_email: "".to_string(),
            },
            TEST_ACTOR,
        )
        .expect("create unused region");
        delete_region(&conn, unused_region.id).expect("delete unused region");
        assert!(list_regions(&conn)
            .expect("list regions")
            .iter()
            .all(|item| item.id != unused_region.id));
    }

    #[test]
    fn school_class_plans_track_delivery_and_admissions() {
        let conn = test_db();
        let model = create_lecture_model(
            &conn,
            &CreateLectureModelInput {
                name: "6x2".to_string(),
                days_per_week: 6,
                lectures_per_day: 2,
            },
        )
        .expect("create lecture model");

        let plan = upsert_school_class_plan(
            &conn,
            &UpsertSchoolClassPlanInput {
                school_id: 1,
                grade_level: "Grade 10".to_string(),
                track: "".to_string(),
                lecture_model_id: model.id,
                batch_pattern: "Both".to_string(),
                aop_admissions: 100,
                registrations: 0,
                actual_admissions: 75,
            },
        )
        .expect("save class plan");
        let dashboard = get_school_program_dashboard(&conn).expect("dashboard");

        assert_eq!(plan.lecture_model_name, "6x2");
        assert_eq!(plan.admission_gap, 25);
        assert_eq!(plan.admission_attainment_percent, 75);
        assert_eq!(dashboard.total_classes, 1);
        assert_eq!(dashboard.total_aop_admissions, 100);
        assert_eq!(dashboard.total_actual_admissions, 75);
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

        let timeline = get_student_timeline(&conn, "Green Valley Public School", "Aarav Shah")
            .expect("get student timeline");

        assert_eq!(timeline.student_name, "Aarav Shah");
        assert_eq!(timeline.school_name, "Green Valley Public School");
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
}
