pub mod crypto;  // M1: Dilithium keygen
pub mod net;     // M2: P2P 
pub mod chain;   // M3: Signed blocks

pub use crypto::Keypair; // ← This blocks lets M2/M3 use M1's Keypair
