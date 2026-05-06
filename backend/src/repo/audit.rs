use rusqlite::{params, Connection};

pub fn record_audit(
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

pub fn insert_audit_log(
    conn: &Connection,
    entity_type: &str,
    entity_id: Option<i64>,
    action: &str,
    actor: &str,
    summary: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, action, actor, summary)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entity_type, entity_id, action, actor, summary],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
