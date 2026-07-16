use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Standard response envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ApiStatus {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: ApiStatus,
    pub description: String,
    pub data: T,
    pub errors: Option<Vec<String>>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: ApiStatus {
                code: 200,
                message: "success".into(),
            },
            description: "OK".into(),
            data,
            errors: None,
        }
    }

    pub fn created(data: T) -> Self {
        Self {
            status: ApiStatus {
                code: 201,
                message: "created".into(),
            },
            description: "Created".into(),
            data,
            errors: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("item not found")]
    NotFound,
    #[error("user not found")]
    UserNotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound | Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.status_code();
        let message = self.to_string();

        let body = ApiResponse::<()> {
            status: ApiStatus {
                code: code.as_u16(),
                message: code.canonical_reason().unwrap_or("error").to_lowercase(),
            },
            description: message.clone(),
            data: (),
            errors: Some(vec![message]),
        };

        (code, Json(body)).into_response()
    }
}
