use engine::types::{Fill, Side, UserId};
use parking_lot::Mutex;
use risk::Caps;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MarketPosition {
    /// Net YES contracts (positive = long YES, negative = short).
    pub qty_yes: i64,
    /// Volume-weighted average entry in cents (meaningful when qty sign matches).
    pub avg_price_cents: i64,
}

#[derive(Default)]
pub struct Ledger {
    positions: Mutex<HashMap<(UserId, String), MarketPosition>>,
    fills: Mutex<Vec<Fill>>,
    realized_pnl_cents: Mutex<HashMap<UserId, i64>>,
}

impl Ledger {
    pub fn check_intent(
        &self,
        user: UserId,
        side: Side,
        qty: u32,
        price_cents: i64,
        market_id: &str,
        caps: &Caps,
    ) -> anyhow::Result<()> {
        let delta = match side {
            Side::Buy => qty as i64,
            Side::Sell => -(qty as i64),
        };
        let p = self.positions.lock();
        let key = (user, market_id.to_string());
        let cur = p.get(&key).map(|m| m.qty_yes).unwrap_or(0);
        let new = cur + delta;
        if new.abs() > caps.max_position {
            anyhow::bail!("position cap");
        }
        if new.abs() * price_cents > caps.max_notional_cents {
            anyhow::bail!("notional cap");
        }
        Ok(())
    }

    pub fn apply_fills(&self, fills: &[Fill]) {
        if fills.is_empty() {
            return;
        }
        let mut p = self.positions.lock();
        let mut log = self.fills.lock();
        for f in fills {
            log.push(f.clone());
            adjust_for_user(&mut p, f.buyer, &f.market_id, f.qty as i64, f.price as i64);
            adjust_for_user(
                &mut p,
                f.seller,
                &f.market_id,
                -(f.qty as i64),
                f.price as i64,
            );
        }
    }

    pub fn positions_for_user(&self, user: UserId) -> HashMap<String, MarketPosition> {
        let p = self.positions.lock();
        p.iter()
            .filter(|((u, _), _)| *u == user)
            .map(|((_, m), pos)| (m.clone(), pos.clone()))
            .collect()
    }

    pub fn recent_fills(&self, limit: usize) -> Vec<Fill> {
        let log = self.fills.lock();
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Binary payoff: YES settles to 100¢, NO to 0¢ per YES contract.
    pub fn apply_settlement(&self, market_id: &str, resolve_yes: bool) {
        let settle_px = if resolve_yes { 100i64 } else { 0i64 };
        let mut p = self.positions.lock();
        let mut pnl = self.realized_pnl_cents.lock();
        let keys: Vec<_> = p.keys().filter(|(_, m)| m == market_id).cloned().collect();
        for key in keys {
            let Some(pos) = p.get(&key).cloned() else {
                continue;
            };
            if pos.qty_yes == 0 {
                continue;
            }
            // Mark-to-settlement: position valued at settle_px vs avg entry.
            let u = key.0;
            let pnl_move = pos.qty_yes * (settle_px - pos.avg_price_cents);
            *pnl.entry(u).or_insert(0) += pnl_move;
            p.remove(&key);
        }
    }
}

fn adjust_for_user(
    p: &mut HashMap<(UserId, String), MarketPosition>,
    user: UserId,
    market_id: &str,
    dq: i64,
    price: i64,
) {
    let key = (user, market_id.to_string());
    let e = p.entry(key).or_insert_with(|| MarketPosition {
        qty_yes: 0,
        avg_price_cents: 0,
    });
    let old = e.qty_yes;
    let new = old + dq;
    if old == 0 || old.signum() == dq.signum() {
        let num = e.avg_price_cents * old.abs() + price * dq.abs();
        let den = new.abs();
        e.qty_yes = new;
        e.avg_price_cents = if den > 0 { num / den } else { 0 };
        return;
    }
    // Reducing or flipping: set remaining at new avg = price for residual.
    e.qty_yes = new;
    e.avg_price_cents = if new == 0 { 0 } else { price };
}
