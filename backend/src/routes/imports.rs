use axum::{
    extract::{Extension, Multipart, State},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{error::AppError, models::*, repositories};

#[derive(Serialize)]
pub struct SchoolImportResult {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}

pub async fn import_schools_csv(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<SchoolImportResult>, AppError> {
    if claims.role == "viewer" {
        return Err(AppError::forbidden("Viewers cannot import schools"));
    }

    let mut content: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::bad_request(format!("Failed to read upload: {e}")))?;
            content = Some(
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| AppError::bad_request("CSV is not valid UTF-8"))?,
            );
            break;
        }
    }

    let content = content.ok_or_else(|| AppError::bad_request("No file field in upload"))?;
    let rows = parse_csv_rows(&content)
        .map_err(|e| AppError::bad_request(format!("CSV parse error: {e}")))?;
    if rows.is_empty() {
        return Err(AppError::bad_request("CSV is empty"));
    }

    let headers: Vec<String> = rows[0].iter().map(|h| normalize_csv_header(h)).collect();
    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (line_no, row) in rows.into_iter().enumerate().skip(1) {
        let values: HashMap<String, String> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), row.get(i).cloned().unwrap_or_default()))
            .collect();
        let input = school_input_from_csv(&values);

        if input.name.trim().is_empty() {
            skipped_count += 1;
            continue;
        }

        match repositories::create_school(&*conn, &input, &claims.display_name) {
            Ok(_) => imported_count += 1,
            Err(e) => {
                skipped_count += 1;
                if errors.len() < 20 {
                    errors.push(format!("Row {}: {} ({})", line_no + 1, input.name, e));
                }
            }
        }
    }

    Ok(Json(SchoolImportResult {
        imported_count,
        skipped_count,
        errors,
    }))
}

// ── SIP master: preview + import ────────────────────────────────────────────

#[derive(Serialize)]
pub struct SipMasterImportPreview {
    pub total_rows: usize,
    pub new_school_count: usize,
    pub existing_school_count: usize,
    pub skipped_row_count: usize,
    pub existing_schools: Vec<String>,
}

#[derive(Serialize)]
pub struct SipMasterImportResult {
    pub imported_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
    pub class_plan_count: usize,
}

async fn read_upload(multipart: &mut Multipart) -> Result<(Option<String>, Option<String>), AppError>
{
    let mut content: Option<String> = None;
    let mut conflict_action: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Multipart error: {e}")))?
    {
        match field.name() {
            Some("file") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Failed to read upload: {e}")))?;
                content = Some(
                    String::from_utf8(bytes.to_vec())
                        .map_err(|_| AppError::bad_request("CSV is not valid UTF-8"))?,
                );
            }
            Some("conflict_action") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Failed to read field: {e}")))?;
                conflict_action = Some(
                    String::from_utf8(bytes.to_vec())
                        .map_err(|_| AppError::bad_request("conflict_action is not UTF-8"))?,
                );
            }
            _ => {}
        }
    }
    Ok((content, conflict_action))
}

