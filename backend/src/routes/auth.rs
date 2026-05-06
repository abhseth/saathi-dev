use axum::{extract::State, Extension, Json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::{
    auth::issue_token,
    error::AppError,
    models::{AppState, Claims, CurrentUser, LoginInput, LoginResponse},
    repositories,
};

static LOGIN_ATTEMPTS: LazyLock<Mutex<HashMap<String, (u32, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60); // 15 minutes

fn check_rate_limit(username: &str) -> Result<(), AppError> {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();

    // Clean up expired entries
    attempts.retain(|_, (_, instant)| now.duration_since(*instant) < LOCKOUT_DURATION);

    if let Some((count, instant)) = attempts.get(username) {
        if *count >= MAX_ATTEMPTS && now.duration_since(*instant) < LOCKOUT_DURATION {
            return Err(AppError::unauthorized("Invalid username or password"));
        }
    }
    Ok(())
}

fn record_failed_attempt(username: &str) {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();
    let entry = attempts.entry(username.to_string()).or_insert((0, now));
    entry.0 += 1;
    entry.1 = now;
}

fn clear_attempts(username: &str) {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap_or_else(|e| e.into_inner());
    attempts.remove(username);
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(input): Json<LoginInput>,
) -> Result<Json<LoginResponse>, AppError> {
    let username = input.username.trim().to_string();
    check_rate_limit(&username)?;

    let conn = state
        .db
        .get()
        .map_err(|e| AppError::internal(format!("DB pool error: {e}")))?;

    let user = match repositories::authenticate_user(&conn, &username, &input.password) {
        Ok(u) => u,
        Err(e) => {
            record_failed_attempt(&username);
            return Err(AppError::unauthorized(e));
        }
    };

    clear_attempts(&username);

    let current = CurrentUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
        school_ids: user.school_ids,
    };

    let token = issue_token(&state, &current)?;

    Ok(Json(LoginResponse {
        token,
        user: current,
    }))
}

pub async fn me(Extension(claims): Extension<Claims>) -> Json<CurrentUser> {
    Json(CurrentUser {
        id: claims.sub.parse().unwrap_or(0),
        username: claims.username,
        display_name: claims.display_name,
        role: claims.role,
        school_ids: claims.school_ids,
    })
}
