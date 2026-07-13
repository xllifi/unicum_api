use log::{error, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::outbound::official::{ModuleError, UnicumApi};

static LOGIN_ROUTE: &str = "https://online.unicum.ru/wjson/iamrobot.json";

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
    pub(super) async fn try_login(&self) -> Result<String, ModuleError> {
        let req = LoginRequest {
            username: &self.username,
            password: &self.password,
        };
        info!("Authenticating with Unicum as {}", req.username);
        #[rustfmt::skip]
        let res = self.http_client
            .post(LOGIN_ROUTE)
            .json(&req)
            .send().await;

        if let Some(status) = res.as_ref().err().and_then(|x| x.status()) {
            error!("(UnicumApi::try_login) Error with status: {status:?}");
            if status == StatusCode::CONFLICT {
                return Err(ModuleError::RetryLogin);
            }
        }

        let res: LoginResponse = res?.json().await?;

        Ok(res.token)
    }
}
