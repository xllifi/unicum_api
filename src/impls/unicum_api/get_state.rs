use serde::{Deserialize, Serialize};

use super::{AddTokenCookie, ModuleError, UnicumApi, User, utils::to_digit_next};
use crate::{entities::{MachineId, Slot, SlotId, State}, impls::unicum_api::internal::product::Product};

static CURSTATE_ROUTE: &str = "https://online.unicum.ru/wjson/curstate.json";

#[derive(Debug, Serialize, Deserialize)]
struct CurstateRequest {
    pub machineguid: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CurstateResponse {
    pub user: User,
    pub products: Vec<Product>,
}

impl UnicumApi {
    pub(super) async fn get_state(&mut self, machine_id: MachineId) -> Result<State, ModuleError> {
        let req = CurstateRequest {
            machineguid: machine_id.to_string(),
        };

        #[rustfmt::skip]
        let res = self.http_client
            .post(CURSTATE_ROUTE)
            .json(&req)
            .add_token_cookie(self.token().await?.into())
            .send().await?;

        println!("{:?}", res);

        let res_str = res.text().await?;

        let res: CurstateResponse = serde_json::from_str(&res_str)?;

        // Update token
        self.update_token(res.user.token);

        let vec: Vec<Slot> = res
            .products
            .into_iter()
            .filter_map(|p| {
                match p {
                    Product::Snack(p) => {
                        let mut chars = p.common.selection.chars();
                        let row: u8 = to_digit_next(&mut chars, 16).unwrap() as u8;
                        let col: u8 = to_digit_next(&mut chars, 16).unwrap() as u8;
                        println!("Mapping {} to row {row} col {col}", p.common.selection);

                        let price = p.common.price as f32 / 10.0_f32.powi(p.common.decimal as i32);

                        Some(Slot {
                            id: SlotId {
                                row,
                                col,
                                name: p.common.name,
                            },
                            price,
                            max: p.common.max,
                            cur: p.common.level.unwrap_or(0),
                        })
                    }
                    _ => None, // Skip anything other than Snack
                }
            })
            .collect();

        return Ok(vec);
    }
}
