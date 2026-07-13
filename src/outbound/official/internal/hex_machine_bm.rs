use base64::Engine;
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};

use crate::{
    entities::MachineId,
    outbound::official::{AddTokenCookie, ModuleError, UnicumApi, User},
};

static GET_MACHINE_ROUTE: &str = "https://online.unicum.ru/wjson/getmachine.json";

#[derive(Debug, Serialize, Deserialize)]
struct GetMachineRequest {
    pub machineguid: String,
}

// We only need bm for this use case, so we just pretend like everything else doesn't exist
#[derive(Debug, Serialize, Deserialize)]
struct GetMachineResponse {
    #[serde(rename = "bm")]
    pub bookmark: String,
    pub user: User,
}

impl UnicumApi {
    pub(crate) async fn get_hex_machine_bm(
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
            .send()
            .await?;

        if !res.status().is_success() {
            error!("Bad status: {}", res.status().as_str())
        }

        let body = res
            .text()
            .await?;

        debug!("{body}");

        let res: GetMachineResponse = serde_json::from_str(&body)?;

        self.update_token(res.user.token);

        Ok(Self::convert_machine_bookmark_to_hex(res.bookmark))
    }
    pub(crate) fn convert_machine_bookmark_to_hex(bm: String) -> String {
        trace!("Decoding machine bookmark from base64..");
        let unbase64 = base64::prelude::BASE64_STANDARD.decode(bm).unwrap();
        let hex: String = unbase64.iter().map(|b| format!("{:02X}", b)).collect();
        hex
    }
}
