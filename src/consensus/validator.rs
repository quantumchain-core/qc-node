// src/consensus/validator.rs
// QTC M10: Block Validator
// Verifies Dilithium2 signature on block header against the ValidatorRegistry.

use crate::chain::Block;
use crate::crypto::verify;
use super::registry::ValidatorRegistry;

/// Validate a block's proposer signature against the validator registry.
pub fn validate_block_sig(block: &Block, registry: &ValidatorRegistry) -> Result<(), String> {
    if block.header.signature.is_empty() {
        return Err("missing signature".into());
    }

    let pubkey = registry
        .get(&block.header.proposer)
        .ok_or_else(|| format!("unknown validator address: 0x{}", hex::encode(block.header.proposer)))?;

    let signable = block.header.to_signable_bytes();
    if !verify(&signable, &block.header.signature, pubkey) {
        return Err("invalid signature".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{Block, BlockHeader};
    use crate::crypto::{generate_keypair, sign};
    use super::super::registry::address_from_pubkey;

    fn make_header(proposer: [u8; 32]) -> BlockHeader {
        BlockHeader {
            parent_hash: [0u8; 32],
            number: 1,
            slot: 1,
            timestamp: 1000,
            proposer,
            tx_root: [0u8; 32],
            state_root: [0u8; 32],
            base_fee: 1000,
            gas_used: 0,
            gas_limit: 30_000_000,
            signature: vec![],
        }
    }

    #[test]
    fn test_empty_sig_rejected() {
        let registry = ValidatorRegistry::new();
        let block = Block { header: make_header([1u8; 32]), transactions: vec![] };
        assert!(validate_block_sig(&block, &registry).is_err());
    }

    #[test]
    fn test_unknown_validator_rejected() {
        let registry = ValidatorRegistry::new(); // empty
        let mut header = make_header([1u8; 32]);
        header.signature = vec![0u8; 2420];
        let block = Block { header, transactions: vec![] };
        assert!(validate_block_sig(&block, &registry).is_err());
    }

    #[test]
    fn test_valid_real_signature_accepted() {
        let (pk, sk) = generate_keypair();
        let registry = ValidatorRegistry::single(&pk);
        let addr = address_from_pubkey(&pk);

        let mut header = make_header(addr);
        header.signature = sign(&sk, &header.to_signable_bytes());

        let block = Block { header, transactions: vec![] };
        assert!(validate_block_sig(&block, &registry).is_ok());
    }

    #[test]
    fn test_tampered_signature_rejected() {
        let (pk, sk) = generate_keypair();
        let registry = ValidatorRegistry::single(&pk);
        let addr = address_from_pubkey(&pk);

        let mut header = make_header(addr);
        header.signature = sign(&sk, &header.to_signable_bytes());
        header.signature[0] ^= 0xFF; // corrupt

        let block = Block { header, transactions: vec![] };
        assert!(validate_block_sig(&block, &registry).is_err());
    }

    #[test]
    fn test_tampered_header_rejected() {
        // signature is valid for the ORIGINAL header, but a signed field
        // changed after signing — verify must catch this.
        let (pk, sk) = generate_keypair();
        let registry = ValidatorRegistry::single(&pk);
        let addr = address_from_pubkey(&pk);

        let mut header = make_header(addr);
        header.signature = sign(&sk, &header.to_signable_bytes());
        header.gas_used = 999_999; // tamper with a signed field

        let block = Block { header, transactions: vec![] };
        assert!(validate_block_sig(&block, &registry).is_err());
    }
}
