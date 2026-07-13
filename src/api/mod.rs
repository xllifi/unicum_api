use std::{net::Ipv4Addr, sync::Arc};

use axum::{
    Router, middleware,
    routing::{get, post},
};
use log::{error, info};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::outbound::official::UnicumApi;

use self::auth::{request_logger, require_auth};

pub mod auth;
pub mod error;
pub mod machines;
pub mod sales;
pub mod state;
pub mod stock;

use error::ApiError;

#[derive(Clone)]
pub struct AppState {
    pub database: PgPool,
    pub unicum: Arc<Mutex<UnicumApi>>,
}

impl AppState {
    pub fn new(database: PgPool, unicum: UnicumApi) -> Self {
        Self {
            database,
            unicum: Arc::new(Mutex::new(unicum)),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/get_machines", get(machines::get).post(machines::get))
        .route("/get_state", post(state::get))
        .route("/get_sales", post(sales::get))
        .route("/get_stock", post(stock::get))
        .route("/set_stock", post(stock::set))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(middleware::from_fn(request_logger))
        .with_state(state)
}

pub async fn serve(port: u16, ip: Ipv4Addr, state: AppState) {
    let address = format!("{ip}:{port}");
    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(error) => {
            error!("Failed to bind {address}: {error}");
            return;
        }
    };

    if let Err(error) = axum::serve(listener, router(state)).await {
        error!("API server stopped: {error}");
    }

    info!("Listening on {address}");
}
