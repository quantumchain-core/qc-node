// src/consensus/validator.rs
// QTC M5: Block Validator
// Verifies Dilithium2 signature on block header

use crate::chain::Block;
use crate::crypto::verify;

/// Validate a block's proposer signature.
/// Returns Ok(()) if valid, Err with reason if not.
pub fn validate_block_sig(block: &Block) -> Result<(), String> {
    let signable = block.header.to_signable_bytes();
    let pk = &block.header.proposer; // 32-byte address — not full pubkey

    // NOTE: proposer field is currently a 32-byte address hash, not full pubkey.
    // For real sig verification we need the full 1312-byte Dilithium2 pubkey.
    // This will be wired when validator registry is added in M7.
    // For now: accept all blocks with non-empty signatures (placeholder).
    if block.header.signature.is_empty() {
        return Err("missing signature".into());
    }

    // TODO M7: look up full pubkey from validator registry by proposer address
    // if !verify(&signable, &block.header.signature, full_pk) {
    //     return Err("invalid signature".into());
    // }
    let _ = (signable, pk); // suppress unused warnings until M7

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{Block, BlockHeader};

    fn make_block(sig: Vec<u8>) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: [0u8; 32],
                number: 1,
                slot: 1,
                timestamp: 1000,
                proposer: [1u8; 32],
                tx_root: [0u8; 32],
                state_root: [0u8; 32],
                base_fee: 1000,
                gas_used: 0,
                gas_limit: 30_000_000,
                signature: sig,
            },
            transactions: vec![],
        }
    }

    #[test]
    fn test_empty_sig_rejected() {
        let block = make_block(vec![]);
        assert!(validate_block_sig(&block).is_err());
    }

    #[test]
    fn test_nonempty_sig_accepted() {
        let block = make_block(vec![0u8; 2420]);
        assert!(validate_block_sig(&block).is_ok());
    }
}
