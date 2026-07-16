use rust_api_light_simple::schema::{CREATE_ITEMS_TABLE_SQL, MIGRATE_USERS_TABLE_SQL, CREATE_USERS_TABLE_SQL};

#[test]
fn items_schema_is_idempotent() {
    assert!(CREATE_ITEMS_TABLE_SQL.contains("create table if not exists items"));
}

#[test]
fn users_schema_is_idempotent() {
    assert!(CREATE_USERS_TABLE_SQL.contains("create table if not exists users"));
}

#[test]
fn users_migration_handles_missing_column() {
    assert!(MIGRATE_USERS_TABLE_SQL.contains("information_schema.columns"));
    assert!(MIGRATE_USERS_TABLE_SQL.contains("username"));
}
