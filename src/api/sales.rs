use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    api::auth::RequirePermissions,
    api::{ApiError, AppState},
    contracts::Contracts,
    entities::{MachineId, MachineSales, Permissions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSalesRequest {
    pub machine_id: MachineId,
    pub since: i64,
    pub until: i64,
}

pub async fn get(
    _auth: RequirePermissions<{ Permissions::SALES_READ.bits() }>,
    State(state): State<AppState>,
    Json(request): Json<GetSalesRequest>,
) -> Result<Json<MachineSales>, ApiError> {
    let sales = state
        .unicum
        .lock()
        .await
        .get_sales(request.machine_id, request.since, request.until)
        .await?;

    Ok(Json(sales))
}
