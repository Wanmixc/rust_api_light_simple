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

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(CREATE_ITEMS_TABLE_SQL).await?;
    pool.execute(CREATE_USERS_TABLE_SQL).await?;
    // Fix any schema drift from older code
    fix_users_table(pool).await?;
    Ok(())
}

async fn fix_users_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Add username column if missing (old table created without it)
    pool.execute(
        r#"
        do $$
        begin
            if not exists (
                select 1 from information_schema.columns
                where table_name = 'users' and column_name = 'username'
            ) then
                alter table users add column username text;
            end if;
        end
        $$
        "#,
    )
    .await?;

    // Add unique constraint if missing
    pool.execute(
        r#"
        do $$
        begin
            if not exists (
                select 1 from information_schema.table_constraints
                where table_name = 'users' and constraint_name = 'users_username_key'
            ) then
                alter table users add constraint users_username_key unique (username);
            end if;
        end
        $$
        "#,
    )
    .await?;

    Ok(())
}