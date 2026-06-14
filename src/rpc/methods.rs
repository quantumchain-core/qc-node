// src/rpc/methods.rs
// QTC M8: JSON-RPC method implementations
// Naming follows Ethereum conventions (eth_*) for wallet/tooling compatibility,
// though QTC is its own chain (see whitepaper).

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::chain::{Address, Block};
use crate::mempool::{Mempool, Transaction};
use crate::state::StateDB;
use crate::net::{handler::GossipMsg};

/// Placeholder chain id — finalize in whitepaper / genesis config.
pub const QTC_CHAIN_ID: u64 = 0x51; // 81 decimal

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorObj>,
    pub id: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcErrorObj {
    pub code: i32,
    pub message: String,
}

impl RpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), result: Some(result), error: None, id }
    }
    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcErrorObj { code, message: message.into() }),
            id,
        }
    }
}

// Standard JSON-RPC error codes
pub const ERR_PARSE: i32 = -32700;
pub const ERR_INVALID_PARAMS: i32 = -32602;
pub const ERR_METHOD_NOT_FOUND: i32 = -32601;
pub const ERR_INTERNAL: i32 = -32603;

// ---------------------------------------------------------------------------
// Shared node state, used by the RPC server and (later) the P2P event loop
// ---------------------------------------------------------------------------

/// Minimal chain head info. The producer/event loop updates this after
/// each block. Kept separate from ChainState (consensus) to avoid
/// coupling RPC to consensus internals.
#[derive(Debug, Clone, Default)]
pub struct ChainHead {
    pub number: u64,
    pub head_hash: [u8; 32],
}

#[derive(Clone)]
pub struct AppState {
    pub state_db: Arc<Mutex<StateDB>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub storage: Arc<crate::state::Storage>,
    pub chain_head: Arc<Mutex<ChainHead>>,
    /// Outbound gossip queue. The RPC layer pushes messages here;
    /// the P2P event loop drains it and publishes to peers.
    /// (Avoids RPC needing direct access to the libp2p Swarm.)
    pub outbox: Arc<Mutex<Vec<GossipMsg>>>,
}

// ---------------------------------------------------------------------------
// Hex helpers
// ---------------------------------------------------------------------------

pub fn hex0x(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn u128_to_hex(v: u128) -> String {
    format!("0x{v:x}")
}

pub fn u64_to_hex(v: u64) -> String {
    format!("0x{v:x}")
}

fn parse_address(s: &str) -> Result<Address, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).map_err(|e| format!("invalid hex address: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("address must be 32 bytes, got {}", bytes.len()));
    }
    let mut addr = [0u8; 32];
    addr.copy_from_slice(&bytes);
    Ok(addr)
}

fn param_str(params: &Value, idx: usize) -> Result<String, String> {
    params
        .get(idx)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing or invalid string param at index {idx}"))
}

fn param_u64(params: &Value, idx: usize) -> Result<u64, String> {
    let v = params.get(idx).ok_or_else(|| format!("missing param at index {idx}"))?;
    if let Some(n) = v.as_u64() {
        return Ok(n);
    }
    if let Some(s) = v.as_str() {
        let s = s.strip_prefix("0x").unwrap_or(s);
        return u64::from_str_radix(s, 16).map_err(|e| format!("invalid u64 hex: {e}"));
    }
    Err(format!("param {idx} is not a number or hex string"))
}

// ---------------------------------------------------------------------------
// Block / Transaction -> JSON
// ---------------------------------------------------------------------------

pub fn tx_to_json(tx: &Transaction) -> Value {
    json!({
        "hash": hex0x(&tx.hash),
        "from": hex0x(&tx.from),
        "to": hex0x(&tx.to),
        "value": u64_to_hex(tx.value),
        "nonce": u64_to_hex(tx.nonce),
        "baseFee": u64_to_hex(tx.base_fee),
        "priorityFee": u64_to_hex(tx.priority_fee),
        "gasLimit": u64_to_hex(tx.gas_limit),
    })
}

pub fn block_to_json(block: &Block) -> Value {
    let h = &block.header;
    json!({
        "number": u64_to_hex(h.number),
        "parentHash": hex0x(&h.parent_hash),
        "slot": u64_to_hex(h.slot),
        "timestamp": u64_to_hex(h.timestamp),
        "proposer": hex0x(&h.proposer),
        "txRoot": hex0x(&h.tx_root),
        "stateRoot": hex0x(&h.state_root),
        "baseFee": u64_to_hex(h.base_fee),
        "gasUsed": u64_to_hex(h.gas_used),
        "gasLimit": u64_to_hex(h.gas_limit),
        "signature": hex0x(&h.signature),
        "transactions": block.transactions.iter().map(tx_to_json).collect::<Vec<_>>(),
    })
}

