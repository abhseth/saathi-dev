use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::{AppState, Claims, CurrentUser},
    repositories,
};

const TOKEN_EXPIRY_SECS: usize = 8 * 60 * 60; // 8 hours

pub fn issue_token(state: &AppState, user: &CurrentUser) -> Result<String, AppError> {
    let exp = (chrono::Utc::now().timestamp() as usize) + TOKEN_EXPIRY_SECS;
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        role: user.role.clone(),
        school_ids: user.school_ids.clone(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::internal(format!("Failed to issue token: {e}")))
}

pub fn decode_token(state: &AppState, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::unauthorized(format!("Invalid token: {e}")))
}

/// Revalidate decoded JWT claims against the current DB user state.
/// Returns fresh Claims with updated role / school_ids, preserving the original expiry.
pub fn revalidate_claims(
    decoded: &Claims,
    conn: &rusqlite::Connection,
) -> Result<Claims, AppError> {
    let user_id: i64 = decoded.sub.parse().unwrap_or(0);
    if user_id == 0 {
        return Err(AppError::unauthorized("Invalid token subject"));
    }
    let user = repositories::get_active_user_by_id(conn, user_id)
        .map_err(|_| AppError::unauthorized("Session invalidated"))?;

    Ok(Claims {
        sub: user.id.to_string(),
        username: user.username,
        display_name: user.display_name,
        role: user.role,
        school_ids: user.school_ids,
        exp: decoded.exp,
    })
}

/// Axum middleware: extract Bearer token, decode it, revalidate against current DB
/// state, and insert fresh Claims into request extensions.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

    let decoded = decode_token(&state, token)?;
    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;
    let fresh_claims = revalidate_claims(&decoded, &*conn)?;

    request.extensions_mut().insert(fresh_claims);
    Ok(next.run(request).await)
}

// ── Role check helpers (centralised so every route module can reuse) ─────────

pub fn require_admin(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" {
        Ok(())
    } else {
        Err(AppError::forbidden("Admin role required"))
    }
}

pub fn require_admin_or_aom(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "aom" {
        Ok(())
    } else {
        Err(AppError::forbidden("Admin or AOM role required"))
    }
}

pub fn require_head_or_admin_or_aom(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "aom" || claims.role == "head" {
        Ok(())
    } else {
        Err(AppError::forbidden("Admin, AOM, or Head role required"))
    }
}

pub fn require_faculty_or_admin(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "faculty" {
        Ok(())
    } else {
        Err(AppError::forbidden("Faculty or admin role required"))
    }
}

pub fn require_faculty_or_admin_or_aom(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "aom" || claims.role == "faculty" {
        Ok(())
    } else {
        Err(AppError::forbidden("Admin, AOM, or faculty role required"))
    }
}

pub fn require_ticket_writer(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "agent" || claims.role == "aom" {
        Ok(())
    } else {
        Err(AppError::forbidden("Ticket write access required"))
    }
}

/// Enforce that the user can access a specific school.
/// Admin, agent, and viewer see everything.
/// Aom, faculty, and head are scoped to their assigned schools.
pub fn enforce_school_scope(claims: &Claims, school_id: i64) -> Result<(), AppError> {
    if claims.role == "admin" || claims.role == "agent" || claims.role == "viewer" {
        return Ok(());
    }
    if claims.school_ids.contains(&school_id) {
        return Ok(());
    }
    Err(AppError::forbidden("Access to this school is denied"))
}

