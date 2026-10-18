use sqlx::FromRow;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub category: String,
    pub price: Decimal,
    pub in_stock: bool,
    pub updated_at: DateTime<Utc>,
}
