// src/node/mod.rs
// QTC M9/M10: Node — ties mempool, state, storage, consensus and gossip together.
//
// This module contains the SYNCHRONOUS core logic (no swarm, no async).
// It is fully unit-testable in-process: two Node instances sharing genesis
// can simulate "node A produces a block" -> "node B receives it via gossip"
// without any real networking.
//
// M10: Node now holds a ValidatorRegistry and uses real Dilithium2 signature
// verification (validate_block_sig) when validating incoming blocks.
//
// src/bin/node.rs wraps this in an async loop that bridges to the libp2p
// swarm (incoming gossip -> on_gossip, outbox -> net::publish).

use crate::chain::{genesis_block, Block};
use crate::consensus::{address_from_pubkey, validate_block_sig, Producer, ValidatorRegistry};
use crate::net::{GossipMsg, HandleResult};
use crate::rpc::AppState;
use crate::state::Executor;

pub struct Node {
    pub app: AppState,
    pub producer: Producer,
    pub registry: ValidatorRegistry,
}

impl Node {
    /// Create a new Node and bootstrap genesis if this is a fresh chain
    /// (chain_head is at its default zero value).
    pub fn new(app: AppState, producer: Producer, registry: ValidatorRegistry) -> Self {
        let node = Self { app, producer, registry };
        node.bootstrap();
        node
    }

    /// If the chain hasn't started yet, persist genesis and point
    /// chain_head at it. Idempotent — safe to call on every startup.
    fn bootstrap(&self) {
        let mut head = self.app.chain_head.lock().unwrap();
        if head.number == 0 && head.head_hash == [0u8; 32] {
            let genesis = genesis_block();
            if self.app.storage.get_block(0).ok().flatten().is_none() {
                if let Err(e) = self.app.storage.put_block(&genesis) {
                    eprintln!("WARNING: failed to persist genesis block: {e:?}");
                }
            }
            head.head_hash = genesis.hash();
        }
    }

    /// Handle a raw gossip message received from a peer.
    /// Routes to tx handling (mempool) or block handling (validate + execute).
    pub fn on_gossip(&mut self, raw: &[u8]) -> HandleResult {
        let msg: GossipMsg = match bincode::deserialize(raw) {
            Ok(m) => m,
            Err(e) => return HandleResult::BlockRejected(format!("deserialize failed: {e}")),
        };

        match msg {
            GossipMsg::NewTx(tx) => {
                let mut mempool = self.app.mempool.lock().unwrap();
                match mempool.add(tx) {
                    Ok(()) => HandleResult::TxAccepted,
                    Err(e) => HandleResult::TxRejected(format!("{e:?}")),
                }
            }
            GossipMsg::NewBlock(block) => self.on_block(block),
        }
    }

    /// Validate, execute, and apply an incoming block.
    /// Execution runs against a CLONE of state; only committed on full success
    /// (a mid-block error never leaves state partially mutated).
    fn on_block(&mut self, block: Block) -> HandleResult {
        let head = self.app.chain_head.lock().unwrap().clone();

        // 1. Chain linkage — parent hash
        if block.header.parent_hash != head.head_hash {
            return HandleResult::BlockRejected(format!(
                "unknown parent: expected {:?}, got {:?}",
                head.head_hash, block.header.parent_hash
            ));
        }

        // 1b. Block number must be exactly head + 1 (AUDIT-015)
        if block.header.number != head.number + 1 {
            return HandleResult::BlockRejected(format!(
                "bad block number: expected {}, got {}",
                head.number + 1, block.header.number
            ));
        }

        // 1c. Timestamp must not be in the future (AUDIT-016)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if block.header.timestamp > now + 60 {
            return HandleResult::BlockRejected(format!(
                "future timestamp: block={}, now={}, drift={}s",
                block.header.timestamp, now,
                block.header.timestamp - now
            ));
        }

        // 2. Signature — M10: real Dilithium2 verify against registry
        if let Err(e) = validate_block_sig(&block, &self.registry) {
            return HandleResult::BlockRejected(format!("bad sig: {e}"));
        }

