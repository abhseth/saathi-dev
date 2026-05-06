use crate::models::{
    AssignmentRule, CommunicationTemplate, CreateSchoolInput, EscalationPolicy, LectureModel,
    Region, School, SchoolClassPlan, SlaPolicy, Ticket, TicketAttachment, TicketComment,
    TicketHistory,
};
use rusqlite::{params, Connection, OptionalExtension};

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
        linked_grade_level: row.get(19).unwrap_or_default(),
        linked_subject: row.get(20).unwrap_or_default(),
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

#[allow(dead_code)]
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

pub fn get_school(conn: &Connection, id: i64) -> Result<School, String> {
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
               mapped_vp_center, vp_tagging,
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

pub fn get_school_by_name(conn: &Connection, name: &str) -> Result<School, String> {
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
               mapped_vp_center, vp_tagging,
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

pub fn school_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<School> {
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
        vp_tagging: row.get(32)?,
        is_dropped: row.get::<_, i64>(33)? == 1,
        dropped_at: row.get(34)?,
        dropped_reason: row.get(35)?,
        created_at: row.get(36)?,
    })
}

pub fn get_school_region_by_name(
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

pub fn record_school_region_history(
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

pub fn region_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Region> {
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

pub fn get_region(conn: &Connection, id: i64) -> Result<Region, String> {
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

pub fn school_class_plan_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SchoolClassPlan> {
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

pub fn get_lecture_model(conn: &Connection, id: i64) -> Result<LectureModel, String> {
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

pub fn get_lecture_model_by_name(conn: &Connection, name: &str) -> Result<LectureModel, String> {
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

pub fn get_school_class_plan(
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

pub fn count_active_schools(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM schools WHERE is_dropped = 0",
        [],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

pub fn count_school_field(conn: &Connection, field_name: &str, value: &str) -> Result<i64, String> {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM schools WHERE is_dropped = 0 AND {field_name} = ?1"),
        params![value],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

pub fn percent(numerator: i64, denominator: i64) -> i64 {
    if denominator <= 0 {
        0
    } else {
        ((numerator as f64 / denominator as f64) * 100.0).round() as i64
    }
}

pub fn validate_batch_pattern(value: &str) -> Result<(), String> {
    let value = value.trim();
    if ALLOWED_BATCH_PATTERNS.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Delivery pattern must be one of: {}",
            ALLOWED_BATCH_PATTERNS.join(", ")
        ))
    }
}

pub fn validate_school_model(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "Aspire" | "Minimum Guarantee") {
        Ok(())
    } else {
        Err("School model must be Aspire or Minimum Guarantee".to_string())
    }
}

pub fn validate_distance_classification(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "Remote" | "Near Proximity") {
        Ok(())
    } else {
        Err("Distance classification must be Remote or Near Proximity".to_string())
    }
}

pub fn validate_email(label: &str, value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || (value.contains('@') && value.contains('.') && !value.contains(' ')) {
        Ok(())
    } else {
        Err(format!("{label} must be a valid email address"))
    }
}

pub fn validate_mobile(label: &str, value: &str) -> Result<(), String> {
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

pub fn validate_email_or_mobile(label: &str, value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.contains('@') {
        validate_email(label, trimmed)
    } else {
        validate_mobile(label, trimmed)
    }
}

pub fn lecture_model_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LectureModel> {
    Ok(LectureModel {
        id: row.get(0)?,
        name: row.get(1)?,
        days_per_week: row.get(2)?,
        lectures_per_day: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn validate_school_contact_fields(input: &CreateSchoolInput) -> Result<(), String> {
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

pub fn escalation_policy_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EscalationPolicy> {
    let auto_assign_on_breach: i64 = row.get(2)?;

    Ok(EscalationPolicy {
        at_risk_hours: row.get(0)?,
        escalation_assignee: row.get(1)?,
        auto_assign_on_breach: auto_assign_on_breach == 1,
        updated_at: row.get(3)?,
    })
}

pub fn communication_template_from_row(
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

pub fn get_communication_template(
    conn: &Connection,
    id: i64,
) -> Result<CommunicationTemplate, String> {
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

pub fn queue_for_category(category: &str) -> &'static str {
    match category {
        "Assessment" | "Academic Support" => "Academic Support",
        "Learning Platform" => "Learning Platform",
        "Device" => "IT / Device",
        "Operations" => "Operations",
        "Attendance" | "Parent Communication" => "Parent Communication",
        _ => "Academic Support",
    }
}

pub fn get_assignment_rule(conn: &Connection, queue: &str) -> Result<AssignmentRule, String> {
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

pub fn active_assignment_for_queue(
    conn: &Connection,
    queue: &str,
) -> Result<Option<String>, String> {
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

pub fn get_sla_policy(conn: &Connection, category: &str) -> Result<SlaPolicy, String> {
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

pub fn get_sla_policy_hours(conn: &Connection, category: &str) -> Result<i64, String> {
    get_sla_policy(conn, category).map(|policy| policy.hours)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repo::common::get_sla_policy_hours;
    use crate::repo::schools::{list_lecture_models, list_regions, list_schools, list_students};
    use crate::repo::tickets::{
        get_escalation_policy, list_assignment_rules, list_communication_templates,
        list_migrations, list_sla_policies, list_tickets,
    };

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        db::initialize_db(&conn).expect("initialize schema");
        conn
    }

    #[test]
    fn initialize_db_seeds_tickets_and_runs_migrations() {
        let conn = test_db();

        let tickets = list_tickets(&conn, None, 1000, 0)
            .expect("list tickets")
            .items;
        let migrations = list_migrations(&conn).expect("list migrations");

        assert_eq!(tickets.len(), 3);
        assert_eq!(
            migrations,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
                46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
                65, 66, 67, 68, 69
            ]
        );
        assert_eq!(tickets[0].school_name, "Sunrise International School");
        assert_eq!(tickets[0].queue, "IT / Device");
        assert!(!tickets[0].sla_due_at.is_empty());

        let policies = list_sla_policies(&conn).expect("list SLA policies");
        assert_eq!(policies.len(), 7);
        assert_eq!(get_sla_policy_hours(&conn, "Assessment"), Ok(24));

        let schools = list_schools(&conn, None).expect("list schools");
        let students = list_students(&conn, None, None).expect("list students");
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
}

// ── Missing helpers reconstructed ────────────────────────────────────────────

pub fn validate_nonempty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label} is required"))
    } else {
        Ok(())
    }
}

pub fn assignment_rule_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssignmentRule> {
    Ok(AssignmentRule {
        queue: row.get(0)?,
        assignee: row.get(1)?,
        is_active: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

pub fn get_region_by_name(conn: &Connection, name: &str) -> Result<Region, String> {
    conn.query_row(
        "SELECT id, name, regional_academic_head_name, regional_academic_head_mobile, regional_academic_head_email, regional_business_head_name, regional_business_head_mobile, regional_business_head_email, updated_at FROM regions WHERE name = ?1",
        params![name],
        region_from_row,
    )
    .map_err(|e| e.to_string())
}
