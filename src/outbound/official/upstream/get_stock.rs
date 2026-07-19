// Url: https://online.unicum.ru/n/vmcloading.html?03<hex_machine_bm>

use scraper::{Html, Selector};

use crate::outbound::official::scraper::GetText;
use crate::outbound::official::upstream::STOCK_ROUTE;
use crate::outbound::official::{ModuleError, RequestBuilderExt, ScraperError, UnicumApi};
use crate::{
    entities::{MachineId, MachineStock, StockEntry},
    outbound::official::scraper::{coordinate_to_row_and_col, parse_required_attr, parse_u8_attr},
};

impl UnicumApi {
    pub(in crate::outbound::official) async fn get_stock_upstream(
        &mut self,
        machine_id: MachineId,
    ) -> Result<MachineStock, ModuleError> {
        let hex_bookmark = self.get_hex_machine_bookmark(machine_id).await?;

        let state = self.get_state_upstream(machine_id).await?;

        let url = STOCK_ROUTE(hex_bookmark);

        let response = self
            .http_client
            .get(url)
            .add_token_cookie(self.token().await?.into())
            .send_checked()
            .await?;
        let document = response.text().await?;

        // Parse the HTML document
        let html = Html::parse_document(&document);

        // Define the selectors
        let row_selector = Selector::parse("form > table tr:not(:first-child)").unwrap();
        let coordinate_selector = Selector::parse("td:first-child").unwrap();
        let input_selector = Selector::parse("input").unwrap();

        let mut stock = MachineStock::new();

        for row in html.select(&row_selector) {
            let coordinate = row
                .select(&coordinate_selector)
                .next()
                .map(|td| td.text_string())
                .unwrap_or_default();

            let input = row
                .select(&input_selector)
                .next()
                .ok_or(ScraperError::MissingElement {
                    element: "input".into(),
                })?;

            let mapped_to = parse_required_attr(input, "name")?.to_owned();
            let cur = parse_u8_attr(input, "value")?;

            let (row, col) = coordinate_to_row_and_col(coordinate.trim())?;

            let name = state
                .iter()
                .find(|x| x.id.col == col && x.id.row == row)
                .map(|x| x.id.name.clone())
                .ok_or(ScraperError::MissingElement {
                    element: format!("name in current state for coordinate {coordinate}"),
                })?;
            stock.push(StockEntry {
                name,
                row,
                col,
                mapped_to,
                cur,
            });
        }

        Ok(stock)
    }
}
