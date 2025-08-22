
use std::{path::PathBuf, fs::{self, File}, io::{Read, Write}};
use anyhow::Result;
use crate::orderbook::BookState;
use std::collections::HashMap;
use crate::types::MarketId;

pub struct Snapshotter { dir: PathBuf }

impl Snapshotter {
    pub fn new(dir: PathBuf) -> Result<Self> { std::fs::create_dir_all(&dir)?; Ok(Self { dir }) }
    pub fn write_all(&self, markets: &HashMap<MarketId, crate::orderbook::OrderBook>) -> Result<()> {
        let mut m: HashMap<MarketId, BookState> = HashMap::new();
        for (k,v) in markets { m.insert(k.clone(), v.state.clone()); }
        let bytes = bincode::serialize(&m)?; let compressed = zstd::encode_all(&bytes[..], 3)?;
        let path = self.dir.join("snapshot-latest.bin.zst"); let mut f = File::create(path)?; f.write_all(&compressed)?; Ok(())
    }
    pub fn try_read_latest(&self) -> Result<Option<HashMap<MarketId, crate::orderbook::OrderBook>>> {
        let path = self.dir.join("snapshot-latest.bin.zst"); if !path.exists() { return Ok(None); }
        let mut f = File::open(path)?; let mut buf=vec![]; f.read_to_end(&mut buf)?; let decompressed = zstd::decode_all(&buf[..])?;
        let map: HashMap<MarketId, BookState> = bincode::deserialize(&decompressed)?;
        let ob_map = map.into_iter().map(|(k,v)| (k, crate::orderbook::OrderBook{ state:v })).collect(); Ok(Some(ob_map))
    }
}
