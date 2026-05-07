use crate::models::{
    AuditLogEntry, Batch, BatchAnalytics, BatchDetail, CreateBatchInput, CreateLectureModelInput,
    CreateSchoolInput, CreateStudentInput, LectureModel, Paginated, Region, School, SchoolClassPlan,
    SchoolDeleteImpact, SchoolDeleteImpactItem, SchoolProgramDashboard, SchoolRegionHistory,
    Student, UpdateBatchInput, UpdateSchoolInput, UpdateStudentInput, UpsertRegionInput,
    UpsertSchoolClassPlanInput,
};
use rusqlite::{params, Connection};
use std::collections::HashSet;

use super::audit::*;
use super::common::*;

fn canonical_grade_level(value: &str) -> String {
    match value.trim() {
        "6" | "Grade 6" => "Grade 6".to_string(),
        "7" | "Grade 7" => "Grade 7".to_string(),
        "8" | "Grade 8" => "Grade 8".to_string(),
        "9" | "Grade 9" => "Grade 9".to_string(),
        "10" | "Grade 10" => "Grade 10".to_string(),
        "11" | "Grade 11" => "Grade 11".to_string(),
        "12" | "Grade 12" => "Grade 12".to_string(),
        "Dropper" => "Dropper".to_string(),
        other => other.to_string(),
    }
}

pub fn list_schools(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<School>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
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
         WHERE is_dropped = 0",
    );
    let mut p: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND schools.id IN ({placeholders})"));
            for id in ids {
                p.push(id);
            }
        }
    }
    sql.push_str(" ORDER BY schools.name LIMIT ?");
    p.push(&MAX_ROWS);

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), school_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn list_dropped_schools(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<School>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT schools.id, schools.name, schools.region_id, COALESCE(regions.name, ''),
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
         WHERE is_dropped = 1",
    );
    let mut p: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND schools.id IN ({placeholders})"));
            for id in ids {
                p.push(id);
            }
        }
    }
    sql.push_str(" ORDER BY datetime(dropped_at) DESC, schools.name LIMIT ?");
    p.push(&MAX_ROWS);

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), school_from_row)
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_school(
    conn: &Connection,
    input: &CreateSchoolInput,
    actor: &str,
) -> Result<School, String> {
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
            mapped_vp_center, vp_tagging
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
            ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31
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
            mapped_vp_center = excluded.mapped_vp_center,
            vp_tagging = excluded.vp_tagging
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
            input.vp_tagging.trim(),
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

pub fn update_school(
    conn: &Connection,
    input: &UpdateSchoolInput,
    actor: &str,
) -> Result<School, String> {
    validate_nonempty("School", &input.name)?;
    validate_school_model(&input.program_model)?;
    validate_distance_classification(&input.distance_classification)?;
    if let Some(region_id) = input.region_id {
        get_region(conn, region_id)?;
    }

    let existing = get_school(conn, input.id)?;
    let previous_region = existing.region_id;

    conn.execute(
        "
        UPDATE schools
        SET name = ?1,
            region_id = ?2,
            program_model = ?3,
            distance_classification = ?4,
            sip_academic_owner_role = ?5,
            sip_academic_owner_name = ?6,
            sip_academic_owner_mobile = ?7,
            sip_academic_owner_email = ?8,
            center_head_name = ?9,
            center_head_mobile = ?10,
            center_head_email = ?11,
            principal_name = ?12,
            principal_mobile = ?13,
            principal_email = ?14,
            school_spoc_name = ?15,
            school_spoc_mobile = ?16,
            school_spoc_email = ?17,
            central_academic_spoc_name = ?18,
            central_academic_spoc_mobile = ?19,
            central_academic_spoc_email = ?20,
            central_business_spoc_name = ?21,
            central_business_spoc_mobile = ?22,
            central_business_spoc_email = ?23,
            bh_name = ?24,
            bh_mobile = ?25,
            bh_email = ?26,
            aom_name = ?27,
            aom_mobile = ?28,
            aom_email = ?29,
            mapped_vp_center = ?30,
            vp_tagging = ?31
        WHERE id = ?32
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
            input.vp_tagging.trim(),
            input.id,
        ],
    )
    .map_err(|error| error.to_string())?;

    let school = get_school(conn, input.id)?;
    if previous_region != input.region_id {
        record_school_region_history(
            conn,
            school.id,
            previous_region,
            &existing.region_name,
            input.region_id,
            &school.region_name,
        )?;
    }
    record_audit(
        conn,
        "school",
        school.id,
        "updated",
        actor,
        &format!("updated school {}", school.name),
    )?;

    Ok(school)
}

