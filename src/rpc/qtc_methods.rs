// src/rpc/qtc_methods.rs
// QTC custom RPC methods for the web dashboard.
//
// qtc_getValidator(address)  — validator stats by address
// qtc_getNetworkStats()      — network-wide statistics  
// qtc_getValidators()        — list all validators (from storage)

use serde_json::{json, Value};
use crate::rpc::methods::AppState;

fn parse_addr(s: &str) -> Result<[u8; 32], String> {
    let clean = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(clean).map_err(|e| format!("invalid address: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("address must be 32 bytes, got {}", bytes.len()));
    }
    let mut addr = [0u8; 32];
    addr.copy_from_slice(&bytes);
    Ok(addr)
}

/// qtc_getValidator(address) → validator stats
pub fn qtc_get_validator(state: &AppState, params: &Value) -> Result<Value, String> {
    let addr_str = params.get(0)
        .and_then(|v| v.as_str())
        .ok_or("missing address param")?;

    let addr = parse_addr(addr_str)?;
    let db = state.state_db.lock().unwrap();
    let account = db.get_account(&addr);
    let head = state.chain_head.lock().unwrap();

    Ok(json!({
        "address": addr_str,
        "balance": format!("0x{:x}", account.balance),
        "nonce": format!("0x{:x}", account.nonce),
        "blocksProduced": format!("0x{:x}", account.nonce),
        "currentBlock": format!("0x{:x}", head.number),
        "status": "active"
    }))
}

/// qtc_getNetworkStats() → network statistics
pub fn qtc_get_network_stats(state: &AppState) -> Value {
    let head = state.chain_head.lock().unwrap();
    let mempool_len = state.mempool.lock().unwrap().len();

    json!({
        "blockNumber": format!("0x{:x}", head.number),
        "blockTime": "0x2",
        "chainId": "0x74",
        "network": "testnet",
        "pendingTxCount": mempool_len,
        "tps": "0x0"
    })
}

/// qtc_getValidators() → validator list from genesis
pub fn qtc_get_validators(state: &AppState) -> Value {
    let head = state.chain_head.lock().unwrap();

    // Read genesis to get validator addresses
    let genesis_path = std::env::var("QC_GENESIS_PATH")
        .unwrap_or_else(|_| "./genesis/testnet.json".to_string());

    let validators = if let Ok(data) = std::fs::read_to_string(&genesis_path) {
        if let Ok(genesis) = serde_json::from_str::<serde_json::Value>(&data) {
            let db = state.state_db.lock().unwrap();
            genesis["validators"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| {
                    let addr_str = v["address"].as_str()?;
                    if let Ok(addr) = parse_addr(addr_str) {
                        let account = db.get_account(&addr);
                        Some(json!({
                            "address": addr_str,
                            "balance": format!("0x{:x}", account.balance),
                            "blocksProduced": format!("0x{:x}", account.nonce),
                            "status": "active"
                        }))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let total = validators.len();
    json!({
        "validators": validators,
        "total": total,
        "currentBlock": format!("0x{:x}", head.number)
    })
}
