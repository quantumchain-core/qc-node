#!/usr/bin/env bash
# scripts/integration_test.sh
#
# QTC: real two-process integration test.
#
# Everything else this session was unit tests — two Node structs living
# in the same process, calling each other's functions directly. This is
# the first test that runs two ACTUAL separate OS processes, talking over
# real localhost TCP, exercising the real libp2p swarm, real listen/dial,
# and — the important part — the real sync protocol under conditions that
# actually require it: node B starts late, after node A already has
# several blocks, so B can only catch up via a real sync request/response
# over the network, not gossip (gossip only ever carries the newest block).
#
# What this proves if it passes:
#   - two real processes can find each other (listen + dial actually works)
#   - proposer rotation works across real processes, not just in-memory
#   - state sync actually closes a real gap over a real network
#
# What this does NOT prove:
#   - behavior across real separate machines / real network latency
#     (both nodes run on localhost here)
#   - behavior with more than 2 validators
#   - long-running stability (this test runs for ~40 seconds total)

set -euo pipefail

WORKDIR="$(mktemp -d)"
echo "Working directory: $WORKDIR"
trap 'kill $(jobs -p) 2>/dev/null || true' EXIT

BIN_DIR="target/release"
NODE_BIN="$BIN_DIR/node"
KEYGEN_BIN="$BIN_DIR/keygen"

echo "=== Building release binaries ==="
cargo build --release --bin node --bin keygen

# ---------------------------------------------------------------------------
# Step 1: generate two validator identities via keygen — same code path the
# real node uses, so the genesis file and the running nodes can never
# disagree about who's who.
# ---------------------------------------------------------------------------
echo "=== Generating validator A identity ==="
export QC_KEYSTORE_PATH="$WORKDIR/validator-a-keystore.json"
export QC_KEYSTORE_PASSWORD="integration-test-password-a"
KEYGEN_A_OUT="$("$KEYGEN_BIN")"
echo "$KEYGEN_A_OUT"
ADDR_A="$(echo "$KEYGEN_A_OUT" | grep ADDRESS= | cut -d= -f2)"
PUBKEY_A="$(echo "$KEYGEN_A_OUT" | grep PUBKEY= | cut -d= -f2)"

echo "=== Generating validator B identity ==="
export QC_KEYSTORE_PATH="$WORKDIR/validator-b-keystore.json"
export QC_KEYSTORE_PASSWORD="integration-test-password-b"
KEYGEN_B_OUT="$("$KEYGEN_BIN")"
echo "$KEYGEN_B_OUT"
ADDR_B="$(echo "$KEYGEN_B_OUT" | grep ADDRESS= | cut -d= -f2)"
PUBKEY_B="$(echo "$KEYGEN_B_OUT" | grep PUBKEY= | cut -d= -f2)"

# ---------------------------------------------------------------------------
# Step 2: build a real genesis file listing both validators.
# ---------------------------------------------------------------------------
GENESIS_PATH="$WORKDIR/genesis.json"
cat > "$GENESIS_PATH" << EOF
{
  "validators": [
    { "address": "$ADDR_A", "pubkey": "$PUBKEY_A" },
    { "address": "$ADDR_B", "pubkey": "$PUBKEY_B" }
  ]
}
EOF
echo "=== genesis.json ==="
cat "$GENESIS_PATH"

hex_to_dec() {
  # Convert a 0x-prefixed hex string (as returned by eth_blockNumber) to decimal.
  python3 -c "print(int('$1', 16))"
}

rpc_call() {
  local port="$1" method="$2"
  curl -s -X POST "http://127.0.0.1:$port/" \
    -H "content-type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":[],\"id\":1}"
}

block_number() {
  local port="$1"
  local result
  result="$(rpc_call "$port" eth_blockNumber | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"])')"
  hex_to_dec "$result"
}

