use crate::chain::{Block, Hash};
use crate::state::StateDB;
use thiserror::Error;
use sled::Db;
use std::path::Path;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("sled db error: {0}")]
    Sled(#[from] sled::Error),
    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

pub struct Storage {
    db: Db,
}

impl Storage {
    /// M6: Open or create database at path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// M6: Save block by hash
    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let hash = block.hash();
        let bytes = bincode::serialize(block)?;
        self.db.insert(hash, bytes)?;
        Ok(())
    }

    /// M6: Load block by hash
    pub fn get_block(&self, hash: &Hash) -> Result<Option<Block>, StorageError> {
        match self.db.get(hash)? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// M6: Save latest state
    pub fn put_state(&self, state: &StateDB) -> Result<(), StorageError> {
        let bytes = bincode::serialize(state)?;
        self.db.insert(b"state", bytes)?;
        Ok(())
    }

    /// M6: Load latest state
    pub fn get_state(&self) -> Result<StateDB, StorageError> {
        match self.db.get(b"state")? {
            Some(bytes) => Ok(bincode::deserialize(&bytes)?),
            None => Ok(StateDB::new()), // empty state on first run
        }
    }
}
