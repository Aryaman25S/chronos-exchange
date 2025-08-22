
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::{fs::{OpenOptions, File}, io::{Write, Read, BufReader}, path::PathBuf};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleRec { pub market_id: MarketId, pub resolve_yes: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord { Place(NewOrder), Cancel { market_id: MarketId, order_id: OrderId }, Replace(ReplaceOrder), Settle { s: SettleRec } }

pub struct Wal { dir: PathBuf, file: File }

impl Wal {
    pub fn new(dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&dir)?;
        let f = OpenOptions::new().create(true).append(true).open(dir.join("engine.wal"))?;
        Ok(Self { dir, file: f })
    }
    pub fn append(&mut self, rec: &WalRecord) -> Result<()> {
        let buf = bincode::serialize(rec)?; let len = (buf.len() as u32).to_le_bytes();
        self.file.write_all(&len)?; self.file.write_all(&buf)?; self.file.flush()?; Ok(())
    }
    pub fn replay<F: FnMut(WalRecord) -> Result<()>>(&self, mut f: F) -> Result<()> {
        let path = self.dir.join("engine.wal"); if !path.exists() { return Ok(()); }
        let mut r = BufReader::new(File::open(path)?);
        loop {
            let mut lenb=[0u8;4]; if r.read_exact(&mut lenb).is_err() { break; }
            let len = u32::from_le_bytes(lenb) as usize; let mut buf=vec![0u8;len]; r.read_exact(&mut buf)?;
            let rec: WalRecord = bincode::deserialize(&buf)?; f(rec)?;
        } Ok(())
    }
}
