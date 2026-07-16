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

pub const CREATE_USERS_TABLE_SQL: &str = r#"
create table if not exists users (
    id uuid primary key,
    username text not null unique,
    password_hash text not null,
    created_at timestamptz not null default now()
)
"#;

/// Run after the initial create-tables to fix any schema drift (e.g. a table
/// created by an older version of the code that omitted a column).
pub const MIGRATE_USERS_TABLE_SQL: &str = r#"
do $$
begin
    if not exists (
        select 1 from information_schema.columns
        where table_name = 'users' and column_name = 'username'
    ) then
        alter table users add column username text;
        update users set username = 'unknown' where username is null;
        alter table users alter column username set not null;
        alter table users add constraint users_username_unique unique (username);
    end if;
end
$$
"#;

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(CREATE_ITEMS_TABLE_SQL).await?;
    pool.execute(CREATE_USERS_TABLE_SQL).await?;
    pool.execute(MIGRATE_USERS_TABLE_SQL).await?;
    Ok(())
}