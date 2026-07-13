// Url: https://online.unicum.ru/n/vmcloading.html?03<hex_machine_bm>
//
// Formdata:
// c<row><col> - how many left in the slot
// m - collection ID
// a - just has to be present (?). can be empty.

use std::collections::HashMap;

use log::info;

use super::{ModuleError, UnicumApi};
use crate::{
    entities::{MachineId, SetStockTarget, Stock}, outbound::official::{AddTokenCookie, utils::parse_err},
};

impl UnicumApi {
    #[allow(non_snake_case)]
    fn SET_STOCK_ROUTE(&mut self, hex_bookmark: String) -> String {
        format!("https://online.unicum.ru/n/vmcloading.html?03{hex_bookmark}")
    }

    pub(super) async fn set_stock(
        &mut self,
        machine_id: MachineId,
        stock: Stock,
        target: SetStockTarget,
    ) -> Result<(), ModuleError> {
        let result = self.get_enchasments_and_bm(machine_id, 1).await?;

        let url = self.SET_STOCK_ROUTE(result.hex_bookmark);

        #[rustfmt::skip]
        let encashment_id = match target {
            SetStockTarget::Latest => result.encashments.first()
                .ok_or(parse_err(format!("No enchasments found! Use future target.")))?
                .id.clone(),
            SetStockTarget::Future => "T".into(),
        };

        let mut req: HashMap<String, String> = HashMap::new();
        for slot in stock {
            req.insert(slot.mapped_to, slot.cur.to_string());
        }
        req.insert("m".into(), encashment_id);
        req.insert("a".into(), String::new());

        self
            .http_client
            .post(url)
            .form(&req)
            .add_token_cookie(self.token().await?.into())
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }
}