/// Returns `None` for unscoped roles (admin, agent, viewer).
/// Returns `Some(&school_ids)` for scoped roles (aom, faculty, head).
/// If a scoped role has no schools assigned, returns `Some(&[-1])`
/// so queries return zero rows.
pub fn scope_filter<'a>(claims: &'a Claims) -> Option<&'a [i64]> {
    if claims.role == "admin" || claims.role == "agent" || claims.role == "viewer" {
        None
    } else if claims.school_ids.is_empty() {
        Some(&[-1]) // impossible id → no rows
    } else {
        Some(&claims.school_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_with_role(role: &str) -> Claims {
        Claims {
            sub: "1".to_string(),
            username: "test".to_string(),
            display_name: "Test".to_string(),
            role: role.to_string(),
            school_ids: vec![1],
            exp: 9999999999,
        }
    }

    #[test]
    fn require_ticket_writer_allows_admin() {
        assert!(require_ticket_writer(&claims_with_role("admin")).is_ok());
    }

    #[test]
    fn require_ticket_writer_allows_agent() {
        assert!(require_ticket_writer(&claims_with_role("agent")).is_ok());
    }

    #[test]
    fn require_ticket_writer_allows_aom() {
        assert!(require_ticket_writer(&claims_with_role("aom")).is_ok());
    }

    #[test]
    fn require_ticket_writer_denies_viewer() {
        assert!(require_ticket_writer(&claims_with_role("viewer")).is_err());
    }

    #[test]
    fn require_ticket_writer_denies_faculty() {
        assert!(require_ticket_writer(&claims_with_role("faculty")).is_err());
    }

    #[test]
    fn require_ticket_writer_denies_head() {
        assert!(require_ticket_writer(&claims_with_role("head")).is_err());
    }

    #[test]
    fn viewer_cannot_delete_region() {
        assert!(require_admin(&claims_with_role("viewer")).is_err());
    }

    #[test]
    fn agent_cannot_delete_region() {
        assert!(require_admin(&claims_with_role("agent")).is_err());
    }

    #[test]
    fn aom_cannot_delete_region() {
        assert!(require_admin(&claims_with_role("aom")).is_err());
    }

    #[test]
    fn aom_can_access_admin_or_aom_route() {
        assert!(require_admin_or_aom(&claims_with_role("aom")).is_ok());
    }

    #[test]
    fn viewer_cannot_access_admin_or_aom_route() {
        assert!(require_admin_or_aom(&claims_with_role("viewer")).is_err());
    }

    #[test]
    fn aom_cannot_access_admin_only_route() {
        assert!(require_admin(&claims_with_role("aom")).is_err());
    }

    #[test]
    fn admin_can_access_admin_only_route() {
        assert!(require_admin(&claims_with_role("admin")).is_ok());
    }

    #[test]
    fn admin_can_pass_create_school_guard() {
        assert!(require_admin(&claims_with_role("admin")).is_ok());
    }

    #[test]
    fn aom_is_denied_create_school_guard() {
        assert!(require_admin(&claims_with_role("aom")).is_err());
    }

    #[test]
    fn aom_scope_allows_assigned_school() {
        let claims = Claims {
            sub: "1".to_string(),
            username: "aom".to_string(),
            display_name: "AOM".to_string(),
            role: "aom".to_string(),
            school_ids: vec![10, 20],
            exp: 9999999999,
        };
        assert!(enforce_school_scope(&claims, 10).is_ok());
        assert!(enforce_school_scope(&claims, 20).is_ok());
    }

    #[test]
    fn aom_scope_denies_unassigned_school() {
        let claims = Claims {
            sub: "1".to_string(),
            username: "aom".to_string(),
            display_name: "AOM".to_string(),
            role: "aom".to_string(),
            school_ids: vec![10],
            exp: 9999999999,
        };
        assert!(enforce_school_scope(&claims, 99).is_err());
    }

    #[test]
    fn revalidate_claims_reflects_current_db_role_and_schools() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        crate::db::initialize_db(&conn).expect("init schema");

        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash, is_active)
             VALUES ('alice', 'Alice', 'agent', 'hash', 1)",
            [],
        )
        .unwrap();
        let user_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO schools (name, region_id, is_dropped)
             VALUES ('Alpha School', 1, 0)",
            [],
        )
        .unwrap();
        let school_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO user_schools (user_id, school_id) VALUES (?1, ?2)",
            rusqlite::params![user_id, school_id],
        )
        .unwrap();

        let decoded = Claims {
            sub: user_id.to_string(),
            username: "stale".to_string(),
            display_name: "Stale".to_string(),
            role: "admin".to_string(),
            school_ids: vec![],
            exp: 9999999999,
        };

        let fresh = revalidate_claims(&decoded, &conn).unwrap();
        assert_eq!(fresh.role, "agent");
        assert_eq!(fresh.username, "alice");
        assert_eq!(fresh.display_name, "Alice");
        assert_eq!(fresh.school_ids, vec![school_id]);
        assert_eq!(fresh.exp, decoded.exp);
    }

    #[test]
    fn revalidate_claims_rejects_inactive_user() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        crate::db::initialize_db(&conn).expect("init schema");

        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash, is_active)
             VALUES ('bob', 'Bob', 'admin', 'hash', 0)",
            [],
        )
        .unwrap();
        let user_id = conn.last_insert_rowid();

        let decoded = Claims {
            sub: user_id.to_string(),
            username: "bob".to_string(),
            display_name: "Bob".to_string(),
            role: "admin".to_string(),
            school_ids: vec![],
            exp: 9999999999,
        };

        assert!(revalidate_claims(&decoded, &conn).is_err());
    }

    #[test]
    fn revalidate_claims_rejects_missing_user() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        crate::db::initialize_db(&conn).expect("init schema");

        let decoded = Claims {
            sub: "9999".to_string(),
            username: "ghost".to_string(),
            display_name: "Ghost".to_string(),
            role: "admin".to_string(),
            school_ids: vec![],
            exp: 9999999999,
        };

        assert!(revalidate_claims(&decoded, &conn).is_err());
    }

    #[test]
    fn revalidate_claims_reflects_role_change() {
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        crate::db::initialize_db(&conn).expect("init schema");

        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash, is_active)
             VALUES ('carol', 'Carol', 'viewer', 'hash', 1)",
            [],
        )
        .unwrap();
        let user_id = conn.last_insert_rowid();

        let decoded = Claims {
            sub: user_id.to_string(),
            username: "carol".to_string(),
            display_name: "Carol".to_string(),
            role: "agent".to_string(),
            school_ids: vec![],
            exp: 9999999999,
        };

        let fresh = revalidate_claims(&decoded, &conn).unwrap();
        assert_eq!(fresh.role, "viewer");
    }
}
