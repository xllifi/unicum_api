use async_trait::async_trait;

mod error;
pub use error::ContractError;

use super::entities::*;

#[async_trait]
pub trait Contracts {
    async fn get_machines(&mut self) -> Result<Vec<Machine>, ContractError>;
    async fn get_state(&mut self, machine_id: MachineId) -> Result<MachineState, ContractError>;
    async fn get_sales(
        &mut self,
        machine_id: MachineId,
        since: i64,
        before: i64,
    ) -> Result<MachineSales, ContractError>;

    async fn get_stock(&mut self, machine_id: MachineId) -> Result<MachineStock, ContractError>;
    async fn set_stock(
        &mut self,
        machine_id: MachineId,
        new_stock: MachineStock,
        target: SetStockForWhichEncashment,
    ) -> Result<(), ContractError>;
}
