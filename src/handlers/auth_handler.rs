use crate::{auth::{hash_password, verify_password, Claims}, error::AppError, models::user::{CreateUser, LoginRequest, User}, AppState};
use axum::{extract::State, Json};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[axum::debug_handler]
pub async fn signup(State(state): State<Arc<AppState>>, Json(payload): Json<CreateUser>) -> Result<Json<User>, AppError> {
    payload.validate()?;
    let password_hash = hash_password(&payload.password)?;
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email, password_hash, role) VALUES ($1, $2, $3, 'user') RETURNING id, username, email, password_hash, role",
        payload.username,
        payload.email,
        password_hash
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.is_unique_violation() {
                return AppError::Conflict("Username atau email sudah digunakan".to_string());
            }
        }
        AppError::DatabaseError(e)
    })?;
    Ok(Json(user))
}

#[axum::debug_handler]
pub async fn signin(State(state): State<Arc<AppState>>, Json(payload): Json<LoginRequest>) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, role FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::InvalidCredentials)?;

    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }
    let token = Claims::new(user.id.to_string(), user.role).encode()?;
    Ok(Json(json!({ "token": token })))
}