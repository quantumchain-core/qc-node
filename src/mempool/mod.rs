// src/mempool/mod.rs
// QTC - M4: Transaction Pool
// Per whitepaper: EIP-1559 style fees (base fee + priority fee), Dilithium2 signatures

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Block gas limit - tune after benchmarks
pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;

// ---------------------------------------------------------------------------
// Types (align with your M1 crypto / M3 chain types)
// ---------------------------------------------------------------------------

/// 32-byte transaction hash (SHA3-256)
pub type TxHash = [u8; 32];

/// QTC account address — bech32m-encoded Dilithium pubkey hash in practice;
/// we use a byte array internally.
pub type Address = [u8; 32];

/// A minimal signed transaction.
/// Extend with contract call data when the VM lands (post-M10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: TxHash,
    pub from: Address,
    pub to: Address,
    pub value: u64, // in smallest QTC unit (nano-QTC)
    pub nonce: u64,
    pub base_fee: u64, // must be >= current protocol base fee
    pub priority_fee: u64, // tip to block proposer
    pub gas_limit: u64,
    pub signature: Vec<u8>, // Dilithium2 signature bytes (2420 bytes for M1)
    pub received_at: u64, // unix timestamp, for eviction ordering
    /// AUDIT-FIX (signature verification): sender's full Dilithium2 public
    /// key (1312 bytes). Required because `from` is only SHA3-256(pubkey) —
    /// a one-way hash — and Dilithium2 signatures don't support recovering
    /// the public key from a signature the way ECDSA does. Appended as the
    /// LAST field (not part of the signed byte layout — see
    /// `signable_bytes()` — so it doesn't change what wallets/faucet sign,
    /// only what they must additionally attach to the wire payload).
    pub from_pubkey: Vec<u8>,
}

impl Transaction {
    /// Effective fee per gas (EIP-1559 style).
    /// Capped at base_fee + priority_fee.
    pub fn effective_fee(&self, base_fee: u64) -> u64 {
        self.base_fee.min(base_fee + self.priority_fee)
    }

    /// Total max fee the sender is willing to pay.
    pub fn max_fee(&self) -> u64 {
        self.base_fee
    }

    /// The exact byte layout that gets Dilithium2-signed by the wallet/
    /// faucet. MUST match qtc-client's `serializeTransaction()` called with
    /// `signature: empty bytes` and `receivedAt: 0` byte-for-byte — see
    /// qtc-client/src/transaction.ts. Any change here requires a matching
    /// change there, and vice versa, or every valid signature will start
    /// failing verification.
    ///
    /// Layout: hash(32) || from(32) || to(32) || value(u64 LE)
    ///       || nonce(u64 LE) || base_fee(u64 LE) || priority_fee(u64 LE)
    ///       || gas_limit(u64 LE) || sig_len_prefix(u64 LE, always 0 here)
    ///       || received_at(u64 LE, always 0 here)
    ///
    /// Deliberately does NOT include `from_pubkey` — that field is
    /// transport-only (added so the node can look up the key to verify
    /// against), not part of the signed content.
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(32 + 32 + 32 + 8 * 5 + 8 + 8);
        v.extend_from_slice(&self.hash);
        v.extend_from_slice(&self.from);
        v.extend_from_slice(&self.to);
        v.extend_from_slice(&self.value.to_le_bytes());
        v.extend_from_slice(&self.nonce.to_le_bytes());
        v.extend_from_slice(&self.base_fee.to_le_bytes());
        v.extend_from_slice(&self.priority_fee.to_le_bytes());
        v.extend_from_slice(&self.gas_limit.to_le_bytes());
        v.extend_from_slice(&0u64.to_le_bytes()); // signature length prefix (empty sig)
        // signature bytes themselves: empty, nothing to append
        v.extend_from_slice(&0u64.to_le_bytes()); // received_at, zeroed for signing
        v
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions across all senders.
    pub global_max: usize,
    /// Maximum pending txs per sender address.
    pub per_sender_max: usize,
    /// Current protocol base fee (updated each block).
    pub base_fee: u64,
    /// Txs older than this many seconds are eligible for eviction.
    pub ttl_seconds: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            global_max: 10_000,
            per_sender_max: 64,
            base_fee: 1_000, // 1000 nano-QTC — tune after economic sim
            ttl_seconds: 3_600, // 1 hour
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum MempoolError {
    /// Transaction already in pool.
    AlreadyKnown,
    /// base_fee field is below the current protocol base fee.
    FeeTooLow,
    /// Nonce already filled by a pending transaction.
    NonceTooLow,
    /// Sender has reached per_sender_max.
    SenderQueueFull,
    /// Pool is at global_max and tx priority is not high enough to evict.
    PoolFull,
    /// Signature verification failed (hook into your M1 crypto).
    InvalidSignature,
    /// from_pubkey does not hash (SHA3-256) to the claimed `from` address.
    InvalidPubkey,
    /// gas_limit is zero or exceeds block gas limit.
    InvalidGas,
}

// ---------------------------------------------------------------------------
// Internal pending-tx wrapper (sortable by fee)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct PendingTx {
    tx: Transaction,
    /// Cached effective fee so sorting is O(1).
    effective_fee: u64,
}