pub fn list_students(
    conn: &Connection,
    school_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Student>, String> {
    Ok(list_students_paginated(conn, school_id, scope_school_ids, None, 1000, 0)?.items)
}

pub fn list_students_paginated(
    conn: &Connection,
    school_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Student>, String> {
    let safe_limit = limit.clamp(1, 250);
    let safe_offset = offset.max(0);
    let search_term = search.unwrap_or("").trim().to_lowercase();

    let mut count_sql = String::from(
        "SELECT COUNT(*)
         FROM students
         JOIN schools ON schools.id = students.school_id
         LEFT JOIN batches ON batches.id = students.batch_ref_id
         WHERE schools.is_dropped = 0",
    );
    let mut sql = String::from(
        "SELECT students.id, students.school_id, schools.name, students.name,
                students.registration_number, students.grade_level, students.program_track,
                students.track, students.student_mobile, students.student_email,
                students.father_name, students.father_email, students.father_mobile,
                students.mother_name, students.mother_email, students.mother_mobile,
                students.batch_ref_id, COALESCE(batches.batch_id, students.batch_id),
                students.batch_id, students.created_at
         FROM students
         JOIN schools ON schools.id = students.school_id
         LEFT JOIN batches ON batches.id = students.batch_ref_id
         WHERE schools.is_dropped = 0",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(id) = school_id {
        count_sql.push_str(" AND students.school_id = ?");
        sql.push_str(" AND students.school_id = ?");
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            count_sql.push_str(&format!(" AND students.school_id IN ({placeholders})"));
            sql.push_str(&format!(" AND students.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    if !search_term.is_empty() {
        let like = format!("%{search_term}%");
        count_sql.push_str(
            " AND (
                lower(students.name) LIKE ?
                OR lower(students.registration_number) LIKE ?
                OR lower(COALESCE(students.student_mobile, '')) LIKE ?
                OR lower(COALESCE(students.student_email, '')) LIKE ?
                OR lower(COALESCE(batches.batch_id, students.batch_id)) LIKE ?
            )",
        );
        sql.push_str(
            " AND (
                lower(students.name) LIKE ?
                OR lower(students.registration_number) LIKE ?
                OR lower(COALESCE(students.student_mobile, '')) LIKE ?
                OR lower(COALESCE(students.student_email, '')) LIKE ?
                OR lower(COALESCE(batches.batch_id, students.batch_id)) LIKE ?
            )",
        );
        for _ in 0..5 {
            p.push(like.clone().into());
        }
    }

    let total_count: i64 = conn
        .query_row(&count_sql, rusqlite::params_from_iter(p.iter()), |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;

    sql.push_str(" ORDER BY schools.name, students.name LIMIT ? OFFSET ?");
    p.push(safe_limit.into());
    p.push(safe_offset.into());

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), student_from_row)
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

pub fn create_student(conn: &Connection, input: &CreateStudentInput) -> Result<Student, String> {
    validate_nonempty("Student", &input.name)?;
    validate_nonempty("Program track", &input.program_track)?;
    let batch = resolve_student_batch(
        conn,
        input.batch_ref_id,
        input.school_id,
        &input.batch_id,
        &input.grade_level,
        &input.track,
    )?;

    conn.execute(
        "
        INSERT INTO students (school_id, name, registration_number, grade_level, program_track, track,
                              student_mobile, student_email, father_name, father_email, father_mobile,
                              mother_name, mother_email, mother_mobile, batch_ref_id, batch_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        ON CONFLICT(school_id, name) DO UPDATE SET
            registration_number = excluded.registration_number,
            grade_level = excluded.grade_level,
            program_track = excluded.program_track,
            track = excluded.track,
            student_mobile = excluded.student_mobile,
            student_email = excluded.student_email,
            father_name = excluded.father_name,
            father_email = excluded.father_email,
            father_mobile = excluded.father_mobile,
            mother_name = excluded.mother_name,
            mother_email = excluded.mother_email,
            mother_mobile = excluded.mother_mobile,
            batch_ref_id = excluded.batch_ref_id,
            batch_id = excluded.batch_id
        ",
        params![
            batch.school_id,
            input.name.trim(),
            input.registration_number.trim(),
            batch.grade_level.trim(),
            input.program_track.trim(),
            batch.track.trim(),
            input.student_mobile.trim(),
            input.student_email.trim(),
            input.father_name.trim(),
            input.father_email.trim(),
            input.father_mobile.trim(),
            input.mother_name.trim(),
            input.mother_email.trim(),
            input.mother_mobile.trim(),
            batch.id,
            batch.batch_id.trim(),
        ],
    )
    .map_err(|error| error.to_string())?;

    get_student_by_school_and_name(conn, batch.school_id, input.name.trim())
}

pub fn update_student(conn: &Connection, input: &UpdateStudentInput) -> Result<Student, String> {
    validate_nonempty("Student", &input.name)?;
    validate_nonempty("Program track", &input.program_track)?;
    let batch = resolve_student_batch(
        conn,
        input.batch_ref_id,
        input.school_id,
        &input.batch_id,
        &input.grade_level,
        &input.track,
    )?;

    conn.execute(
        "
        UPDATE students
        SET name = ?1, registration_number = ?2, grade_level = ?3, program_track = ?4, track = ?5,
            student_mobile = ?6, student_email = ?7, father_name = ?8, father_email = ?9,
            father_mobile = ?10, mother_name = ?11, mother_email = ?12, mother_mobile = ?13,
            batch_ref_id = ?14, batch_id = ?15
        WHERE id = ?16
        ",
        params![
            input.name.trim(),
            input.registration_number.trim(),
            batch.grade_level.trim(),
            input.program_track.trim(),
            batch.track.trim(),
            input.student_mobile.trim(),
            input.student_email.trim(),
            input.father_name.trim(),
            input.father_email.trim(),
            input.father_mobile.trim(),
            input.mother_name.trim(),
            input.mother_email.trim(),
            input.mother_mobile.trim(),
            batch.id,
            batch.batch_id.trim(),
            input.id,
        ],
    )
    .map_err(|error| error.to_string())?;

    get_student(conn, input.id)
}

fn resolve_student_batch(
    conn: &Connection,
    batch_ref_id: i64,
    school_id: i64,
    batch_name: &str,
    grade_level: &str,
    track: &str,
) -> Result<Batch, String> {
    if batch_ref_id > 0 {
        let batch = get_batch(conn, batch_ref_id)?;
        if school_id > 0 && batch.school_id != school_id {
            return Err("Selected batch does not belong to the selected school".to_string());
        }
        return Ok(batch);
    }
    validate_nonempty("Batch", batch_name)?;
    validate_nonempty("Grade", grade_level)?;
    let _ = get_school(conn, school_id)?;
    create_batch(
        conn,
        &CreateBatchInput {
            school_id,
            batch_id: batch_name.trim().to_string(),
            grade_level: grade_level.trim().to_string(),
            track: track.trim().to_string(),
            batch_pattern: "Weekday".to_string(),
            capacity: 0,
        },
    )
    .or_else(|_| {
        list_batches(conn, Some(school_id), None)?
            .into_iter()
            .find(|batch| batch.batch_id == batch_name.trim())
            .ok_or_else(|| "Could not resolve student batch".to_string())
    })
}

pub fn delete_student(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM students WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_batches(
    conn: &Connection,
    school_id: Option<i64>,
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<Batch>, String> {
    const MAX_ROWS: i64 = 1000;
    let mut sql = String::from(
        "SELECT batches.id, batches.school_id, schools.name, batches.batch_id,
                batches.grade_level, batches.track, batches.batch_pattern, batches.capacity,
                batches.created_at
         FROM batches
         JOIN schools ON schools.id = batches.school_id
         WHERE batches.deleted_at = ''",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(id) = school_id {
        sql.push_str(" AND batches.school_id = ?");
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND batches.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY schools.name, batches.grade_level, batches.batch_id LIMIT ?");
    p.push(MAX_ROWS.into());

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(p.iter()), |row| {
            Ok(Batch {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                batch_id: row.get(3)?,
                grade_level: row.get(4)?,
                track: row.get(5)?,
                batch_pattern: row.get(6)?,
                capacity: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn get_batch(conn: &Connection, id: i64) -> Result<Batch, String> {
    conn.query_row(
        "SELECT batches.id, batches.school_id, schools.name, batches.batch_id,
                batches.grade_level, batches.track, batches.batch_pattern, batches.capacity,
                batches.created_at
         FROM batches
         JOIN schools ON schools.id = batches.school_id
         WHERE batches.id = ?1 AND batches.deleted_at = ''",
        params![id],
        |row| {
            Ok(Batch {
                id: row.get(0)?,
                school_id: row.get(1)?,
                school_name: row.get(2)?,
                batch_id: row.get(3)?,
                grade_level: row.get(4)?,
                track: row.get(5)?,
                batch_pattern: row.get(6)?,
                capacity: row.get(7)?,
                created_at: row.get(8)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub fn create_batch(conn: &Connection, input: &CreateBatchInput) -> Result<Batch, String> {
    validate_nonempty("Batch", &input.batch_id)?;
    validate_nonempty("Class", &input.grade_level)?;
    validate_batch_pattern(&input.batch_pattern)?;
    if input.capacity < 0 {
        return Err("Capacity cannot be negative".to_string());
    }
    conn.execute(
        "INSERT INTO batches (school_id, batch_id, grade_level, track, batch_pattern, capacity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.school_id,
            input.batch_id.trim(),
            canonical_grade_level(&input.grade_level),
            input.track.trim(),
            input.batch_pattern.trim(),
            input.capacity,
        ],
    )
    .map_err(|error| error.to_string())?;
    get_batch(conn, conn.last_insert_rowid())
}

pub fn update_batch(conn: &Connection, input: &UpdateBatchInput) -> Result<Batch, String> {
    validate_nonempty("Batch", &input.batch_id)?;
    validate_nonempty("Class", &input.grade_level)?;
    validate_batch_pattern(&input.batch_pattern)?;
    if input.capacity < 0 {
        return Err("Capacity cannot be negative".to_string());
    }
    conn.execute(
        "UPDATE batches
         SET school_id = ?2, batch_id = ?3, grade_level = ?4, track = ?5, batch_pattern = ?6, capacity = ?7
         WHERE id = ?1 AND deleted_at = ''",
        params![
            input.id,
            input.school_id,
            input.batch_id.trim(),
            canonical_grade_level(&input.grade_level),
            input.track.trim(),
            input.batch_pattern.trim(),
            input.capacity,
        ],
    )
    .map_err(|error| error.to_string())?;
    get_batch(conn, input.id)
}

pub fn archive_batch(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE batches SET deleted_at = datetime('now', 'localtime') WHERE id = ?1 AND deleted_at = ''",
        params![id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn get_batch_students(
    conn: &Connection,
    batch_id: i64,
) -> Result<Vec<Student>, String> {
    let mut stmt = conn.prepare(
        "SELECT students.id, students.school_id, schools.name, students.name,
                students.registration_number, students.grade_level, students.program_track,
                students.track, students.student_mobile, students.student_email,
                students.father_name, students.father_email, students.father_mobile,
                students.mother_name, students.mother_email, students.mother_mobile,
                students.batch_ref_id, COALESCE(batches.batch_id, students.batch_id),
                students.batch_id, students.created_at
         FROM students
         JOIN schools ON schools.id = students.school_id
         LEFT JOIN batches ON batches.id = students.batch_ref_id
         WHERE students.batch_ref_id = ?1 AND schools.is_dropped = 0
         ORDER BY students.name"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![batch_id], |row| {
        Ok(Student {
            id: row.get(0)?,
            school_id: row.get(1)?,
            school_name: row.get(2)?,
            name: row.get(3)?,
            registration_number: row.get(4)?,
            grade_level: row.get(5)?,
            program_track: row.get(6)?,
            track: row.get(7)?,
            student_mobile: row.get(8)?,
            student_email: row.get(9)?,
            father_name: row.get(10)?,
            father_email: row.get(11)?,
            father_mobile: row.get(12)?,
            mother_name: row.get(13)?,
            mother_email: row.get(14)?,
            mother_mobile: row.get(15)?,
            batch_ref_id: row.get(16)?,
            batch_name: row.get(17)?,
            batch_id: row.get(18)?,
            created_at: row.get(19)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_batch_analytics(
    conn: &Connection,
    scope_school_ids: Option<&[i64]>,
) -> Result<BatchAnalytics, String> {
    let mut sql = String::from(
        "SELECT batches.id, batches.school_id, schools.name, batches.batch_id,
                batches.grade_level, batches.track, batches.batch_pattern, batches.capacity,
                batches.created_at
         FROM batches
         JOIN schools ON schools.id = batches.school_id
         WHERE batches.deleted_at = ''"
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND batches.school_id IN ({placeholders})"));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY schools.name, batches.grade_level, batches.batch_id");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let batch_rows = stmt.query_map(rusqlite::params_from_iter(p.iter()), |row| {
        Ok(Batch {
            id: row.get(0)?,
            school_id: row.get(1)?,
            school_name: row.get(2)?,
            batch_id: row.get(3)?,
            grade_level: row.get(4)?,
            track: row.get(5)?,
            batch_pattern: row.get(6)?,
            capacity: row.get(7)?,
            created_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    let batches: Vec<Batch> = batch_rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    let mut details: Vec<BatchDetail> = Vec::new();
    let mut total_students: i64 = 0;
    let mut total_capacity: i64 = 0;

    for batch in batches {
        let student_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM students WHERE batch_ref_id = ?1",
            params![batch.id],
            |row| row.get(0),
        ).unwrap_or(0);

        let faculty_count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT faculty_id) FROM faculty_assignments WHERE batch_id = ?1 AND is_active = 1",
            params![batch.id],
            |row| row.get(0),
        ).unwrap_or(0);

        let active_ticket_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tickets
             JOIN students ON students.id = tickets.student_id
             WHERE students.batch_ref_id = ?1 AND tickets.status != 'Resolved' AND tickets.status != 'Closed'",
            params![batch.id],
            |row| row.get(0),
        ).unwrap_or(0);

        let upcoming_session_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM timetable_slots
             WHERE batch_ref_id = ?1 AND session_date >= date('now', 'localtime') AND is_cancelled = 0",
            params![batch.id],
            |row| row.get(0),
        ).unwrap_or(0);

        total_students += student_count;
        total_capacity += batch.capacity;

        details.push(BatchDetail {
            batch,
            student_count,
            faculty_count,
            active_ticket_count,
            upcoming_session_count,
        });
    }

    let overall_utilization = if total_capacity > 0 {
        (total_students as f64 / total_capacity as f64) * 100.0
    } else {
        0.0
    };

    Ok(BatchAnalytics {
        batches: details,
        total_students,
        total_capacity,
        overall_utilization,
    })
}

pub fn list_regions(conn: &Connection) -> Result<Vec<Region>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, name, regional_academic_head_name, regional_academic_head_mobile,
                   regional_academic_head_email, regional_business_head_name,
                   regional_business_head_mobile, regional_business_head_email,
                   regional_deputy_academic_head_name, regional_deputy_academic_head_mobile,
                   regional_deputy_academic_head_email, updated_at
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

pub fn upsert_region(
    conn: &Connection,
    input: &UpsertRegionInput,
    actor: &str,
) -> Result<Region, String> {
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

fn count_school_dependency(conn: &Connection, sql: &str, school_id: i64) -> Result<i64, String> {
    conn.query_row(sql, params![school_id], |row| row.get(0))
        .map_err(|error| error.to_string())
}

pub fn get_school_delete_impact(
    conn: &Connection,
    id: i64,
) -> Result<SchoolDeleteImpact, String> {
    let school = get_school(conn, id)?;
    let checks = [
        ("Students", "SELECT COUNT(*) FROM students WHERE school_id = ?1"),
        ("Tickets", "SELECT COUNT(*) FROM tickets WHERE school_id = ?1"),
        ("Class offerings", "SELECT COUNT(*) FROM school_class_plans WHERE school_id = ?1"),
        ("Batches", "SELECT COUNT(*) FROM batches WHERE school_id = ?1"),
        ("Faculty memberships", "SELECT COUNT(*) FROM faculty_school_memberships WHERE school_id = ?1"),
        ("Faculty assignments", "SELECT COUNT(*) FROM faculty_assignments WHERE school_id = ?1"),
        ("Recurring timetable slots", "SELECT COUNT(*) FROM timetable_slots WHERE school_id = ?1"),
        ("Weekly timetable slots", "SELECT COUNT(*) FROM timetable_weekly_slots WHERE school_id = ?1"),
        (
            "Lecture sessions",
            "
            SELECT COUNT(*)
            FROM lecture_sessions ls
            JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
            WHERE ts.school_id = ?1
            ",
        ),
        (
            "Attendance records",
            "
            SELECT COUNT(*)
            FROM attendance_records ar
            JOIN lecture_sessions ls ON ls.id = ar.lecture_session_id
            JOIN timetable_slots ts ON ts.id = ls.timetable_slot_id
            WHERE ts.school_id = ?1
            ",
        ),
        ("User-school access links", "SELECT COUNT(*) FROM user_schools WHERE school_id = ?1"),
        ("School holidays", "SELECT COUNT(*) FROM holidays WHERE school_id = ?1"),
        ("Optional subject settings", "SELECT COUNT(*) FROM school_optional_subjects WHERE school_id = ?1"),
        ("Region history entries", "SELECT COUNT(*) FROM school_region_history WHERE school_id = ?1"),
    ];

    let mut total_linked_records = 0;
    let mut items = Vec::new();
    for (label, sql) in checks {
        let count = count_school_dependency(conn, sql, id)?;
        total_linked_records += count;
        items.push(SchoolDeleteImpactItem {
            label: label.to_string(),
            count,
        });
    }

    Ok(SchoolDeleteImpact {
        school_id: school.id,
        school_name: school.name,
        total_linked_records,
        items,
    })
}

pub fn drop_school(
    conn: &Connection,
    id: i64,
    reason: &str,
    actor: &str,
) -> Result<School, String> {
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

pub fn list_audit_log(
    conn: &Connection,
    limit: i64,
    offset: i64,
) -> Result<Paginated<AuditLogEntry>, String> {
    let safe_limit = limit.clamp(1, 500);
    let safe_offset = offset.max(0);

    let total_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "
            SELECT id, entity_type, entity_id, action, actor, summary, created_at
            FROM audit_log
            ORDER BY datetime(created_at) DESC, id DESC
            LIMIT ?1 OFFSET ?2
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![safe_limit, safe_offset], |row| {
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
    scope_school_ids: Option<&[i64]>,
) -> Result<Vec<SchoolClassPlan>, String> {
    let mut sql = String::from(
        "SELECT school_class_plans.id, school_class_plans.school_id, schools.name,
                school_class_plans.grade_level, school_class_plans.track,
                school_class_plans.lecture_model_id,
                lecture_models.name, lecture_models.days_per_week, lecture_models.lectures_per_day,
                school_class_plans.batch_pattern, school_class_plans.aop_admissions,
                school_class_plans.registrations, school_class_plans.actual_admissions,
                school_class_plans.updated_at
         FROM school_class_plans
         JOIN schools ON schools.id = school_class_plans.school_id
         JOIN lecture_models ON lecture_models.id = school_class_plans.lecture_model_id
         WHERE schools.is_dropped = 0",
    );
    let mut p: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(id) = school_id {
        sql.push_str(" AND school_class_plans.school_id = ?");
        p.push(id.into());
    }
    if let Some(ids) = scope_school_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(
                " AND school_class_plans.school_id IN ({placeholders})"
            ));
            for id in ids {
                p.push((*id).into());
            }
        }
    }
    sql.push_str(" ORDER BY schools.name, school_class_plans.grade_level");

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(p.iter()),
            school_class_plan_from_row,
        )
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>();

    rows.map_err(|error| error.to_string())
}

pub fn upsert_school_class_plan(
    conn: &Connection,
    input: &UpsertSchoolClassPlanInput,
) -> Result<SchoolClassPlan, String> {
    get_school(conn, input.school_id)?;
    let grade_level = canonical_grade_level(&input.grade_level);
    validate_nonempty("Grade", &grade_level)?;
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
            grade_level.as_str(),
            input.track.trim(),
            input.lecture_model_id,
            input.batch_pattern.trim(),
            input.aop_admissions,
            input.registrations,
            input.actual_admissions
        ],
    )
    .map_err(|error| error.to_string())?;

    get_school_class_plan(
        conn,
        input.school_id,
        grade_level.as_str(),
        input.track.trim(),
    )
}

pub fn get_school_program_dashboard(conn: &Connection) -> Result<SchoolProgramDashboard, String> {
    let plans = list_school_class_plans(conn, None, None)?;
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

pub fn student_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Student> {
    Ok(Student {
        id: row.get(0)?,
        school_id: row.get(1)?,
        school_name: row.get(2)?,
        name: row.get(3)?,
        registration_number: row.get(4)?,
        grade_level: row.get(5)?,
        program_track: row.get(6)?,

        track: row.get(7)?,
        student_mobile: row.get(8)?,
        student_email: row.get(9)?,
        father_name: row.get(10)?,
        father_email: row.get(11)?,
        father_mobile: row.get(12)?,
        mother_name: row.get(13)?,
        mother_email: row.get(14)?,
        mother_mobile: row.get(15)?,
        batch_ref_id: row.get(16)?,
        batch_name: row.get(17)?,
        batch_id: row.get(18)?,
        created_at: row.get(19)?,
    })
}

pub fn get_student_by_school_and_name(
    conn: &Connection,
    school_id: i64,
    name: &str,
) -> Result<Student, String> {
    conn.query_row(
        "SELECT students.id, students.school_id, schools.name, students.name,
                students.registration_number, students.grade_level, students.program_track,
                students.track, students.student_mobile, students.student_email,
                students.father_name, students.father_email, students.father_mobile,
                students.mother_name, students.mother_email, students.mother_mobile,
                students.batch_ref_id, COALESCE(batches.batch_id, students.batch_id),
                students.batch_id, students.created_at
         FROM students
         JOIN schools ON schools.id = students.school_id
         LEFT JOIN batches ON batches.id = students.batch_ref_id
         WHERE students.school_id = ?1 AND students.name = ?2",
        params![school_id, name],
        student_from_row,
    )
    .map_err(|e| e.to_string())
}

pub fn get_student(conn: &Connection, id: i64) -> Result<Student, String> {
    conn.query_row(
        "SELECT students.id, students.school_id, schools.name, students.name,
               students.registration_number, students.grade_level, students.program_track,
               students.track, students.student_mobile, students.student_email,
               students.father_name, students.father_email, students.father_mobile,
               students.mother_name, students.mother_email, students.mother_mobile,
               students.batch_ref_id, COALESCE(batches.batch_id, students.batch_id),
               students.batch_id, students.created_at
         FROM students
         JOIN schools ON schools.id = students.school_id
         LEFT JOIN batches ON batches.id = students.batch_ref_id
         WHERE students.id = ?1",
        params![id],
        student_from_row,
    )
    .map_err(|error| error.to_string())
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

    #[test]
    fn school_and_student_master_data_can_be_created_and_listed() {
        let conn = test_db();

        // Create a region
        let region = upsert_region(
            &conn,
            &UpsertRegionInput {
                id: None,
                name: "North Zone".to_string(),
                regional_academic_head_name: "Rahul".to_string(),
                regional_academic_head_mobile: "9876543210".to_string(),
                regional_academic_head_email: "rahul@example.com".to_string(),
                regional_business_head_name: "Priya".to_string(),
                regional_business_head_mobile: "9876543211".to_string(),
                regional_business_head_email: "priya@example.com".to_string(),
                regional_deputy_academic_head_name: String::new(),
                regional_deputy_academic_head_mobile: String::new(),
                regional_deputy_academic_head_email: String::new(),
            },
            TEST_ACTOR,
        )
        .expect("create region");
        assert_eq!(region.name, "North Zone");

        // Create a school
        let school = create_school(
            &conn,
            &CreateSchoolInput {
                name: "Green Valley Public School".to_string(),
                region_id: Some(region.id),
                program_model: "Aspire".to_string(),
                distance_classification: "Near Proximity".to_string(),
                sip_academic_owner_role: "Academic Coordinator".to_string(),
                sip_academic_owner_name: "Anita".to_string(),
                sip_academic_owner_mobile: "9876543212".to_string(),
                sip_academic_owner_email: "anita@example.com".to_string(),
                center_head_name: "Ravi".to_string(),
                center_head_mobile: "9876543213".to_string(),
                center_head_email: "ravi@example.com".to_string(),
                principal_name: "Sunita".to_string(),
                principal_mobile: "9876543214".to_string(),
                principal_email: "sunita@example.com".to_string(),
                school_spoc_name: "Vikram".to_string(),
                school_spoc_mobile: "9876543215".to_string(),
                school_spoc_email: "vikram@example.com".to_string(),
                central_academic_spoc_name: "Deepa".to_string(),
                central_academic_spoc_mobile: "9876543216".to_string(),
                central_academic_spoc_email: "deepa@example.com".to_string(),
                central_business_spoc_name: "Arjun".to_string(),
                central_business_spoc_mobile: "9876543217".to_string(),
                central_business_spoc_email: "arjun@example.com".to_string(),
                bh_name: "Meera".to_string(),
                bh_mobile: "9876543218".to_string(),
                bh_email: "meera@example.com".to_string(),
                aom_name: "Kiran".to_string(),
                aom_mobile: "9876543219".to_string(),
                aom_email: "kiran@example.com".to_string(),
                mapped_vp_center: String::new(),
                vp_tagging: String::new(),
            },
            TEST_ACTOR,
        )
        .expect("create school");
        assert_eq!(school.name, "Green Valley Public School");
        assert_eq!(school.program_model, "Aspire");

        // Create a student
        let student = create_student(
            &conn,
            &CreateStudentInput {
                school_id: school.id,
                name: "Aarav Shah".to_string(),
                registration_number: "GV2024001".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE Foundation".to_string(),
                track: "JEE".to_string(),
                student_mobile: "9876543220".to_string(),
                student_email: "aarav@example.com".to_string(),
                father_name: "Rajesh Shah".to_string(),
                father_email: "rajesh@example.com".to_string(),
                father_mobile: "9876543221".to_string(),
                mother_name: "Pooja Shah".to_string(),
                mother_email: "pooja@example.com".to_string(),
                mother_mobile: "9876543222".to_string(),
                batch_ref_id: 0,
                batch_id: "Batch A".to_string(),
            },
        )
        .expect("create student");
        assert_eq!(student.name, "Aarav Shah");
        assert_eq!(student.school_name, "Green Valley Public School");

        // List schools and students
        let schools = list_schools(&conn, None).expect("list schools");
        let students = list_students(&conn, None, None).expect("list students");
        assert!(schools.len() >= 1);
        assert!(students.len() >= 1);

        // Verify school is linked to region
        let fetched = get_school(&conn, school.id).expect("get school");
        assert_eq!(fetched.region_id, Some(region.id));
    }

    #[test]
    fn school_class_plans_track_delivery_and_admissions() {
        let conn = test_db();

        // Use a seeded school
        let schools = list_schools(&conn, None).expect("list schools");
        let school = schools.first().expect("at least one seeded school");

        // Create a lecture model
        let model = create_lecture_model(
            &conn,
            &CreateLectureModelInput {
                name: "6x2 Intensive".to_string(),
                days_per_week: 6,
                lectures_per_day: 2,
            },
        )
        .expect("create lecture model");
        assert_eq!(model.name, "6x2 Intensive");

        // Create a class plan
        let plan = upsert_school_class_plan(
            &conn,
            &UpsertSchoolClassPlanInput {
                school_id: school.id,
                grade_level: "Grade 11".to_string(),
                track: "JEE".to_string(),
                lecture_model_id: model.id,
                batch_pattern: "Weekday".to_string(),
                aop_admissions: 100,
                registrations: 80,
                actual_admissions: 75,
            },
        )
        .expect("upsert class plan");
        assert_eq!(plan.lecture_model_name, "6x2 Intensive");
        assert_eq!(plan.grade_level, "Grade 11");
        assert_eq!(plan.aop_admissions, 100);
        assert_eq!(plan.registrations, 80);
        assert_eq!(plan.actual_admissions, 75);
        assert_eq!(plan.admission_gap, 25);
        assert_eq!(plan.admission_attainment_percent, 75);

        // Verify dashboard
        let dashboard = get_school_program_dashboard(&conn).expect("dashboard");
        assert!(dashboard.total_schools >= 1);
        assert!(dashboard.total_classes >= 1);
        assert!(dashboard.total_aop_admissions >= 100);
        assert!(dashboard.total_actual_admissions >= 75);
    }

    #[test]
    fn school_delete_impact_counts_linked_records_before_hard_delete() {
        let conn = test_db();
        let school = list_schools(&conn, None)
            .expect("list schools")
            .into_iter()
            .next()
            .expect("seeded school");

        create_student(
            &conn,
            &CreateStudentInput {
                school_id: school.id,
                name: "Delete Impact Student".to_string(),
                registration_number: "DIS001".to_string(),
                grade_level: "Grade 11".to_string(),
                program_track: "JEE".to_string(),
                track: "JEE".to_string(),
                student_mobile: String::new(),
                student_email: String::new(),
                father_name: String::new(),
                father_email: String::new(),
                father_mobile: String::new(),
                mother_name: String::new(),
                mother_email: String::new(),
                mother_mobile: String::new(),
                batch_ref_id: 0,
                batch_id: "Delete Impact Batch".to_string(),
            },
        )
        .expect("create student");

        conn.execute(
            "
            INSERT INTO tickets
                (title, description, requester, school_id, school_name, issue_category)
            VALUES
                ('Delete impact ticket', 'Check impact', 'Tester', ?1, ?2, 'Operations')
            ",
            params![school.id, school.name],
        )
        .expect("insert ticket");

        let impact = get_school_delete_impact(&conn, school.id).expect("delete impact");
        assert_eq!(impact.school_id, school.id);
        assert!(impact.total_linked_records >= 2);
        assert!(impact.items.iter().any(|item| item.label == "Students" && item.count >= 1));
        assert!(impact.items.iter().any(|item| item.label == "Tickets" && item.count >= 1));
    }

    #[test]
    fn class_plan_upsert_canonicalizes_numeric_grade() {
        let conn = test_db();
        let school = list_schools(&conn, None)
            .expect("list schools")
            .into_iter()
            .next()
            .expect("seeded school");
        let model = create_lecture_model(
            &conn,
            &CreateLectureModelInput {
                name: "3x3 Canonical".to_string(),
                days_per_week: 3,
                lectures_per_day: 3,
            },
        )
        .expect("create lecture model");

        let plan = upsert_school_class_plan(
            &conn,
            &UpsertSchoolClassPlanInput {
                school_id: school.id,
                grade_level: "11".to_string(),
                track: "JEE".to_string(),
                lecture_model_id: model.id,
                batch_pattern: "Weekday".to_string(),
                aop_admissions: 50,
                registrations: 45,
                actual_admissions: 40,
            },
        )
        .expect("upsert class plan");

        assert_eq!(plan.grade_level, "Grade 11");
    }

    #[test]
    fn batches_can_be_created_updated_and_archived() {
        let conn = test_db();
        let school = list_schools(&conn, None)
            .expect("list schools")
            .into_iter()
            .next()
            .expect("seeded school");

        let batch = create_batch(
            &conn,
            &CreateBatchInput {
                school_id: school.id,
                batch_id: "XI-JEE-WD-A".to_string(),
                grade_level: "11".to_string(),
                track: "JEE".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 40,
            },
        )
        .expect("create batch");

        assert_eq!(batch.grade_level, "Grade 11");
        assert_eq!(batch.capacity, 40);

        let updated = update_batch(
            &conn,
            &UpdateBatchInput {
                id: batch.id,
                school_id: school.id,
                batch_id: "XI-JEE-WD-B".to_string(),
                grade_level: "Grade 11".to_string(),
                track: "JEE".to_string(),
                batch_pattern: "Weekday".to_string(),
                capacity: 45,
            },
        )
        .expect("update batch");

        assert_eq!(updated.batch_id, "XI-JEE-WD-B");
        assert_eq!(updated.capacity, 45);

        archive_batch(&conn, updated.id).expect("archive batch");
        let active = list_batches(&conn, Some(school.id), None).expect("list batches");
        assert!(!active.iter().any(|item| item.id == updated.id));
    }
}
