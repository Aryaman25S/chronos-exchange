
pub mod types;
pub mod orderbook;
pub mod wal;
pub mod snapshot;
pub mod hash;

use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc, path::PathBuf};
use types::*;
use orderbook::OrderBook;
use wal::{Wal, WalRecord};
use snapshot::Snapshotter;

#[derive(Clone)]
pub struct Engine { pub(crate) inner: Arc<Mutex<Inner>> }
pub(crate) struct Inner { pub markets: HashMap<MarketId, OrderBook>, pub wal: Wal, pub snap: Snapshotter }

impl Engine {
    pub fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let wal = Wal::new(data_dir.join("wal"))?;
        let snap = Snapshotter::new(data_dir.join("snapshots"))?;
        Ok(Self { inner: Arc::new(Mutex::new(Inner { markets: HashMap::new(), wal, snap })) })
    }
    pub fn ensure_market(&self, market: Market) { let mut g = self.inner.lock(); g.markets.entry(market.id.clone()).or_insert_with(|| OrderBook::new(market)); }
    pub fn place_order(&self, o: NewOrder) -> anyhow::Result<Vec<Fill>> {
        let mut g = self.inner.lock(); let book = g.markets.get_mut(&o.market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?;
        g.wal.append(&WalRecord::Place(o.clone()))?; let fills = book.place(o.clone())?; Ok(fills)
    }
    pub fn cancel_order(&self, market_id: MarketId, order_id: OrderId) -> anyhow::Result<()> {
        let mut g = self.inner.lock(); let book = g.markets.get_mut(&market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?;
        g.wal.append(&WalRecord::Cancel { market_id: market_id.clone(), order_id })?; book.cancel(order_id)
    }
    pub fn replace_order(&self, r: ReplaceOrder) -> anyhow::Result<()> {
        let mut g = self.inner.lock(); let book = g.markets.get_mut(&r.market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?;
        g.wal.append(&WalRecord::Replace(r.clone()))?; book.replace(r)
    }
    pub fn snapshot_all(&self) -> anyhow::Result<()> { let g = self.inner.lock(); g.snap.write_all(&g.markets)?; Ok(()) }
    pub fn state_hash(&self) -> anyhow::Result<[u8; 32]> { let g = self.inner.lock(); Ok(crate::hash::state_hash(&g.markets)) }
    pub fn restore_from_latest(&self) -> anyhow::Result<()> {
        let mut g = self.inner.lock(); if let Some(state) = g.snap.try_read_latest()? { g.markets = state; }
        g.wal.replay(|rec| { match rec {
            WalRecord::Place(o) => { let book = g.markets.get_mut(&o.market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?; let _ = book.place(o); },
            WalRecord::Cancel{market_id, order_id} => { let book = g.markets.get_mut(&market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?; let _ = book.cancel(order_id); },
            WalRecord::Replace(r) => { let book = g.markets.get_mut(&r.market_id).ok_or_else(|| anyhow::anyhow!("unknown market"))?; let _ = book.replace(r); },
            WalRecord::Settle{s} => { if let Some(book) = g.markets.get_mut(&s.market_id) { book.settle(s.resolve_yes); } }
        } Ok(()) })?; Ok(())
    }
}
