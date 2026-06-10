pub mod crypto; // M1
pub mod net; // M2 
pub mod chain; // M3

// M1: Export the actual types you have in dilithium.rs
pub use crypto::{
    DilithiumPubKey,
    DilithiumSecretKey, 
    DilithiumSig,
    generate_keypair, // ← This is your function name, not Keypair::generate
    sign,
    verify,
};
