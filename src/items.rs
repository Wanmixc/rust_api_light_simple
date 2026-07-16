use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, AppState};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemPayload {
    pub name: String,
    pub description: Option<String>,
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
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("name is required")]
    NameRequired,
}

pub async fn list_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, ApiError> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        select id, name, description, created_at, updated_at
        from items
        order by created_at desc
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(items))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<ItemPayload>,
) -> Result<(StatusCode, Json<Item>), ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();
    let id = Uuid::new_v4();

    let item = sqlx::query_as::<_, Item>(
        r#"
        insert into items (id, name, description)
        values ($1, $2, $3)
        returning id, name, description, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, ApiError> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        select id, name, description, created_at, updated_at
        from items
        where id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ItemPayload>,
) -> Result<Json<Item>, ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();

    let item = sqlx::query_as::<_, Item>(
        r#"
        update items
        set name = $2,
            description = $3,
            updated_at = now()
        where id = $1
        returning id, name, description, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("delete from items where id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}
