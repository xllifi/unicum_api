use axum::{Json, extract::State};

use crate::{
    api::{ApiError, AppState},
    contracts::Contracts,
    entities::Machine,
};

pub async fn get(State(state): State<AppState>) -> Result<Json<Vec<Machine>>, ApiError> {
    let machines = state.unicum.lock().await.get_machines().await?;

    Ok(Json(machines))
}
