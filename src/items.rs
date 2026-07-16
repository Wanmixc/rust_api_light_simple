use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, error::ApiResponse, AppState};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemPayload {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

impl ItemPayload {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.trim().is_empty() {
            return Err(ValidationError::NameRequired);
        }

        Ok(())
    }

    fn clean(self) -> Self {
        Self {
            name: self.name.trim().to_string(),
            description: self
                .description
                .and_then(|value| non_empty_trimmed(value.as_str())),
            is_active: self.is_active,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("name is required")]
    NameRequired,
}

pub async fn list_items(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Item>>>, ApiError> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        select id, name, description, is_active, created_at, updated_at
        from items
        order by created_at desc
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(ApiResponse::ok(items)))
}

pub async fn create_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ItemPayload>,
) -> Result<(StatusCode, Json<ApiResponse<Item>>), ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();
    let id = Uuid::new_v4();

    let item = sqlx::query_as::<_, Item>(
        r#"
        insert into items (id, name, description, is_active)
        values ($1, $2, $3, $4)
        returning id, name, description, is_active, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.is_active.unwrap_or(true))
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::created(item))))
}

pub async fn get_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Item>>, ApiError> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        select id, name, description, is_active, created_at, updated_at
        from items
        where id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(ApiResponse::ok(item)))
}

pub async fn update_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ItemPayload>,
) -> Result<Json<ApiResponse<Item>>, ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();

    let item = sqlx::query_as::<_, Item>(
        r#"
        update items
        set name = $2,
            description = $3,
            is_active = $4,
            updated_at = now()
        where id = $1
        returning id, name, description, is_active, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.is_active.unwrap_or(true))
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(ApiResponse::ok(item)))
}

pub async fn delete_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let result = sqlx::query("delete from items where id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(ApiResponse::ok(())))
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}