// src/sync/mod.rs
// QTC: State sync — request/response protocol for backfilling missing
// blocks. A brand-new gap: qc-node had NO mechanism for a node to catch up
// on history it missed (crashed, just joined, network blip). Gossipsub
// only ever carries the newest block as it's produced; there was nothing
// for a node that's behind to ask a peer "send me blocks N..M".
//
// This module is deliberately networking-agnostic: SyncRequest/SyncResponse
// are plain serializable data, and build_sync_response is a pure function
// over Storage. The actual peer-to-peer wiring (libp2p request-response
// Behaviour + Codec) lives in net::sync_codec — that's the only part of
// this feature that needs a real multi-node network to test end-to-end,
// so keeping the actual sync logic here means it's fully unit-testable
// without any networking at all.

use serde::{Deserialize, Serialize};
use crate::state::Storage;

/// Max blocks returned per sync response. Keeps any single response
/// bounded in size/time regardless of how far behind a peer has fallen —
/// a node 100,000 blocks behind syncs in batches, not one giant response.
pub const MAX_SYNC_BATCH: u64 = 128;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncRequest {
    /// First block number requested (inclusive).
    pub from: u64,
    /// Last block number requested (inclusive). The responder clamps
    /// this to `from + MAX_SYNC_BATCH - 1` regardless of what's asked.
    pub to: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Blocks in ascending order by number, starting at the request's
    /// `from`. May be shorter than requested if the responder doesn't
    /// have all of them (stops at the first gap in ITS OWN storage), or
    /// the range was clamped to MAX_SYNC_BATCH.
    pub blocks: Vec<crate::chain::Block>,
}

/// Build a response to a SyncRequest by reading sequentially from storage.
/// Stops at the first missing block rather than erroring out entirely —
/// a partial response covering the blocks the responder DOES have is more
/// useful to the requester than an all-or-nothing failure.
pub fn build_sync_response(storage: &Storage, req: &SyncRequest) -> SyncResponse {
    if req.to < req.from {
        return SyncResponse { blocks: Vec::new() };
    }
    let capped_to = req.to.min(req.from.saturating_add(MAX_SYNC_BATCH - 1));

    let mut blocks = Vec::new();
    for number in req.from..=capped_to {
        match storage.get_block(number) {
            Ok(Some(block)) => blocks.push(block),
            _ => break, // first gap (or storage error) — stop, return what we have
        }
    }
    SyncResponse { blocks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{genesis_block, Block, BlockHeader};

    fn fresh_storage() -> Storage {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());
        Storage::new().unwrap()
    }

    fn block_at(number: u64, parent_hash: [u8; 32]) -> Block {
        Block {
            header: BlockHeader {
                parent_hash,
                number,
                slot: number,
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
    fn test_build_sync_response_returns_available_blocks_in_order() {
        let storage = fresh_storage();
        let genesis = genesis_block();
        storage.put_block(&genesis).unwrap();
        let b1 = block_at(1, genesis.hash());
        let b2 = block_at(2, b1.hash());
        storage.put_block(&b1).unwrap();
        storage.put_block(&b2).unwrap();

        let resp = build_sync_response(&storage, &SyncRequest { from: 1, to: 2 });
        assert_eq!(resp.blocks.len(), 2);
        assert_eq!(resp.blocks[0].header.number, 1);
        assert_eq!(resp.blocks[1].header.number, 2);
    }

    #[test]
    fn test_build_sync_response_stops_at_first_gap() {
        let storage = fresh_storage();
        let genesis = genesis_block();
        storage.put_block(&genesis).unwrap();
        let b1 = block_at(1, genesis.hash());
        storage.put_block(&b1).unwrap();
        // block 2 deliberately never stored — gap

        let resp = build_sync_response(&storage, &SyncRequest { from: 1, to: 5 });
        assert_eq!(resp.blocks.len(), 1);
        assert_eq!(resp.blocks[0].header.number, 1);
    }

    #[test]
    fn test_build_sync_response_empty_when_nothing_stored() {
        let storage = fresh_storage();
        let resp = build_sync_response(&storage, &SyncRequest { from: 1, to: 10 });
        assert!(resp.blocks.is_empty());
    }

    #[test]
    fn test_build_sync_response_clamped_to_max_batch() {
        let storage = fresh_storage();
        let genesis = genesis_block();
        storage.put_block(&genesis).unwrap();
        let mut parent_hash = genesis.hash();
        for n in 1..=(MAX_SYNC_BATCH + 10) {
            let b = block_at(n, parent_hash);
            parent_hash = b.hash();
            storage.put_block(&b).unwrap();
        }

        let resp = build_sync_response(&storage, &SyncRequest { from: 1, to: MAX_SYNC_BATCH + 10 });
        assert_eq!(resp.blocks.len() as u64, MAX_SYNC_BATCH);
    }

    #[test]
    fn test_build_sync_response_invalid_range_returns_empty() {
        let storage = fresh_storage();
        let resp = build_sync_response(&storage, &SyncRequest { from: 10, to: 5 });
        assert!(resp.blocks.is_empty());
    }
}