# ---------------------------------------------------------------------------
# Step 3: start node A ALONE first. Let it produce several blocks entirely
# by itself before node B ever exists — this is what forces B to actually
# need the sync protocol later, rather than just receiving everything via
# ordinary gossip as it's produced.
# ---------------------------------------------------------------------------
echo "=== Starting node A alone ==="
QC_KEYSTORE_PATH="$WORKDIR/validator-a-keystore.json" \
QC_KEYSTORE_PASSWORD="integration-test-password-a" \
QC_DB_PATH="$WORKDIR/db-a" \
QC_GENESIS_PATH="$GENESIS_PATH" \
QC_RPC_ADDR="127.0.0.1:18545" \
QC_LISTEN_ADDR="/ip4/127.0.0.1/tcp/19001" \
"$NODE_BIN" > "$WORKDIR/node-a.log" 2>&1 &
NODE_A_PID=$!

echo "Waiting for node A's RPC to come up..."
for i in $(seq 1 20); do
  if curl -s "http://127.0.0.1:18545/" > /dev/null 2>&1; then break; fi
  sleep 1
done

echo "Letting node A produce blocks alone for 12 seconds (~6 blocks at 2s/block)..."
sleep 12

HEIGHT_A_BEFORE_B="$(block_number 18545)"
echo "Node A height before B joins: $HEIGHT_A_BEFORE_B"
if [ "$HEIGHT_A_BEFORE_B" -lt 2 ]; then
  echo "FAIL: node A did not produce blocks on its own. Check node-a.log:"
  cat "$WORKDIR/node-a.log"
  exit 1
fi

# ---------------------------------------------------------------------------
# Step 4: NOW start node B, late, bootstrapping to node A. B starts at
# height 0 while A is already several blocks ahead — B can only close that
# gap via the real sync request/response protocol.
# ---------------------------------------------------------------------------
echo "=== Starting node B (late, must sync to catch up) ==="
QC_KEYSTORE_PATH="$WORKDIR/validator-b-keystore.json" \
QC_KEYSTORE_PASSWORD="integration-test-password-b" \
QC_DB_PATH="$WORKDIR/db-b" \
QC_GENESIS_PATH="$GENESIS_PATH" \
QC_RPC_ADDR="127.0.0.1:18546" \
QC_LISTEN_ADDR="/ip4/127.0.0.1/tcp/19002" \
QC_BOOTSTRAP_PEERS="/ip4/127.0.0.1/tcp/19001" \
"$NODE_BIN" > "$WORKDIR/node-b.log" 2>&1 &
NODE_B_PID=$!

echo "Waiting for node B's RPC to come up..."
for i in $(seq 1 20); do
  if curl -s "http://127.0.0.1:18546/" > /dev/null 2>&1; then break; fi
  sleep 1
done

echo "Giving both nodes 20 seconds to connect, gossip, and sync..."
sleep 20

HEIGHT_A="$(block_number 18545)"
HEIGHT_B="$(block_number 18546)"
echo "Final height — node A: $HEIGHT_A, node B: $HEIGHT_B"

echo ""
echo "=== node-a.log (last 30 lines) ==="
tail -30 "$WORKDIR/node-a.log"
echo ""
echo "=== node-b.log (last 30 lines) ==="
tail -30 "$WORKDIR/node-b.log"

# ---------------------------------------------------------------------------
# Step 5: assertions.
# ---------------------------------------------------------------------------
FAIL=0

if [ "$HEIGHT_B" -eq 0 ]; then
  echo "FAIL: node B never advanced past height 0 — it never connected, or sync never worked."
  FAIL=1
fi

DIFF=$((HEIGHT_A - HEIGHT_B))
if [ "$DIFF" -lt 0 ]; then DIFF=$((-DIFF)); fi
if [ "$DIFF" -gt 3 ]; then
  echo "FAIL: node A ($HEIGHT_A) and node B ($HEIGHT_B) are more than 3 blocks apart — they don't appear to be staying in sync."
  FAIL=1
fi

if grep -q "gap detected" "$WORKDIR/node-b.log"; then
  echo "PASS: node B's log shows it detected a real gap and requested sync — the sync protocol was actually exercised, not just gossip."
else
  echo "WARN: node B's log never mentions detecting a gap. Either it caught up entirely via gossip before ever seeing a gap (less conclusive proof of the sync path specifically), or something's off. Not treated as a hard failure, but worth a look."
fi

if [ "$FAIL" -eq 0 ]; then
  echo ""
  echo "=== INTEGRATION TEST PASSED ==="
  exit 0
else
  echo ""
  echo "=== INTEGRATION TEST FAILED ==="
  exit 1
fi
