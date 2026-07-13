use colored::Colorize;
use fern::{
    Dispatch,
    colors::{Color, ColoredLevelConfig},
};
use log::LevelFilter;
use time::macros::format_description;

pub fn setup() -> Dispatch {
    let colors = ColoredLevelConfig::new()
        .debug(Color::BrightBlack)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    Dispatch::new()
        .format(move |out, message, record| {
            let timestamp = time::OffsetDateTime::now_utc()
                .format(format_description!("[hour]:[minute]:[second]"))
                .unwrap_or_else(|_| "--:--:--".to_owned());
            out.finish(format_args!(
                "{} | {} | {} > {}",
                timestamp.bright_black(),
                format!("{:^5}", colors.color(record.level())),
                record.target().cyan(),
                message
            ));
        })
        .level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .level_for("html5ever", LevelFilter::Warn)
        .level_for("selectors", LevelFilter::Warn)
        .level_for("h2", LevelFilter::Warn)
        .chain(std::io::stdout())
}
