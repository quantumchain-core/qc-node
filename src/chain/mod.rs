// src/chain/mod.rs
// QTC - M3: Chain Types (unified, replaces header.rs)
// Dilithium2: sig=2420 bytes, pk=1312 bytes

use serde::{Deserialize, Serialize};
pub use crate::mempool::Transaction;

/// 32-byte hash (SHA3-256 / SHA2-256)
pub type Hash = [u8; 32];

/// Account address — 32-byte Dilithium2 pubkey hash
pub type Address = [u8; 32];

/// Block header — all consensus-critical fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub number: u64,
    pub slot: u64,
    pub timestamp: u64,
    pub proposer: Address,      // 32-byte hash of validator pubkey
    pub tx_root: Hash,
    pub state_root: Hash,       // M6: updated after execution
    pub base_fee: u64,          // EIP-1559
    pub gas_used: u64,
    pub gas_limit: u64,
    pub signature: Vec<u8>,     // Dilithium2 sig over header bytes (2420 bytes)
}

impl BlockHeader {
    /// Serialize header fields WITHOUT signature for signing/verification
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.parent_hash);
        v.extend_from_slice(&self.number.to_le_bytes());
        v.extend_from_slice(&self.slot.to_le_bytes());
        v.extend_from_slice(&self.timestamp.to_le_bytes());
        v.extend_from_slice(&self.proposer);
        v.extend_from_slice(&self.tx_root);
        v.extend_from_slice(&self.state_root);
        v.extend_from_slice(&self.base_fee.to_le_bytes());
        v.extend_from_slice(&self.gas_used.to_le_bytes());
        v.extend_from_slice(&self.gas_limit.to_le_bytes());
        v // signature NOT included
    }
}

/// Full block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn hash(&self) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.header.to_signable_bytes());
        hasher.finalize().into()
    }
}

/// Binary merkle root over transaction hashes. Standard pairwise-hash
/// construction: odd node out at any level is paired with itself.
/// Returns the zero hash for an empty transaction list (matches genesis
/// and any other intentionally-empty block).
pub fn merkle_root(transactions: &[Transaction]) -> Hash {
    use sha2::{Digest, Sha256};

    if transactions.is_empty() {
        return [0u8; 32];
    }

    let mut level: Vec<Hash> = transactions.iter().map(|tx| tx.hash).collect();

    while level.len() > 1 {
        let mut next_level = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(pair[0]);
            // Odd one out at this level: pair with itself rather than
            // dropping it, so every transaction still affects the root.
            hasher.update(pair.get(1).unwrap_or(&pair[0]));
            next_level.push(hasher.finalize().into());
        }
        level = next_level;
    }

    level[0]
}

/// The fixed genesis block. Every node computes the same hash for this,
/// so it acts as the universal "block 0" / chain root.
///
/// M9: Node::bootstrap() persists this to storage on first run and sets
/// ChainHead to point at it, so the first produced/received block
/// (number = 1) has a well-defined parent_hash.
pub fn genesis_block() -> Block {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_with_hash(h: u8) -> Transaction {
        Transaction {
            hash: [h; 32],
            from: [0u8; 32],
            to: [0u8; 32],
            value: 0,
            nonce: 0,
            base_fee: 0,
            priority_fee: 0,
            gas_limit: 0,
            action: crate::mempool::TxAction::Transfer,
            signature: Vec::new(),
            received_at: 0,
            from_pubkey: Vec::new(),
        }
    }

    #[test]
    fn test_merkle_root_empty_is_zero_hash() {
        assert_eq!(merkle_root(&[]), [0u8; 32]);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let txs = vec![tx_with_hash(1), tx_with_hash(2), tx_with_hash(3)];
        assert_eq!(merkle_root(&txs), merkle_root(&txs));
    }

    #[test]
    fn test_merkle_root_changes_with_different_txs() {
        let txs_a = vec![tx_with_hash(1), tx_with_hash(2)];
        let txs_b = vec![tx_with_hash(1), tx_with_hash(3)];
        assert_ne!(merkle_root(&txs_a), merkle_root(&txs_b));
    }

    #[test]
    fn test_merkle_root_order_sensitive() {
        // Same transactions, different order -> different root. A block
        // with reordered (but otherwise identical) transactions must not
        // hash to the same tx_root.
        let forward = vec![tx_with_hash(1), tx_with_hash(2), tx_with_hash(3)];
        let reversed = vec![tx_with_hash(3), tx_with_hash(2), tx_with_hash(1)];
        assert_ne!(merkle_root(&forward), merkle_root(&reversed));
    }

    #[test]
    fn test_merkle_root_odd_count_does_not_panic() {
        // 1, 3, and 5 transactions all exercise the "odd node paired with
        // itself" branch at some level. Using hashes starting at 1 (not
        // 0) so a single-tx tree's zero-round-trip (see the test below)
        // doesn't make this coincidentally compare against the very
        // all-zero hash we're checking against.
        for n in [1usize, 3, 5] {
            let txs: Vec<Transaction> = (1..=n as u8).map(tx_with_hash).collect();
            let root = merkle_root(&txs);
            assert_ne!(root, [0u8; 32]);
        }
    }

    #[test]
    fn test_merkle_root_single_tx_equals_its_hash() {
        // By standard merkle-tree convention (same as Bitcoin), a
        // single-leaf tree's root IS that leaf — there's nothing to pair
        // it with, so no hashing round happens. My original version of
        // this test asserted the opposite (assert_ne), which was simply
        // wrong about the convention and about what merkle_root() above
        // actually does: its `while level.len() > 1` loop never runs for
        // exactly one transaction, so it returns the raw hash unchanged.
        // That's correct behavior, not a bug — documenting it here.
        let tx = tx_with_hash(7);
        let root = merkle_root(&[tx.clone()]);
        assert_eq!(root, tx.hash);
    }
    #[test]
    fn test_signable_bytes_excludes_sig() {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 1000,
            proposer: [1u8; 32],
            tx_root: [2u8; 32],
            state_root: [3u8; 32],
            base_fee: 1000,
            gas_used: 21000,
            gas_limit: 30_000_000,
            signature: vec![0u8; 2420],
        };
        let bytes = header.to_signable_bytes();
        // sig (2420 bytes) must NOT be in signable bytes
        assert!(!bytes.windows(2420).any(|w| w == vec![0u8; 2420].as_slice()));
    }

    #[test]
    fn test_block_hash_deterministic() {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 1000,
            proposer: [1u8; 32],
            tx_root: [2u8; 32],
            state_root: [3u8; 32],
            base_fee: 1000,
            gas_used: 0,
            gas_limit: 30_000_000,
            signature: vec![0u8; 2420],
        };
        let block = Block { header, transactions: vec![] };
        assert_eq!(block.hash(), block.hash());
    }

    #[test]
    fn test_genesis_block_hash_stable_and_nonzero() {
        let g1 = genesis_block();
        let g2 = genesis_block();
        // Same inputs -> same hash every time
        assert_eq!(g1.hash(), g2.hash());
        // The hash itself is a real SHA256 digest, not the zero placeholder
        assert_ne!(g1.hash(), [0u8; 32]);
    }
}
