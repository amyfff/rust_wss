use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub body: Option<String>,
    pub sent_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
pub struct CreateEmail {
    #[validate(email)]
    pub sender: String,
    #[validate(email)]
    pub recipient: String,
    #[validate(length(min = 1))]
    pub subject: String,
    pub body: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateEmail {
    pub subject: Option<String>,
    pub body: Option<String>,
}