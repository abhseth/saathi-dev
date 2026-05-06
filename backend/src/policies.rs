use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::models::CentralPolicy;

/// Get a policy value from DB, falling back to the provided default.
pub fn get_policy_value(conn: &Connection, key: &str, default: &str) -> Result<String, String> {
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM central_policies WHERE key = ?1 LIMIT 1",
            params![key],
            |row| row.get(0),
        )
        .ok();
    Ok(val.unwrap_or_else(|| default.to_string()))
}

pub fn get_policy_value_as_i64(conn: &Connection, key: &str, default: i64) -> Result<i64, String> {
    let raw = get_policy_value(conn, key, &default.to_string())?;
    Ok(raw.parse().unwrap_or(default))
}

pub fn list_policies(conn: &Connection) -> Result<Vec<CentralPolicy>, String> {
    let mut stmt = conn
        .prepare("SELECT id, key, value, region_id, updated_at FROM central_policies ORDER BY key")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CentralPolicy {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                region_id: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn upsert_policy(
    conn: &Connection,
    key: &str,
    value: &str,
    region_id: Option<i64>,
) -> Result<CentralPolicy, String> {
    conn.execute(
        "INSERT INTO central_policies (key, value, region_id, updated_at)
         VALUES (?1, ?2, ?3, datetime('now', 'localtime'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, region_id = excluded.region_id, updated_at = excluded.updated_at",
        params![key, value, region_id],
    ).map_err(|e| e.to_string())?;
    let id: i64 = conn.last_insert_rowid();
    Ok(CentralPolicy {
        id,
        key: key.to_string(),
        value: value.to_string(),
        region_id,
        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Reload all policies into a HashMap for fast lookup during alert calculation.
pub fn load_policy_map(conn: &Connection) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let policies = list_policies(conn)?;
    for p in policies {
        map.insert(p.key, p.value);
    }
    Ok(map)
}
