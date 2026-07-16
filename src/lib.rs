pub mod config;
pub mod error;
pub mod items;
pub mod schema;

use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::items::{create_item, delete_item, get_item, list_items, update_item};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn create_router(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/health", get(health))
        .route("/api/items", get(list_items).post(create_item))
        .route(
            "/api/items/{id}",
            get(get_item).put(update_item).delete(delete_item),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
