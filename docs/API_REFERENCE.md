# QTC JSON-RPC API Reference

**Endpoint:** `http://<node-ip>:8545`
**Method:** POST
**Content-Type:** application/json
**Protocol:** JSON-RPC 2.0

All method names follow Ethereum conventions for wallet/tooling compatibility.

---

## Quick Start

```bash
# Check node is running
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

## Rate Limiting

Default: **100 requests per second** per node instance.
Override: set `QC_RPC_RATE_LIMIT` env var before starting the node.
Exceeded: returns HTTP 429 Too Many Requests.

---

## Methods

### eth_chainId

Returns the chain ID of the current network.

**Params:** none

**Returns:** `string` — chain ID as hex

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_chainId",
  "params": [],
  "id": 1
}

// Response (testnet)
{
  "jsonrpc": "2.0",
  "result": "0x74",
  "id": 1
}

// Response (mainnet)
{
  "jsonrpc": "2.0",
  "result": "0x51",
  "id": 1
}
```

**Chain IDs:**
| Network | Chain ID | Decimal |
|---|---|---|
| Testnet | `0x74` | 116 |
| Mainnet | `0x51` | 81 |

Set via `QC_NETWORK=testnet` (default) or `QC_NETWORK=mainnet`.

---

### eth_blockNumber

Returns the number of the most recent block.

**Params:** none

**Returns:** `string` — block number as hex

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_blockNumber",
  "params": [],
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": "0x1a4",
  "id": 1
}
```

**Notes:**
- Returns `"0x0"` if no blocks have been produced yet
- Increments by 1 every ~2 seconds (BLOCK_TIME_SECS=2)

---

### eth_getBalance

Returns the QTC balance of an account in nano-QTC.

**Params:**
1. `address` — `string` — 0x-prefixed 64 hex char address (32 bytes)
2. `tag` — `string` — block tag (currently only `"latest"` supported)

**Returns:** `string` — balance in nano-QTC as hex (1 QTC = 1,000,000,000 nano-QTC)

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_getBalance",
  "params": [
    "0x<your-64-char-address>",
    "latest"
  ],
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": "0x2386f26fc10000",
  "id": 1
}
```

**Convert result to QTC:**
```javascript
const nanoQTC = parseInt(result, 16);
const QTC = nanoQTC / 1_000_000_000;
```

**Notes:**
- Returns `"0x0"` for addresses with no balance
- Address must be exactly 64 hex characters after `0x` (32 bytes)
- Address = SHA3-256(Dilithium2 public key) per FIPS 202

---

### eth_getTransactionCount

Returns the nonce (transaction count) of an address.

**Params:**
1. `address` — `string` — 0x-prefixed 64 hex char address
2. `tag` — `string` — `"latest"`

**Returns:** `string` — nonce as hex

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_getTransactionCount",
  "params": [
    "0x<your-64-char-address>",
    "latest"
  ],
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": "0x5",
  "id": 1
}
```

**Notes:**
- Nonce starts at 0 for new addresses
- Every successful transaction increments the nonce by 1
- Use this before signing a transaction to get the correct nonce

---

### eth_getBlockByNumber

Returns full block information by block number.

**Params:**
1. `number` — `string` — block number as hex, or `"latest"`
2. `full` — `boolean` — include full transaction objects (currently ignored, always returns full)

**Returns:** block object or `null` if not found

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_getBlockByNumber",
  "params": ["0x1", true],
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "number": "0x1",
    "parentHash": "0x<64 hex chars>",
    "slot": "0x1",
    "timestamp": "0x<unix timestamp hex>",
    "proposer": "0x<validator address>",
    "txRoot": "0x<tx merkle root>",
    "stateRoot": "0x<state root hash>",
    "baseFee": "0x3e8",
    "gasUsed": "0x5208",
    "gasLimit": "0x1c9c380",
    "signature": "0x<2420 byte Dilithium2 sig>",
    "transactions": []
  },
  "id": 1
}
```

**Block fields:**

| Field | Type | Description |
|---|---|---|
| `number` | hex | Block height |
| `parentHash` | hex | SHA256 of parent block |
| `slot` | hex | Slot number (same as number currently) |
| `timestamp` | hex | Unix timestamp (seconds) |
| `proposer` | hex | Validator address that produced this block |
| `txRoot` | hex | SHA256 of all transactions |
| `stateRoot` | hex | SHA256 of all account states |
| `baseFee` | hex | EIP-1559 base fee in nano-QTC per gas unit |
| `gasUsed` | hex | Total gas consumed by transactions |
| `gasLimit` | hex | Maximum gas per block (30,000,000) |
| `signature` | hex | Dilithium2 signature (2420 bytes) |
| `transactions` | array | List of transactions in this block |

---

### eth_sendRawTransaction

Submits a signed transaction to the mempool.

**Params:**
1. `data` — `string` — 0x-prefixed hex-encoded bincode-serialized Transaction

**Returns:** `string` — transaction hash as hex, or error

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "eth_sendRawTransaction",
  "params": ["0x<hex encoded signed transaction>"],
  "id": 1
}

// Response (success)
{
  "jsonrpc": "2.0",
  "result": "0x<transaction hash>",
  "id": 1
}

// Response (error — invalid hash)
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "tx hash mismatch: declared 0x..., computed 0x..."
  },
  "id": 1
}
```

**Transaction structure (for qtc-client):**

```typescript
interface Transaction {
  hash: Uint8Array;        // SHA256(from||to||value||nonce||base_fee||priority_fee||gas_limit)
  from: Uint8Array;        // 32 bytes — sender address
  to: Uint8Array;          // 32 bytes — recipient address
  value: bigint;           // nano-QTC to transfer
  nonce: bigint;           // sender's current nonce
  base_fee: bigint;        // must match current block base fee
  priority_fee: bigint;    // tip to validator (can be 0)
  gas_limit: bigint;       // 21000 for simple transfers
  signature: Uint8Array;   // 2420 bytes — Dilithium2 signature
}
```

**Notes:**
- Transaction hash is verified server-side (AUDIT-018 fix)
- Signature must be Dilithium2 over bincode-serialized transaction fields
- Use qtc-client TypeScript library for transaction construction
- Mempool cap: 10,000 transactions (highest fee first)
- Duplicate transactions rejected

---

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| -32700 | Parse Error | Invalid JSON |
| -32601 | Method Not Found | Unknown method name |
| -32602 | Invalid Params | Wrong params (address format, tx hash, etc.) |
| -32603 | Internal Error | Storage error or node internal failure |

---

## Connecting qtc-client

```typescript
import { QTCClient } from 'qtc-client';

const client = new QTCClient('http://<node-ip>:8545');

// Get block number
const block = await client.getBlockNumber();

// Get balance
const balance = await client.getBalance('0x<address>');

// Send transaction (requires keypair)
const txHash = await client.sendTransaction({
  to: '0x<recipient>',
  value: 1_000_000_000n, // 1 QTC in nano-QTC
  keypair: myKeypair,
});
```

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `QC_RPC_ADDR` | `0.0.0.0:8545` | Bind address for RPC server |
| `QC_NETWORK` | `testnet` | Network selection (testnet/mainnet) |
| `QC_RPC_RATE_LIMIT` | `100` | Max requests per second |

---

## Firewall

Open port `8545/tcp` for RPC access.
Open port `30333/tcp` for P2P gossip (not yet configurable).
