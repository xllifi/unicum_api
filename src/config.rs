use thiserror::Error;

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