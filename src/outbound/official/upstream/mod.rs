mod get_colls;
mod get_hex_machine_bm;
mod get_machines;
mod get_sales;
mod get_state;
mod get_stock;
mod get_token;
mod set_stock;

// Official JSON
pub static LOGIN_ROUTE: &str = "https://online.unicum.ru/wjson/iamrobot.json";
pub static GET_MACHINE_ROUTE: &str = "https://online.unicum.ru/wjson/getmachine.json";
pub static GET_MACHINES_ROUTE: &str = "https://online.unicum.ru/wjson/getmachines.json";
pub static CURSTATE_ROUTE: &str = "https://online.unicum.ru/wjson/curstate.json";
pub static COLL_LIST_ROUTE: &str = "https://online.unicum.ru/wjson/coll_list.json";

// Scraper
#[allow(non_snake_case)]
pub fn STOCK_ROUTE(hex_bookmark: String) -> String {
    format!("https://online.unicum.ru/n/vmcloading.html?03{hex_bookmark}")
}
#[allow(non_snake_case)]
pub fn SALES_ROUTE(hex_bookmark: String) -> String {
    format!("https://online.unicum.ru/n/sgraph.html?V03{hex_bookmark}03")
}
