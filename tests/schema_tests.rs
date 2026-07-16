use rust_api_light_simple::schema::CREATE_ITEMS_TABLE_SQL;

#[test]
fn items_schema_is_idempotent() {
    assert!(CREATE_ITEMS_TABLE_SQL.contains("create table if not exists items"));
}
