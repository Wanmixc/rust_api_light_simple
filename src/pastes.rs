use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, error::ApiResponse, AppState};

pub const PASTE_ID_LEN: usize = 5;
const PASTE_ID_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const MAX_ID_ATTEMPTS: usize = 5;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Paste {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PastePayload {
    pub content: String,
}

impl PastePayload {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.content.trim().is_empty() {
            return Err(ValidationError::ContentRequired);
        }

        Ok(())
    }

    fn clean(self) -> Self {
        Self {
            content: self.content.trim().to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("content is required")]
    ContentRequired,
}

pub fn generate_paste_id() -> String {
    let uuid = Uuid::new_v4();

    uuid.as_bytes()
        .iter()
        .take(PASTE_ID_LEN)
        .map(|byte| {
            let index = usize::from(*byte) % PASTE_ID_CHARS.len();
            char::from(PASTE_ID_CHARS[index])
        })
        .collect()
}

pub async fn create_paste(
    State(state): State<AppState>,
    Json(payload): Json<PastePayload>,
) -> Result<(StatusCode, Json<ApiResponse<Paste>>), ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();

    for _ in 0..MAX_ID_ATTEMPTS {
        let id = generate_paste_id();
        let result = sqlx::query_as::<_, Paste>(
            r#"
            insert into pastes (id, content)
            values ($1, $2)
            returning id, content, created_at
            "#,
        )
        .bind(id)
        .bind(&payload.content)
        .fetch_one(&state.pool)
        .await;

        match result {
            Ok(paste) => return Ok((StatusCode::CREATED, Json(ApiResponse::created(paste)))),
            Err(error) if is_unique_violation(&error) => continue,
            Err(error) => return Err(error.into()),
        }
    }

    Err(ApiError::BadRequest(
        "failed to create unique paste id".into(),
    ))
}

pub async fn get_paste(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Paste>>, ApiError> {
    let paste =
        sqlx::query_as::<_, Paste>("select id, content, created_at from pastes where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::NotFound)?;

    Ok(Json(ApiResponse::ok(paste)))
}

pub async fn delete_paste(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let result = sqlx::query("delete from pastes where id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(ApiResponse::ok(())))
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_unique_violation())
}
