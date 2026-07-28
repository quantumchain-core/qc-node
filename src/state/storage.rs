use sled::Db;
use thiserror::Error;
use crate::chain::Block;
use crate::state::StateDB;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
}

pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new() -> Result<Self, StorageError> {
        // M6: Use QC_DB_PATH in CI, fallback to./qc-data locally
        let path = std::env::var("QC_DB_PATH").unwrap_or_else(|_| "./qc-data".to_string());
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// Open storage at an explicit path, bypassing QC_DB_PATH entirely.
    ///
    /// Added because `new()` reading a process-global env var is a real
    /// race under `cargo test`'s default parallel threads: every test
    /// helper that did `std::env::set_var("QC_DB_PATH", tmp.path());
    /// Storage::new()` could have its path silently overwritten by a
    /// DIFFERENT concurrently-running test before its own `new()` call
    /// read the var — causing one test's storage to open in another
    /// test's temp directory. Tests should use this constructor instead;
    /// `new()` stays as-is for production use (a real node process only
    /// ever sets QC_DB_PATH once, at startup, so no race there).
    pub fn open_at<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let key = format!("block_{}", block.header.number);
        let value = bincode::serialize(block)?;
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn get_block(&self, number: u64) -> Result<Option<Block>, StorageError> {
        let key = format!("block_{}", number);
        match self.db.get(key)? {
            Some(ivec) => Ok(Some(bincode::deserialize(&ivec)?)),
            None => Ok(None),
        }
    }

    pub fn put_state(&self, state: &StateDB) -> Result<(), StorageError> {
        let value = bincode::serialize(state)?;
        self.db.insert("state", value)?;
        Ok(())
    }

    pub fn get_state(&self) -> Result<Option<StateDB>, StorageError> {
        match self.db.get("state")? {
            Some(ivec) => Ok(Some(bincode::deserialize(&ivec)?)),
            None => Ok(None),
        }
    }
                     }
