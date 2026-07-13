use std::{
    ffi::OsString,
    fs,
    net::Ipv4Addr,
    path::PathBuf,
    process::exit,
    sync::LazyLock,
};

use axum::{
    Extension, Json, Router, extract::Request, http::HeaderMap, middleware::Next, response::Response, routing::{get, post},
};
use clap::{Parser, Subcommand};
use colored::Colorize;
use fern::{
    Dispatch,
    colors::{Color, ColoredLevelConfig},
};
use log::{LevelFilter, debug, error, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use time::macros::format_description;
use uuid::Uuid;

use crate::{
    contracts::Contracts, entities::{Permissions, Stock}, outbound::official::UnicumApi, state::get_state_dir, user_mgmt::{RequirePermissions, require_auth, useradd, userdel},
};

mod contracts;
mod entities;
mod outbound;
mod state;
mod user_mgmt;

pub const APP_NAME: &str = "unicum_api";
const ENV_PREFIX: &str = "UNCIUM_API";

pub const ENV_STATE_DIR: LazyLock<EnvVar<Option<OsString>>> =
    LazyLock::new(|| load_runtime_env("STATE_DIR"));
pub const ENV_SALT_B64: LazyLock<EnvVar<String>> = LazyLock::new(|| {
    let var = load_runtime_env("SALT_B64");
    if let Some(value) = var.value {
        if let Some(value) = value.to_str() {
            return EnvVar {
                name: var.name,
                value: value.to_string(),
            };
        } else {
            error!("{ENV_PREFIX}_SALT_B64 env variable's value must be a valid base64 string.");
            exit(2);
        }
    } else {
        error!("Please specify a password salt with {ENV_PREFIX}_SALT_B64 env variable.");
        exit(2);
    }
});
pub const ENV_POSTGRES_URL: LazyLock<EnvVar<String>> = LazyLock::new(|| {
    let var = load_runtime_env("POSTGRES_URL");
    if let Some(value) = var.value {
        if let Some(value) = value.to_str() {
            return EnvVar {
                name: var.name,
                value: value.to_string(),
            };
        } else {
            error!("{ENV_PREFIX}_POSTGRES_URL env variable's value must be a valid ASCII string.");
            exit(2);
        }
    } else {
        error!("Please specify a Postgres URL with {ENV_PREFIX}_POSTGRES_URL env variable.");
        exit(2);
    }
});

pub const ENV_UNICUM_USERNAME: LazyLock<EnvVar<String>> = LazyLock::new(|| {
    let var = load_runtime_env("UNICUM_USERNAME");
    if let Some(value) = var.value {
        if let Some(value) = value.to_str() {
            return EnvVar {
                name: var.name,
                value: value.to_string(),
            };
        } else {
            error!("{ENV_PREFIX}_UNICUM_USERNAME env variable's value must be a valid ASCII string.");
            exit(2);
        }
    } else {
        error!("Please specify a Unicum username with {ENV_PREFIX}_UNICUM_USERNAME env variable.");
        exit(2);
    }
});
pub const ENV_UNICUM_PASSWORD: LazyLock<EnvVar<String>> = LazyLock::new(|| {
    let var = load_runtime_env("UNICUM_PASSWORD");
    if let Some(value) = var.value {
        if let Some(value) = value.to_str() {
            return EnvVar {
                name: var.name,
                value: value.to_string(),
            };
        } else {
            error!("{ENV_PREFIX}_UNICUM_PASSWORD env variable's value must be a valid ASCII string.");
            exit(2);
        }
    } else {
        error!("Please specify a Unicum password with {ENV_PREFIX}_UNICUM_PASSWORD env variable.");
        exit(2);
    }
});

pub struct EnvVar<T> {
    name: String,
    value: T,
}

fn load_runtime_env(suffix: &str) -> EnvVar<Option<OsString>> {
    let name = format!("{ENV_PREFIX}_{suffix}");
    let value = std::env::var_os(&name);
    EnvVar { name, value }
}

pub static STATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let state_dir = get_state_dir();

    if let Err(err) = fs::create_dir_all(&state_dir) {
        use std::io::ErrorKind;
        let state_dir_string = state_dir.to_string_lossy();
        match err.kind() {
            ErrorKind::PermissionDenied => error!(
                "Permission denied while creating state directory {state_dir_string}! Can't continue."
            ),
            ErrorKind::ReadOnlyFilesystem => error!(
                "Failed to create state directory {state_dir_string} because it's a read-only filesystem! Can't continue."
            ),
            ErrorKind::StorageFull => error!(
                "Failed to create state directory {state_dir_string} because storage is full! Can't continue."
            ),
            kind => error!(
                "Failed to create state directory {state_dir_string} for unhandled reason: {kind:?}! Can't continue."
            ),
        }
        if ENV_STATE_DIR.value.is_none() {
            info!(
                "TIP: You can define a custom state directory with \
                {name} environment variable. \
                It'll take preference over the default one.",
                name = ENV_STATE_DIR.name
            )
        }
        exit(2);
    }
    state_dir
});

/// A convenient wrapper for Unicum's API
#[derive(Parser)]
#[command(name = "unicum_api")]
#[command(author = "xllifi <xllifi@kitten.red>")]
#[command(version = "1.0")]
#[command(about = "A convenient wrapper for Unicum's API", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// The subcommands available in this tool
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Starts an API server.
    Serve {
        /// Port number to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        /// Port number to listen on
        #[arg(short, long, default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
        ip: Ipv4Addr,
    },
    /// Add a user
    Useradd { username: Option<String> },
    /// Delere a user
    Userdel { username: Option<String> },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    setup_logger().apply().unwrap();
    LazyLock::force(&STATE_DIR);

    let cli = Cli::parse();

    let pool = init_db().await;

    match cli.command {
        Commands::Serve { port, ip } => listen(port, ip, pool).await,
        Commands::Useradd { username } => useradd(username, pool).await,
        Commands::Userdel { username } => userdel(username, pool).await,
    }
}

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
        .level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(std::io::stdout())
}

#[rustfmt::skip]
async fn init_db() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&ENV_POSTGRES_URL.value)
        .await
        .unwrap();

    // Ensure users table exists
    let _ = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT NOT NULL PRIMARY KEY,
            password_hash TEXT NOT NULL,
            permissions INTEGER NOT NULL
        )"
    )
    .fetch(&pool);

    pool
}

async fn listen(port: u16, ip: Ipv4Addr, pg_pool: PgPool) {
    let unicum_api = UnicumApi::new(ENV_UNICUM_USERNAME.value.clone(), ENV_UNICUM_PASSWORD.value.clone());

    let app = Router::new()
        .route("/get_stock", post(get_stock))
        .layer(axum::middleware::from_fn(require_auth))
        .layer(axum::middleware::from_fn(logger))
        .layer(Extension(unicum_api))
        .layer(Extension(pg_pool));

    let listener = tokio::net::TcpListener::bind(format!("{ip}:{port}"))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    info!("Listening on {ip}:{port}");
}

#[derive(Serialize, Deserialize)]
struct GetStockRequest {
    machine_id: Uuid,
}

async fn get_stock(
    _auth: RequirePermissions<{ Permissions::STOCK_READ.bits() }>,
    Extension(mut unicum_api): Extension<UnicumApi>,
    Json(payload): Json<GetStockRequest>,
) -> Result<Json<Stock>, StatusCode> {
    let stock = unicum_api.get_stock(payload.machine_id).await.map_err(|e| {
        error!("Unicum API error: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(stock))
}

async fn logger(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let uri = req.uri();
    info!("Got a request for {uri}");
    Ok(next.run(req).await)
}