pub mod types;
pub mod orderbook;
pub mod wal;
pub mod snapshot;
pub mod hash;

use parking_lot::Mutex;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tracing::info;

use orderbook::OrderBook;
use snapshot::Snapshotter;
use types::*;
use wal::{Wal, WalRecord};

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

    pub fn place_order(&self, o: NewOrder) -> anyhow::Result<Vec<Fill>> {
        let mut g = self.inner.lock();
        // WAL first, then borrow the book mutably
        g.wal.append(&WalRecord::Place(o.clone()))?;
        let fills = {
            let book = g
                .markets
                .get_mut(&o.market_id)
                .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
            book.place(o.clone())?
        };
        Ok(fills)
    }

    pub fn cancel_order(&self, market_id: MarketId, order_id: OrderId) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        g.wal
            .append(&WalRecord::Cancel { market_id: market_id.clone(), order_id })?;
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
        {
            let book = g
                .markets
                .get_mut(&r.market_id)
                .ok_or_else(|| anyhow::anyhow!("unknown market"))?;
            book.replace(r)?;
        }
        Ok(())
    }

    pub fn snapshot_all(&self) -> anyhow::Result<()> {
        let g = self.inner.lock();
        g.snap.write_all(&g.markets)?;
        Ok(())
    }

    pub fn state_hash(&self) -> anyhow::Result<[u8; 32]> {
        let g = self.inner.lock();
        Ok(crate::hash::state_hash(&g.markets))
    }

    pub fn restore_from_latest(&self) -> anyhow::Result<()> {
        let mut g = self.inner.lock();
        if let Some(state) = g.snap.try_read_latest()? {
            g.markets = state;
        }
        // Collect WAL records first, then apply them — avoids nested borrows.
        let mut recs = Vec::new();
        g.wal.replay(|rec| {
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
                WalRecord::Cancel { market_id, order_id } => {
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
        info!("Restored from snapshot+WAL");
        Ok(())
    }

    /// Safe accessor for WS/MD paths: returns (L2Book, last_trade, seq)
    pub fn get_market_snapshot(
        &self,
        market_id: &str,
        depth: usize,
    ) -> Option<(L2Book, Option<u32>, u64)> {
        let g = self.inner.lock();
        g.markets.get(market_id).map(|ob| (ob.l2(depth), ob.state.last_trade, ob.state.seq))
    }
}