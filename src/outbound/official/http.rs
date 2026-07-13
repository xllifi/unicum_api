use reqwest::{RequestBuilder, Response};

use super::ModuleError;

pub(crate) trait RequestBuilderExt {
    fn add_token_cookie(self, token: String) -> Self;

    async fn send_checked(self) -> Result<Response, ModuleError>;
}

impl RequestBuilderExt for RequestBuilder {
    fn add_token_cookie(self, token: String) -> Self {
        self.header("Cookie", format!("nvmc_login={token}"))
    }

    async fn send_checked(self) -> Result<Response, ModuleError> {
        let response = self.send().await?;
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            Err(ModuleError::HttpStatus { status })
        }
    }
}
