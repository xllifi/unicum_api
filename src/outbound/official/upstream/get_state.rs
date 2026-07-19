use serde::{Deserialize, Serialize};

use crate::outbound::official::upstream::CURSTATE_ROUTE;
use crate::outbound::official::{
    ModuleError, RequestBuilderExt, UnicumApi, UserInResponse, scraper::to_digit_next,
};
use crate::{
    entities::{MachineId, MachineState, MachineStateEntry, Product},
    outbound::official::types::UpstreamProduct,
};

#[derive(Debug, Serialize, Deserialize)]
struct CurstateRequest {
    pub machineguid: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CurstateResponse {
    pub user: UserInResponse,
    pub products: Vec<UpstreamProduct>,
}

impl UnicumApi {
    pub(in crate::outbound::official) async fn get_state_upstream(
        &mut self,
        machine_id: MachineId,
    ) -> Result<MachineState, ModuleError> {
        let req = CurstateRequest {
            machineguid: machine_id.to_string(),
        };

        #[rustfmt::skip]
        let res = self.http_client
            .post(CURSTATE_ROUTE)
            .json(&req)
            .add_token_cookie(self.token().await?.into())
            .send_checked().await?;

        let res_str = res.text().await?;

        let res: CurstateResponse = serde_json::from_str(&res_str)?;

        // Update token
        self.update_token(res.user.token);

        let mut state_entries = Vec::new();
        for product in res.products {
            let UpstreamProduct::Snack(product) = product else {
                continue;
            };

            let mut chars = product.common.selection.chars();
            let row = to_digit_next(&mut chars, 16)? as u8;
            let col = to_digit_next(&mut chars, 16)? as u8;
            let price = product.common.price as f32 / 10.0_f32.powi(product.common.decimal as i32);

            state_entries.push(MachineStateEntry {
                id: Product::new(row, col, product.common.name),
                price,
                max: product.common.max,
                cur: product.common.level.unwrap_or(0),
            });
        }

        state_entries.sort_by(|a, b| {
            a.id.row.cmp(&b.id.row).then(a.id.col.cmp(&b.id.col))
        });

        Ok(state_entries)
    }
}
