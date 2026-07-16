use sqlx::{Executor, PgPool};

pub const CREATE_ITEMS_TABLE_SQL: &str = r#"
create table if not exists items (
    id uuid primary key,
    name text not null check (length(trim(name)) > 0),
    description text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
)
"#;

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(CREATE_ITEMS_TABLE_SQL).await?;
    Ok(())
}
