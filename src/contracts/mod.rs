use async_trait::async_trait;

use super::entities::{InternalError, MachineId, Sales, SetStockTarget, State, Stock};

#[async_trait]
pub trait Contracts {
    async fn get_state(&mut self, machine_id: MachineId) -> Result<State, InternalError>;
    async fn get_sales(
        &mut self,
        machine_id: MachineId,
        since: i64,
        before: i64,
    ) -> Result<Sales, InternalError>;

    async fn get_stock(&mut self, machine_id: MachineId) -> Result<Stock, InternalError>;
    async fn set_stock(
        &mut self,
        machine_id: MachineId,
        new_stock: Stock,
        target: SetStockTarget,
    ) -> Result<(), InternalError>;
}