// ---------------------------------------------------------------------------
// Method implementations
// ---------------------------------------------------------------------------

pub fn eth_chain_id() -> Value {
    json!(u64_to_hex(QTC_CHAIN_ID))
}

pub fn eth_block_number(state: &AppState) -> Value {
    let head = state.chain_head.lock().unwrap();
    json!(u64_to_hex(head.number))
}

pub fn eth_get_balance(state: &AppState, params: &Value) -> Result<Value, String> {
    let addr_str = param_str(params, 0)?;
    let addr = parse_address(&addr_str)?;
    let db = state.state_db.lock().unwrap();
    let account = db.get_account(&addr);
    Ok(json!(u128_to_hex(account.balance)))
}

pub fn eth_get_transaction_count(state: &AppState, params: &Value) -> Result<Value, String> {
    let addr_str = param_str(params, 0)?;
    let addr = parse_address(&addr_str)?;
    let db = state.state_db.lock().unwrap();
    let account = db.get_account(&addr);
    Ok(json!(u64_to_hex(account.nonce)))
}

pub fn eth_get_block_by_number(state: &AppState, params: &Value) -> Result<Value, String> {
    let number = param_u64(params, 0)?;
    match state.storage.get_block(number) {
        Ok(Some(block)) => Ok(block_to_json(&block)),
        Ok(None) => Ok(Value::Null),
        Err(e) => Err(format!("storage error: {e:?}")),
    }
}

