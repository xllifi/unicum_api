use core::fmt;
use std::fmt::Display;

use bitflags::bitflags;
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    #[sqlx(try_from = "i32")]
    pub permissions: Permissions,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Permissions: u32 {
        const STOCK_READ  = 0b0000_0001;
        const STOCK_WRITE = 0b0000_0010;
        const STATE_READ  = 0b0000_0100;
        const SALES_READ  = 0b0000_1000;
    }
}

impl Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl TryFrom<i32> for Permissions {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let bits = value as u32;
        Permissions::from_bits(bits).ok_or("Invalid bitmask value".into())
    }
}

impl From<Permissions> for i32 {
    fn from(value: Permissions) -> Self {
        value.bits() as i32
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InternalError {
    NetworkError { cause: String },
    ParseError { cause: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrontFacingError {
    NoCredentials,
    InvadlidCredentials,
}
