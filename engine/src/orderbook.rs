
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, VecDeque};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookState {
    pub market: Market,
    pub bids: BTreeMap<u32, VecDeque<Order>>,
    pub asks: BTreeMap<u32, VecDeque<Order>>,
    pub last_trade: Option<u32>,
    pub seq: u64,
    pub settled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct OrderBook { pub state: BookState }

impl OrderBook {
    pub fn new(market: Market) -> Self {
        Self { state: BookState { market, bids: BTreeMap::new(), asks: BTreeMap::new(), last_trade: None, seq: 0, settled: None } }
    }
    fn best_bid(&self) -> Option<u32> { self.state.bids.keys().rev().next().copied() }
    fn best_ask(&self) -> Option<u32> { self.state.asks.keys().next().copied() }

    pub fn l2(&self, depth: usize) -> L2Book {
        let bids = self.state.bids.iter().rev().take(depth).map(|(p,q)| L2Level{price:*p, qty:q.iter().map(|o| o.qty as u64).sum()}).collect();
        let asks = self.state.asks.iter().take(depth).map(|(p,q)| L2Level{price:*p, qty:q.iter().map(|o| o.qty as u64).sum()}).collect();
        L2Book { bids, asks }
    }

    pub fn place(&mut self, o: NewOrder) -> Result<Vec<Fill>> {
        if self.state.settled.is_some() { return Ok(vec![]); }
        let mut incoming = Order::from(&o);
        let mut fills = vec![];
        let cross = || match o.side {
            Side::Buy => self.best_ask().map(|a| incoming.price >= a).unwrap_or(false),
            Side::Sell => self.best_bid().map(|b| incoming.price <= b).unwrap_or(false),
        };
        while incoming.qty > 0 && cross() {
            match o.side {
                Side::Buy => {
                    let best_ask = self.best_ask().unwrap();
                    let q = self.state.asks.get_mut(&best_ask).unwrap();
                    if let Some(maker) = q.front_mut() {
                        if maker.user_id == incoming.user_id { incoming.qty = 0; break; }
                        let traded = maker.qty.min(incoming.qty);
                        maker.qty -= traded; incoming.qty -= traded;
                        self.state.last_trade = Some(best_ask); self.state.seq += 1;
                        fills.push(Fill{ market_id:o.market_id.clone(), taker_order_id:o.id, maker_order_id:maker.id, price:best_ask, qty:traded, buyer:incoming.user_id, seller:maker.user_id });
                        if maker.qty == 0 { q.pop_front(); } if q.is_empty() { self.state.asks.remove(&best_ask); }
                    }
                }
                Side::Sell => {
                    let best_bid = self.best_bid().unwrap();
                    let q = self.state.bids.get_mut(&best_bid).unwrap();
                    if let Some(maker) = q.front_mut() {
                        if maker.user_id == incoming.user_id { incoming.qty = 0; break; }
                        let traded = maker.qty.min(incoming.qty);
                        maker.qty -= traded; incoming.qty -= traded;
                        self.state.last_trade = Some(best_bid); self.state.seq += 1;
                        fills.push(Fill{ market_id:o.market_id.clone(), taker_order_id:o.id, maker_order_id:maker.id, price:best_bid, qty:traded, buyer:maker.user_id, seller:incoming.user_id });
                        if maker.qty == 0 { q.pop_front(); } if q.is_empty() { self.state.bids.remove(&best_bid); }
                    }
                }
            }
        }
        match o.tif {
            Tif::Fok => { incoming.qty = 0; }
            Tif::Ioc => { incoming.qty = 0; }
            Tif::Gtc => {
                if incoming.qty > 0 {
                    match o.side {
                        Side::Buy => self.state.bids.entry(incoming.price).or_default().push_back(incoming),
                        Side::Sell => self.state.asks.entry(incoming.price).or_default().push_back(incoming),
                    }
                    self.state.seq += 1;
                }
            }
        }
        Ok(fills)
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Result<()> {
        let mut removed = false;
        for q in self.state.bids.values_mut() {
            if let Some(pos) = q.iter().position(|o| o.id == order_id) { q.remove(pos); removed = true; break; }
        }
        if !removed {
            for q in self.state.asks.values_mut() {
                if let Some(pos) = q.iter().position(|o| o.id == order_id) { q.remove(pos); removed = true; break; }
            }
        }
        if removed { self.state.seq += 1; }
        Ok(())
    }

    pub fn replace(&mut self, r: ReplaceOrder) -> Result<()> {
        Ok(()) // simplified placeholder
    }

    pub fn settle(&mut self, yes: bool) { self.state.settled = Some(yes); self.state.seq += 1; }
}