/// Submit a raw transaction.
/// `params[0]` is a hex string of a bincode-serialized Transaction.
/// Returns the tx hash on success, and queues the tx for gossip to peers.
pub fn eth_send_raw_transaction(state: &AppState, params: &Value) -> Result<Value, String> {
    let raw_hex = param_str(params, 0)?;
    let raw_hex = raw_hex.strip_prefix("0x").unwrap_or(&raw_hex);
    let bytes = hex::decode(raw_hex).map_err(|e| format!("invalid hex: {e}"))?;
    let tx: Transaction = bincode::deserialize(&bytes)
        .map_err(|e| format!("invalid transaction encoding: {e}"))?;

    let tx_hash = tx.hash;

    {
        let mut mempool = state.mempool.lock().unwrap();
        mempool.add(tx.clone()).map_err(|e| format!("{e:?}"))?;
    }

    // Queue for gossip — the P2P event loop drains `outbox` and publishes.
    state.outbox.lock().unwrap().push(GossipMsg::NewTx(tx));

    Ok(json!(hex0x(&tx_hash)))
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn dispatch(state: &AppState, req: RpcRequest) -> RpcResponse {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        "eth_chainId" => Ok(eth_chain_id()),
        "eth_blockNumber" => Ok(eth_block_number(state)),
        "eth_getBalance" => eth_get_balance(state, &req.params),
        "eth_getTransactionCount" => eth_get_transaction_count(state, &req.params),
        "eth_getBlockByNumber" => eth_get_block_by_number(state, &req.params),
        "eth_sendRawTransaction" => eth_send_raw_transaction(state, &req.params),
        other => return RpcResponse::err(id, ERR_METHOD_NOT_FOUND, format!("method not found: {other}")),
    };

    match result {
        Ok(value) => RpcResponse::ok(id, value),
        Err(msg) => RpcResponse::err(id, ERR_INVALID_PARAMS, msg),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Account, Storage};

    fn test_state() -> AppState {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("QC_DB_PATH", tmp.path());
        AppState {
            state_db: Arc::new(Mutex::new(StateDB::new())),
            mempool: Arc::new(Mutex::new(Mempool::new(Default::default()))),
            storage: Arc::new(Storage::new().unwrap()),
            chain_head: Arc::new(Mutex::new(ChainHead::default())),
            outbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn make_tx(from: u8, nonce: u64) -> Transaction {
        let mut from_addr = [0u8; 32];
        from_addr[0] = from;
        Transaction {
            hash: [from, nonce as u8, 7, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            from: from_addr,
            to: [2u8; 32],
            value: 100,
            nonce,
            base_fee: 1_000,
            priority_fee: 50,
            gas_limit: 21_000,
            signature: vec![0u8; 2420],
            received_at: 0,
        }
    }

    #[test]
    fn test_chain_id() {
        let v = eth_chain_id();
        assert_eq!(v, json!(u64_to_hex(QTC_CHAIN_ID)));
    }

    #[test]
    fn test_block_number_default() {
        let state = test_state();
        let v = eth_block_number(&state);
        assert_eq!(v, json!("0x0"));
    }

    #[test]
    fn test_get_balance_unknown_account_is_zero() {
        let state = test_state();
        let addr = "0x".to_string() + &"00".repeat(32);
        let params = json!([addr]);
        let v = eth_get_balance(&state, &params).unwrap();
        assert_eq!(v, json!("0x0"));
    }

    #[test]
    fn test_get_balance_known_account() {
        let state = test_state();
        let mut addr = [0u8; 32];
        addr[0] = 5;
        state.state_db.lock().unwrap().set_account(addr, Account {
            balance: 1234,
            ..Default::default()
        });
        let addr_hex = hex0x(&addr);
        let params = json!([addr_hex]);
        let v = eth_get_balance(&state, &params).unwrap();
        assert_eq!(v, json!(u128_to_hex(1234)));
    }

    #[test]
    fn test_get_balance_invalid_address() {
        let state = test_state();
        let params = json!(["not_hex"]);
        assert!(eth_get_balance(&state, &params).is_err());
    }

    #[test]
    fn test_get_transaction_count() {
        let state = test_state();
        let mut addr = [0u8; 32];
        addr[0] = 7;
        state.state_db.lock().unwrap().set_account(addr, Account {
            nonce: 42,
            ..Default::default()
        });
        let params = json!([hex0x(&addr)]);
        let v = eth_get_transaction_count(&state, &params).unwrap();
        assert_eq!(v, json!(u64_to_hex(42)));
    }

    #[test]
    fn test_get_block_by_number_not_found() {
        let state = test_state();
        let params = json!(["0x5"]);
        let v = eth_get_block_by_number(&state, &params).unwrap();
        assert_eq!(v, Value::Null);
    }

    #[test]
    fn test_get_block_by_number_found() {
        let state = test_state();
        let block = Block {
            header: crate::chain::BlockHeader {
                parent_hash: [0u8; 32],
                number: 1,
                slot: 1,
                timestamp: 100,
                proposer: [9u8; 32],
                tx_root: [0u8; 32],
                state_root: [0u8; 32],
                base_fee: 1000,
                gas_used: 0,
                gas_limit: 30_000_000,
                signature: vec![0u8; 2420],
            },
            transactions: vec![],
        };
        state.storage.put_block(&block).unwrap();

        let params = json!(["0x1"]);
        let v = eth_get_block_by_number(&state, &params).unwrap();
        assert_eq!(v["number"], json!("0x1"));
        assert_eq!(v["proposer"], json!(hex0x(&[9u8; 32])));
    }

    #[test]
    fn test_send_raw_transaction_accepted_and_queued() {
        let state = test_state();
        let tx = make_tx(1, 0);
        let bytes = bincode::serialize(&tx).unwrap();
        let params = json!([hex0x(&bytes)]);

        let v = eth_send_raw_transaction(&state, &params).unwrap();
        assert_eq!(v, json!(hex0x(&tx.hash)));

        // tx is in mempool
        assert_eq!(state.mempool.lock().unwrap().len(), 1);

        // tx is queued for gossip
        let outbox = state.outbox.lock().unwrap();
        assert_eq!(outbox.len(), 1);
        assert!(matches!(outbox[0], GossipMsg::NewTx(_)));
    }

    #[test]
    fn test_send_raw_transaction_invalid_hex() {
        let state = test_state();
        let params = json!(["not_hex_at_all!"]);
        assert!(eth_send_raw_transaction(&state, &params).is_err());
    }

    #[test]
    fn test_send_raw_transaction_duplicate_rejected() {
        let state = test_state();
        let tx = make_tx(1, 0);
        let bytes = bincode::serialize(&tx).unwrap();
        let params = json!([hex0x(&bytes)]);
        eth_send_raw_transaction(&state, &params).unwrap();
        assert!(eth_send_raw_transaction(&state, &params).is_err());
    }

    #[test]
    fn test_dispatch_method_not_found() {
        let state = test_state();
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            method: "eth_doesNotExist".into(),
            params: json!([]),
            id: json!(1),
        };
        let resp = dispatch(&state, req);
        assert!(resp.result.is_none());
        assert_eq!(resp.error.unwrap().code, ERR_METHOD_NOT_FOUND);
    }

    #[test]
    fn test_dispatch_chain_id() {
        let state = test_state();
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            method: "eth_chainId".into(),
            params: json!([]),
            id: json!(1),
        };
        let resp = dispatch(&state, req);
        assert_eq!(resp.result.unwrap(), json!(u64_to_hex(QTC_CHAIN_ID)));
    }
}
