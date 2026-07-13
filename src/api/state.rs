use axum::{Json, extract::State};

use crate::{
    api::stock::MachineRequest,
    api::{ApiError, AppState},
    contracts::Contracts,
    entities::MachineState,
};

pub async fn get(
    State(state): State<AppState>,
    Json(request): Json<MachineRequest>,
) -> Result<Json<MachineState>, ApiError> {
    let machine_state = state
        .unicum
        .lock()
        .await
        .get_state(request.machine_id)
        .await?;

    Ok(Json(machine_state))
}
