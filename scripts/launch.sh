#!/bin/bash
# scripts/launch.sh
# QTC Node Launch Script — M15
#
# Usage:
#   Testnet: ./launch.sh testnet
#   Mainnet: ./launch.sh mainnet
#
# Environment variables (optional overrides):
#   QC_KEYSTORE_PATH  — default: ./qc-keystore.json
#   QC_DB_PATH        — default: ./qc-data
#   QC_RPC_ADDR       — default: 0.0.0.0:8545
#   QC_COINBASE       — default: derived from validator pubkey

set -e

NETWORK=${1:-testnet}

if [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "mainnet" ]; then
    echo "Usage: ./launch.sh [testnet|mainnet]"
    exit 1
fi

echo "================================================"
echo "  QTC NODE LAUNCHER"
echo "  Network: $NETWORK"
echo "================================================"

# --- Paths ---
BINARY="./target/release/node"
KEYSTORE="${QC_KEYSTORE_PATH:-./qc-keystore.json}"
DB_PATH="${QC_DB_PATH:-./qc-data}"
RPC_ADDR="${QC_RPC_ADDR:-0.0.0.0:8545}"

if [ "$NETWORK" = "testnet" ]; then
    GENESIS="./genesis/testnet.json"
else
    GENESIS="./genesis/mainnet.json"
fi

# --- Pre-flight checks ---
echo ""
echo "Running pre-flight checks..."

# Binary exists
if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Building..."
    cargo build --release
    echo "Build complete."
fi

# Genesis file exists
if [ ! -f "$GENESIS" ]; then
    echo "ERROR: Genesis file not found at $GENESIS"
    echo "For testnet: genesis/testnet.json"
    echo "For mainnet: genesis/mainnet.json"
    exit 1
fi

# Mainnet extra checks
if [ "$NETWORK" = "mainnet" ]; then
    echo ""
    echo "MAINNET LAUNCH CHECKLIST"
    echo "========================"
    echo "Before launching mainnet, confirm ALL of these:"
    echo ""
    echo "  [ ] Professional security audit completed"
    echo "  [ ] Legal utility token opinion obtained"
    echo "  [ ] Minimum 3 independent validators ready"
    echo "  [ ] Testnet stable for 30+ days"
    echo "  [ ] TOKENOMICS.md hash verified"
    echo "  [ ] SECURITY_AUDIT.md hash verified"
    echo ""
    read -p "Have all requirements been met? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Mainnet launch aborted. Complete requirements first."
        exit 1
    fi
fi

echo ""
echo "Pre-flight checks passed."
echo ""
echo "Configuration:"
echo "  Network  : $NETWORK"
echo "  Binary   : $BINARY"
echo "  Keystore : $KEYSTORE"
echo "  Database : $DB_PATH"
echo "  RPC      : $RPC_ADDR"
echo "  Genesis  : $GENESIS"
echo ""

# --- Launch ---
echo "Starting QTC node..."
echo ""

QC_NETWORK="$NETWORK" \
QC_KEYSTORE_PATH="$KEYSTORE" \
QC_DB_PATH="$DB_PATH" \
QC_RPC_ADDR="$RPC_ADDR" \
QC_GENESIS_PATH="$GENESIS" \
"$BINARY"
launch.sh
