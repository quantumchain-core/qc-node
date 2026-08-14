// src/consensus/producer.rs
// QTC M5/M6/M10: Block Producer
// Signs blocks with Dilithium2, executes txs, saves to disk.
// M10: proposer field is now derived from the validator's own pubkey
// (address = SHA3-256(pubkey)), so validate_block_sig() can look it up
// in the ValidatorRegistry.

use std::time::{SystemTime, UNIX_EPOCH};
use crate::chain::{merkle_root, Block, BlockHeader, Address};
use crate::mempool::Mempool;
use crate::state::{StateDB, Executor, Storage};
use crate::crypto::sign;
use super::registry::address_from_pubkey;

/// Floor for the protocol base fee. Without a floor, a sustained run of
/// empty/low-usage blocks would let base_fee decay toward (and, with
/// integer division, potentially hit) zero — at which point the ±1/8
/// adjustment step itself becomes zero forever (0 * anything / 8 == 0),
/// permanently stuck. 1 keeps the adjustment mechanism always able to move.
pub const MIN_BASE_FEE: u64 = 1;

/// P3 FIX (core-dev review): EIP-1559-style base fee adjustment.
/// MILESTONES.md already claimed this existed ("target = GAS_LIMIT/2,
/// ±1/8 per block") — it didn't; base_fee was just carried forward from
/// the parent unchanged. This is the actual implementation of what was
/// documented.
///
/// target_gas = gas_limit / 2. If the parent block used more than target,
/// base_fee rises (up to +1/8); less than target, it falls (down to
/// -1/8, floored at MIN_BASE_FEE); exactly target, unchanged.
pub fn next_base_fee(parent: &BlockHeader) -> u64 {
    let target_gas = parent.gas_limit / 2;
    if target_gas == 0 {
        return parent.base_fee.max(MIN_BASE_FEE);
    }

    let parent_base_fee = parent.base_fee as u128;

    if parent.gas_used == target_gas {
        parent.base_fee.max(MIN_BASE_FEE)
    } else if parent.gas_used > target_gas {
        let gas_delta = (parent.gas_used - target_gas) as u128;
        let delta = ((parent_base_fee * gas_delta) / (target_gas as u128) / 8).max(1);
        (parent_base_fee + delta).min(u64::MAX as u128) as u64
    } else {
        let gas_delta = (target_gas - parent.gas_used) as u128;
        let delta = (parent_base_fee * gas_delta) / (target_gas as u128) / 8;
        parent.base_fee.saturating_sub(delta as u64).max(MIN_BASE_FEE)
    }
}

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
            tx_root: merkle_root(&txs),
            state_root: [0u8; 32],
            // P3 FIX: was `parent.header.base_fee` (static forever) —
            // now actually adjusts with demand, per next_base_fee() above.
            base_fee: next_base_fee(&parent.header),
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
            action: crate::mempool::TxAction::Transfer,
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
        let storage = Storage::open_at(tmp.path()).unwrap();
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
        let storage = Storage::open_at(tmp.path()).unwrap();
        let parent = genesis();

        let block = producer.produce_block(&mut mempool, &mut state, &storage, &parent).unwrap();

        assert_eq!(block.header.proposer, address_from_pubkey(&producer.validator_pk));
    }

    // -----------------------------------------------------------------
    // P3 FIX: next_base_fee tests (core-dev review)
    // -----------------------------------------------------------------

    fn header_with(base_fee: u64, gas_used: u64, gas_limit: u64) -> BlockHeader {
        BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 0,
            proposer: [0u8; 32],
            tx_root: [0u8; 32],
            state_root: [0u8; 32],
            base_fee,
            gas_used,
            gas_limit,
            signature: vec![],
        }
    }

    #[test]
    fn test_base_fee_unchanged_at_target() {
        // gas_used exactly at target (gas_limit / 2) -> no change
        let parent = header_with(1_000, 5_000_000, 10_000_000);
        assert_eq!(next_base_fee(&parent), 1_000);
    }

    #[test]
    fn test_base_fee_rises_when_full() {
        // gas_used == gas_limit (double the target) -> base fee rises
        let parent = header_with(1_000, 10_000_000, 10_000_000);
        let next = next_base_fee(&parent);
        assert!(next > 1_000, "expected base fee to rise, got {next}");
        // max theoretical single-block increase is +1/8 (=125 here)
        assert!(next <= 1_000 + 125);
    }

    #[test]
    fn test_base_fee_falls_when_empty() {
        // gas_used == 0 -> base fee falls
        let parent = header_with(1_000, 0, 10_000_000);
        let next = next_base_fee(&parent);
        assert!(next < 1_000, "expected base fee to fall, got {next}");
        assert!(next >= 1_000 - 125);
    }

    #[test]
    fn test_base_fee_never_drops_below_floor() {
        // starting already at the floor, a sequence of empty blocks must
        // never push it to zero or below MIN_BASE_FEE.
        let mut base_fee = MIN_BASE_FEE;
        for _ in 0..50 {
            let parent = header_with(base_fee, 0, 10_000_000);
            base_fee = next_base_fee(&parent);
            assert!(base_fee >= MIN_BASE_FEE);
        }
    }

    #[test]
    fn test_base_fee_converges_toward_target_over_many_blocks() {
        // A sustained run of full blocks should keep pushing base_fee up
        // block after block (not just a one-time bump).
        let mut base_fee = 1_000u64;
        for _ in 0..10 {
            let parent = header_with(base_fee, 10_000_000, 10_000_000);
            let next = next_base_fee(&parent);
            assert!(next > base_fee);
            base_fee = next;
        }
    }
}
