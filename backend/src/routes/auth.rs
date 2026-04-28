use axum::{extract::State, Extension, Json};
use std::sync::Arc;

use crate::{
    auth::issue_token,
    error::AppError,
    models::{AppState, Claims, CurrentUser, LoginInput, LoginResponse},
    repositories,
};

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(input): Json<LoginInput>,
) -> Result<Json<LoginResponse>, AppError> {
    let conn = state.db.lock().map_err(|_| AppError::internal("DB lock poisoned"))?;

    let user = repositories::authenticate_user(&conn, &input.username, &input.password)
        .map_err(|e| AppError::unauthorized(e))?;

    let current = CurrentUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    };

    let token = issue_token(&state, &current)?;

    Ok(Json(LoginResponse { token, user: current }))
}

pub async fn me(
    Extension(claims): Extension<Claims>,
) -> Json<CurrentUser> {
    Json(CurrentUser {
        id: claims.sub.parse().unwrap_or(0),
        username: claims.username,
        display_name: claims.display_name,
        role: claims.role,
    })
}
