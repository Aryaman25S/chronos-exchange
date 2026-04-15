use crate::orderbook::BookState;
use crate::types::MarketId;
use anyhow::Result;
use std::collections::HashMap;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

/// v2 snapshot: magic + WAL end offset + zstd(bincode book states).
const MAGIC: &[u8; 4] = b"CHS2";

pub struct Snapshotter {
    dir: PathBuf,
}

impl Snapshotter {
    pub fn new(dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// `wal_end_offset` is the engine.wal file length at snapshot time (all bytes before this are
    /// incorporated into the serialized book state; replay WAL from this offset onward).
    pub fn write_all(
        &self,
        markets: &HashMap<MarketId, crate::orderbook::OrderBook>,
        wal_end_offset: u64,
    ) -> Result<()> {
        let mut m: HashMap<MarketId, BookState> = HashMap::new();
        for (k, v) in markets {
            m.insert(k.clone(), v.state.clone());
        }
        let bytes = bincode::serialize(&m)?;
        let compressed = zstd::encode_all(&bytes[..], 3)?;
        let path = self.dir.join("snapshot-latest.bin.zst");
        let mut f = File::create(path)?;
        f.write_all(MAGIC)?;
        f.write_all(&wal_end_offset.to_le_bytes())?;
        f.write_all(&compressed)?;
        Ok(())
    }

    /// Returns book state and WAL byte offset to replay from. Legacy v1 files (no magic) are
    /// ignored so recovery replays the full WAL only (avoids double-apply); delete old snapshots
    /// if you need the fast path without a compatible WAL.
    pub fn try_read_latest(
        &self,
    ) -> Result<Option<(HashMap<MarketId, crate::orderbook::OrderBook>, u64)>> {
        let path = self.dir.join("snapshot-latest.bin.zst");
        if !path.exists() {
            return Ok(None);
        }
        let mut f = File::open(&path)?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        if &magic != MAGIC {
            tracing::warn!(
                "Ignoring legacy snapshot at {}; delete this file or replace with a v2 snapshot",
                path.display()
            );
            return Ok(None);
        }
        let mut offb = [0u8; 8];
        f.read_exact(&mut offb)?;
        let wal_end_offset = u64::from_le_bytes(offb);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let decompressed = zstd::decode_all(&buf[..])?;
        let map: HashMap<MarketId, BookState> = bincode::deserialize(&decompressed)?;
        let ob_map = map
            .into_iter()
            .map(|(k, v)| (k, crate::orderbook::OrderBook { state: v }))
            .collect();
        Ok(Some((ob_map, wal_end_offset)))
    }
}
