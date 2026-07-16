use rust_api_light_simple::config::{build_database_url, AppConfig};

#[test]
fn uses_database_url_when_it_is_present() {
    let config = AppConfig::from_pairs([
        (
            "DATABASE_URL",
            "postgres://direct:secret@example.com:5432/app",
        ),
        ("DATABASE_HOST", "ignored.local"),
        ("DATABASE_USER", "ignored"),
        ("DATABASE_PASSWORD", "ignored"),
        ("DATABASE_NAME", "ignored"),
    ])
    .expect("config should load");

    assert_eq!(
        config.database_url,
        "postgres://direct:secret@example.com:5432/app"
    );
}

#[test]
fn builds_database_url_from_split_postgres_fields() {
    let url = build_database_url("db.example.com", "app user", "p@ss word", "app_db");

    assert_eq!(
        url,
        "postgres://app%20user:p%40ss%20word@db.example.com:5432/app_db?sslmode=require"
    );
}

#[test]
fn reads_host_and_port_with_defaults() {
    let config = AppConfig::from_pairs([
        (
            "DATABASE_URL",
            "postgres://user:password@localhost:5432/app",
        ),
        ("PORT", "8080"),
    ])
    .expect("config should load");

    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
}

#[test]
fn jwt_secret_defaults_when_not_set() {
    let config = AppConfig::from_pairs([
        (
            "DATABASE_URL",
            "postgres://user:password@localhost:5432/app",
        ),
    ])
    .expect("config should load");

    assert_eq!(config.jwt_secret, "dev-secret-change-in-production");
}

#[test]
fn jwt_secret_from_env() {
    let config = AppConfig::from_pairs([
        (
            "DATABASE_URL",
            "postgres://user:password@localhost:5432/app",
        ),
        ("JWT_SECRET", "my-secret-key"),
    ])
    .expect("config should load");

    assert_eq!(config.jwt_secret, "my-secret-key");
}
