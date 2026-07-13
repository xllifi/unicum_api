use std::{fs, path::PathBuf};

use thiserror::Error;

use crate::APP_NAME;

const ENV_PREFIX: &str = "UNICUM_API";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Please specify {description} with {name} env variable.")]
    Missing {
        name: String,
        description: &'static str,
    },

    #[error("{name} env variable's value must be valid Unicode.")]
    NotUnicode { name: String },

    #[error("Failed to create state directory {path}: {source}")]
    CreateStateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

fn env_name(suffix: &str) -> String {
    format!("{ENV_PREFIX}_{suffix}")
}

fn required_env(suffix: &str, description: &'static str) -> Result<String, ConfigError> {
    let name = env_name(suffix);
    let value = std::env::var_os(&name).ok_or_else(|| ConfigError::Missing {
        name: name.clone(),
        description,
    })?;
    value
        .into_string()
        .map_err(|_| ConfigError::NotUnicode { name })
}

pub fn postgres_url() -> Result<String, ConfigError> {
    required_env("POSTGRES_URL", "a Postgres URL")
}

pub fn password_salt() -> Result<String, ConfigError> {
    required_env("SALT_B64", "a password salt")
}

pub fn unicum_username() -> Result<String, ConfigError> {
    required_env("UNICUM_USERNAME", "a Unicum username")
}

pub fn unicum_password() -> Result<String, ConfigError> {
    required_env("UNICUM_PASSWORD", "a Unicum password")
}

#[cfg(target_os = "windows")]
fn default_state_dir() -> PathBuf {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join(APP_NAME)
}

#[cfg(target_os = "macos")]
fn default_state_dir() -> PathBuf {
    PathBuf::from("/Library/Application Support").join(APP_NAME)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn default_state_dir() -> PathBuf {
    PathBuf::from("/var/lib").join(APP_NAME)
}

pub fn initialize_state_dir() -> Result<PathBuf, ConfigError> {
    let state_dir = std::env::var_os(env_name("STATE_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(default_state_dir);

    fs::create_dir_all(&state_dir).map_err(|io_err| ConfigError::CreateStateDirectory {
        path: state_dir.clone(),
        source: io_err,
    })?;

    Ok(state_dir)
}
