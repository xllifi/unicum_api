use crate::contracts::ContractError;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ScraperError {
    #[error("Jagged table: {cause}")]
    JaggedTable { cause: String },

    #[error("Missing required element: {element}")]
    MissingElement { element: String },

    #[error("Missing required attribute: {attribute}")]
    MissingAttribute { attribute: String },

    #[error("Empty value for {what}")]
    EmptyValue { what: String },

    #[error("Invalid {field} value {value:?}")]
    InvalidValue { field: String, value: String },

    #[error("Invalid date {value:?}: {cause}")]
    InvalidDate { value: String, cause: String },

    #[error("Sales row for slot {slot:?} has {actual} date cells, expected {expected}")]
    SalesColumnCountMismatch {
        slot: String,
        expected: usize,
        actual: usize,
    },

    #[error("Invalid sales cell {value:?}: {cause}")]
    InvalidSalesCell { value: String, cause: String },

    #[error("Sales count cannot be zero")]
    ZeroSalesCount,

    #[error("Failed to parse: {cause}")]
    ParseError { cause: String },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ModuleError {
    #[error("Retry the login request")]
    RetryLogin,

    #[error("Official API returned HTTP status {status}")]
    HttpStatus { status: StatusCode },

    #[error("Scraper error: {0}")]
    Scraper(#[from] ScraperError),

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<ModuleError> for ContractError {
    fn from(error: ModuleError) -> Self {
        match error {
            ModuleError::HttpStatus { status } => Self::Network {
                cause: format!("Official API returned HTTP status {status}"),
            },
            ModuleError::ReqwestError(error) => Self::Network {
                cause: error.to_string(),
            },
            ModuleError::JsonError(error) => Self::Parse {
                cause: error.to_string(),
            },
            ModuleError::RetryLogin => Self::Network {
                cause: "Login busy, try again".into(),
            },
            ModuleError::Scraper(error) => Self::Parse {
                cause: error.to_string(),
            },
        }
    }
}