pub async fn preview_sip_master_import(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<SipMasterImportPreview>, AppError> {
    if claims.role == "viewer" {
        return Err(AppError::forbidden("Viewers cannot import SIP master"));
    }

    let (content, _) = read_upload(&mut multipart).await?;
    let content = content.ok_or_else(|| AppError::bad_request("No file field in upload"))?;
    let rows = parse_csv_rows(&content)
        .map_err(|e| AppError::bad_request(format!("CSV parse error: {e}")))?;
    if rows.is_empty() {
        return Err(AppError::bad_request("SIP master file is empty"));
    }

    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let existing_names = existing_school_names(&*conn)?;
    let mut existing_schools: Vec<String> = Vec::new();
    let mut new_school_count = 0;
    let mut skipped_row_count = 0;

    for values in tabular_values(&rows).into_iter() {
        let school_name = csv_value(&values, &["school_name", "name"]);
        if school_name.is_empty() {
            skipped_row_count += 1;
            continue;
        }
        if existing_names.contains(&normalize_school_key(&school_name)) {
            existing_schools.push(school_name);
        } else {
            new_school_count += 1;
        }
    }
    existing_schools.sort();
    existing_schools.dedup();

    Ok(Json(SipMasterImportPreview {
        total_rows: rows.len().saturating_sub(1),
        new_school_count,
        existing_school_count: existing_schools.len(),
        skipped_row_count,
        existing_schools,
    }))
}

pub async fn import_sip_master(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<SipMasterImportResult>, AppError> {
    if claims.role == "viewer" {
        return Err(AppError::forbidden("Viewers cannot import SIP master"));
    }

    let (content, conflict_action) = read_upload(&mut multipart).await?;
    let content = content.ok_or_else(|| AppError::bad_request("No file field in upload"))?;
    let conflict_action = conflict_action
        .ok_or_else(|| AppError::bad_request("Missing conflict_action field"))?;
    let update_existing = match conflict_action.trim() {
        "update_existing" => true,
        "skip_existing" => false,
        other => {
            return Err(AppError::bad_request(format!(
                "Unsupported conflict_action: {other}"
            )))
        }
    };

    let rows = parse_csv_rows(&content)
        .map_err(|e| AppError::bad_request(format!("CSV parse error: {e}")))?;
    if rows.is_empty() {
        return Err(AppError::bad_request("SIP master file is empty"));
    }

    let conn = state.db.get().map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let mut existing_names = existing_school_names(&*conn)?;
    let mut imported_count = 0;
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut class_plan_count = 0;

    for values in tabular_values(&rows).into_iter() {
        let mut input = school_input_from_csv(&values);
        if input.name.trim().is_empty() {
            skipped_count += 1;
            continue;
        }

        if let Some(region_id) = import_region_id_from_row(&*conn, &values, &claims.display_name)? {
            input.region_id = Some(region_id);
        }

        let exists = existing_names.contains(&normalize_school_key(&input.name));
        if exists && !update_existing {
            skipped_count += 1;
            continue;
        }

        let school = repositories::create_school(&*conn, &input, &claims.display_name)?;
        if exists {
            updated_count += 1;
        } else {
            imported_count += 1;
            existing_names.push(normalize_school_key(&input.name));
        }

        class_plan_count += import_school_class_plans_from_master_row(&*conn, school.id, &values)?;
    }

    Ok(Json(SipMasterImportResult {
        imported_count,
        updated_count,
        skipped_count,
        class_plan_count,
    }))
}

// ── SIP master helpers ──────────────────────────────────────────────────────

fn tabular_values(rows: &[Vec<String>]) -> Vec<HashMap<String, String>> {
    let headers: Vec<String> = rows[0].iter().map(|h| normalize_csv_header(h)).collect();
    rows.iter()
        .skip(1)
        .map(|row| {
            headers
                .iter()
                .enumerate()
                .map(|(i, h)| (h.clone(), row.get(i).cloned().unwrap_or_default()))
                .collect()
        })
        .collect()
}

fn existing_school_names(conn: &rusqlite::Connection) -> Result<Vec<String>, String> {
    let mut schools = repositories::list_schools(conn, None)?;
    schools.extend(repositories::list_dropped_schools(conn, None)?);
    Ok(schools
        .into_iter()
        .map(|school| normalize_school_key(&school.name))
        .collect())
}

fn normalize_school_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn import_region_id_from_row(
    conn: &rusqlite::Connection,
    values: &HashMap<String, String>,
    actor: &str,
) -> Result<Option<i64>, String> {
    let region_name = csv_value(values, &["region", "region_name"]);
    if region_name.is_empty() {
        return Ok(None);
    }
    let region = repositories::upsert_region(
        conn,
        &UpsertRegionInput {
            id: None,
            name: region_name,
            regional_academic_head_name: String::new(),
            regional_academic_head_mobile: String::new(),
            regional_academic_head_email: String::new(),
            regional_business_head_name: String::new(),
            regional_business_head_mobile: String::new(),
            regional_business_head_email: String::new(),
        },
        actor,
    )?;
    Ok(Some(region.id))
}

fn import_school_class_plans_from_master_row(
    conn: &rusqlite::Connection,
    school_id: i64,
    values: &HashMap<String, String>,
) -> Result<usize, String> {
    // (grade_level, track) pairs. Grades 6-10 have no JEE/NEET split (track="").
    // Grades 11/12/Dropper split into JEE and NEET.
    let plan_slots: &[(&str, &str)] = &[
        ("Grade 6", ""),
        ("Grade 7", ""),
        ("Grade 8", ""),
        ("Grade 9", ""),
        ("Grade 10", ""),
        ("Grade 11", "JEE"),
        ("Grade 11", "NEET"),
        ("Grade 12", "JEE"),
        ("Grade 12", "NEET"),
        ("Dropper", "JEE"),
        ("Dropper", "NEET"),
    ];
    let mut saved_count = 0;

    for (grade, track) in plan_slots {
        // Header column prefix: "grade_8" or "grade_11_jee" or "dropper_neet".
        let prefix = if track.is_empty() {
            normalize_csv_header(grade)
        } else {
            format!("{}_{}", normalize_csv_header(grade), normalize_csv_header(track))
        };
        let lecture_model_name = csv_value(values, &[&format!("{prefix}_lecture_model")]);
        let batch_pattern = csv_value(values, &[&format!("{prefix}_batch_pattern")]);
        let aop_admissions = parse_optional_i64(&csv_value(
            values,
            &[&format!("{prefix}_aop_admissions"), &format!("{prefix}_aop")],
        ))?;
        let registrations = parse_optional_i64(&csv_value(
            values,
            &[&format!("{prefix}_registrations"), &format!("{prefix}_registration")],
        ))?;
        let actual_admissions = parse_optional_i64(&csv_value(
            values,
            &[&format!("{prefix}_actual_admissions"), &format!("{prefix}_actual")],
        ))?;

        if lecture_model_name.is_empty()
            && batch_pattern.is_empty()
            && aop_admissions.is_none()
            && registrations.is_none()
            && actual_admissions.is_none()
        {
            continue;
        }

        let lecture_model = repositories::create_lecture_model(
            conn,
            &CreateLectureModelInput {
                name: if lecture_model_name.is_empty() {
                    "3x3".to_string()
                } else {
                    lecture_model_name.clone()
                },
                days_per_week: lecture_model_part(&lecture_model_name, 0).unwrap_or(3),
                lectures_per_day: lecture_model_part(&lecture_model_name, 1).unwrap_or(3),
            },
        )?;

        repositories::upsert_school_class_plan(
            conn,
            &UpsertSchoolClassPlanInput {
                school_id,
                grade_level: grade.to_string(),
                track: track.to_string(),
                lecture_model_id: lecture_model.id,
                batch_pattern: if batch_pattern.is_empty() {
                    "Weekday".to_string()
                } else {
                    normalize_batch_pattern(&batch_pattern)
                },
                aop_admissions: aop_admissions.unwrap_or(0),
                registrations: registrations.unwrap_or(0),
                actual_admissions: actual_admissions.unwrap_or(0),
            },
        )?;
        saved_count += 1;
    }

    Ok(saved_count)
}

fn parse_optional_i64(value: &str) -> Result<Option<i64>, String> {
    let trimmed = value.trim().trim_end_matches('%');
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<i64>()
        .map(Some)
        .map_err(|_| format!("Expected a whole number but found '{value}'"))
}

fn lecture_model_part(value: &str, part_index: usize) -> Option<i64> {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "");
    let parts = normalized.split('x').collect::<Vec<_>>();
    parts.get(part_index)?.parse::<i64>().ok()
}

fn normalize_batch_pattern(value: &str) -> String {
    match normalize_csv_header(value).as_str() {
        "weekday" | "weekdays" | "weekday_only" | "weekday_batches" => "Weekday".to_string(),
        "weekend" | "weekends" | "weekend_only" | "weekend_batches" => "Weekend".to_string(),
        "both" | "weekday_weekend" | "weekend_weekday" => "Both".to_string(),
        _ => value.trim().to_string(),
    }
}

// ── CSV parsing helpers (ported from desktop commands.rs) ───────────────────

fn parse_csv_rows(content: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut chars = content.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                row.push(cell.trim().to_string());
                cell.clear();
            }
            '\n' if !in_quotes => {
                row.push(cell.trim().to_string());
                cell.clear();
                if row.iter().any(|v| !v.is_empty()) {
                    rows.push(row);
                }
                row = Vec::new();
            }
            '\r' if !in_quotes => {}
            _ => cell.push(ch),
        }
    }

    if in_quotes {
        return Err("Unterminated quoted value".to_string());
    }

    row.push(cell.trim().to_string());
    if row.iter().any(|v| !v.is_empty()) {
        rows.push(row);
    }
    Ok(rows)
}

