pub mod dilithium;

// Re-export M1 types so M2/M3 can use them
pub use dilithium::{
    DilithiumPubKey, 
    DilithiumSecretKey, 
    DilithiumSig,
    generate_keypair,
    sign, 
    verify,
    hash_sha3,
    RAMPURA_TESTNET_0_CHAIN_ID,
};
