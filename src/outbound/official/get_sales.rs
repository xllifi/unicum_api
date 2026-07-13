// This one is going to get ugly. You've been warned.
//
// Since unicum doesn't provide a convenient API for sales data, we'll be scraping.
// Luckily, they only use basic HTML in old UI.

// Url: https://online.unicum.ru/n/sgraph.html?03<hex_machine_bm><group_mode>
// group_mode - We'll be explicitly using 00 (none). Also available: 01 - group by hour, 02 - group by week day, 03 - group by month day, 04 - group by month.
//
// Formdata:
// c<row><col> - how many left in the slot
// m - collection ID
// a - just has to be present (?). can be empty.

use std::collections::HashMap;

use log::{debug, info};
use scraper::{Html, Selector};
use time::{Date, macros::format_description, parsing::Parsed};

use super::{AddTokenCookie, ModuleError, UnicumApi, utils::parse_next};
use crate::{entities::{self, MachineId, Sale, Sales, SlotId}, outbound::official::utils::to_digit_next};

impl UnicumApi {
    #[allow(non_snake_case)]
    async fn GET_SALES_ROUTE(&mut self, machine_id: MachineId) -> Result<String, ModuleError> {
        let hex_bm = self.get_hex_machine_bm(machine_id).await?;

        Ok(format!(
            "https://online.unicum.ru/n/sgraph.html?V03{hex_bm}00"
        ))
    }

    /// since - inclusive, until - exclusive
    pub(super) async fn get_sales(
        &mut self,
        machine_id: MachineId,
        since: Date,
        until: Date,
    ) -> Result<Sales, ModuleError> {
        if until <= since {
            return Ok(vec![]);
        }
        let url = self.GET_SALES_ROUTE(machine_id).await?;
        
        let mut req = HashMap::new();
        // Since
        req.insert("day", since.day().to_string());
        req.insert("month", (since.month() as u8).to_string());
        req.insert("year", since.year().to_string());
        // Until
        req.insert("day1", until.day().to_string());
        req.insert("month1", (until.month() as u8).to_string());
        req.insert("year1", until.year().to_string());

        let document = self
            .http_client
            .post(url)
            .form(&req)
            .add_token_cookie(self.token().await?.into())
            .send()
            .await?
            .text()
            .await?;

        let html = Html::parse_document(&document);

        let table_selector = Selector::parse("table").unwrap();
        let date_row_selector = Selector::parse("tr:first-child").unwrap();

        let sale_rows_selector =
            Selector::parse("tr:not(:first-child, :last-child, :nth-last-child(2))").unwrap();
        let slot_cell_selector = Selector::parse("td:first-child font").unwrap();
        let name_cell_selector = Selector::parse("td:first-child b").unwrap();
        
        let informative_cell_selector =
            Selector::parse("td:not(:first-child, :last-child, :nth-last-child(2))").unwrap();

        let mut sales: Vec<Sale> = vec![];
        if let Some(table) = html.select(&table_selector).next() {
            debug!("Found table!");
            let mut idx_to_date: HashMap<usize, Date> = HashMap::new();
            if let Some(date_row) = table.select(&date_row_selector).next() {
                debug!("Found dates row!");
                let description = format_description!("[day padding:zero]/[month padding:zero repr:numerical]/[year padding:zero repr:last_two]");
                for (i, cell) in date_row.select(&informative_cell_selector).enumerate() {
                    let cell_text: String = cell.text().next().unwrap_or("").trim().into();
                    debug!("Parsing date cell {cell_text}");
                    if cell_text.is_empty() {
                        continue;
                    }

                    let mut parsed = Parsed::new();
                    parsed.parse_items(cell_text.as_bytes(), description).map_err(|e| {
                        ModuleError::ParseError {
                            cause: format!("failed to parse {cell_text}: {e}"),
                        }
                    })?;

                    parsed.set_year_century(20, false);
                    let date = Date::try_from(parsed).map_err(|e| {
                        ModuleError::ParseError {
                            cause: format!("failed to create date from a Parsed: {e}"),
                        }
                    })?;

                    idx_to_date.insert(i, date);
                }
            }
            debug!("idx_to_unix: {idx_to_date:?}");
            for row in table.select(&sale_rows_selector) {
                let name = row
                    .select(&name_cell_selector)
                    .next()
                    .map(|x| x.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or(String::new());

                let slot_id: SlotId = {
                    let slot = row
                        .select(&slot_cell_selector)
                        .next()
                        .and_then(|x| x.text().next().map(|x| x.into()))
                        .unwrap_or(String::new());
                    let mut chars = slot[1..slot.len() - 2].chars();
                    let row: u8 = to_digit_next(&mut chars, 16)? as u8;
                    let col: u8 = to_digit_next(&mut chars, 16)? as u8;

                    SlotId { row, col, name }
                };
                debug!("Parsing sales row for slot C{}R{} {}", slot_id.col, slot_id.row, slot_id.name);


                for (i, cell) in row.select(&informative_cell_selector).enumerate() {
                    let cell_text: String =
                        cell.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    debug!("Parsing sale {}", cell_text);
                    if cell_text.is_empty() {
                        continue;
                    }
                    let mut split = cell_text.split("/");
                    let sales_count: u32 = parse_next(&mut split)?;
                    let sales_price: f32 = parse_next(&mut split)?;
                    let price = sales_price / sales_count as f32;

                    for _ in 0..sales_count {
                        sales.push(Sale {
                            date: entities::Date::from(*idx_to_date.get(&i).unwrap()),
                            slot_id: slot_id.clone(),
                            price,
                        });
                    }
                }
            }
        } else {
            return Err(ModuleError::ParseError {
                cause: "Couldn't find <table>".into(),
            });
        }

        Ok(sales)
    }
}
