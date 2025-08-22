
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MktDelta { pub seq: u64, pub bids_upd: Vec<(u32,u64)>, pub asks_upd: Vec<(u32,u64)>, pub last_trade: Option<(u32,u32)> }

#[derive(Clone)]
pub struct Broadcaster { tx: broadcast::Sender<String>, ring: Arc<Mutex<VecDeque<String>>>, ring_cap: usize }

impl Broadcaster {
    pub fn new(capacity: usize) -> Self { let (tx, _rx) = broadcast::channel(1024); Self { tx, ring: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))), ring_cap: capacity } }
    pub fn publish_snapshot(&self, seq: u64, l2: engine::types::L2Book, last: Option<u32>) {
        let snap = serde_json::json!({ "type":"snapshot","seq":seq,"bids": l2.bids.into_iter().map(|l| [l.price,l.qty]).collect::<Vec<_>>(), "asks": l2.asks.into_iter().map(|l| [l.price,l.qty]).collect::<Vec<_>>(), "last_trade": last });
        self.push(snap.to_string());
    }
    pub fn publish_delta(&self, d: &MktDelta) { let s = serde_json::to_string(&serde_json::json!({ "type":"delta","seq": d.seq,"bids_upd": d.bids_upd,"asks_upd": d.asks_upd,"last_trade": d.last_trade })).unwrap(); self.push(s); }
    fn push(&self, s: String) { { let mut ring = self.ring.lock(); ring.push_back(s.clone()); while ring.len() > self.ring_cap { ring.pop_front(); } } let _ = self.tx.send(s); }
    pub fn subscribe(&self) -> broadcast::Receiver<String> { self.tx.subscribe() }
    pub fn snapshot_from_seq(&self, _from_seq: u64) -> Vec<String> { self.ring.lock().iter().cloned().collect() }
}
