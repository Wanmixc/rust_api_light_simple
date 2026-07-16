pub mod auth;
pub mod config;
pub mod error;
pub mod items;
pub mod schema;

use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::{
    auth::{login, me, register},
    items::{create_item, delete_item, get_item, list_items, update_item},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

pub fn create_router(pool: PgPool, jwt_secret: String) -> Router {
    let state = AppState { pool, jwt_secret };

    Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", axum::routing::post(register))
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/me", get(me))
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