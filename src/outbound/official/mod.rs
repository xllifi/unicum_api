use std::thread::sleep;

use async_trait::async_trait;
use log::info;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::{
    contracts::Contracts,
    entities::{InternalError, MachineId, Sales, SetStockTarget, State, Stock},
};

mod get_sales;
mod get_state;
mod get_stock;
mod login;
mod set_stock;
mod utils;

mod internal;

#[derive(Debug, Clone)]
pub struct UnicumApi {
    http_client: reqwest::Client,

    username: String,
    password: String,

    token: String,
    token_updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    token: String,
}

#[derive(Debug, thiserror::Error, derive_more::From)]
pub(crate) enum ModuleError {
    #[error("Retry the login request")]
    RetryLogin,

    #[error("Failed to parse: {cause}")]
    ParseError { cause: String },

    #[from(reqwest::Error)]
    #[error("Reqwest error: {0}")]
    ReqwestError(reqwest::Error),

    #[from(serde_json::Error)]
    #[error("Reqwest error: {0}")]
    JsonError(serde_json::Error),
}

impl From<ModuleError> for InternalError {
    fn from(value: ModuleError) -> Self {
        use ModuleError::*;
        match value {
            ReqwestError(e) => Self::NetworkError {
                cause: e.to_string(),
            },
            JsonError(e) => Self::ParseError {
                cause: e.to_string(),
            },
            RetryLogin => Self::NetworkError {
                cause: "Login busy, try again".into(),
            },
            ParseError { cause } => Self::ParseError { cause },
        }
    }
}

impl UnicumApi {
    pub fn new(username: String, password: String) -> Self {
        Self {
            http_client: Client::new(),
            username,
            password,
            token: "".into(),
            token_updated_at: OffsetDateTime::from_unix_timestamp(0).unwrap(),
        }
    }
    fn update_token(&mut self, new_token: String) {
        info!("Setting new token: {}", new_token);
        self.token = new_token;
        self.token_updated_at = OffsetDateTime::now_utc();
    }
    async fn token(&mut self) -> Result<&str, ModuleError> {
        let token_expires_at = self.token_updated_at.saturating_add(Duration::minutes(39));
        if OffsetDateTime::now_utc() > token_expires_at {
            info!("Token expired, need to re-login");
            loop {
                let new_token = self.try_login().await;
                match new_token {
                    Err(ModuleError::RetryLogin) => {
                        info!("API busy, retrying in 200ms");
                        sleep(std::time::Duration::from_millis(200));
                    }
                    Ok(token) => {
                        self.update_token(token);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(&self.token)
    }
}

#[async_trait]
impl Contracts for UnicumApi {
    async fn get_state(&mut self, machine_id: MachineId) -> Result<State, InternalError> {
        self.get_state_internal(machine_id)
            .await
            .map_err(|e| e.into())
    }

    async fn get_sales(
        &mut self,
        machine_id: MachineId,
        since: i64,
        until: i64,
    ) -> Result<Sales, InternalError> {
        let since_date = OffsetDateTime::from_unix_timestamp(since).unwrap().date();
        let until_date = OffsetDateTime::from_unix_timestamp(until).unwrap().date();
        self.get_sales(machine_id, since_date, until_date)
            .await
            .map_err(|e| e.into())
    }

    async fn get_stock(&mut self, machine_id: MachineId) -> Result<Stock, InternalError> {
        self.get_stock(machine_id).await.map_err(|e| e.into())
    }
    async fn set_stock(
        &mut self,
        machine_id: MachineId,
        stock: Stock,
        target: SetStockTarget,
    ) -> Result<(), InternalError> {
        self.set_stock(machine_id, stock, target)
            .await
            .map_err(|e| e.into())
    }
}

pub trait AddTokenCookie {
    fn add_token_cookie(self, token: String) -> Self;
}

impl AddTokenCookie for RequestBuilder {
    fn add_token_cookie(self, token: String) -> Self {
        self.header("Cookie", format!("nvmc_login={token}; nvmc_root=/n/"))
    }
}
