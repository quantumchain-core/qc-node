pub mod crypto;  // M1
pub mod net;     // M2 
pub mod chain;   // M3

// Make M1 available to everyone
pub use crypto::{
    DilithiumPubKey,
    DilithiumSecretKey, 
    DilithiumSig,
    generate_keypair,
};
