// src/keystore.rs
// QTC: encrypted validator keystore (AES-256-GCM + Argon2id).
//
// Moved here from src/bin/node.rs, verbatim, so a second binary
// (src/bin/keygen.rs) can load/generate a validator identity the exact
// same way the real node does — critical for integration testing, where
// a genesis file needs to list real validator pubkeys BEFORE the full
// node process starts. Sharing this code path means the genesis file and
// the running node can never disagree about who a validator actually is.
//
// Behavior is unchanged from the original bin/node.rs version — same
// function signatures, same tests, moved as-is rather than rewritten.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use argon2::Argon2;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use rand::RngCore;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
struct Keystore {
    pk_hex: String,
    encrypted_sk: String,
    salt_hex: String,
    nonce_hex: String,
}

const AES_KEY_LEN: usize = 32; // AES-256

pub fn keystore_path() -> PathBuf {
    let path = std::env::var("QC_KEYSTORE_PATH")
        .unwrap_or_else(|_| "./qc-keystore.json".to_string());
    PathBuf::from(path)
}

/// Derive a 32-byte AES key from `password` + `salt` via Argon2id.
/// Uses the low-level raw-bytes API (`hash_password_into`), NOT the
/// high-level `hash_password` (which expects a PHC-formatted `SaltString`,
/// not raw salt bytes — that mismatch was the original compile error here).
fn derive_key(argon2: &Argon2, password: &str, salt: &[u8]) -> Result<[u8; AES_KEY_LEN], Box<dyn std::error::Error>> {
    let mut key = [0u8; AES_KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("key derivation failed: {e}"))?;
    Ok(key)
}

/// Read `QC_KEYSTORE_PASSWORD` from the environment. Unlike the previous
/// version, this does NOT fall back to a hardcoded default — a default
/// baked into public source code isn't a secret, so a silent fallback here
/// would mean "encrypted" keystores are only as safe as a string anyone can
/// read on GitHub. Refusing to start is safer than starting insecurely.
pub fn require_keystore_password() -> Result<String, Box<dyn std::error::Error>> {
    std::env::var("QC_KEYSTORE_PASSWORD").map_err(|_| {
        "QC_KEYSTORE_PASSWORD is not set. Refusing to start: there is no safe default \
         for the keystore encryption password. Set QC_KEYSTORE_PASSWORD before launching the node."
            .into()
    })
}

pub fn load_or_generate_keypair() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let path = keystore_path();
    let password = require_keystore_password()?;
    let argon2 = Argon2::default();

    if path.exists() {
        restrict_keystore_permissions(&path)?;
        let json = std::fs::read_to_string(&path)?;
        let ks: Keystore = serde_json::from_str(&json)?;

        let salt = hex::decode(&ks.salt_hex)?;
        let nonce_bytes = hex::decode(&ks.nonce_hex)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut key_bytes = derive_key(&argon2, &password, &salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        key_bytes.zeroize();

        let ciphertext = hex::decode(&ks.encrypted_sk)?;
        let sk = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "incorrect QC_KEYSTORE_PASSWORD or corrupted keystore file")?;

        println!("✅ Loaded encrypted keystore from {}", path.display());
        Ok((hex::decode(&ks.pk_hex)?, sk))
    } else {
        let (pk, sk) = crate::crypto::generate_keypair();

        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let mut key_bytes = derive_key(&argon2, &password, &salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        key_bytes.zeroize();

        let encrypted = cipher
            .encrypt(&nonce, sk.as_ref())
            .map_err(|_| "keystore encryption failed")?;

        let ks = Keystore {
            pk_hex: hex::encode(&pk),
            encrypted_sk: hex::encode(encrypted),
            salt_hex: hex::encode(salt),
            nonce_hex: hex::encode(nonce.as_slice()),
        };

        std::fs::write(&path, serde_json::to_string_pretty(&ks)?)?;
        restrict_keystore_permissions(&path)?;
        println!("✅ Created encrypted keystore at {}", path.display());
        Ok((pk, sk))
    }
}

/// Restrict the keystore file to owner read/write only (0600). Without
/// this, the file inherits the process umask — commonly 0644, meaning
/// any other local user on the box can read the encrypted blob and mount
/// an offline password-guessing attack against it, with no need to
/// exploit anything else first. Unix-only; Windows ACLs would need a
/// different mechanism (not implemented here since RUN_VALIDATOR.md only
/// documents Ubuntu deployment).
#[cfg(unix)]
pub fn restrict_keystore_permissions(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn restrict_keystore_permissions(_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_restrict_keystore_permissions_sets_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        // Start deliberately permissive, to prove this actually tightens
        // permissions rather than just happening to already be 0600.
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).unwrap();

        restrict_keystore_permissions(tmp.path()).unwrap();

        let mode = std::fs::metadata(tmp.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    #[cfg(unix)]
    fn test_load_or_generate_keypair_creates_keystore_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = tmp_dir.path().join("qc-keystore.json");

        std::env::set_var("QC_KEYSTORE_PATH", &keystore_path);
        std::env::set_var("QC_KEYSTORE_PASSWORD", "test-password-for-unit-test-only");

        let (pk, sk) = load_or_generate_keypair().unwrap();
        assert!(!pk.is_empty());
        assert!(!sk.is_empty());

        let mode = std::fs::metadata(&keystore_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        std::env::remove_var("QC_KEYSTORE_PATH");
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
    }

    #[test]
    #[cfg(unix)]
    fn test_load_or_generate_keypair_self_heals_loose_permissions_on_reload() {
        // Simulates a keystore that predates this fix, or got copied in
        // some other way that lost its restrictive permissions — loading
        // it should tighten permissions, not just leave them as-is.
        use std::os::unix::fs::PermissionsExt;
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = tmp_dir.path().join("qc-keystore.json");

        std::env::set_var("QC_KEYSTORE_PATH", &keystore_path);
        std::env::set_var("QC_KEYSTORE_PASSWORD", "test-password-for-unit-test-only");

        // First call creates it (already 0600 per the test above).
        load_or_generate_keypair().unwrap();
        // Deliberately loosen it, as if copied from elsewhere.
        std::fs::set_permissions(&keystore_path, std::fs::Permissions::from_mode(0o644)).unwrap();

        // Second call takes the "load existing" branch.
        load_or_generate_keypair().unwrap();

        let mode = std::fs::metadata(&keystore_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        std::env::remove_var("QC_KEYSTORE_PATH");
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
    }

    #[test]
    fn test_require_keystore_password_errors_when_unset() {
        std::env::remove_var("QC_KEYSTORE_PASSWORD");
        assert!(require_keystore_password().is_err());
    }
}
