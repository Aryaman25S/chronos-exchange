use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type MarketId = String;
pub type OrderId = Uuid;
pub type UserId = Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tif {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub name: String,
    pub tick_size: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrder {
    pub id: OrderId,
    pub user_id: UserId,
    pub market_id: MarketId,
    pub side: Side,
    pub price: u32,
    pub qty: u32,
    pub tif: Tif,
    pub idempotency: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceOrder {
    pub market_id: MarketId,
    pub order_id: OrderId,
    pub new_price: Option<u32>,
    pub new_qty: Option<u32>,
}
/// Result of [`crate::orderbook::OrderBook::place`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceResult {
    pub fills: Vec<Fill>,
    /// True if the matcher hit self-trade prevention (incoming canceled vs own quote).
    pub self_trade_prevented: bool,
    /// True if the remaining size was posted to the book (GTC).
    pub rested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub market_id: MarketId,
    pub taker_order_id: OrderId,
    pub maker_order_id: OrderId,
    pub price: u32,
    pub qty: u32,
    pub buyer: UserId,
    pub seller: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Level {
    pub price: u32,
    pub qty: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Book {
    pub bids: Vec<L2Level>,
    pub asks: Vec<L2Level>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub price: u32,
    pub qty: u32,
    pub ts: u64,
}
/// Resting (open) order for a user, for API listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrderRow {
    pub market_id: MarketId,
    pub order_id: OrderId,
    pub side: Side,
    pub price: u32,
    pub qty: u32,
}

impl From<&NewOrder> for Order {
    fn from(o: &NewOrder) -> Self {
        // Deterministic so snapshot+WAL replay yields identical `state_hash`.
        let ts = o.id.as_u128() as u64;
        Self {
            id: o.id,
            user_id: o.user_id,
            price: o.price,
            qty: o.qty,
            ts,
        }
    }
}
