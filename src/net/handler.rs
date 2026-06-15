// src/net/handler.rs
// QTC M7/M10: Gossip Message Handler
//
// Lightweight, standalone validation of incoming gossip messages — checks
// the wire format and a cheap non-empty-signature check on blocks.
//
// NOTE (M10): Full validation (real Dilithium2 verification against the
// ValidatorRegistry, state execution, chain-head updates) happens in
// node::Node::on_block, which has access to AppState + ValidatorRegistry.
// This handler remains a minimal, registry-free check used for isolated
// wire-format tests.

use crate::chain::Block;
use crate::mempool::{Mempool, Transaction};

/// Messages gossiped over the qc-blocks and qc-txs topics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GossipMsg {
    NewBlock(Block),
    NewTx(Transaction),
}

/// Result of processing an incoming gossip message
#[derive(Debug, PartialEq)]
pub enum HandleResult {
    BlockAccepted,
    BlockRejected(String),
    TxAccepted,
    TxRejected(String),
}

/// Process a raw gossip message received from a peer.
/// Returns HandleResult so the caller knows what happened.
pub fn handle_gossip(
    raw: &[u8],
    mempool: &mut Mempool,
    head_hash: &[u8; 32],
) -> HandleResult {
    // 1. Deserialize
    let msg: GossipMsg = match bincode::deserialize(raw) {
        Ok(m) => m,
        Err(e) => return HandleResult::BlockRejected(format!("deserialize failed: {e}")),
    };

    match msg {
        GossipMsg::NewBlock(block) => handle_block(block, mempool, head_hash),
        GossipMsg::NewTx(tx) => handle_tx(tx, mempool),
    }
}

fn handle_block(
    block: Block,
    mempool: &mut Mempool,
    head_hash: &[u8; 32],
) -> HandleResult {
    // 1. Check parent links to our current head
    if &block.header.parent_hash != head_hash {
        return HandleResult::BlockRejected(format!(
            "unknown parent: expected {:?}, got {:?}",
            head_hash, block.header.parent_hash
        ));
    }

    // 2. Cheap signature presence check (full verify happens in node::Node::on_block)
    if block.header.signature.is_empty() {
        return HandleResult::BlockRejected("missing signature".into());
    }

    // 3. Remove txs that are now included in this block from mempool
    for tx in &block.transactions {
        mempool.remove(&tx.hash);
    }

    HandleResult::BlockAccepted
}

fn handle_tx(tx: Transaction, mempool: &mut Mempool) -> HandleResult {
    match mempool.add(tx) {
        Ok(()) => HandleResult::TxAccepted,
        Err(e) => HandleResult::TxRejected(format!("{e:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{Block, BlockHeader};
    use crate::mempool::{Mempool, MempoolConfig, Transaction};

    fn make_tx(from: u8, nonce: u64) -> Transaction {
        let mut from_addr = [0u8; 32];
        from_addr[0] = from;
        Transaction {
            hash: [from, nonce as u8, 1, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            from: from_addr,
            to: [2u8; 32],
            value: 100,
            nonce,
            base_fee: 1_000,
            priority_fee: 100,
            gas_limit: 21_000,
            signature: vec![0u8; 2420],
            received_at: 0,
        }
    }

    fn make_block(parent: [u8; 32], sig: Vec<u8>) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: parent,
                number: 1,
                slot: 1,
                timestamp: 0,
                proposer: [1u8; 32],
                tx_root: [0u8; 32],
                state_root: [0u8; 32],
                base_fee: 1_000,
                gas_used: 21_000,
                gas_limit: 30_000_000,
                signature: sig,
            },
            transactions: vec![],
        }
    }

    #[test]
    fn test_valid_block_accepted() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let head = [0u8; 32];
        let block = make_block(head, vec![1u8; 2420]);
        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        assert_eq!(handle_gossip(&raw, &mut mempool, &head), HandleResult::BlockAccepted);
    }

    #[test]
    fn test_block_wrong_parent_rejected() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let head = [0u8; 32];
        let wrong_parent = [9u8; 32];
        let block = make_block(wrong_parent, vec![1u8; 2420]);
        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        assert!(matches!(
            handle_gossip(&raw, &mut mempool, &head),
            HandleResult::BlockRejected(_)
        ));
    }

    #[test]
    fn test_block_empty_sig_rejected() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let head = [0u8; 32];
        let block = make_block(head, vec![]);
        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        assert!(matches!(
            handle_gossip(&raw, &mut mempool, &head),
            HandleResult::BlockRejected(_)
        ));
    }

    #[test]
    fn test_valid_tx_accepted() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let head = [0u8; 32];
        let tx = make_tx(1, 0);
        let raw = bincode::serialize(&GossipMsg::NewTx(tx)).unwrap();
        assert_eq!(handle_gossip(&raw, &mut mempool, &head), HandleResult::TxAccepted);
    }

    #[test]
    fn test_duplicate_tx_rejected() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let head = [0u8; 32];
        let tx = make_tx(1, 0);
        let raw = bincode::serialize(&GossipMsg::NewTx(tx)).unwrap();
        handle_gossip(&raw, &mut mempool, &head); // first
        let tx2 = make_tx(1, 0);
        let raw2 = bincode::serialize(&GossipMsg::NewTx(tx2)).unwrap();
        assert!(matches!(
            handle_gossip(&raw2, &mut mempool, &head),
            HandleResult::TxRejected(_)
        ));
    }

    #[test]
    fn test_block_removes_txs_from_mempool() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let tx = make_tx(1, 0);
        let tx_hash = tx.hash;
        mempool.add(tx.clone()).unwrap();
        assert_eq!(mempool.len(), 1);

        let head = [0u8; 32];
        let mut block = make_block(head, vec![1u8; 2420]);
        block.transactions.push(tx);
        let raw = bincode::serialize(&GossipMsg::NewBlock(block)).unwrap();
        handle_gossip(&raw, &mut mempool, &head);

        assert!(mempool.get(&tx_hash).is_none());
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_corrupt_bytes_rejected() {
        let mut mempool = Mempool::new(MempoolConfig::default());
        let raw = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(matches!(
            handle_gossip(&raw, &mut mempool, &[0u8; 32]),
            HandleResult::BlockRejected(_)
        ));
    }
}
