use std::{net::Ipv4Addr, process::exit};

use clap::{Parser, Subcommand};
use log::{error, info};

use crate::{
    api::AppState,
    outbound::official::UnicumApi,
    users::{useradd, userdel},
};

mod api;
mod config;
mod contracts;
mod database;
mod entities;
mod logging;
mod outbound;
mod users;

pub const APP_NAME: &str = "unicum_api";

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
    if let Err(error) = dotenvy::dotenv() {
        if error.not_found() {
            info!("No .env file found; using process environment");
        } else {
            error!("Failed to load .env file: {error}");
            exit(2);
        }
    }
    if let Err(error) = logging::setup().apply() {
        eprintln!("Failed to initialize logger: {error}");
        exit(2);
    }
    if let Err(error) = config::initialize_state_dir() {
        error!("Failed to initialize state directory: {error}");
        exit(2);
    }

    let cli = Cli::parse();

    let postgres_url = config::postgres_url().unwrap_or_else(|error| {
        error!("{error}");
        exit(2);
    });
    let pool = match database::connect(&postgres_url).await {
        Ok(pool) => pool,
        Err(error) => {
            error!("Failed to initialize database: {error}");
            exit(2);
        }
    };

    match cli.command {
        Commands::Serve { port, ip } => {
            let username = config::unicum_username().unwrap_or_else(|error| {
                error!("{error}");
                exit(2);
            });
            let password = config::unicum_password().unwrap_or_else(|error| {
                error!("{error}");
                exit(2);
            });
            let unicum = UnicumApi::new(username, password);
            api::serve(port, ip, AppState::new(pool, unicum)).await;
        }
        Commands::Useradd { username } => {
            let salt = config::password_salt().unwrap_or_else(|error| {
                error!("{error}");
                exit(2);
            });
            useradd(username, pool, &salt).await;
        }
        Commands::Userdel { username } => userdel(username, pool).await,
    }
}