        // 2b. tx_root must match the actual transaction list. The signature
        // only covers the tx_root *field* — without this check, a peer
        // could swap in different transactions during gossip and it would
        // go undetected as long as gas accounting still summed correctly.
        let computed_tx_root = crate::chain::merkle_root(&block.transactions);
        if computed_tx_root != block.header.tx_root {
            return HandleResult::BlockRejected(format!(
                "tx_root mismatch: header declares {:?}, computed {:?}",
                block.header.tx_root, computed_tx_root
            ));
        }

        // 3. Gas accounting must match the declared header BEFORE touching state
        let declared_gas: u64 = block.transactions.iter().map(|t| t.gas_limit).sum();
        if declared_gas != block.header.gas_used || declared_gas > block.header.gas_limit {
            return HandleResult::BlockRejected("gas mismatch".into());
        }

        // 4. Execute on a clone — commit only if everything succeeds
        let mut state_clone = self.app.state_db.lock().unwrap().clone();
        let gas_used = match Executor::execute_block(&mut state_clone, &block, &block.header.proposer) {
            Ok(g) => g,
            Err(e) => return HandleResult::BlockRejected(format!("exec failed: {e:?}")),
        };
        if gas_used != block.header.gas_used {
            return HandleResult::BlockRejected("gas_used mismatch after execution".into());
        }

        // 5. Commit state
        *self.app.state_db.lock().unwrap() = state_clone;

        // 6. Remove included txs from our mempool
        {
            let mut mempool = self.app.mempool.lock().unwrap();
            for tx in &block.transactions {
                mempool.remove(&tx.hash);
            }
        }

        // 7. Persist block + state
        if let Err(e) = self.app.storage.put_block(&block) {
            return HandleResult::BlockRejected(format!("storage failed: {e:?}"));
        }
        {
            let state_guard = self.app.state_db.lock().unwrap();
            if let Err(e) = self.app.storage.put_state(&state_guard) {
                return HandleResult::BlockRejected(format!("state save failed: {e:?}"));
            }
        }

        // 8. Advance chain head
        {
            let mut head_lock = self.app.chain_head.lock().unwrap();
            head_lock.number = block.header.number;
            head_lock.head_hash = block.hash();
        }

