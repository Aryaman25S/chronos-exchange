use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleRec {
    pub market_id: MarketId,
    pub resolve_yes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord {
    Place(NewOrder),
    Cancel {
        market_id: MarketId,
        order_id: OrderId,
    },
    Replace(ReplaceOrder),
    Settle {
        s: SettleRec,
    },
}

pub struct Wal {
    dir: PathBuf,
    file: File,
}

impl Wal {
    pub fn new(dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&dir)?;
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("engine.wal"))?;
        Ok(Self { dir, file: f })
    }
    pub fn append(&mut self, rec: &WalRecord) -> Result<()> {
        let buf = bincode::serialize(rec)?;
        let len = (buf.len() as u32).to_le_bytes();
        self.file.write_all(&len)?;
        self.file.write_all(&buf)?;
        self.file.flush()?;
        Ok(())
    }

    /// Flush the append handle so [`Self::len_bytes`] reflects all appended records.
    pub fn flush(&mut self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }

    /// Current WAL file length in bytes (end offset for the next record).
    pub fn len_bytes(&mut self) -> Result<u64> {
        self.flush()?;
        Ok(self.file.metadata()?.len())
    }

    /// Replay every record from the beginning (same as `replay_from(0, ...)`).
    pub fn replay<F: FnMut(WalRecord) -> Result<()>>(&self, f: F) -> Result<()> {
        self.replay_from(0, f)
    }

    /// Replay records starting at byte offset `start` (typically the offset stored in a snapshot).
    pub fn replay_from<F: FnMut(WalRecord) -> Result<()>>(
        &self,
        start: u64,
        mut f: F,
    ) -> Result<()> {
        let path = self.dir.join("engine.wal");
        if !path.exists() {
            return Ok(());
        }
        let mut file = File::open(&path)?;
        let len = file.metadata()?.len();
        if start > len {
            anyhow::bail!("WAL replay start offset {start} past file length {len}");
        }
        file.seek(SeekFrom::Start(start))?;
        let mut r = BufReader::new(file);
        loop {
            let mut lenb = [0u8; 4];
            if r.read_exact(&mut lenb).is_err() {
                break;
            }
            let reclen = u32::from_le_bytes(lenb) as usize;
            let mut buf = vec![0u8; reclen];
            r.read_exact(&mut buf)?;
            let rec: WalRecord = bincode::deserialize(&buf)?;
            f(rec)?;
        }
        Ok(())
    }
}
