use crate::error::AppError;
use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    // Path import yang benar untuk axum 0.7+
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    middleware::Next,
    response::Response,
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
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

// Signature middleware ini sudah benar. Error sebelumnya adalah efek samping dari error lain.
#[axum::debug_handler]
pub async fn auth_middleware(next: Next, request: Request) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();
    let token = get_token_from_header(&parts)?;
    let claims = decode_token(&token)?;
    parts.extensions.insert(claims);
    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

fn get_token_from_header(parts: &Parts) -> Result<String, AppError> {
    TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
        .map(|TypedHeader(Authorization(bearer))| bearer.token().to_string())
        .map_err(|_| AppError::Unauthorized)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims where S: Send + Sync {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = get_token_from_header(parts)?;
        let claims = decode_token(&token)?;
        Ok(claims)
    }
}