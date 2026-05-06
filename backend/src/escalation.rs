use rusqlite::{params, Connection};
use serde_json::Value;

use crate::models::{CreateEscalationRuleInput, EscalationRule, UpdateEscalationRuleInput};

pub fn list_escalation_rules(conn: &Connection) -> Result<Vec<EscalationRule>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, conditions_json, action, assignee_role, hours_threshold, is_active, created_at, updated_at FROM escalation_rules ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EscalationRule {
                id: row.get(0)?,
                name: row.get(1)?,
                conditions_json: row.get(2)?,
                action: row.get(3)?,
                assignee_role: row.get(4)?,
                hours_threshold: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_escalation_rule(
    conn: &Connection,
    input: &CreateEscalationRuleInput,
) -> Result<EscalationRule, String> {
    conn.execute(
        "INSERT INTO escalation_rules (name, conditions_json, action, assignee_role, hours_threshold, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now', 'localtime'), datetime('now', 'localtime'))",
        params![input.name, input.conditions_json, input.action, input.assignee_role, input.hours_threshold, if input.is_active { 1 } else { 0 }],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(EscalationRule {
        id,
        name: input.name.clone(),
        conditions_json: input.conditions_json.clone(),
        action: input.action.clone(),
        assignee_role: input.assignee_role.clone(),
        hours_threshold: input.hours_threshold,
        is_active: input.is_active,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

pub fn update_escalation_rule(
    conn: &Connection,
    input: &UpdateEscalationRuleInput,
) -> Result<EscalationRule, String> {
    conn.execute(
        "UPDATE escalation_rules SET name = ?1, conditions_json = ?2, action = ?3, assignee_role = ?4, hours_threshold = ?5, is_active = ?6, updated_at = datetime('now', 'localtime') WHERE id = ?7",
        params![input.name, input.conditions_json, input.action, input.assignee_role, input.hours_threshold, if input.is_active { 1 } else { 0 }, input.id],
    ).map_err(|e| e.to_string())?;
    Ok(EscalationRule {
        id: input.id,
        name: input.name.clone(),
        conditions_json: input.conditions_json.clone(),
        action: input.action.clone(),
        assignee_role: input.assignee_role.clone(),
        hours_threshold: input.hours_threshold,
        is_active: input.is_active,
        created_at: String::new(),
        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Evaluate all active rules against a ticket and return the first matching assignee_role.
pub fn evaluate_rules_for_ticket(
    conn: &Connection,
    queue: &str,
    priority: &str,
    hours_open: i64,
) -> Result<Option<String>, String> {
    let rules = list_escalation_rules(conn)?;
    for rule in rules.into_iter().filter(|r| r.is_active) {
        let cond: Value = serde_json::from_str(&rule.conditions_json).unwrap_or(Value::Null);
        if let Some(obj) = cond.as_object() {
            let mut matched = true;
            if let Some(q) = obj.get("queue").and_then(|v| v.as_str()) {
                if q != queue {
                    matched = false;
                }
            }
            if let Some(p) = obj.get("priority").and_then(|v| v.as_str()) {
                if p != priority {
                    matched = false;
                }
            }
            if matched && hours_open >= rule.hours_threshold {
                return Ok(Some(rule.assignee_role));
            }
        }
    }
    Ok(None)
}
