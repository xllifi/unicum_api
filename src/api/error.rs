use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::contracts::ContractError;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiError {
    BadRequest { message: String },
    Unauthorized,
    Forbidden,
    Internal,
    Upstream,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }
}

impl From<ApiError> for StatusCode {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::BadRequest { .. } => Self::BAD_REQUEST,
            ApiError::Unauthorized => Self::UNAUTHORIZED,
            ApiError::Forbidden => Self::FORBIDDEN,
            ApiError::Internal => Self::INTERNAL_SERVER_ERROR,
            ApiError::Upstream => Self::BAD_GATEWAY,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from(self.clone());
        (status, Json(self)).into_response()
    }
}

impl From<ContractError> for ApiError {
    fn from(error: ContractError) -> Self {
        log::error!("Unicum API error: {error:?}");
        Self::Upstream
    }
}
