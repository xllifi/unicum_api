use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    api::auth::RequirePermissions,
    api::{ApiError, AppState},
    contracts::Contracts,
    entities::{MachineId, MachineStock, Permissions, SetStockForWhichEncashment},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineRequest {
    pub machine_id: MachineId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStockRequest {
    pub machine_id: MachineId,
    pub stock: MachineStock,
    pub target: SetStockForWhichEncashment,
}

pub async fn get(
    _auth: RequirePermissions<{ Permissions::STOCK_READ.bits() }>,
    State(state): State<AppState>,
    Json(request): Json<MachineRequest>,
) -> Result<Json<MachineStock>, ApiError> {
    let stock = state
        .unicum
        .lock()
        .await
        .get_stock(request.machine_id)
        .await?;

    Ok(Json(stock))
}

pub async fn set(
    _auth: RequirePermissions<{ Permissions::STOCK_WRITE.bits() }>,
    State(state): State<AppState>,
    Json(request): Json<SetStockRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .unicum
        .lock()
        .await
        .set_stock(request.machine_id, request.stock, request.target)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