// ---------------------------------------------------------------------------
// Mempool
// ---------------------------------------------------------------------------

pub struct Mempool {
    config: MempoolConfig,

    /// All transactions indexed by hash for O(1) lookup.
    by_hash: HashMap<TxHash, PendingTx>,

    /// Per-sender nonce queue: sender → (nonce → hash).
    /// BTreeMap keeps nonces in order for cheap gap detection.
    by_sender: HashMap<Address, BTreeMap<u64, TxHash>>,

    /// Fee-priority index for block building and eviction.
    /// Key: (effective_fee DESC, received_at ASC) → hash.
    /// We store as (u64::MAX - effective_fee, received_at) so BTreeMap
    /// gives us highest-fee first with natural ordering.
    fee_index: BTreeMap<(u64, u64), TxHash>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            by_hash: HashMap::new(),
            by_sender: HashMap::new(),
            fee_index: BTreeMap::new(),
        }
    }

    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    /// Add a transaction to the pool.
    /// Returns Ok(()) on success, or a MempoolError describing rejection.
    pub fn add(&mut self, tx: Transaction) -> Result<(), MempoolError> {
        // 1. Duplicate check
        if self.by_hash.contains_key(&tx.hash) {
            return Err(MempoolError::AlreadyKnown);
        }

        // 2. Base fee check
        if tx.base_fee < self.config.base_fee {
            return Err(MempoolError::FeeTooLow);
        }

        // 3. Gas sanity
        if tx.gas_limit == 0 || tx.gas_limit > BLOCK_GAS_LIMIT {
            return Err(MempoolError::InvalidGas);
        }

        // 4. Check nonce + per-sender cap WITHOUT holding the borrow
        {
            let sender_queue = self.by_sender.get(&tx.from);
            if let Some(queue) = sender_queue {
                if queue.contains_key(&tx.nonce) {
                    return Err(MempoolError::NonceTooLow);
                }
                if queue.len() >= self.config.per_sender_max {
                    return Err(MempoolError::SenderQueueFull);
                }
            }
        } // borrow drops here

        // 5. Global cap — evict lowest-fee tx if needed
        if self.by_hash.len() >= self.config.global_max {
            let new_effective = tx.effective_fee(self.config.base_fee);
            if!self.try_evict_for(new_effective) {
                return Err(MempoolError::PoolFull);
            }
        }

        // 6. Signature verification (AUDIT-FIX: was a TODO / no-op).
        //    Two checks, both required:
        //    a) the claimed pubkey actually hashes to the claimed sender address
        //    b) the Dilithium2 signature is valid for that pubkey over the
        //       exact bytes the wallet/faucet signed (see signable_bytes())
        if crate::consensus::address_from_pubkey(&tx.from_pubkey) != tx.from {
            return Err(MempoolError::InvalidPubkey);
        }
        if !crate::crypto::verify(&tx.signable_bytes(), &tx.signature, &tx.from_pubkey) {
            return Err(MempoolError::InvalidSignature);
        }

        // 7. Insert - now safe to mutably borrow again
        let effective_fee = tx.effective_fee(self.config.base_fee);
        let hash = tx.hash;
        let fee_key = (u64::MAX - effective_fee, tx.received_at);

        self.by_sender
           .entry(tx.from)
           .or_default()
           .insert(tx.nonce, hash);
        self.fee_index.insert(fee_key, hash);
        self.by_hash.insert(hash, PendingTx { tx, effective_fee });

        Ok(())
    }

    /// Remove a transaction by hash (e.g. after inclusion in a block).
    pub fn remove(&mut self, hash: &TxHash) -> Option<Transaction> {
        let pending = self.by_hash.remove(hash)?;
        let tx = pending.tx.clone();

        // Clean sender queue
        if let Some(queue) = self.by_sender.get_mut(&tx.from) {
            queue.remove(&tx.nonce);
            if queue.is_empty() {
                self.by_sender.remove(&tx.from);
            }
        }

        // Clean fee index
        let fee_key = (u64::MAX - pending.effective_fee, tx.received_at);
        self.fee_index.remove(&fee_key);

        Some(tx)
    }

    /// Return up to `limit` transactions ordered by priority (highest fee first).
    /// Used by the block proposer to fill a block.
    pub fn peek_best(&self, limit: usize) -> Vec<&Transaction> {
        self.fee_index
           .values()
           .take(limit)
           .filter_map(|hash| self.by_hash.get(hash))
           .map(|p| &p.tx)
           .collect()
    }

    /// Evict transactions older than config.ttl_seconds.
    /// Call once per block.
    pub fn evict_expired(&mut self) {
        let now = now_secs();
        let cutoff = now.saturating_sub(self.config.ttl_seconds);
        let stale: Vec<TxHash> = self
           .by_hash
           .values()
           .filter(|p| p.tx.received_at < cutoff)
           .map(|p| p.tx.hash)
           .collect();
        for hash in stale {
            self.remove(&hash);
        }
    }

    /// Update the protocol base fee (called after each block, EIP-1559 style).
    /// Evicts any transactions whose base_fee is now below the new floor.
    pub fn update_base_fee(&mut self, new_base_fee: u64) {
        self.config.base_fee = new_base_fee;
        let stale: Vec<TxHash> = self
           .by_hash
           .values()
           .filter(|p| p.tx.base_fee < new_base_fee)
           .map(|p| p.tx.hash)
           .collect();
        for hash in stale {
            self.remove(&hash);
        }
    }

    /// Count of pending transactions.
    pub fn len(&self) -> usize {
        self.by_hash.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_hash.is_empty()
    }

    /// Lookup a transaction by hash.
    pub fn get(&self, hash: &TxHash) -> Option<&Transaction> {
        self.by_hash.get(hash).map(|p| &p.tx)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Try to evict the lowest-priority transaction to make room.
    /// Returns true if eviction succeeded and new tx has higher priority.
    fn try_evict_for(&mut self, new_effective_fee: u64) -> bool {
        // Lowest priority is at the end of the BTreeMap (highest fee_key value)
        if let Some((&fee_key, &hash)) = self.fee_index.iter().next_back() {
            let lowest_fee = u64::MAX - fee_key.0;
            if new_effective_fee > lowest_fee {
                self.remove(&hash);
                return true;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn now_secs() -> u64 {
    SystemTime::now()
       .duration_since(UNIX_EPOCH)
       .unwrap_or_default()
       .as_secs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// Returns a real Dilithium2 keypair for a given test "identity" byte,
    /// generating it once and reusing it on subsequent calls so the same
    /// `from` id always maps to the same address (needed for nonce/queue
    /// tests that send multiple txs "from" the same sender).
    fn keypair_for(id: u8) -> (Vec<u8>, Vec<u8>) {
        static KEYS: OnceLock<Mutex<HashMap<u8, (Vec<u8>, Vec<u8>)>>> = OnceLock::new();
        let map = KEYS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = map.lock().unwrap();
        map.entry(id)
            .or_insert_with(crate::crypto::generate_keypair)
            .clone()
    }

    /// Builds a transaction with a REAL, verifying Dilithium2 signature —
    /// necessary now that Mempool::add() actually checks it (previously
    /// this test suite used a `vec![0u8; 2420]` placeholder signature,
    /// which only worked because verification was a no-op TODO).
    fn make_tx(from: u8, nonce: u64, base_fee: u64, priority_fee: u64) -> Transaction {
        let (pk, sk) = keypair_for(from);
        let from_addr = crate::consensus::address_from_pubkey(&pk);
        let mut tx = Transaction {
            hash: [from, nonce as u8, base_fee as u8, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0],
            from: from_addr,
            to: [1u8; 32],
            value: 1_000,
            nonce,
            base_fee,
            priority_fee,
            gas_limit: 21_000,
            signature: Vec::new(), // filled in below, after signable_bytes() is stable
            received_at: now_secs(),
            from_pubkey: pk,
        };
        tx.signature = crate::crypto::sign(&sk, &tx.signable_bytes());
        tx
    }

    #[test]
    fn test_add_and_get() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 0, 2_000, 100);
        let hash = tx.hash;
        assert!(pool.add(tx).is_ok());
        assert!(pool.get(&hash).is_some());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 0, 2_000, 100);
        pool.add(tx.clone()).unwrap();
        assert_eq!(pool.add(tx), Err(MempoolError::AlreadyKnown));
    }

    #[test]
    fn test_fee_too_low() {
        let mut pool = Mempool::new(MempoolConfig::default()); // base_fee = 1000
        let tx = make_tx(1, 0, 500, 100); // base_fee 500 < 1000
        assert_eq!(pool.add(tx), Err(MempoolError::FeeTooLow));
    }

    #[test]
    fn test_invalid_gas() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let mut tx = make_tx(1, 0, 2_000, 100);
        tx.gas_limit = 0;
        assert_eq!(pool.add(tx.clone()), Err(MempoolError::InvalidGas));
        tx.gas_limit = BLOCK_GAS_LIMIT + 1;
        assert_eq!(pool.add(tx), Err(MempoolError::InvalidGas));
    }

    #[test]
    fn test_remove() {
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 0, 2_000, 100);
        let hash = tx.hash;
        pool.add(tx).unwrap();
        assert!(pool.remove(&hash).is_some());
        assert!(pool.is_empty());
    }

    #[test]
    fn test_peek_best_ordering() {
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.add(make_tx(1, 0, 2_000, 50)).unwrap();
        pool.add(make_tx(2, 0, 2_000, 200)).unwrap();
        pool.add(make_tx(3, 0, 2_000, 100)).unwrap();

        let best = pool.peek_best(3);
        assert_eq!(best[0].priority_fee, 200);
        assert_eq!(best[1].priority_fee, 100);
        assert_eq!(best[2].priority_fee, 50);
    }

    #[test]
    fn test_base_fee_update_evicts() {
        let mut pool = Mempool::new(MempoolConfig::default()); // base_fee = 1000
        pool.add(make_tx(1, 0, 1_500, 100)).unwrap();
        pool.add(make_tx(2, 0, 3_000, 100)).unwrap();
        pool.update_base_fee(2_000);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_sender_queue_full() {
        let mut config = MempoolConfig::default();
        config.per_sender_max = 2;
        let mut pool = Mempool::new(config);
        pool.add(make_tx(1, 0, 2_000, 100)).unwrap();
        pool.add(make_tx(1, 1, 2_000, 100)).unwrap();
        assert_eq!(pool.add(make_tx(1, 2, 2_000, 100)), Err(MempoolError::SenderQueueFull));
    }

    #[test]
    fn test_global_eviction_by_fee() {
        let mut config = MempoolConfig::default();
        config.global_max = 2;
        let mut pool = Mempool::new(config);
        pool.add(make_tx(1, 0, 2_000, 50)).unwrap(); // effective = 2050
        pool.add(make_tx(2, 0, 2_000, 100)).unwrap(); // effective = 2100
        // This one has effective = 2200, should evict the 2050 one
        pool.add(make_tx(3, 0, 2_000, 200)).unwrap();
        assert_eq!(pool.len(), 2);
        assert!(pool.get(&make_tx(1, 0, 2_000, 50).hash).is_none());
    }
        }
