use crate::error::AppError;
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap}, // Import HeaderMap
    middleware::Next,
    response::Response,
};
// NEW: Import from axum-extra
// use axum_extra::{
//     headers::{authorization::Bearer, Authorization},
//     TypedHeader,
// };
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// The Claims struct and its impl block remain unchanged...
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    // ... no changes here
    pub fn new(sub: String, role: String) -> Self {
        let iat = Utc::now();
        let exp_seconds: i64 = std::env::var("JWT_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);
        let exp = iat + Duration::seconds(exp_seconds);
        Self { sub, role, iat: iat.timestamp(), exp: exp.timestamp() }
    }

    pub fn encode(&self) -> Result<String, AppError> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET harus diatur");
        encode(&Header::default(), self, &EncodingKey::from_secret(secret.as_ref()))
            .map_err(|_| AppError::InternalServerError(anyhow::anyhow!("Gagal membuat token")))
    }
}

// The decode_token, hash_password, and verify_password functions remain unchanged...
pub fn decode_token(token: &str) -> Result<Claims, AppError> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET harus diatur");
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError(anyhow::anyhow!("Gagal hash password")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|_| AppError::InternalServerError(anyhow::anyhow!("Gagal verifikasi password")))
}


// MODIFIED: Updated middleware function signature and logic
pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, AppError> {
    let token = get_token_from_headers(request.headers())?;
    let claims = decode_token(&token)?;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

// MODIFIED: Changed function to be synchronous and accept &HeaderMap
fn get_token_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_owned())
        .ok_or(AppError::Unauthorized)
}


// MODIFIED: Updated to use the new helper function
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = get_token_from_headers(&parts.headers)?;
        let claims = decode_token(&token)?;
        Ok(claims)
    }
}