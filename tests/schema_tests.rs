use rust_api_light_simple::schema::{CREATE_ITEMS_TABLE_SQL, CREATE_USERS_TABLE_SQL};

#[test]
fn items_schema_is_idempotent() {
    assert!(CREATE_ITEMS_TABLE_SQL.contains("create table if not exists items"));
}

#[test]
fn users_schema_recreates_table() {
    assert!(CREATE_USERS_TABLE_SQL.contains("drop table if exists users"));
    assert!(CREATE_USERS_TABLE_SQL.contains("create table users"));
}
