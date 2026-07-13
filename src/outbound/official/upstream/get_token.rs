use log::{error, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::outbound::official::{ModuleError, RequestBuilderExt, UnicumApi, upstream::LOGIN_ROUTE};

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest<'a> {
    #[serde(rename = "login")]
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

impl UnicumApi {
    pub(in crate::outbound::official) async fn try_login(&self) -> Result<String, ModuleError> {
        let req = LoginRequest {
            username: &self.username,
            password: &self.password,
        };
        info!("Authenticating with Unicum as {}", req.username);
        let response = self
            .http_client
            .post(LOGIN_ROUTE)
            .json(&req)
            .send_checked()
            .await;

        if let Err(ModuleError::HttpStatus { status }) = response.as_ref() {
            error!("(UnicumApi::try_login) Error with status: {status:?}");
            if *status == StatusCode::CONFLICT {
                return Err(ModuleError::RetryLogin);
            }
        }

        let response = response?;
        let res: LoginResponse = response.json().await?;

        Ok(res.token)
    }
}
