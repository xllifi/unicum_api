use tokio::time::{Duration as TokioDuration, sleep};

use log::info;
use reqwest::Client;
use time::{Duration, OffsetDateTime};

use super::ModuleError;

#[derive(Debug, Clone)]
pub struct UnicumApi {
    pub(super) http_client: Client,
    pub(super) username: String,
    pub(super) password: String,
    token: String,
    token_updated_at: OffsetDateTime,
}

impl UnicumApi {
    pub fn new(username: String, password: String) -> Self {
        Self {
            http_client: Client::new(),
            username,
            password,
            token: String::new(),
            token_updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    pub(super) fn update_token(&mut self, new_token: String) {
        info!("Setting a new Unicum API token: {}", new_token);
        self.token = new_token;
        self.token_updated_at = OffsetDateTime::now_utc();
    }

    pub(super) async fn token(&mut self) -> Result<&str, ModuleError> {
        let token_expires_at = self.token_updated_at.saturating_add(Duration::minutes(39));
        if OffsetDateTime::now_utc() > token_expires_at {
            info!("Token expired, need to re-login");
            loop {
                match self.try_login().await {
                    Err(ModuleError::RetryLogin) => {
                        info!("API busy, retrying in 200ms");
                        sleep(TokioDuration::from_millis(200)).await;
                    }
                    Ok(token) => {
                        self.update_token(token);
                        break;
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        Ok(&self.token)
    }
}
