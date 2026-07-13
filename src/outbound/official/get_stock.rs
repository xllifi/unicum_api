// Url: https://online.unicum.ru/n/vmcloading.html?03<hex_machine_bm>

use log::info;
use scraper::{Html, Selector};

use super::{ModuleError, UnicumApi};
use crate::{
    entities::{MachineId, Stock, StockSlot}, outbound::official::{
        AddTokenCookie,
        utils::{parse_err, sel_to_row_and_col},
    },
};

impl UnicumApi {
    #[allow(non_snake_case)]
    fn GET_STOCK_ROUTE(&mut self, hex_bookmark: String) -> String {
        format!("https://online.unicum.ru/n/vmcloading.html?03{hex_bookmark}")
    }

    pub(super) async fn get_stock(
        &mut self,
        machine_id: MachineId,
    ) -> Result<Stock, ModuleError> {
        let hex_bookmark = self.get_hex_machine_bm(machine_id).await?;

        let url = self.GET_STOCK_ROUTE(hex_bookmark);

        let document = self
            .http_client
            .get(url)
            .add_token_cookie(self.token().await?.into())
            .send()
            .await?
            .text()
            .await?;

        // Parse the HTML document
        let html = Html::parse_document(&document);

        // Define the selectors
        let row_selector = Selector::parse("form > table tr:not(:first-child)").unwrap();
        let sel_td_selector = Selector::parse("td:first-child").unwrap();
        let input_selector = Selector::parse("input").unwrap();

        let mut stock: Stock = Vec::new();

        for row in html.select(&row_selector) {
            let sel = row
                .select(&sel_td_selector)
                .next()
                .map(|td| td.inner_html())
                .unwrap_or_default();

            let input = row
                .select(&input_selector)
                .next()
                .ok_or(parse_err("Failed to find input element"))?;

            let mapped_to = input
                .value().attr("name")
                .ok_or(parse_err("Failed to find name on input element"))?
                .to_string();

            let cur: u8 = input
                .value().attr("value")
                .ok_or(parse_err("Failed to find value on input element"))?
                .parse()
                .map_err(|e| parse_err(format!("Failed to parse value of input as u8: {e}")))?;

            let (row, col) = sel_to_row_and_col(sel)?;
            stock.push(StockSlot {
                row,
                col,
                mapped_to,
                cur,
            });
        }

        Ok(stock)
    }
}