fn normalize_csv_header(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn csv_value(values: &HashMap<String, String>, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| values.get(*key).cloned())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn normalize_school_model(value: &str) -> String {
    match normalize_csv_header(value).as_str() {
        "aspire" => "Aspire".to_string(),
        "minimum_guarantee" | "minimum_gaurantee" | "mg" => "Minimum Guarantee".to_string(),
        _ => value.trim().to_string(),
    }
}

fn normalize_school_distance(value: &str) -> String {
    match normalize_csv_header(value).as_str() {
        "remote" | "_30_km" | "30_km" | "greater_than_30_km" | "distance_from_vp_30_km" => {
            "Remote".to_string()
        }
        "near" | "near_proximity" | "within_30_km" | "less_than_30_km" => {
            "Near Proximity".to_string()
        }
        _ => value.trim().to_string(),
    }
}

fn sip_role_for_distance(value: &str) -> String {
    match value {
        "Remote" => "SIP Academic Head".to_string(),
        "Near Proximity" => "SIP Academic Lead".to_string(),
        _ => String::new(),
    }
}

fn school_input_from_csv(values: &HashMap<String, String>) -> CreateSchoolInput {
    let distance_classification =
        normalize_school_distance(&csv_value(values, &["distance_classification", "distance"]));
    let explicit_role = csv_value(
        values,
        &[
            "sip_academic_owner_role",
            "sip_academic_head_lead_role",
            "sip_academic_role",
        ],
    );

    CreateSchoolInput {
        name: csv_value(values, &["name", "school_name"]),
        region_id: None,
        program_model: normalize_school_model(&csv_value(values, &["model", "program_model"])),
        distance_classification: distance_classification.clone(),
        sip_academic_owner_role: if explicit_role.is_empty() {
            sip_role_for_distance(&distance_classification)
        } else {
            explicit_role
        },
        sip_academic_owner_name: csv_value(
            values,
            &["sip_academic_owner_name", "sip_academic_head_lead_name"],
        ),
        sip_academic_owner_mobile: csv_value(
            values,
            &["sip_academic_owner_mobile", "sip_academic_head_lead_mobile"],
        ),
        sip_academic_owner_email: csv_value(
            values,
            &["sip_academic_owner_email", "sip_academic_head_lead_email"],
        ),
        center_head_name: csv_value(values, &["center_head_name"]),
        center_head_mobile: csv_value(values, &["center_head_mobile"]),
        center_head_email: csv_value(values, &["center_head_email"]),
        principal_name: csv_value(values, &["principal_name", "school_principal_name"]),
        principal_mobile: csv_value(values, &["principal_mobile", "school_principal_mobile"]),
        principal_email: csv_value(values, &["principal_email", "school_principal_email"]),
        school_spoc_name: csv_value(values, &["school_spoc_name"]),
        school_spoc_mobile: csv_value(values, &["school_spoc_mobile"]),
        school_spoc_email: csv_value(values, &["school_spoc_email"]),
        central_academic_spoc_name: csv_value(
            values,
            &[
                "central_academic_spoc_name",
                "central_sip_academic_spoc_name",
            ],
        ),
        central_academic_spoc_mobile: csv_value(
            values,
            &[
                "central_academic_spoc_mobile",
                "central_sip_academic_spoc_mobile",
            ],
        ),
        central_academic_spoc_email: csv_value(
            values,
            &[
                "central_academic_spoc_email",
                "central_sip_academic_spoc_email",
            ],
        ),
        central_business_spoc_name: csv_value(values, &["central_business_spoc_name"]),
        central_business_spoc_mobile: csv_value(values, &["central_business_spoc_mobile"]),
        central_business_spoc_email: csv_value(values, &["central_business_spoc_email"]),
        bh_name: csv_value(values, &["bh_name", "business_head_name"]),
        bh_mobile: csv_value(values, &["bh_mobile", "business_head_mobile"]),
        bh_email: csv_value(values, &["bh_email", "business_head_email"]),
        aom_name: csv_value(values, &["aom_name", "academic_operations_manager_name"]),
        aom_mobile: csv_value(
            values,
            &["aom_mobile", "academic_operations_manager_mobile"],
        ),
        aom_email: csv_value(values, &["aom_email", "academic_operations_manager_email"]),
        mapped_vp_center: csv_value(
            values,
            &["mapped_vp_center", "vp_center", "vp_centre", "mapped_vp_centre"],
        ),
    }
}
