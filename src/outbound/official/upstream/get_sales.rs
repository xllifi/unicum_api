// TODO: add tests for this

use std::num::ParseIntError;

use scraper::{ElementRef, Html, Selector};
use time::{Date, Month};

use crate::{
    entities::{MachineId, MachineSales, MachineSalesEntry, Product}, outbound::official::{
        ModuleError, RequestBuilderExt, ScraperError, UnicumApi, scraper::{GetText, to_digit_next}, upstream::SALES_ROUTE,
    },
};

const TABLE_SELECTOR: &str = "table";

const HEADER_ROW_SELECTOR: &str =
    "tr:first-child > :is(td,th):not(:first-child, :last-child, :nth-last-child(2))";
const HEADER_CELL_SELECTOR: &str =
    "tr:not(:first-child, :last-child, :nth-last-child(2)) > :is(td, th):first-child";

const NON_HEADER_ROW_SELECTOR: &str = //
    "tr:not(:first-child, :last-child, :nth-last-child(2))";
const NON_HEADER_CELL_SELECTOR: &str =
    ":is(td, th):not(:first-child, :last-child, :nth-last-child(2))";

impl UnicumApi {
    /// Returns sales from `since` (inclusive) until `until` (exclusive).
    pub(in crate::outbound::official) async fn get_sales_upstream(
        &mut self,
        machine_id: MachineId,
        since: Date,
        until: Date,
    ) -> Result<MachineSales, ModuleError> {
        if until < since {
            return Ok(Vec::new());
        }

        let months_to_query = get_months_between(since, until);

        let bookmark = self.get_hex_machine_bookmark(machine_id).await?;
        let url = SALES_ROUTE(bookmark);
        let since = [
            ("day", "1".into()),
            ("month", (since.month() as u8).to_string()),
            ("year", since.year().to_string()),
        ];
        let mut results = Vec::new();
        for (month, year) in months_to_query {
            let until = [
                ("day1", month.length(year).to_string()),
                ("month1", (month as u8).to_string()),
                ("year1", year.to_string()),
            ];
            let mut iter = since.clone().into_iter().chain(until);
            let req: [(&str, String); 6] = std::array::from_fn(|_| iter.next().unwrap());

            let response = self
                .http_client
                .post(&url)
                .form(&req)
                .add_token_cookie(self.token().await?.to_owned())
                .send_checked()
                .await?;
            let document = response.text().await?;

            let result = parse_sales_report(&document, month, year).map_err(ScraperError::from)?;

            result.into_iter().for_each(|e| results.push(e));
        }

        Ok(results)
    }
}

fn get_months_between(since: Date, until: Date) -> Vec<(Month, i32)> {
    // Return empty if the start date is after the end date
    if since > until {
        return Vec::new();
    }

    let mut current_year = since.year();
    let mut current_month = since.month();

    let end_year = until.year();
    let end_month = until.month();

    let mut results = Vec::new();

    // Loop until the current year/month surpasses the end year/month
    while (current_year < end_year) || (current_year == end_year && current_month <= end_month) {
        results.push((current_month, current_year));

        // If we are at December, increment the year before moving to January
        if current_month == Month::December {
            current_year += 1;
        }
        current_month = current_month.next(); // Built-in method that wraps December to January
    }

    results
}

fn parse_sales_report(
    document: &str,
    month: Month,
    year: i32,
) -> Result<MachineSales, ScraperError> {
    let html = Html::parse_document(document);
    let table = html.select_first(TABLE_SELECTOR, "sales table")?;

    let dates_row: Vec<Date> = table
        .select(&Selector::parse(HEADER_ROW_SELECTOR).expect("static selector should parse"))
        .map(|el| parse_date_value(el.text_string(), month, year))
        .filter(|res| {
            // Remove all dates which are invalid - the HTML has days 1-31 regardless of month.
            if let Err(ScraperError::InvalidDate { value: _, cause: _ }) = res {
                false
            } else {
                true
            }
        })
        .collect::<Result<_, _>>()?;
    let products_col: Vec<Product> = table
        .select(&Selector::parse(HEADER_CELL_SELECTOR).expect("static selector should parse"))
        .map(|el| parse_product_value(el.text_string()))
        .collect::<Result<_, _>>()?;

    let table: Vec<Vec<String>> = table
        .select(&Selector::parse(NON_HEADER_ROW_SELECTOR).expect("static selector should be valid"))
        .map(|row| {
            row.select(
                &Selector::parse(NON_HEADER_CELL_SELECTOR)
                    .expect("static selector should be valid"),
            )
            .map(|cell| cell.text_string())
            .collect()
        })
        .collect();

    let capacity = table.len() * products_col.len();
    let sales = table.into_iter().enumerate().try_fold(
        Vec::with_capacity(capacity),
        |acc, (row_index, row)| {
            let product = products_col
                .get(row_index)
                .ok_or_else(|| ScraperError::JaggedTable {
                    cause: format!("Missing product for row index {row_index}"),
                })?;

            row.into_iter()
                .enumerate()
                .try_fold(acc, |mut acc, (col_index, col)| {
                    if col_index >= month.length(year) as usize {
                        // Skip this column since HTML includes columns for days 1-31 regardless of month
                        return Ok(acc);
                    }
                    let date =
                        dates_row
                            .get(col_index)
                            .ok_or_else(|| ScraperError::JaggedTable {
                                cause: format!("Missing date for col index {col_index}"),
                            })?;

                    let sales = parse_sales_value(&col, date, product)?;
                    acc.extend(sales);
                    Ok(acc)
                })
        },
    )?;
    Ok(sales)
}

