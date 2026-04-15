use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MktDelta {
    pub market_id: String,
    pub seq: u64,
    pub bids_upd: Vec<(u32, u64)>,
    pub asks_upd: Vec<(u32, u64)>,
    pub last_trade: Option<(u32, u32)>,
}

#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<String>,
    ring: Arc<Mutex<VecDeque<(u64, String)>>>,
    ring_cap: usize,
}

impl Broadcaster {
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(4096);
        Self {
            tx,
            ring: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            ring_cap: capacity,
        }
    }

    pub fn publish_snapshot(
        &self,
        market_id: &str,
        seq: u64,
        l2: engine::types::L2Book,
        last: Option<u32>,
    ) {
        let snap = serde_json::json!({
            "type":"snapshot",
            "market_id": market_id,
            "seq":seq,
            "bids": l2.bids.into_iter().map(|l| (l.price, l.qty)).collect::<Vec<(u32,u64)>>(),
            "asks": l2.asks.into_iter().map(|l| (l.price, l.qty)).collect::<Vec<(u32,u64)>>(),
            "last_trade": last
        });
        self.push(seq, snap.to_string());
    }

    pub fn publish_delta(&self, d: &MktDelta) {
        let s = serde_json::to_string(&serde_json::json!({
            "type":"delta",
            "market_id": d.market_id,
            "seq": d.seq,
            "bids_upd": d.bids_upd,
            "asks_upd": d.asks_upd,
            "last_trade": d.last_trade
        }))
        .unwrap();
        self.push(d.seq, s);
    }

    pub fn publish_trade(&self, market_id: &str, seq: u64, price: u32, qty: u32) {
        let s = serde_json::json!({
            "type": "trade",
            "market_id": market_id,
            "seq": seq,
            "price": price,
            "qty": qty,
        })
        .to_string();
        self.push(seq, s);
    }

    fn push(&self, seq: u64, s: String) {
        {
            let mut ring = self.ring.lock();
            ring.push_back((seq, s.clone()));
            while ring.len() > self.ring_cap {
                ring.pop_front();
            }
        }
        let _ = self.tx.send(s);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    /// Messages with seq strictly greater than `from_seq` (typically deltas/snapshots after a resync point).
    pub fn snapshot_from_seq(&self, from_seq: u64) -> Vec<String> {
        self.ring
            .lock()
            .iter()
            .filter(|(s, _)| *s > from_seq)
            .map(|(_, m)| m.clone())
            .collect()
    }
}
