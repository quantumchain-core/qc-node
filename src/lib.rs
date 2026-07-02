pub mod crypto;    // M1 - locked
pub mod net;       // M2/M7 - swarm + gossip handler
pub mod chain;     // M3 - done
pub mod mempool;   // M4 - done
pub mod consensus; // M5 - done
pub mod state;     // M6 - done
pub mod rpc;       // M8 - JSON-RPC HTTP server
pub mod node;      // M9 - event-loop core (sync, unit-testable)
pub mod vesting;   // M14 - cliff/linear vesting + TimelockedOpsFund