        HandleResult::BlockAccepted
    }

    /// Look up the block at chain_head.number (or genesis if at height 0).
    fn parent_block(&self, head_number: u64) -> Result<Block, String> {
        if head_number == 0 {
            return match self.app.storage.get_block(0) {
                Ok(Some(b)) => Ok(b),
                Ok(None) => Ok(genesis_block()),
                Err(e) => Err(format!("{e:?}")),
            };
        }
        match self.app.storage.get_block(head_number) {
            Ok(Some(b)) => Ok(b),
            Ok(None) => Err(format!("missing block {head_number} in storage")),
            Err(e) => Err(format!("{e:?}")),
        }
    }

    /// Try to produce a new block from the mempool.
    /// Returns Ok(None) if the mempool is empty, or if it isn't this
    /// validator's turn to propose — neither is an error, just nothing to do.
    pub fn try_produce_block(&mut self) -> Result<Option<Block>, String> {
        let head = self.app.chain_head.lock().unwrap().clone();
        let parent = self.parent_block(head.number)?;

        // Proposer-turn check: round-robin by slot, same rule as
        // consensus::Consensus::is_proposer, but actually wired in here.
        if self.registry.len() > 1 {
            let my_address = address_from_pubkey(&self.producer.validator_pk);
            let my_index = self.registry.get_index(&my_address)
                .ok_or_else(|| "this validator is not in the registry".to_string())?;
            let next_slot = parent.header.slot + 1;
            if next_slot % (self.registry.len() as u64) != my_index {
                return Ok(None); // not our turn this slot
            }
        }

        {
            let mempool = self.app.mempool.lock().unwrap();
            if mempool.is_empty() {
                return Ok(None);
            }
        }

        // Execute against a clone — Producer mutates state in place, and we
        // only want to commit if produce_block succeeds end-to-end.
        let mut state_clone = self.app.state_db.lock().unwrap().clone();
        let block = {
            let mut mempool = self.app.mempool.lock().unwrap();
            match self.producer.produce_block(&mut mempool, &mut state_clone, &self.app.storage, &parent) {
                Ok(b) => b,
                Err(e) if e == "mempool empty" => return Ok(None),
                Err(e) => return Err(e),
            }
        };

        // Commit state (producer already persisted block+state via storage,
        // but the in-memory copy used by RPC must be updated too)
        *self.app.state_db.lock().unwrap() = state_clone;

        // Advance head
        {
            let mut head_lock = self.app.chain_head.lock().unwrap();
            head_lock.number = block.header.number;
            head_lock.head_hash = block.hash();
        }

        // Queue for gossip to peers
        self.app.outbox.lock().unwrap().push(GossipMsg::NewBlock(block.clone()));

        Ok(Some(block))
    }

    /// Drain queued outbound gossip messages (new txs from RPC, new blocks
    /// from try_produce_block). The async event loop calls this and
    /// publishes each message via net::publish().
    pub fn drain_outbox(&mut self) -> Vec<GossipMsg> {
        let mut outbox = self.app.outbox.lock().unwrap();
        std::mem::take(&mut *outbox)
    }

    /// If an incoming block announces a number further ahead than we can
    /// use directly (a gap — we're missing one or more blocks in between),
    /// return the sync request that would close it. Returns None if
    /// there's no gap (the block is usable as-is, or it's from a peer
    /// that's behind US, in which case sync doesn't apply).
    ///
    /// Pure and network-agnostic on purpose — the caller (the async event
    /// loop in bin/node.rs) decides what to do with the request, e.g. pick
    /// a peer and send it via the request-response Behaviour.
    pub fn sync_request_for_gap(&self, announced_number: u64) -> Option<crate::sync::SyncRequest> {
        let head_number = self.app.chain_head.lock().unwrap().number;
        if announced_number <= head_number + 1 {
            return None; // no gap: either exactly next, or already behind us
        }
        let from = head_number + 1;
        let to = announced_number.min(from + crate::sync::MAX_SYNC_BATCH - 1);
        Some(crate::sync::SyncRequest { from, to })
    }

    /// Apply a batch of blocks received via sync, in order, reusing the
    /// same validation/execution/commit path as gossiped blocks
    /// (`on_block`). Stops at the first rejected block — a partial batch
    /// still advances the chain by however many blocks DID apply cleanly,
    /// rather than discarding a valid prefix because a later block in the
    /// batch had a problem.
    pub fn apply_sync_blocks(&mut self, blocks: Vec<Block>) -> Result<u64, String> {
        let mut applied = 0u64;
        for block in blocks {
            let number = block.header.number;
            match self.on_block(block) {
                HandleResult::BlockAccepted => applied += 1,
                HandleResult::BlockRejected(e) => {
                    return Err(format!(
                        "sync block {number} rejected after applying {applied} block(s): {e}"
                    ));
                }
                // on_block only ever returns one of the two variants above.
                other => unreachable!("on_block returned unexpected {other:?}"),
            }
        }
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    use crate::mempool::{Mempool, MempoolConfig, Transaction};
    use crate::rpc::ChainHead;
    use crate::state::{Account, StateDB, Storage};
    use std::sync::{Arc, Mutex};

    fn fresh_app_state() -> AppState {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());
        AppState {
            state_db: Arc::new(Mutex::new(StateDB::new())),
            mempool: Arc::new(Mutex::new(Mempool::new(MempoolConfig {
                base_fee: 1_000,
                ..Default::default()
            }))),
            storage: Arc::new(Storage::new().unwrap()),
            chain_head: Arc::new(Mutex::new(ChainHead::default())),
            outbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn make_producer(coinbase: [u8; 32]) -> Producer {
        let (pk, sk) = generate_keypair();
        Producer::new(sk, pk, coinbase)
    }

    fn make_tx(from: u8, nonce: u64) -> Transaction {
        let (pk, sk) = generate_keypair();
        let from_addr = crate::consensus::address_from_pubkey(&pk);
        let mut tx = Transaction {
            hash: [from, nonce as u8, 3, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            from: from_addr,
            to: [2u8; 32],
            value: 10,
            nonce,
            base_fee: 1_000,
            priority_fee: 50,
            gas_limit: 21_000,
            signature: Vec::new(),
            received_at: 0,
            from_pubkey: pk,
        };
        tx.signature = crate::crypto::sign(&sk, &tx.signable_bytes());
        tx
    }

    #[test]
    fn test_bootstrap_sets_genesis_head() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let node = Node::new(app.clone(), producer, registry);

        let head = node.app.chain_head.lock().unwrap().clone();
        assert_eq!(head.number, 0);
        assert_eq!(head.head_hash, genesis_block().hash());

        // genesis was persisted
        let g = app.storage.get_block(0).unwrap().unwrap();
        assert_eq!(g.header.number, 0);
    }

    #[test]
    fn test_on_gossip_tx_accepted() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app, producer, registry);

        let tx = make_tx(1, 0);
        let raw = bincode::serialize(&GossipMsg::NewTx(tx)).unwrap();
        assert_eq!(node.on_gossip(&raw), HandleResult::TxAccepted);
        assert_eq!(node.app.mempool.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_on_gossip_block_wrong_parent_rejected() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app, producer, registry);

        let mut block = genesis_block();
        block.header.number = 1;
        block.header.parent_hash = [0xFFu8; 32]; // wrong on purpose
        block.header.signature = vec![1u8; 2420];

        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        assert!(matches!(node.on_gossip(&raw), HandleResult::BlockRejected(_)));
    }

    #[test]
    fn test_on_gossip_block_tx_root_mismatch_rejected() {
        // A block whose transaction list doesn't match its declared
        // tx_root must be rejected — this is what actually enforces the
        // tx_root field now that it's a real merkle root instead of an
        // always-zero placeholder.
        let app_a = fresh_app_state();
        let app_b = fresh_app_state();

        let producer_a = make_producer([9u8; 32]);
        let producer_b = make_producer([8u8; 32]);

        // node_a gets sole ownership of every slot so this test deterministically
        // produces a block — proposer turn-taking is covered separately by
        // test_try_produce_block_respects_proposer_turn.
        let registry_a = ValidatorRegistry::single(&producer_a.validator_pk);
        let mut registry_b = ValidatorRegistry::new();
        registry_b.insert(producer_a.validator_pk.clone());

        let mut node_a = Node::new(app_a.clone(), producer_a, registry_a);
        let mut node_b = Node::new(app_b.clone(), producer_b, registry_b);

        let tx = make_tx(1, 0);
        app_a.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        app_a.mempool.lock().unwrap().add(tx).unwrap();

        let mut block = node_a.try_produce_block().unwrap().expect("block produced");

        // Tamper with the transaction list after the fact, WITHOUT
        // recomputing tx_root — simulates a peer swapping transactions
        // in transit.
        let extra_tx = make_tx(2, 0);
        block.transactions.push(extra_tx);

        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        let result = node_b.on_gossip(&raw);
        assert!(
            matches!(&result, HandleResult::BlockRejected(msg) if msg.contains("tx_root mismatch")),
            "expected tx_root mismatch rejection, got {result:?}"
        );
    }


    #[test]
    fn test_try_produce_block_empty_mempool_returns_none() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app, producer, registry);
        assert_eq!(node.try_produce_block().unwrap(), None);
    }

    #[test]
    fn test_try_produce_block_advances_head_and_queues_gossip() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app.clone(), producer, registry);

        // Fund sender: needs value(10) + gas_limit(21_000)*base_fee(1_000)
        let tx = make_tx(1, 0);
        let from_addr = tx.from;
        app.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });

        app.mempool.lock().unwrap().add(tx).unwrap();

        let block = node.try_produce_block().unwrap().expect("block produced");
        assert_eq!(block.header.number, 1);
        assert_eq!(block.header.parent_hash, genesis_block().hash());

        let head = app.chain_head.lock().unwrap().clone();
        assert_eq!(head.number, 1);
        assert_eq!(head.head_hash, block.hash());

        // Block was queued for gossip
        let outbox = node.drain_outbox();
        assert_eq!(outbox.len(), 1);
        assert!(matches!(outbox[0], GossipMsg::NewBlock(_)));

        // Sender balance updated: 100_000_000 - 10 - 21_000*1_000
        let alice = app.state_db.lock().unwrap().get_account(&from_addr);
        assert_eq!(alice.balance, 100_000_000 - 10 - 21_000_000);
        assert_eq!(alice.nonce, 1);
    }

    #[test]
    fn test_try_produce_block_respects_proposer_turn() {
        // Two validators registered; only whichever one's turn it is (same
        // round-robin rule as consensus::Consensus::is_proposer) should
        // produce a block — even though BOTH have a pending mempool tx.
        let app_a = fresh_app_state();
        let app_b = fresh_app_state();

        let producer_a = make_producer([9u8; 32]);
        let producer_b = make_producer([8u8; 32]);

        let addr_a = crate::consensus::address_from_pubkey(&producer_a.validator_pk);
        let addr_b = crate::consensus::address_from_pubkey(&producer_b.validator_pk);

        let mut registry = ValidatorRegistry::new();
        registry.insert(producer_a.validator_pk.clone());
        registry.insert(producer_b.validator_pk.clone());

        let index_a = registry.get_index(&addr_a).unwrap();
        let index_b = registry.get_index(&addr_b).unwrap();

        // Genesis is slot 0, so the first block either node tries to
        // produce is for slot 1.
        let next_slot = 1u64;
        let count = registry.len() as u64;
        let a_is_proposer = next_slot % count == index_a;
        let b_is_proposer = next_slot % count == index_b;
        // Sanity check on the test setup itself: with 2 validators and
        // distinct indices (0 and 1), exactly one of them owns slot 1.
        assert_ne!(a_is_proposer, b_is_proposer);

        let mut node_a = Node::new(app_a.clone(), producer_a, registry.clone());
        let mut node_b = Node::new(app_b.clone(), producer_b, registry.clone());

        // Fund a sender and queue a tx on BOTH nodes — so the only thing
        // that can stop the "wrong" node from producing is the turn check.
        let tx_a = make_tx(1, 0);
        app_a.state_db.lock().unwrap().set_account(tx_a.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        app_a.mempool.lock().unwrap().add(tx_a).unwrap();

        let tx_b = make_tx(1, 0);
        app_b.state_db.lock().unwrap().set_account(tx_b.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        app_b.mempool.lock().unwrap().add(tx_b).unwrap();

        let result_a = node_a.try_produce_block().unwrap();
        let result_b = node_b.try_produce_block().unwrap();

        assert_eq!(result_a.is_some(), a_is_proposer, "node A produced a block only if it was A's turn");
        assert_eq!(result_b.is_some(), b_is_proposer, "node B produced a block only if it was B's turn");
    }

    #[test]
    fn test_try_produce_block_errors_if_validator_not_registered() {
        // Fail closed: if this node's own key isn't in the registry it
        // should get an explicit error, not silently produce forever
        // (or silently never produce, which would be just as confusing).
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let other_1 = make_producer([1u8; 32]);
        let other_2 = make_producer([2u8; 32]);

        let mut registry = ValidatorRegistry::new();
        registry.insert(other_1.validator_pk.clone());
        registry.insert(other_2.validator_pk.clone());
        // `producer`'s own pubkey was never inserted into the registry.

        let mut node = Node::new(app.clone(), producer, registry);

        let tx = make_tx(1, 0);
        app.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        app.mempool.lock().unwrap().add(tx).unwrap();

        let err = node.try_produce_block().unwrap_err();
        assert!(err.contains("not in the registry"));
    }


    #[test]
    fn test_produce_then_gossip_to_second_node() {
        // Node A produces a block; Node B (separate state, same genesis)
        // receives it via on_gossip and reaches the same head.
        // M10: B's registry must contain A's pubkey to verify A's signature.
        let app_a = fresh_app_state();
        let app_b = fresh_app_state();

        let producer_a = make_producer([9u8; 32]);
        let producer_b = make_producer([8u8; 32]);

        // Node A gets a single-validator registry so it always owns every
        // slot (this test is about gossip/execution convergence, not
        // proposer turn-taking — that's covered by
        // test_try_produce_block_respects_proposer_turn). Node B just
        // needs A's pubkey in its registry to verify A's signature.
        let registry_a = ValidatorRegistry::single(&producer_a.validator_pk);
        let mut registry_b = ValidatorRegistry::new();
        registry_b.insert(producer_a.validator_pk.clone());

        let mut node_a = Node::new(app_a.clone(), producer_a, registry_a);
        let mut node_b = Node::new(app_b.clone(), producer_b, registry_b);

        // Both start from the same genesis hash
        assert_eq!(
            node_a.app.chain_head.lock().unwrap().head_hash,
            node_b.app.chain_head.lock().unwrap().head_hash
        );

        let tx = make_tx(1, 0);
        app_a.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        // B needs the same starting state to validate the same transfer
        app_b.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });

        app_a.mempool.lock().unwrap().add(tx.clone()).unwrap();

        let block = node_a.try_produce_block().unwrap().expect("block produced");
        let raw = bincode::serialize(&GossipMsg::NewBlock(block.clone())).unwrap();

        let result = node_b.on_gossip(&raw);
        assert_eq!(result, HandleResult::BlockAccepted);

        let head_a = node_a.app.chain_head.lock().unwrap().clone();
        let head_b = node_b.app.chain_head.lock().unwrap().clone();
        assert_eq!(head_a.head_hash, head_b.head_hash);
        assert_eq!(head_b.number, 1);

        // B's state converged with A's after executing the same block
        let bal_a = app_a.state_db.lock().unwrap().get_account(&tx.from).balance;
        let bal_b = app_b.state_db.lock().unwrap().get_account(&tx.from).balance;
        assert_eq!(bal_a, bal_b);
    }

    #[test]
    fn test_sync_request_for_gap_none_when_no_gap() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let node = Node::new(app, producer, registry);
        // head is 0 (genesis); announced = 1 is exactly next, no gap
        assert_eq!(node.sync_request_for_gap(1), None);
        // announced = 0 (behind or equal to us) is also not a gap for us
        assert_eq!(node.sync_request_for_gap(0), None);
    }

    #[test]
    fn test_sync_request_for_gap_detected() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let node = Node::new(app, producer, registry);
        // head is 0; a peer announces block 5 — blocks 1..4 are missing
        let req = node.sync_request_for_gap(5).expect("gap should be detected");
        assert_eq!(req.from, 1);
        assert_eq!(req.to, 5);
    }

    #[test]
    fn test_sync_request_for_gap_clamped_to_max_batch() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let node = Node::new(app, producer, registry);
        let far_ahead = crate::sync::MAX_SYNC_BATCH * 10;
        let req = node.sync_request_for_gap(far_ahead).unwrap();
        assert_eq!(req.from, 1);
        assert_eq!(req.to - req.from + 1, crate::sync::MAX_SYNC_BATCH);
    }

    #[test]
    fn test_apply_sync_blocks_advances_head() {
        // Producer node builds up a short real chain; a second, fresh
        // node applies those blocks via apply_sync_blocks (simulating
        // what happens after a sync response arrives) and ends up at the
        // same head, without ever calling try_produce_block itself.
        let app_producer = fresh_app_state();
        let app_syncer = fresh_app_state();

        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);

        let mut producer_node = Node::new(app_producer.clone(), producer, registry.clone());

        let tx = make_tx(1, 0);
        app_producer.state_db.lock().unwrap().set_account(tx.from, Account {
            balance: 100_000_000,
            nonce: 0,
            ..Default::default()
        });
        app_producer.mempool.lock().unwrap().add(tx).unwrap();
        let block = producer_node.try_produce_block().unwrap().expect("block produced");

        // Syncer needs the producer's pubkey in ITS registry to verify
        // the block's signature, same as the gossip path requires.
        let mut syncer_node = Node::new(app_syncer.clone(), make_producer([1u8; 32]), registry);

        let applied = syncer_node.apply_sync_blocks(vec![block.clone()]).unwrap();
        assert_eq!(applied, 1);

        let syncer_head = app_syncer.chain_head.lock().unwrap().clone();
        assert_eq!(syncer_head.number, 1);
        assert_eq!(syncer_head.head_hash, block.hash());
    }

    #[test]
    fn test_apply_sync_blocks_stops_at_first_rejected_block() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app, producer, registry);

        let mut bad_block = genesis_block();
        bad_block.header.number = 1;
        bad_block.header.parent_hash = [0xFFu8; 32]; // wrong parent, will be rejected
        bad_block.header.signature = vec![1u8; 2420];

        let err = node.apply_sync_blocks(vec![bad_block]).unwrap_err();
        assert!(err.contains("rejected after applying 0 block"));
    }
    #[test]
    fn test_drain_outbox_empties_queue() {
        let app = fresh_app_state();
        let producer = make_producer([9u8; 32]);
        let registry = ValidatorRegistry::single(&producer.validator_pk);
        let mut node = Node::new(app, producer, registry);
        node.app.outbox.lock().unwrap().push(GossipMsg::NewTx(make_tx(1, 0)));
        assert_eq!(node.drain_outbox().len(), 1);
        assert_eq!(node.drain_outbox().len(), 0); // drained
    }
}
