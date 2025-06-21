use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("kesalahan internal server")]
    InternalServerError(#[from] anyhow::Error),
    #[error("kesalahan database: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("input tidak valid: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("kredensial salah")]
    InvalidCredentials,
    #[error("token tidak valid atau kedaluwarsa")]
    InvalidToken,
    #[error("diperlukan otentikasi")]
    Unauthorized,
    #[error("tidak ditemukan: {0}")]
    NotFound(String),
    #[error("konflik: {0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Kesalahan Internal Server".to_string())
            }
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Kesalahan Database".to_string())
            }
            AppError::ValidationError(e) => (StatusCode::BAD_REQUEST, format!("Input tidak valid: {}", e)),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Kredensial salah".to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Token tidak valid".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Diperlukan otentikasi".to_string()),
            AppError::NotFound(entity) => (StatusCode::NOT_FOUND, format!("{} tidak ditemukan", entity)),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}