
use anyhow::Result;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Caps {
    pub max_position: i64,
    pub max_notional_cents: i64,
    pub rate_per_sec: u32,
    pub burst: u32,
}

#[derive(Default)]
pub struct Risk {
    caps: Caps,
    positions: Mutex<HashMap<Uuid, i64>>,
    bucket: Mutex<HashMap<Uuid, (u64, u32)>>,
    idempo: Mutex<HashMap<String, bool>>,
}

impl Risk {
    pub fn new(caps: Caps) -> Self { Self { caps, positions: Default::default(), bucket: Default::default(), idempo: Default::default() } }
    pub fn check_rate_limit(&self, user: Uuid) -> Result<()> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut g = self.bucket.lock();
        let (mut last, mut tokens) = g.get(&user).cloned().unwrap_or((now, self.caps.burst));
        let elapsed = now.saturating_sub(last); tokens = std::cmp::min(self.caps.burst, tokens + (elapsed as u32) * self.caps.rate_per_sec);
        if tokens == 0 { anyhow::bail!("rate limit exceeded"); } tokens -= 1; g.insert(user, (now, tokens)); Ok(())
    }
    pub fn check_idempotency(&self, key: &str) -> Result<()> {
        let mut g = self.idempo.lock(); if g.contains_key(key) { anyhow::bail!("duplicate idempotency key"); } g.insert(key.to_string(), true); Ok(())
    }
    pub fn check_position(&self, user: Uuid, delta_contracts: i64, price_cents: i64) -> Result<()> {
        let mut p = self.positions.lock(); let pos = p.entry(user).or_insert(0); let new = *pos + delta_contracts;
        if new.abs() > self.caps.max_position { anyhow::bail!("position cap"); }
        let notional = new.abs() as i64 * price_cents; if notional > self.caps.max_notional_cents { anyhow::bail!("notional cap"); } *pos = new; Ok(())
    }
}
