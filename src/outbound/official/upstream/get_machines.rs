use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::Machine,
    outbound::official::{
        ModuleError, RequestBuilderExt, UnicumApi, UserInResponse, upstream::GET_MACHINES_ROUTE,
    },
};

impl UnicumApi {
    pub(crate) async fn get_machines_upstream(&mut self) -> Result<Vec<Machine>, ModuleError> {
        let token = self.token().await?.into();
        debug!("Sending request to {GET_MACHINES_ROUTE} with token {token}");
        let res = self
            .http_client
            .post(GET_MACHINES_ROUTE)
            .add_token_cookie(token)
            .send_checked()
            .await?;

        let body = res.text().await?;

        debug!("{body}");

        let res: GetMachinesResponse = serde_json::from_str(&body)?;

        self.update_token(res.user.token);

        Ok(res
            .machines
            .into_iter()
            .map(|x| Machine {
                guid: x.guid,
                comment: x.comment,
            })
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GetMachinesResponse {
    pub machines: Vec<UpstreamMachine>,
    pub user: UserInResponse,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpstreamMachine {
    pub guid: Uuid,
    pub comment: String,
}
