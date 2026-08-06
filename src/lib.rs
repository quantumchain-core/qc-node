pub mod crypto;     // M1
pub mod net;        // M2/M7
pub mod chain;      // M3
pub mod mempool;    // M4
pub mod consensus;  // M5
pub mod state;      // M6
pub mod rpc;        // M8
pub mod node;       // M9
pub mod keystore;   // moved from bin/node.rs so keygen helper can share it
pub mod sync;       // M17: state sync (backfilling missing blocks)
pub mod vesting;    // M14 - vesting + TimelockedOpsFund
pub mod governance; // M14 - proposals, 5/7 multisig, immutable rules
