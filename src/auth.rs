use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub username: String,
}

impl From<UserRow> for UserPublic {
    fn from(u: UserRow) -> Self {
        Self {
            id: u.id,
            username: u.username,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}

/// Extractor that validates the JWT and exposes the authenticated user id.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn register(
    State(state): State<super::AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    validate_credentials(&payload.username, &payload.password)?;

    let password_hash = hash_password(&payload.password)?;

    let id = Uuid::new_v4();

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        insert into users (id, username, password_hash)
        values ($1, $2, $3)
        on conflict (username) do nothing
        returning id, username, password_hash, created_at
        "#,
    )
    .bind(id)
    .bind(payload.username.trim())
    .bind(password_hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::BadRequest("username already taken".into()))?;

    let token = create_token(user.id, &state.jwt_secret)?;
    let user_public = UserPublic::from(user);

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user_public,
        }),
    ))
}

pub async fn login(
    State(state): State<super::AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, ApiError> {
    let username = payload.username.trim().to_lowercase();

    let user = sqlx::query_as::<_, UserRow>(
        "select id, username, password_hash, created_at from users where username = $1",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::Unauthorized("invalid username or password".into()))?;

    verify_password(&payload.password, &user.password_hash)
        .map_err(|_| ApiError::Unauthorized("invalid username or password".into()))?;

    let token = create_token(user.id, &state.jwt_secret)?;
    let user_public = UserPublic::from(user);

    Ok(Json(AuthResponse {
        token,
        user: user_public,
    }))
}

pub async fn me(
    auth: AuthUser,
    State(state): State<super::AppState>,
) -> Result<Json<UserPublic>, ApiError> {
    let user = sqlx::query_as::<_, UserRow>(
        "select id, username, password_hash, created_at from users where id = $1",
    )
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(UserPublic::from(user)))
}

// ---------------------------------------------------------------------------
// AuthUser extractor
// ---------------------------------------------------------------------------

impl FromRequestParts<super::AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &super::AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::Unauthorized("missing or invalid Authorization header".into()))?;

        let token_data = decode::<Claims>(
            header,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            let msg = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "token expired",
                _ => "invalid token",
            };
            ApiError::Unauthorized(msg.into())
        })?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_credentials(username: &str, password: &str) -> Result<(), ApiError> {
    let username = username.trim();
    if username.is_empty() {
        return Err(ApiError::BadRequest("username is required".into()));
    }
    if username.len() < 3 {
        return Err(ApiError::BadRequest(
            "username must be at least 3 characters".into(),
        ));
    }
    if password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::BadRequest(format!("password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| ApiError::Unauthorized("invalid credentials".into()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized("invalid credentials".into()))
}

fn create_token(user_id: Uuid, secret: &str) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::BadRequest("failed to create token".into()))
}