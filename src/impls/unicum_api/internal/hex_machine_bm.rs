use base64::Engine;
use log::trace;
use serde::{Deserialize, Serialize};

use crate::{
    entities::MachineId,
    impls::unicum_api::{AddTokenCookie, ModuleError, UnicumApi, User},
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
        let res: GetMachineResponse = self
            .http_client
            .post(GET_MACHINE_ROUTE)
            .json(&req)
            .add_token_cookie(self.token().await?.into())
            .send()
            .await?
            .json()
            .await?;

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
