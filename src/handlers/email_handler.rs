use crate::{auth::Claims, error::AppError, models::email::{CreateEmail, Email, UpdateEmail}, ws, AppState};
use axum::{extract::{Path, State}, Extension, Json}; // Added Extension
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[axum::debug_handler]
pub async fn get_all_emails(State(state): State<Arc<AppState>>, _claims: Extension<Claims>) -> Result<Json<Vec<Email>>, AppError> {
    let emails = sqlx::query_as!(Email, "SELECT * FROM emails ORDER BY sent_at DESC")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(Json(emails))
}

#[axum::debug_handler]
pub async fn create_email(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Json(payload): Json<CreateEmail>) -> Result<Json<Email>, AppError> {
    payload.validate()?;
    let email = sqlx::query_as!(
        Email,
        "INSERT INTO emails (sender, recipient, subject, body) VALUES ($1, $2, $3, $4) RETURNING *",
        payload.sender,
        payload.recipient,
        payload.subject,
        payload.body
    )
    .fetch_one(&state.db_pool)
    .await?;
    ws::broadcast_event(ws::WsEvent::EmailCreated(email.clone()));
    Ok(Json(email))
}

#[axum::debug_handler]
pub async fn get_email_by_id(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>) -> Result<Json<Email>, AppError> {
    let email = sqlx::query_as!(Email, "SELECT * FROM emails WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Email".to_string()))?;
    Ok(Json(email))
}

#[axum::debug_handler]
pub async fn update_email(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>, Json(payload): Json<UpdateEmail>) -> Result<Json<Email>, AppError> {
    payload.validate()?;
    let email = sqlx::query_as!(Email, "SELECT * FROM emails WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Email".to_string()))?;

    let subject = payload.subject.unwrap_or(email.subject);
    let body = payload.body.or(email.body);

    let updated_email = sqlx::query_as!(
        Email,
        "UPDATE emails SET subject = $1, body = $2 WHERE id = $3 RETURNING *",
        subject,
        body,
        id
    )
    .fetch_one(&state.db_pool)
    .await?;
    ws::broadcast_event(ws::WsEvent::EmailUpdated(updated_email.clone()));
    Ok(Json(updated_email))
}

#[axum::debug_handler]
pub async fn delete_email(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM emails WHERE id = $1", id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Email".to_string()));
    }
    ws::broadcast_event(ws::WsEvent::EmailDeleted(id));
    Ok(())
}