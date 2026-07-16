use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use rust_api_light_simple::create_router;

async fn setup() -> (PgPool, Router) {
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/test".into());

    let pool = PgPool::connect(&db_url)
        .await
        .expect("failed to connect to test database");

    rust_api_light_simple::schema::ensure_schema(&pool)
        .await
        .expect("schema setup failed");

    let jwt_secret = "test-secret-for-integration-tests".to_string();
    let router = create_router(pool.clone(), jwt_secret);

    (pool, router)
}

async fn register(router: &Router, username: &str, password: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": username, "password": password}).to_string(),
        ))
        .unwrap();

    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or_default();
    (status, json)
}

async fn login(router: &Router, username: &str, password: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": username, "password": password}).to_string(),
        ))
        .unwrap();

    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or_default();
    (status, json)
}

#[tokio::test]
#[ignore = "requires a running PostgreSQL"]
async fn register_creates_user_and_returns_token() {
    let (_pool, router) = setup().await;

    let (status, body) = register(&router, "testuser", "password123").await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["username"], "testuser");
}

#[tokio::test]
#[ignore = "requires a running PostgreSQL"]
async fn login_with_correct_credentials_returns_token() {
    let (_pool, router) = setup().await;

    // Register first
    register(&router, "logintest", "password123").await;

    // Then login
    let (status, body) = login(&router, "logintest", "password123").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
}

#[tokio::test]
#[ignore = "requires a running PostgreSQL"]
async fn register_duplicate_username_is_rejected() {
    let (_pool, router) = setup().await;

    register(&router, "dupe", "password123").await;
    let (status, _body) = register(&router, "dupe", "differentpass").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires a running PostgreSQL"]
async fn protected_route_without_token_returns_401() {
    let (_pool, router) = setup().await;

    let req = Request::builder()
        .method(http::Method::GET)
        .uri("/api/items")
        .body(Body::empty())
        .unwrap();

    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires a running PostgreSQL"]
async fn can_create_item_with_valid_token() {
    let (_pool, router) = setup().await;

    let (_status, body) = register(&router, "itemuser", "password123").await;
    let token = body["token"].as_str().unwrap();

    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/api/items")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({"name": "My Item", "description": "Test"}).to_string(),
        ))
        .unwrap();

    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}