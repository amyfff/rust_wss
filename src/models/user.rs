use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

// PERBAIKAN DI SINI
#[derive(Deserialize, Validate)] // <-- Ditambahkan
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))] // <-- Diperbaiki
    pub password: String,
}