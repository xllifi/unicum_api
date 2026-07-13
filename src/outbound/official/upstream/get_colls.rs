use base64::Engine;
use log::trace;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    entities::MachineId,
    outbound::official::{
        ModuleError, RequestBuilderExt, ScraperError, UnicumApi, UserInResponse,
        upstream::COLL_LIST_ROUTE,
    },
};

impl UnicumApi {
    pub(crate) async fn get_enchasments_and_bm(
        &mut self,
        machine_id: MachineId,
        count: u8,
    ) -> Result<EncashmentsAndBookmark, ModuleError> {
        let req = CollListRequest {
            machineguid: machine_id.to_string(),
            collcount: count,
            timestamp: OffsetDateTime::now_utc(),
        };
        let res = self
            .http_client
            .post(COLL_LIST_ROUTE)
            .json(&req)
            .add_token_cookie(self.token().await?.into())
            .send_checked()
            .await?;

        let json: CollListResponse = res.json().await?;

        self.update_token(json.user.token);

        trace!("Decoding machine bm from base64..");
        let bytes = base64::prelude::BASE64_STANDARD
            .decode(json.bookmark)
            .map_err(|error| ScraperError::ParseError {
                cause: format!("Invalid bookmark: {error}"),
            })?;
        let hex: String = bytes.iter().map(|byte| format!("{byte:02X}")).collect();
        Ok(EncashmentsAndBookmark {
            hex_bookmark: hex,
            encashments: json.encashments,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CollListRequest {
    pub machineguid: String,
    pub collcount: u8,
    #[serde(rename = "date", with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct CollListResponse {
    pub user: UserInResponse,
    #[serde(rename = "bm")]
    pub bookmark: String,
    #[serde(rename = "collections")]
    pub encashments: Vec<Encashment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Encashment {
    #[serde(rename = "collection")]
    pub id: String,
    #[serde(rename = "startstime", with = "time::serde::rfc3339")]
    pub start_time: OffsetDateTime,
}

pub struct EncashmentsAndBookmark {
    pub encashments: Vec<Encashment>,
    pub hex_bookmark: String,
}
