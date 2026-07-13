use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Product {
    #[serde(rename = "ingredient")]
    Ingredient(IngredientProduct),

    #[serde(rename = "out")]
    Out(OutProduct),

    #[serde(rename = "coffee")]
    Coffee(Box<CoffeeProduct>),

    #[serde(rename = "snack")]
    Snack(Box<SnackProduct>),

    #[serde(rename = "combo")]
    Combo(Box<ComboProduct>),
}

// =========================================================================
// Shared Base Components
// =========================================================================

/// Core fields shared by Coffee, Snacks, and Combos
#[derive(Debug, Serialize, Deserialize)]
pub struct CommonProductFields {
    #[serde(rename = "productID")]
    pub product_id: u16,
    pub selection: String,
    pub name: String,
    pub guid: Option<String>,
    pub max: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i8>,
    pub vends: u32,
    pub blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failures: Option<u32>,
    pub decimal: u8,
    pub price: i64,
    pub price_cl1: i64,
    pub price_cl2: i64,
    pub price_cl3: i64,
    pub ingredients: Vec<f64>,
}

/// Dynamic metrics only populated for standalone single items (Coffee/Snacks)
#[derive(Debug, Serialize, Deserialize)]
pub struct SingleItemMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub av_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cl1_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub av_price_cl1: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cl2_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub av_price_cl2: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cl3_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub av_price_cl3: Option<i64>,
}

// =========================================================================
// Type-Specific Structural Payloads
// =========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CoffeeProduct {
    #[serde(flatten)]
    pub common: CommonProductFields,

    #[serde(rename = "articleID", skip_serializing_if = "Option::is_none")]
    pub article_id: Option<u16>,

    #[serde(flatten)]
    pub metrics: SingleItemMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnackProduct {
    #[serde(flatten)]
    pub common: CommonProductFields,

    /// Используется для соединения ячеек спиралей (0 = НЕ СОЕДИНЯТЬ)
    #[serde(rename = "articleID", skip_serializing_if = "Option::is_none")]
    pub article_id: Option<u16>,

    #[serde(flatten)]
    pub metrics: SingleItemMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComboProduct {
    #[serde(flatten)]
    pub common: CommonProductFields,

    #[serde(rename = "articleID", skip_serializing_if = "Option::is_none")]
    pub article_id: Option<u16>,

    /// Состав комбо-продажи (Отсутствует у Coffee/Snack)
    pub contents: Vec<ComboContent>,
}

// =========================================================================
// Minor Supporting Structs
// =========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientProduct {
    pub selection: String,
    pub name: String,
    pub guid: Option<String>,
    pub max: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<f64>,
    pub vends: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutProduct {
    #[serde(rename = "productID")]
    pub product_id: u16,
    pub vends: u32,
    pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComboContent {
    #[serde(rename = "type")]
    pub item_type: ComboItemType,
    pub selection: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComboItemType {
    Coffee,
    Snack,
}
