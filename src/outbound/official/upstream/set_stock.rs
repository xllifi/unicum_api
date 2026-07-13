// Url: https://online.unicum.ru/n/vmcloading.html?03<hex_machine_bm>
//
// Formdata:
// c<row><col> - how many left in the slot
// m - collection ID
// a - just has to be present (?). can be empty.

use std::collections::HashMap;

use crate::entities::{MachineId, MachineStock, SetStockForWhichEncashment};
use crate::outbound::official::upstream::STOCK_ROUTE;
use crate::outbound::official::{ModuleError, RequestBuilderExt, ScraperError, UnicumApi};

impl UnicumApi {
    pub(in crate::outbound::official) async fn set_stock_upstream(
        &mut self,
        machine_id: MachineId,
        stock: MachineStock,
        target: SetStockForWhichEncashment,
    ) -> Result<(), ModuleError> {
        let result = self.get_enchasments_and_bm(machine_id, 1).await?;

        let url = STOCK_ROUTE(result.hex_bookmark);

        #[rustfmt::skip]
        let encashment_id = match target {
            SetStockForWhichEncashment::Latest => result.encashments.first()
                .ok_or(ScraperError::MissingElement {
                    element: "encashment".into(),
                })?
                .id.clone(),
            SetStockForWhichEncashment::Future => "T".into(),
        };

        let mut req: HashMap<String, String> = HashMap::new();
        for slot in stock {
            req.insert(slot.mapped_to, slot.cur.to_string());
        }
        req.insert("m".into(), encashment_id);
        req.insert("a".into(), String::new());

        let response = self
            .http_client
            .post(url)
            .form(&req)
            .add_token_cookie(self.token().await?.into())
            .send_checked()
            .await?;
        response.text().await?;

        Ok(())
    }
}
