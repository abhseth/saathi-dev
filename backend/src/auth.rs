use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;

use crate::{error::AppError, models::{AppState, Claims, CurrentUser}};

const TOKEN_EXPIRY_SECS: usize = 8 * 60 * 60; // 8 hours

pub fn issue_token(state: &AppState, user: &CurrentUser) -> Result<String, AppError> {
    let exp = (chrono::Utc::now().timestamp() as usize) + TOKEN_EXPIRY_SECS;
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        role: user.role.clone(),
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

/// Axum middleware: extract Bearer token, decode it, insert Claims into request extensions.
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

    let claims = decode_token(&state, token)?;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Helper used inside route handlers: get claims or return 401.
pub fn current_claims(request: &Request) -> Result<&Claims, AppError> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::unauthorized("Not authenticated"))
}
