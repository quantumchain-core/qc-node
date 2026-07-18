// src/consensus/producer.rs
// QTC M5/M6/M10: Block Producer
// Signs blocks with Dilithium2, executes txs, saves to disk.
// M10: proposer field is now derived from the validator's own pubkey
// (address = SHA3-256(pubkey)), so validate_block_sig() can look it up
// in the ValidatorRegistry.

use std::time::{SystemTime, UNIX_EPOCH};
use crate::chain::{Block, BlockHeader, Address};
use crate::mempool::Mempool;
use crate::state::{StateDB, Executor, Storage};
use crate::crypto::sign;
use super::registry::address_from_pubkey;

pub struct Producer {
    pub validator_sk: Vec<u8>,  // Dilithium2 secret key (2560 bytes)
    pub validator_pk: Vec<u8>,  // Dilithium2 public key (1312 bytes)
    pub coinbase: Address,      // fee recipient (may differ from proposer address)
}

impl Producer {
    pub fn new(sk: Vec<u8>, pk: Vec<u8>, coinbase: Address) -> Self {
        Self { validator_sk: sk, validator_pk: pk, coinbase }
    }

    pub fn produce_block(
        &self,
        mempool: &mut Mempool,
        state: &mut StateDB,
        storage: &Storage,
        parent: &Block,
    ) -> Result<Block, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let txs: Vec<_> = mempool.peek_best(1000).into_iter().cloned().collect();
        if txs.is_empty() {
            return Err("mempool empty".into());
        }

        let mut header = BlockHeader {
            parent_hash: parent.hash(),
            number: parent.header.number + 1,
            slot: parent.header.slot + 1,
            timestamp,
            proposer: address_from_pubkey(&self.validator_pk), // M10: derived from pk
            tx_root: [0u8; 32],
            state_root: [0u8; 32],
            base_fee: parent.header.base_fee,
            gas_used: 0,
            gas_limit: 10_000_000,
            signature: vec![],
        };

        let mut block = Block {
            header: header.clone(),
            transactions: txs.clone(),
        };

        let gas_used = Executor::execute_block(state, &block, &self.coinbase)
            .map_err(|e| format!("exec failed: {e:?}"))?;

        header.gas_used = gas_used;
        header.state_root = state.state_root();

        let signable = header.to_signable_bytes();
        header.signature = sign(&self.validator_sk, &signable);

        block.header = header;

        for tx in &block.transactions {
            mempool.remove(&tx.hash);
        }

        storage.put_block(&block).map_err(|e| format!("storage failed: {e:?}"))?;
        storage.put_state(state).map_err(|e| format!("state save failed: {e:?}"))?;

        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::{Mempool, MempoolConfig, Transaction};
    use crate::state::{StateDB, Account, Storage};
    use crate::crypto::generate_keypair;

    fn make_producer() -> Producer {
        let (pk, sk) = generate_keypair();
        let coinbase = [9u8; 32];
        Producer::new(sk, pk, coinbase)
    }

    fn make_tx(from: u8, nonce: u64) -> Transaction {
        let (pk, sk) = generate_keypair();
        let from_addr = crate::consensus::address_from_pubkey(&pk);
        let mut tx = Transaction {
            hash: [from, nonce as u8, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            from: from_addr,
            to: [2u8; 32],
            value: 10,
            nonce,
            base_fee: 1_000,
            priority_fee: 100,
            gas_limit: 21_000,
            signature: Vec::new(),
            received_at: 0,
            from_pubkey: pk,
        };
        tx.signature = crate::crypto::sign(&sk, &tx.signable_bytes());
        tx
    }

    fn genesis() -> Block {
        Block {
            header: BlockHeader {
                parent_hash: [0u8; 32],
                number: 0,
                slot: 0,
                timestamp: 0,
                proposer: [0u8; 32],
                tx_root: [0u8; 32],
                state_root: [0u8; 32],
                base_fee: 1_000,
                gas_used: 0,
                gas_limit: 10_000_000,
                signature: vec![0u8; 2420],
            },
            transactions: vec![],
        }
    }

    #[test]
    fn test_produce_block_signs_and_saves() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());

        let producer = make_producer();
        let mut mempool = Mempool::new(MempoolConfig {
            base_fee: 1_000,
            ..Default::default()
        });

        let mut state = StateDB::new();
        let tx = make_tx(1, 0);

        // gas_cost = gas_limit * base_fee = 21_000 * 1_000 = 21_000_000
        // value = 10 -> total needed = 21_000_010
        state.set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });

        mempool.add(tx).unwrap();
        let storage = Storage::new().unwrap();
        let parent = genesis();

        let block = producer.produce_block(
            &mut mempool, &mut state, &storage, &parent
        ).unwrap();

        assert_eq!(block.header.number, 1);
        assert!(!block.header.signature.is_empty());
        assert_eq!(block.header.signature.len(), 2420);
        assert!(mempool.is_empty());
    }

    #[test]
    fn test_proposer_matches_pubkey_address() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());

        let producer = make_producer();
        let mut mempool = Mempool::new(MempoolConfig {
            base_fee: 1_000,
            ..Default::default()
        });

        let mut state = StateDB::new();
        let tx = make_tx(1, 0);
        state.set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });

        mempool.add(tx).unwrap();
        let storage = Storage::new().unwrap();
        let parent = genesis();

        let block = producer.produce_block(&mut mempool, &mut state, &storage, &parent).unwrap();

        assert_eq!(block.header.proposer, address_from_pubkey(&producer.validator_pk));
    }
}
