use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    contracts::{ContractError, Contracts},
    entities::{
        Machine, MachineId, MachineSales, MachineState, MachineStock, SetStockForWhichEncashment,
    },
};

use super::UnicumApi;

#[async_trait]
impl Contracts for UnicumApi {
    async fn get_machines(&mut self) -> Result<Vec<Machine>, ContractError> {
        self.get_machines_upstream().await.map_err(Into::into)
    }
    async fn get_state(&mut self, machine_id: MachineId) -> Result<MachineState, ContractError> {
        self.get_state_upstream(machine_id)
            .await
            .map_err(Into::into)
    }

    async fn get_sales(
        &mut self,
        machine_id: MachineId,
        since: i64,
        until: i64,
    ) -> Result<MachineSales, ContractError> {
        let since_date = OffsetDateTime::from_unix_timestamp(since)
            .map_err(|error| ContractError::Parse {
                cause: error.to_string(),
            })?
            .date();
        let until_date = OffsetDateTime::from_unix_timestamp(until)
            .map_err(|error| ContractError::Parse {
                cause: error.to_string(),
            })?
            .date();
        self.get_sales_upstream(machine_id, since_date, until_date)
            .await
            .map_err(Into::into)
    }

    async fn get_stock(&mut self, machine_id: MachineId) -> Result<MachineStock, ContractError> {
        self.get_stock_upstream(machine_id)
            .await
            .map_err(Into::into)
    }

    async fn set_stock(
        &mut self,
        machine_id: MachineId,
        stock: MachineStock,
        target: SetStockForWhichEncashment,
    ) -> Result<(), ContractError> {
        self.set_stock_upstream(machine_id, stock, target)
            .await
            .map_err(Into::into)
    }
}