fn parse_date_value(value: String, month: Month, year: i32) -> Result<Date, ScraperError> {
    let day = value
        .as_str()
        .parse()
        .map_err(|e: ParseIntError| ScraperError::ParseError {
            cause: format!("for string {value}: {e}"),
        })?;
    Date::from_calendar_date(year, month, day).map_err(|e| ScraperError::InvalidDate {
        value: format!("{day}/{month}/{year}"),
        cause: e.to_string(),
    })
}

fn parse_product_value(value: String) -> Result<Product, ScraperError> {
    let raw_coordinate: String = value.chars().take(4).collect();
    let coordinate = raw_coordinate
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| ScraperError::InvalidValue {
            field: "product coordinate".into(),
            value: raw_coordinate.clone(),
        })?;

    let mut characters = coordinate.chars();
    let row = to_digit_next(&mut characters, 16)? as u8;
    let col = to_digit_next(&mut characters, 16)? as u8;

    let name = value
        .strip_prefix(&raw_coordinate)
        .ok_or_else(|| ScraperError::InvalidValue {
            field: "sales product name for coordinate".into(),
            value: coordinate.into(),
        })?
        .trim();
    if name.is_empty() {
        return Err(ScraperError::EmptyValue {
            what: format!("sales product name for coordinate {value}"),
        });
    }

    Ok(Product::new(row, col, name))
}

fn parse_sales_value(
    value: &str,
    date: &Date,
    product: &Product,
) -> Result<Vec<MachineSalesEntry>, ScraperError> {
    if value.is_empty() {
        // No sales for this day
        return Ok(Vec::new());
    }

    let (count, total_price) =
        value
            .split_once('/')
            .ok_or_else(|| ScraperError::InvalidSalesCell {
                value: value.into(),
                cause: format!("expected <count>/<total_price>, got: {value}"),
            })?;
    if total_price.contains('/') {
        return Err(ScraperError::InvalidSalesCell {
            value: value.into(),
            cause: format!("expected <count>/<total_price>, got: {value}"),
        });
    }

    let count = count
        .trim()
        .parse::<u32>()
        .map_err(|error| ScraperError::InvalidSalesCell {
            value: value.into(),
            cause: format!("unparseable count: {error}"),
        })?;
    if count == 0 {
        return Err(ScraperError::ZeroSalesCount);
    }
    let total_price =
        total_price
            .trim()
            .parse::<f32>()
            .map_err(|error| ScraperError::InvalidSalesCell {
                value: value.into(),
                cause: format!("unparseable total price: {error}"),
            })?;

    let sale = MachineSalesEntry {
        date: date.clone().into(),
        product: product.clone(),
        price: total_price / count as f32,
    };
    Ok(vec![sale; count as usize])
}

trait SelectFirst<'a> {
    fn select_first(
        &'a self,
        selector: &str,
        element: &str,
    ) -> Result<ElementRef<'a>, ScraperError>;
}

impl<'a> SelectFirst<'a> for Html {
    fn select_first(
        &'a self,
        selector: &str,
        element: &str,
    ) -> Result<ElementRef<'a>, ScraperError> {
        self.select(&Selector::parse(selector).expect("static selector should be valid"))
            .next()
            .ok_or_else(|| ScraperError::MissingElement {
                element: element.to_owned(),
            })
    }
}

impl<'a> SelectFirst<'a> for ElementRef<'_> {
    fn select_first(
        &'a self,
        selector: &str,
        element: &str,
    ) -> Result<ElementRef<'a>, ScraperError> {
        self.select(&Selector::parse(selector).expect("static selector should be valid"))
            .next()
            .ok_or_else(|| ScraperError::MissingElement {
                element: element.to_owned(),
            })
    }
}
