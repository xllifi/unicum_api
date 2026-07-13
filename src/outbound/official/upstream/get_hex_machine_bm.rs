use base64::Engine;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    entities::MachineId,
    outbound::official::{
        ModuleError, RequestBuilderExt, ScraperError, UnicumApi, UserInResponse,
        upstream::GET_MACHINE_ROUTE,
    },
};

impl UnicumApi {
    pub(crate) async fn get_hex_machine_bookmark(
        &mut self,
        machine_id: MachineId,
    ) -> Result<String, ModuleError> {
        let req = GetMachineRequest {
            machineguid: machine_id.to_string(),
        };
        let token = self.token().await?.into();
        debug!("Sending request {req:?} to {GET_MACHINE_ROUTE} with token {token}");
        let res = self
            .http_client
            .post(GET_MACHINE_ROUTE)
            .json(&req)
            .add_token_cookie(token)
            .send_checked()
            .await?;

        let body = res.text().await?;

        debug!("{body}");

        let res: GetMachineResponse = serde_json::from_str(&body)?;

        self.update_token(res.user.token);

        Ok(Self::convert_machine_bookmark_to_hex(res.bookmark)?)
    }

    pub(crate) fn convert_machine_bookmark_to_hex(bookmark: String) -> Result<String, ModuleError> {
        trace!("Decoding machine bookmark from base64..");
        let bytes = base64::prelude::BASE64_STANDARD
        .decode(bookmark)
        .map_err(|error| ScraperError::ParseError {
            cause: format!("Invalid bookmark: {error}"),
        })?;
        let bookmark = bytes.iter().map(|byte| format!("{byte:02X}")).collect();
        debug!("Machine bookmark: {bookmark}");
        Ok(bookmark)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GetMachineRequest {
    pub machineguid: String,
}

// We only need bm for this use case, so we just pretend like everything else doesn't exist
#[derive(Debug, Serialize, Deserialize)]
struct GetMachineResponse {
    #[serde(rename = "bm")]
    pub bookmark: String,
    pub user: UserInResponse,
}
