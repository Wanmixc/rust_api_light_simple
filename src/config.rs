use std::{collections::HashMap, env};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub max_db_connections: u32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Self::from_pairs(env::vars())
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values: HashMap<String, String> = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        let database_url = match values.get("DATABASE_URL") {
            Some(value) if !value.trim().is_empty() => value.clone(),
            _ => {
                let host = required(&values, "DATABASE_HOST")?;
                let user = required(&values, "DATABASE_USER")?;
                let password = required(&values, "DATABASE_PASSWORD")?;
                let name = required(&values, "DATABASE_NAME")?;
                let ssl_mode = values
                    .get("DATABASE_SSL_MODE")
                    .map(String::as_str)
                    .unwrap_or("require");

                build_database_url_with_ssl(host, user, password, name, ssl_mode)
            }
        };

        let host = values
            .get("HOST")
            .cloned()
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let port = parse_or_default(values.get("PORT"), 3010, "PORT")?;
        let max_db_connections =
            parse_or_default(values.get("MAX_DB_CONNECTIONS"), 2, "MAX_DB_CONNECTIONS")?;

        Ok(Self {
            database_url,
            host,
            port,
            max_db_connections,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn build_database_url(host: &str, user: &str, password: &str, name: &str) -> String {
    build_database_url_with_ssl(host, user, password, name, "require")
}

fn build_database_url_with_ssl(
    host: &str,
    user: &str,
    password: &str,
    name: &str,
    ssl_mode: &str,
) -> String {
    format!(
        "postgres://{}:{}@{}:5432/{}?sslmode={}",
        urlencoding::encode(user),
        urlencoding::encode(password),
        host,
        urlencoding::encode(name),
        ssl_mode
    )
}

fn required<'a>(values: &'a HashMap<String, String>, key: &str) -> Result<&'a str> {
    values
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("{key} is required"))
}

fn parse_or_default<T>(value: Option<&String>, default: T, key: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match value {
        Some(value) if !value.trim().is_empty() => value
            .parse::<T>()
            .with_context(|| format!("{key} must be valid")),
        _ => Ok(default),
    }
}
