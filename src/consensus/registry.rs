// src/consensus/registry.rs
// QTC M10: Validator Registry
//
// Maps a 32-byte validator Address -> full Dilithium2 public key (1312 bytes).
// Address = SHA3-256(pubkey) — per whitepaper sec 4.2, addresses are a hash
// of the Dilithium public key, and SHA3-256 is the spec'd hash (NIST FIPS 202).

use std::collections::HashMap;
use std::fs;
use sha3::{Digest, Sha3_256};
use serde::Deserialize;
use crate::chain::Address;

/// Derive a 32-byte address from a Dilithium2 public key via SHA3-256.
pub fn address_from_pubkey(pk: &[u8]) -> Address {
    let mut hasher = Sha3_256::new();
    hasher.update(pk);
    hasher.finalize().into()
}

#[derive(Debug, Clone, Default)]
pub struct ValidatorRegistry {
    validators: HashMap<Address, Vec<u8>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a validator by its full public key. Returns the derived address.
    pub fn insert(&mut self, pubkey: Vec<u8>) -> Address {
        let addr = address_from_pubkey(&pubkey);
        self.validators.insert(addr, pubkey);
        addr
    }

    /// Look up a validator's public key by address.
    pub fn get(&self, addr: &Address) -> Option<&Vec<u8>> {
        self.validators.get(addr)
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Convenience for single-validator setups (local dev, tests).
    pub fn single(pubkey: &[u8]) -> Self {
        let mut r = Self::new();
        r.insert(pubkey.to_vec());
        r
    }

    /// Load a multi-validator registry from a genesis JSON file.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "validators": [
    ///     { "address": "0x<64 hex chars>", "pubkey": "0x<2624 hex chars>" }
    ///   ]
    /// }
    /// ```
    /// Each entry's `address` MUST equal SHA3-256(pubkey) — mismatches are
    /// rejected, catching copy/paste errors in genesis config.
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let data = fs::read_to_string(path)
            .map_err(|e| format!("failed to read genesis file {path}: {e}"))?;
        Self::from_json(&data)
    }

    pub fn from_json(data: &str) -> Result<Self, String> {
        let genesis: GenesisConfig =
            serde_json::from_str(data).map_err(|e| format!("invalid genesis JSON: {e}"))?;

        let mut registry = Self::new();
        for entry in genesis.validators {
            let pubkey_hex = entry.pubkey.strip_prefix("0x").unwrap_or(&entry.pubkey);
            let pubkey = hex::decode(pubkey_hex)
                .map_err(|e| format!("invalid pubkey hex for {}: {e}", entry.address))?;
            if pubkey.len() != 1312 {
                return Err(format!(
                    "validator {} has pubkey of {} bytes, expected 1312 (Dilithium2)",
                    entry.address,
                    pubkey.len()
                ));
            }

            let addr_hex = entry.address.strip_prefix("0x").unwrap_or(&entry.address);
            let addr_bytes = hex::decode(addr_hex)
                .map_err(|e| format!("invalid address hex {}: {e}", entry.address))?;
            if addr_bytes.len() != 32 {
                return Err(format!("address {} must be 32 bytes", entry.address));
            }
            let mut declared_addr = [0u8; 32];
            declared_addr.copy_from_slice(&addr_bytes);

            let derived_addr = address_from_pubkey(&pubkey);
            if derived_addr != declared_addr {
                return Err(format!(
                    "address mismatch for {}: declared does not equal SHA3-256(pubkey) = 0x{}",
                    entry.address,
                    hex::encode(derived_addr)
                ));
            }

            registry.validators.insert(declared_addr, pubkey);
        }

        Ok(registry)
    }
}

#[derive(Debug, Deserialize)]
struct GenesisConfig {
    validators: Vec<GenesisValidator>,
}

#[derive(Debug, Deserialize)]
struct GenesisValidator {
    address: String,
    pubkey: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    #[test]
    fn test_address_derivation_deterministic() {
        let (pk, _) = generate_keypair();
        assert_eq!(address_from_pubkey(&pk), address_from_pubkey(&pk));
    }

    #[test]
    fn test_single_registry_lookup() {
        let (pk, _) = generate_keypair();
        let registry = ValidatorRegistry::single(&pk);
        let addr = address_from_pubkey(&pk);
        assert_eq!(registry.get(&addr), Some(&pk));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_unknown_address_returns_none() {
        let registry = ValidatorRegistry::new();
        assert_eq!(registry.get(&[0u8; 32]), None);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_load_from_json_valid() {
        let (pk, _) = generate_keypair();
        let addr = address_from_pubkey(&pk);
        let json = format!(
            r#"{{"validators":[{{"address":"0x{}","pubkey":"0x{}"}}]}}"#,
            hex::encode(addr),
            hex::encode(&pk)
        );
        let registry = ValidatorRegistry::from_json(&json).unwrap();
        assert_eq!(registry.get(&addr), Some(&pk));
    }

    #[test]
    fn test_load_from_json_address_mismatch_rejected() {
        let (pk, _) = generate_keypair();
        // deliberately wrong address (all zeros)
        let json = format!(
            r#"{{"validators":[{{"address":"0x{}","pubkey":"0x{}"}}]}}"#,
            hex::encode([0u8; 32]),
            hex::encode(&pk)
        );
        assert!(ValidatorRegistry::from_json(&json).is_err());
    }

    #[test]
    fn test_load_from_json_wrong_pubkey_length_rejected() {
        let json = r#"{"validators":[{"address":"0x00","pubkey":"0xdead"}]}"#;
        assert!(ValidatorRegistry::from_json(json).is_err());
    }

    #[test]
    fn test_load_from_json_multiple_validators() {
        let (pk1, _) = generate_keypair();
        let (pk2, _) = generate_keypair();
        let addr1 = address_from_pubkey(&pk1);
        let addr2 = address_from_pubkey(&pk2);
        let json = format!(
            r#"{{"validators":[{{"address":"0x{}","pubkey":"0x{}"}},{{"address":"0x{}","pubkey":"0x{}"}}]}}"#,
            hex::encode(addr1), hex::encode(&pk1),
            hex::encode(addr2), hex::encode(&pk2),
        );
        let registry = ValidatorRegistry::from_json(&json).unwrap();
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get(&addr1), Some(&pk1));
        assert_eq!(registry.get(&addr2), Some(&pk2));
    }

    #[test]
    fn test_load_from_file_not_found() {
        assert!(ValidatorRegistry::load_from_file("/nonexistent/genesis.json").is_err());
    }
}
