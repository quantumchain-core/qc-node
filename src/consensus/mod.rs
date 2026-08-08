// src/consensus/mod.rs
// QTC M5/M10: Consensus Engine
//
// This file used to also contain a second, complete proposer-selection
// and block-production implementation (`Consensus` struct, `try_propose`,
// `is_proposer`, etc.) alongside the one actually used by the live node.
// That second implementation was NEVER called by anything outside its own
// tests — real proposer-turn enforcement lives in
// `crate::node::Node::try_produce_block`, wired directly to
// `ValidatorRegistry` (see registry.rs). The old copy here checked a
// legacy `QC_VALIDATOR_COUNT` env var instead of the real registry, which
// is exactly the kind of thing that causes confusion (and, per a recent
// review, got mistaken for an active bug) precisely because it looked
// real but could never actually run. Removed entirely rather than left
// as a trap for the next person reading this file.
//
// Also removed: `MAX_TXS_PER_BLOCK`, `ValidatorId`, and `SlotProposer`,
// which had zero usages anywhere outside this file, and a duplicate,
// never-wired-in `calculate_next_base_fee` — the live block-production
// path (`producer.rs`) currently just carries `parent.header.base_fee`
// forward unchanged rather than adjusting it based on network
// congestion, unlike this file's old (tested, but dead) implementation.
// That's a real, separate gap worth fixing on its own, not folded into
// this cleanup.

pub mod producer;
pub mod registry;
pub mod validator;

pub use producer::Producer;
pub use registry::{address_from_pubkey, ValidatorRegistry};
pub use validator::validate_block_sig;

pub const BLOCK_TIME_SECS: u64 = 2;

/// Round-robin proposer-turn check: is the validator at `my_address` the
/// proposer for `slot`, given the current `registry`?
///
/// This is the actual proposer-selection rule the live node uses (see
/// `node::Node::try_produce_block`, which calls this) — living here in
/// the consensus module, not buried inline in node/mod.rs, so
/// "consensus" actually contains the consensus decision rather than
/// being just a pass-through of re-exports.
///
/// Returns:
///   - `Some(true)`  — it's this validator's turn (including the trivial
///                     single-validator case: nobody to rotate with)
///   - `Some(false)` — registered, but it's someone else's turn
///   - `None`        — `my_address` isn't in the registry at all
pub fn is_proposer_for_slot(
    registry: &ValidatorRegistry,
    my_address: &crate::chain::Address,
    slot: u64,
) -> Option<bool> {
    if registry.len() <= 1 {
        return Some(true);
    }
    let my_index = registry.get_index(my_address)?;
    Some(slot % (registry.len() as u64) == my_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    #[test]
    fn test_is_proposer_for_slot_single_validator_always_true() {
        let (pk, _sk) = generate_keypair();
        let registry = ValidatorRegistry::single(&pk);
        let addr = address_from_pubkey(&pk);
        assert_eq!(is_proposer_for_slot(&registry, &addr, 0), Some(true));
        assert_eq!(is_proposer_for_slot(&registry, &addr, 999), Some(true));
    }

    #[test]
    fn test_is_proposer_for_slot_round_robin_two_validators() {
        let (pk_a, _) = generate_keypair();
        let (pk_b, _) = generate_keypair();
        let mut registry = ValidatorRegistry::new();
        registry.insert(pk_a.clone());
        registry.insert(pk_b.clone());
        let addr_a = address_from_pubkey(&pk_a);
        let addr_b = address_from_pubkey(&pk_b);

        let index_a = registry.get_index(&addr_a).unwrap();
        let index_b = registry.get_index(&addr_b).unwrap();
        assert_ne!(index_a, index_b);

        // Exactly one of the two should own any given slot.
        for slot in 0..10u64 {
            let a_turn = is_proposer_for_slot(&registry, &addr_a, slot).unwrap();
            let b_turn = is_proposer_for_slot(&registry, &addr_b, slot).unwrap();
            assert_ne!(a_turn, b_turn, "slot {slot}: exactly one validator should own it");
        }
    }

    #[test]
    fn test_is_proposer_for_slot_unregistered_address_returns_none() {
        let (pk_a, _) = generate_keypair();
        let (pk_b, _) = generate_keypair();
        let (pk_unregistered, _) = generate_keypair();
        let mut registry = ValidatorRegistry::new();
        registry.insert(pk_a);
        registry.insert(pk_b);
        let unregistered_addr = address_from_pubkey(&pk_unregistered);
        assert_eq!(is_proposer_for_slot(&registry, &unregistered_addr, 0), None);
    }
}
