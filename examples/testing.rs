use colored::Colorize;
use fern::{
    Dispatch,
    colors::{Color, ColoredLevelConfig},
};
use log::{LevelFilter};
use time::macros::format_description;
use uuid::Uuid;

use unicum_api::{
    contracts::Contracts,
    entities::{SetStockTarget, StockSlot},
    impls::unicum_api::UnicumApi,
};

fn setup_logger() -> Dispatch {
    let colors: ColoredLevelConfig = ColoredLevelConfig::new()
        .debug(Color::BrightBlack)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} | {} | {} > {}",
                Colorize::bright_black(
                    time::OffsetDateTime::now_utc()
                        .format(format_description!("[hour]:[minute]:[second]"))
                        .unwrap()
                        .as_str()
                ),
                format!("{:^5}", colors.color(record.level())),
                Colorize::cyan(record.target()),
                message
            ));
        })
        .level(LevelFilter::Trace)
        .level_for("html5ever", LevelFilter::Info)
        .level_for("selectors", LevelFilter::Info)
        .level_for("reqwest", LevelFilter::Info)
        .chain(std::io::stdout())
}

#[tokio::main]
async fn main() {
    setup_logger().apply().unwrap();

    let mut unicum = UnicumApi::new("username".into(), "password".into());

    let mut stock = unicum
        .get_stock(Uuid::from_u128(0x00000000000000000000000000000000))
        .await
        .unwrap();

    println!("{:?}", stock);

    stock[0] = StockSlot {
        row: stock[0].row,
        col: stock[0].col,
        mapped_to: stock[0].mapped_to.clone(),
        cur: 1,
    };

    unicum.set_stock(
        Uuid::from_u128(0x00000000000000000000000000000000),
        stock,
        SetStockTarget::Latest,
    ).await.unwrap();
}
