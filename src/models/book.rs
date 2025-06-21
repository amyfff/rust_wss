use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub publication_year: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
pub struct CreateBook {
    #[validate(length(min = 1, message = "Judul tidak boleh kosong"))]
    pub title: String,
    #[validate(length(min = 1, message = "Penulis tidak boleh kosong"))]
    pub author: String,
    pub publication_year: Option<i32>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateBook {
    #[validate(length(min = 1, message = "Judul tidak boleh kosong"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "Penulis tidak boleh kosong"))]
    pub author: Option<String>,
    pub publication_year: Option<i32>,
}