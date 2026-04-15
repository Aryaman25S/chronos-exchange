pub mod hash;
pub mod orderbook;
pub mod snapshot;
pub mod types;
pub mod wal;

use parking_lot::Mutex;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tracing::info;

use orderbook::OrderBook;
use snapshot::Snapshotter;
use types::*;
use wal::{Wal, WalRecord};

pub use types::PlaceResult;
pub use wal::SettleRec;

#[derive(Clone)]
pub struct Engine {
    pub(crate) inner: Arc<Mutex<Inner>>,
}

pub(crate) struct Inner {
    pub markets: HashMap<MarketId, OrderBook>,
    pub wal: Wal,
    pub snap: Snapshotter,
}

impl Engine {
    pub fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let wal = Wal::new(data_dir.join("wal"))?;
        let snap = Snapshotter::new(data_dir.join("snapshots"))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                markets: HashMap::new(),
                wal,
                snap,
            })),
        })
    }

    pub fn ensure_market(&self, market: Market) {
        let mut g = self.inner.lock();
        g.markets
            .entry(market.id.clone())
            .or_insert_with(|| OrderBook::new(market));
    }

    pub fn place_order(&self, o: NewOrder) -> anyhow::Result<PlaceResult> {
        let mut g = self.inner.lock();
        {
            let book = g
                .markets
                .get_mut(&o.market_id)
                .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
            if matches!(o.tif, Tif::Fok) && !book.can_fok_fill(&o) {
                return Ok(PlaceResult {
                    fills: vec![],
                    self_trade_prevented: false,
                    rested: false,
                });
            }
        }
        g.wal.append(&WalRecord::Place(o.clone()))?;
        let out = {
            let book = g
                .markets
                .get_mut(&o.market_id)
                .expect("market exists after check");
            book.place(o)?
        };
        Ok(out)
    }

    pub fn cancel_order(&self, market_id: MarketId, order_id: OrderId) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        g.wal.append(&WalRecord::Cancel {
            market_id: market_id.clone(),
            order_id,
        })?;
        {
            let book = g
                .markets
                .get_mut(&market_id)
                .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
            book.cancel(order_id)?;
        }
        Ok(())
    }

    pub fn replace_order(&self, r: ReplaceOrder) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        g.wal.append(&WalRecord::Replace(r.clone()))?;
        let book = g
            .markets
            .get_mut(&r.market_id)
            .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
        book.replace(r)?;
        Ok(())
    }

    pub fn settle_market(&self, market_id: MarketId, resolve_yes: bool) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        g.wal.append(&WalRecord::Settle {
            s: crate::wal::SettleRec {
                market_id: market_id.clone(),
                resolve_yes,
            },
        })?;
        let book = g
            .markets
            .get_mut(&market_id)
            .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
        book.settle(resolve_yes);
        Ok(())
    }

    pub fn snapshot_all(&self) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        g.wal.flush()?;
        let wal_end = g.wal.len_bytes()?;
        g.snap.write_all(&g.markets, wal_end)?;
        Ok(())
    }

    pub fn state_hash(&self) -> anyhow::Result<[u8; 32]> {
        let g = self.inner.lock();
        Ok(crate::hash::state_hash(&g.markets))
    }

    pub fn restore_from_latest(&self) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        let mut wal_start = 0u64;
        if let Some((state, wal_off)) = g.snap.try_read_latest()? {
            g.markets = state;
            wal_start = wal_off;
        }
        let mut recs = Vec::new();
        g.wal.replay_from(wal_start, |rec| {
            recs.push(rec);
            Ok(())
        })?;
        for rec in recs {
            match rec {
                WalRecord::Place(o) => {
                    if let Some(book) = g.markets.get_mut(&o.market_id) {
                        let _ = book.place(o);
                    }
                }
                WalRecord::Cancel {
                    market_id,
                    order_id,
                } => {
                    if let Some(book) = g.markets.get_mut(&market_id) {
                        let _ = book.cancel(order_id);
                    }
                }
                WalRecord::Replace(r) => {
                    if let Some(book) = g.markets.get_mut(&r.market_id) {
                        let _ = book.replace(r);
                    }
                }
                WalRecord::Settle { s } => {
                    if let Some(book) = g.markets.get_mut(&s.market_id) {
                        book.settle(s.resolve_yes);
                    }
                }
            }
        }
        info!("Restored from snapshot (WAL replay from byte {})", wal_start);
        Ok(())
    }

    /// Safe accessor for WS/MD paths: returns (L2Book, last_trade, seq)
    pub fn get_market_snapshot(
        &self,
        market_id: &str,
        depth: usize,
    ) -> Option<(L2Book, Option<u32>, u64)> {
        let g = self.inner.lock();
        g.markets
            .get(market_id)
            .map(|ob| (ob.l2(depth), ob.state.last_trade, ob.state.seq))
    }

    /// Markets with settlement status (`None` = open).
    pub fn list_markets_detail(&self) -> Vec<(Market, Option<bool>)> {
        let g = self.inner.lock();
        let mut v: Vec<_> = g
            .markets
            .values()
            .map(|ob| (ob.state.market.clone(), ob.state.settled))
            .collect();
        v.sort_by(|a, b| a.0.id.cmp(&b.0.id));
        v
    }

    /// Open resting orders for `user_id`, optionally scoped to one market.
    pub fn resting_orders_for_user(
        &self,
        user_id: UserId,
        market_id: Option<&str>,
    ) -> Vec<OpenOrderRow> {
        let g = self.inner.lock();
        let mut rows = Vec::new();
        let keys: Vec<String> = match market_id {
            Some(m) => vec![m.to_string()],
            None => g.markets.keys().cloned().collect(),
        };
        for mid in keys {
            let Some(ob) = g.markets.get(&mid) else {
                continue;
            };
            for (side, o) in ob.resting_orders_for_user(user_id) {
                rows.push(OpenOrderRow {
                    market_id: mid.clone(),
                    order_id: o.id,
                    side,
                    price: o.price,
                    qty: o.qty,
                });
            }
        }
        rows
    }
}
