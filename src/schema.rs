use sqlx::{Executor, PgPool};

pub const CREATE_ITEMS_TABLE_SQL: &str = r#"
create table if not exists items (
    id uuid primary key,
    name text not null check (length(trim(name)) > 0),
    description text,
    is_active boolean not null default true,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    created_by uuid references users(id),
    created_date timestamptz,
    updated_by uuid references users(id),
    updated_date timestamptz,
    inactivated_by uuid references users(id),
    inactivated_date timestamptz
)
"#;

pub const CREATE_USERS_TABLE_SQL: &str = r#"
create table if not exists users (
    id uuid primary key,
    username text not null unique,
    password_hash text not null,
    is_active boolean not null default true,
    created_at timestamptz not null default now()
)
"#;

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute(CREATE_ITEMS_TABLE_SQL).await?;
    pool.execute(CREATE_USERS_TABLE_SQL).await?;
    fix_missing_columns(pool).await?;
    Ok(())
}

async fn fix_missing_columns(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Add columns that may be missing from older table versions
    let migrations: &[(&str, &str, &str)] = &[
        ("users", "username", "text"),
        ("users", "is_active", "boolean not null default true"),
        ("items", "is_active", "boolean not null default true"),
        ("items", "created_by", "uuid"),
        ("items", "created_date", "timestamptz"),
        ("items", "updated_by", "uuid"),
        ("items", "updated_date", "timestamptz"),
        ("items", "inactivated_by", "uuid"),
        ("items", "inactivated_date", "timestamptz"),
    ];

    for (table, column, col_type) in migrations {
        let sql = format!(
            r#"
            do $$
            begin
                if not exists (
                    select 1 from information_schema.columns
                    where table_name = '{table}' and column_name = '{column}'
                ) then
                    alter table {table} add column {column} {col_type};
                end if;
            end
            $$
            "#
        );
        pool.execute(sql.as_str()).await?;
    }

    // Add unique constraint on users.username if missing
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