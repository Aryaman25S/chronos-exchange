use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

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
pub struct OrderBook {
    pub state: BookState,
}

impl OrderBook {
    pub fn new(market: Market) -> Self {
        Self {
            state: BookState {
                market,
                bids: BTreeMap::new(),
                asks: BTreeMap::new(),
                last_trade: None,
                seq: 0,
                settled: None,
            },
        }
    }

    #[inline]
    fn best_bid(&self) -> Option<u32> {
        self.state.bids.keys().rev().next().copied()
    }
    #[inline]
    fn best_ask(&self) -> Option<u32> {
        self.state.asks.keys().next().copied()
    }

    /// Full fill possible without self-trade blocking completion (read-only).
    pub fn can_fok_fill(&self, o: &NewOrder) -> bool {
        let mut need = o.qty;
        if need == 0 {
            return true;
        }
        match o.side {
            Side::Buy => {
                for (&price, queue) in self.state.asks.iter() {
                    if price > o.price {
                        break;
                    }
                    for maker in queue.iter() {
                        if maker.user_id == o.user_id {
                            return false;
                        }
                        let take = need.min(maker.qty);
                        need -= take;
                        if need == 0 {
                            return true;
                        }
                    }
                }
                false
            }
            Side::Sell => {
                for (&price, queue) in self.state.bids.iter().rev() {
                    if price < o.price {
                        break;
                    }
                    for maker in queue.iter() {
                        if maker.user_id == o.user_id {
                            return false;
                        }
                        let take = need.min(maker.qty);
                        need -= take;
                        if need == 0 {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    pub fn l2(&self, depth: usize) -> L2Book {
        let bids = self
            .state
            .bids
            .iter()
            .rev()
            .take(depth)
            .map(|(p, q)| L2Level {
                price: *p,
                qty: q.iter().map(|o| o.qty as u64).sum(),
            })
            .collect();
        let asks = self
            .state
            .asks
            .iter()
            .take(depth)
            .map(|(p, q)| L2Level {
                price: *p,
                qty: q.iter().map(|o| o.qty as u64).sum(),
            })
            .collect();
        L2Book { bids, asks }
    }

    /// Resting GTC orders for `user` on both sides.
    pub fn resting_orders_for_user(&self, user: UserId) -> Vec<(Side, Order)> {
        let mut out = Vec::new();
        for q in self.state.bids.values() {
            for o in q.iter() {
                if o.user_id == user && o.qty > 0 {
                    out.push((Side::Buy, o.clone()));
                }
            }
        }
        for q in self.state.asks.values() {
            for o in q.iter() {
                if o.user_id == user && o.qty > 0 {
                    out.push((Side::Sell, o.clone()));
                }
            }
        }
        out
    }

    /// Price–time priority matching with STP (cancel incoming vs self).
    pub fn place(&mut self, o: NewOrder) -> Result<PlaceResult> {
        if self.state.settled.is_some() {
            return Ok(PlaceResult {
                fills: vec![],
                self_trade_prevented: false,
                rested: false,
            });
        }
        if matches!(o.tif, Tif::Fok) && !self.can_fok_fill(&o) {
            return Ok(PlaceResult {
                fills: vec![],
                self_trade_prevented: false,
                rested: false,
            });
        }
        let mut incoming = Order::from(&o);
        let mut fills = Vec::new();
        let mut stp_hit = false;

        while incoming.qty > 0 {
            let is_cross = match o.side {
                Side::Buy => self
                    .best_ask()
                    .map(|a| incoming.price >= a)
                    .unwrap_or(false),
                Side::Sell => self
                    .best_bid()
                    .map(|b| incoming.price <= b)
                    .unwrap_or(false),
            };
            if !is_cross {
                break;
            }

            match o.side {
                Side::Buy => {
                    let best_ask = match self.best_ask() {
                        Some(px) => px,
                        None => break,
                    };

                    let trade: Option<(OrderId, UserId, u32, u32)> = {
                        let q = self.state.asks.get_mut(&best_ask).unwrap();
                        if let Some(maker) = q.front_mut() {
                            if maker.user_id == incoming.user_id {
                                stp_hit = true;
                                incoming.qty = 0;
                                None
                            } else {
                                let traded = maker.qty.min(incoming.qty);
                                maker.qty -= traded;
                                incoming.qty -= traded;

                                let maker_id = maker.id;
                                let maker_user = maker.user_id;
                                if maker.qty == 0 {
                                    q.pop_front();
                                }
                                Some((maker_id, maker_user, traded, best_ask))
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(q) = self.state.asks.get(&best_ask) {
                        if q.is_empty() {
                            self.state.asks.remove(&best_ask);
                        }
                    }

                    if let Some((maker_id, maker_user, traded_qty, px)) = trade {
                        self.state.last_trade = Some(px);
                        self.state.seq += 1;
                        fills.push(Fill {
                            market_id: o.market_id.clone(),
                            taker_order_id: o.id,
                            maker_order_id: maker_id,
                            price: px,
                            qty: traded_qty,
                            buyer: incoming.user_id,
                            seller: maker_user,
                        });
                    } else if incoming.qty == 0 {
                        break;
                    }
                }

                Side::Sell => {
                    let best_bid = match self.best_bid() {
                        Some(px) => px,
                        None => break,
                    };

                    let trade: Option<(OrderId, UserId, u32, u32)> = {
                        let q = self.state.bids.get_mut(&best_bid).unwrap();
                        if let Some(maker) = q.front_mut() {
                            if maker.user_id == incoming.user_id {
                                stp_hit = true;
                                incoming.qty = 0;
                                None
                            } else {
                                let traded = maker.qty.min(incoming.qty);
                                maker.qty -= traded;
                                incoming.qty -= traded;

                                let maker_id = maker.id;
                                let maker_user = maker.user_id;
                                if maker.qty == 0 {
                                    q.pop_front();
                                }
                                Some((maker_id, maker_user, traded, best_bid))
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(q) = self.state.bids.get(&best_bid) {
                        if q.is_empty() {
                            self.state.bids.remove(&best_bid);
                        }
                    }

                    if let Some((maker_id, maker_user, traded_qty, px)) = trade {
                        self.state.last_trade = Some(px);
                        self.state.seq += 1;
                        fills.push(Fill {
                            market_id: o.market_id.clone(),
                            taker_order_id: o.id,
                            maker_order_id: maker_id,
                            price: px,
                            qty: traded_qty,
                            buyer: maker_user,
                            seller: incoming.user_id,
                        });
                    } else if incoming.qty == 0 {
                        break;
                    }
                }
            }
        }

        let mut rested = false;
        match o.tif {
            Tif::Fok => {
                incoming.qty = 0;
            }
            Tif::Ioc => {
                incoming.qty = 0;
            }
            Tif::Gtc => {
                if incoming.qty > 0 {
                    rested = true;
                    match o.side {
                        Side::Buy => self
                            .state
                            .bids
                            .entry(incoming.price)
                            .or_default()
                            .push_back(incoming),
                        Side::Sell => self
                            .state
                            .asks
                            .entry(incoming.price)
                            .or_default()
                            .push_back(incoming),
                    }
                    self.state.seq += 1;
                }
            }
        }

        Ok(PlaceResult {
            fills,
            self_trade_prevented: stp_hit,
            rested,
        })
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Result<()> {
        let mut removed = false;

        for q in self.state.bids.values_mut() {
            if let Some(pos) = q.iter().position(|o| o.id == order_id) {
                q.remove(pos);
                removed = true;
                break;
            }
        }
        if !removed {
            for q in self.state.asks.values_mut() {
                if let Some(pos) = q.iter().position(|o| o.id == order_id) {
                    q.remove(pos);
                    removed = true;
                    break;
                }
            }
        }
        if removed {
            self.state.seq += 1;
        }
        Ok(())
    }

    /// Cancel and re-place at new price/qty (loses time priority at new level).
    pub fn replace(&mut self, r: ReplaceOrder) -> Result<()> {
        if self.state.settled.is_some() {
            return Ok(());
        }
        let mut found: Option<(Side, Order)> = None;
        for q in self.state.bids.values_mut() {
            if let Some(pos) = q.iter().position(|o| o.id == r.order_id) {
                let o = q.remove(pos).unwrap();
                found = Some((Side::Buy, o));
                break;
            }
        }
        if found.is_none() {
            for q in self.state.asks.values_mut() {
                if let Some(pos) = q.iter().position(|o| o.id == r.order_id) {
                    let o = q.remove(pos).unwrap();
                    found = Some((Side::Sell, o));
                    break;
                }
            }
        }
        let (side, old) = found.ok_or_else(|| anyhow::anyhow!("order not found"))?;

        let empty_bids: Vec<u32> = self
            .state
            .bids
            .iter()
            .filter(|(_, q)| q.is_empty())
            .map(|(p, _)| *p)
            .collect();
        for p in empty_bids {
            self.state.bids.remove(&p);
        }
        let empty_asks: Vec<u32> = self
            .state
            .asks
            .iter()
            .filter(|(_, q)| q.is_empty())
            .map(|(p, _)| *p)
            .collect();
        for p in empty_asks {
            self.state.asks.remove(&p);
        }

        self.state.seq += 1;

        let new_price = r.new_price.unwrap_or(old.price);
        let new_qty = r.new_qty.unwrap_or(old.qty);
        if new_qty == 0 {
            anyhow::bail!("invalid new_qty");
        }
        let new_o = NewOrder {
            id: r.order_id,
            user_id: old.user_id,
            market_id: r.market_id,
            side,
            price: new_price,
            qty: new_qty,
            tif: Tif::Gtc,
            idempotency: None,
        };
        let _ = self.place(new_o)?;
        Ok(())
    }

    pub fn settle(&mut self, yes: bool) {
        self.state.settled = Some(yes);
        self.state.seq += 1;
    }
}
