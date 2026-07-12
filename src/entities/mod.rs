use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type MachineId = Uuid;

pub type State = Vec<Slot>;
pub type Sales = Vec<Sale>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotId {
    pub row: u8,
    pub col: u8,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub id: SlotId,
    pub price: f32,

    /// How many items can be in this slot
    pub max: u8,
    /// How many items are in this slot now
    pub cur: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub date: Date,
    pub slot_id: SlotId,
    /// Slot's price at the time of sale
    pub price: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: i32,
}

pub type Stock = Vec<StockSlot>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSlot {
    pub row: u8,
    pub col: u8,
    pub mapped_to: String,
    pub cur: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetStockTarget {
    Latest,
    Future,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    NetworkError { cause: String },
    ParseError { cause: String },
}