use sqlx::{ sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs::read_to_string, path::Path};

use axum::extract::FromRef;
use leptos::LeptosOptions;
use serde::Deserialize;

use crate::auth::AuthBackend;

/// A... normal number of connections?
fn default_max_connections() -> u32 {
    10
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default = "LeptosOptions::default")]
    pub leptos: LeptosOptions,
}

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub pool: SqlitePool,
    pub auth: AuthBackend,
}

// Must be implemented to be able to use this struct as the router state.
// It REALLY wants leptops options
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.config.leptos.to_owned()
    }
}


/// Defines the struct that holds the global state.
/// The server configuration is part of the state.
impl AppState {
    pub fn new(path: &Path) -> Result<Self, String> {
        // Read user config
        let raw_config = match read_to_string(path) {
            Ok(s) => s,
            Err(err) => {
                return Err(format!("Error reading {path:?}: {err}"));
            }
        };

        let config: Config = match toml::from_str(&raw_config) {
            Ok(c) => c,
            Err(err) => {
                return Err(format!("Error parsing {}: {}", path.to_string_lossy(), err));
            }
        };

        // Connect to database
        sqlx::any::install_default_drivers();
        let pool: SqlitePool = match SqlitePoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect_lazy(&config.database.url)
        {
            Ok(p) => p,
            Err(err) => {
                return Err(format!("Could not create database pool: {}", err));
            }
        };

        let auth = AuthBackend { pool: pool.clone() };

        Ok(AppState { config, pool, auth })
    }
}
