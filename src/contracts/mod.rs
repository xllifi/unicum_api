use std::collections::HashMap;

use crate::entities::{Error, MachineId, Sales, SetStockTarget, SlotId, State, Stock};

pub trait Contracts {
    async fn get_state(&mut self, machine_id: MachineId) -> Result<State, Error>;
    async fn get_sales(&mut self, machine_id: MachineId, since: i64, before: i64) -> Result<Sales, Error>;

    async fn get_stock(&mut self, machine_id: MachineId) -> Result<Stock, Error>;
    async fn set_stock(&mut self, machine_id: MachineId, new_stock: Stock, target: SetStockTarget) -> Result<(), Error>;
}