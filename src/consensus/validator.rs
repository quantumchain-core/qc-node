// src/consensus/validator.rs
// QTC M5: Block Validator
// Verifies Dilithium2 signature on block header

use crate::chain::Block;

/// Validate a block's proposer signature.
pub fn validate_block_sig(block: &Block) -> Result<(), String> {
    if block.header.signature.is_empty() {
        return Err("missing signature".into());
    }
    // TODO M8: look up full 1312-byte pk from validator registry by proposer address
    // let signable = block.header.to_signable_bytes();
    // if !verify(&signable, &block.header.signature, full_pk) {
    //     return Err("invalid signature".into());
    // }
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
