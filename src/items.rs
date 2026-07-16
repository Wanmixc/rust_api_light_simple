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
    pub created_by: Option<Uuid>,
    pub created_date: Option<DateTime<Utc>>,
    pub updated_by: Option<Uuid>,
    pub updated_date: Option<DateTime<Utc>>,
    pub inactivated_by: Option<Uuid>,
    pub inactivated_date: Option<DateTime<Utc>>,
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

// Shared column list so we don't repeat ourselves.
const ITEM_COLUMNS: &str = "\
    id, name, description, is_active, \
    created_at, updated_at, \
    created_by, created_date, \
    updated_by, updated_date, \
    inactivated_by, inactivated_date";

pub async fn list_items(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Item>>>, ApiError> {
    let items = sqlx::query_as::<_, Item>(&format!(
        "select {ITEM_COLUMNS} from items order by created_at desc"
    ))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(ApiResponse::ok(items)))
}

pub async fn create_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ItemPayload>,
) -> Result<(StatusCode, Json<ApiResponse<Item>>), ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let item = sqlx::query_as::<_, Item>(&format!(
        r#"
        insert into items (id, name, description, is_active,
                           created_by, created_date)
        values ($1, $2, $3, $4, $5, $6)
        returning {ITEM_COLUMNS}
        "#,
    ))
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.is_active.unwrap_or(true))
    .bind(auth.user_id)
    .bind(now)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::created(item))))
}

pub async fn get_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Item>>, ApiError> {
    let item = sqlx::query_as::<_, Item>(&format!(
        "select {ITEM_COLUMNS} from items where id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(ApiResponse::ok(item)))
}

pub async fn update_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ItemPayload>,
) -> Result<Json<ApiResponse<Item>>, ApiError> {
    payload
        .validate()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload = payload.clean();
    let now = Utc::now();
    let new_is_active = payload.is_active.unwrap_or(true);

    // Fetch current is_active to decide whether to set inactivated_* fields.
    let current: Option<(bool,)> =
        sqlx::query_as("select is_active from items where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;

    let (current_is_active,) = current.ok_or(ApiError::NotFound)?;

    let item = sqlx::query_as::<_, Item>(&format!(
        r#"
        update items
        set name = $2,
            description = $3,
            is_active = $4,
            updated_at = now(),
            updated_by = $5,
            updated_date = $6,
            inactivated_by = case when $7 and not $4 then $5 else inactivated_by end,
            inactivated_date = case when $7 and not $4 then $6 else inactivated_date end
        where id = $1
        returning {ITEM_COLUMNS}
        "#,
    ))
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(new_is_active)
    .bind(auth.user_id)
    .bind(now)
    .bind(current_is_active) // true = was active before this update
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