# Run a QTC Testnet Validator

**Network:** QTC Testnet
**Token:** tQTC (test tokens — zero monetary value)
**Purpose:** Validate blocks, earn tQTC rewards, help build the network

---

## How Rewards Work

Performance-based only. No time locks. No minimum stake.

```
Monthly Reward =
  (your_blocks / total_blocks) × 60% of monthly pool
+ (your_uptime%) × 40% of monthly pool
```

**Year 1 monthly pool:** ~4,166,666 tQTC (from 50M annual)

**Example with 10 validators, equal performance:**
Each earns ~416,666 tQTC/month at 100% uptime.
At 50% uptime you earn proportionally less.
Go offline for a week → earn nothing that week.

**No bonuses for just registering. Earn only while running.**

**tQTC → QTC conversion:**
When mainnet launches, DAO votes on conversion ratio.
Early validators with best uptime get best ratio.
Announced publicly before mainnet. No surprises.

---

## Requirements

| Item | Minimum | Recommended |
|---|---|---|
| CPU | 1 vCPU | 2 vCPU |
| RAM | 512 MB | 1 GB |
| Disk | 10 GB | 20 GB |
| OS | Ubuntu 22.04 | Ubuntu 22.04 |
| Network | 10 Mbps | 100 Mbps |
| Cost | $0 (Oracle Cloud Always Free) | $0 |

**Oracle Cloud Always Free tier covers all requirements perfectly.**
Sign up at cloud.oracle.com — no credit card charge for Always Free VMs.

---

## Step 1 — Get a Free Server

1. Go to **cloud.oracle.com** → Sign up (free, $1.08 verification hold — refunded)
2. Create a VM instance:
   - Shape: **VM.Standard.A1.Flex** (Always Free)
   - OCPU: 1, RAM: 1 GB
   - OS: **Ubuntu 22.04**
   - Enable public IP
3. Note your public IP address

---

## Step 2 — Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

---

## Step 3 — Clone and Build qc-node

```bash
git clone https://github.com/quantumchain-core/qc-node
cd qc-node
cargo build --release
```

Build takes 5-6 minutes on first run.

---

## Step 4 — Generate Your Validator Identity

```bash
export QC_KEYSTORE_PASSWORD=your_strong_password_here
export QC_NETWORK=testnet
export QC_KEYSTORE_PATH=./qc-keystore.json
export QC_DB_PATH=./qc-data
export QC_GENESIS_PATH=./genesis/testnet.json
export QC_RPC_ADDR=0.0.0.0:8545

./target/release/node
```

You will see something like:
```
================================================
  QTC NODE -- TESTNET 
================================================
✅ Created encrypted keystore at ./qc-keystore.json
validator address: 0x<your-64-char-address>
Validator registry: 1 validator(s)
```

(On every later restart, once `./qc-keystore.json` already exists, that
third line reads `✅ Loaded encrypted keystore from ./qc-keystore.json`
instead — same address, no new identity generated.)

**Copy your validator address immediately** (the `validator address: 0x...` line).
**Backup qc-keystore.json — losing it means losing your validator identity and all earned rewards.**

Press Ctrl+C after copying the address.

---

## Step 5 — Register as a Validator

**Option A — GitHub Issue:**
Open an issue at github.com/quantumchain-core/qc-node/issues/new

Title: `[Validator Registration] 0x<your-address>`

Include:
- Your validator address (0x + 64 hex chars)
- Your pk_hex from qc-keystore.json
- Your server public IP
- Your country/region

**Option B — Landing Page (coming soon):**
Register directly at qtc.network with GitHub login.

You will be added to genesis/testnet.json within 48 hours.

---

## Step 6 — Download Updated Genesis File

After registration, download the updated genesis:

```bash
curl -o genesis/testnet.json \
  https://raw.githubusercontent.com/quantumchain-core/qc-node/main/genesis/testnet.json
```

---

## Configuration Options

The variables in Step 4 are the minimum to get a node running — everything
below is optional, but **`QC_BOOTSTRAP_PEERS` matters more than it looks**:
without it, your node has no way to discover any other peer at all (there's
no other discovery mechanism yet, e.g. no mDNS) and will just run alone,
never actually joining the network even though it starts up cleanly with
no errors.

| Variable | Default if unset | What it does |
|---|---|---|
| `QC_BOOTSTRAP_PEERS` | *(empty — no peers)* | **Get this from your registration confirmation or ask in the validator channel.** Comma-separated libp2p multiaddrs of already-running peers, e.g. `/ip4/1.2.3.4/tcp/30333/p2p/12D3KooW...`. Without at least one, your node is network-isolated. |
| `QC_LISTEN_ADDR` | `/ip4/0.0.0.0/tcp/30333` | The libp2p multiaddr your node listens on for incoming P2P connections. Only change this if you need a non-default port (e.g. running two nodes on one host). |
| `QC_COINBASE` | *(unset — fee recipient = your validator address)* | 32-byte hex address (with or without `0x`) to receive block rewards/fees at, if different from your validator identity address. Most operators should leave this unset. |
| `QC_RPC_RATE_LIMIT` | `100` (requests/sec) | Global cap across all RPC callers to your node — not per-caller. Raise if your own tooling is hitting it; be cautious raising it much on a publicly-reachable RPC port. |
| `QC_RPC_ADDR` | `127.0.0.1:8545` (localhost-only) | **Must be set explicitly to `0.0.0.0:8545` (as in Step 4 above) if you want to query your node's RPC from outside the server itself.** The localhost-only default is deliberate — a missing/dropped env var on restart should fail safe (no RPC exposure), not silently open the port to the public internet. |

---

## Step 7 — Run Your Validator

Create a launch script:

```bash
cat > run-validator.sh << 'SCRIPT'
#!/bin/bash
export QC_KEYSTORE_PASSWORD=your_strong_password_here
export QC_NETWORK=testnet
export QC_KEYSTORE_PATH=./qc-keystore.json
export QC_DB_PATH=./qc-data
export QC_GENESIS_PATH=./genesis/testnet.json
export QC_RPC_ADDR=0.0.0.0:8545
export QC_BOOTSTRAP_PEERS="/ip4/1.2.3.4/tcp/30333/p2p/12D3KooW..."   # from Step 5/6 — required to actually join the network
./target/release/node
SCRIPT
chmod +x run-validator.sh
```

Run in background:
```bash
nohup env QC_KEYSTORE_PASSWORD=your_strong_password_here \
  QC_NETWORK=testnet \
  QC_KEYSTORE_PATH=./qc-keystore.json \
  QC_DB_PATH=./qc-data \
  QC_GENESIS_PATH=./genesis/testnet.json \
  QC_RPC_ADDR=0.0.0.0:8545 \
  QC_BOOTSTRAP_PEERS="/ip4/1.2.3.4/tcp/30333/p2p/12D3KooW..." \
  ./target/release/node > validator.log 2>&1 &
echo "Validator PID: $!"
```

---

## Step 8 — Keep It Running (Systemd)

```bash
sudo tee /etc/systemd/system/qtc-validator.service << 'UNIT'
[Unit]
Description=QTC Testnet Validator
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/qc-node
EnvironmentFile=/home/ubuntu/qtc.env
ExecStart=/home/ubuntu/qc-node/target/release/node
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

cat > /home/ubuntu/qtc.env << 'ENV'
QC_KEYSTORE_PASSWORD=your_strong_password_here
QC_NETWORK=testnet
QC_KEYSTORE_PATH=/home/ubuntu/qc-node/qc-keystore.json
QC_DB_PATH=/home/ubuntu/qc-node/qc-data
QC_GENESIS_PATH=/home/ubuntu/qc-node/genesis/testnet.json
QC_RPC_ADDR=0.0.0.0:8545
QC_BOOTSTRAP_PEERS=/ip4/1.2.3.4/tcp/30333/p2p/12D3KooW...
ENV

sudo systemctl daemon-reload
sudo systemctl enable qtc-validator
sudo systemctl start qtc-validator
sudo journalctl -u qtc-validator -f
```

---

## Verify Your Node

```bash
# Check block number
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check your balance
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x<your-address>"],"id":1}'
```

(`eth_getBalance` currently only reads the address param — a second
`"latest"`/block-tag argument, if you pass one, is accepted but ignored;
there's no historical-balance-by-block support yet.)

---

## Firewall Rules (Oracle Cloud)

Add these ingress rules in Oracle Cloud console:

| Port | Protocol | Purpose |
|---|---|---|
| 8545 | TCP | JSON-RPC API |
| 30333 | TCP | P2P gossip (matches `QC_LISTEN_ADDR`'s default port — update this rule too if you changed that) |

---

## Troubleshooting

**"✅ Loaded encrypted keystore from..."** — Good, identity is stable.

**"✅ Created encrypted keystore at..."** — New identity, first run at this
`QC_KEYSTORE_PATH`. If you'd already registered a different address, make
sure `QC_KEYSTORE_PATH` points at the *same* keystore file as before —
this message means it's about to generate a new (unregistered) identity.

**Node starts cleanly, no errors, but block number never advances and you
never see any peer activity** — almost always `QC_BOOTSTRAP_PEERS` is
unset or unreachable. See Configuration Options above; this is the most
common way a validator silently never actually joins.

**"unknown parent" errors** — Chain out of sync. Delete qc-data/ and restart.

**Sled lock error** — Another node instance running. Run: `pkill -f "target/release/node"`

---

## Bug Bounty

| Severity | Reward |
|---|---|
| Critical | 500,000 tQTC |
| High | 100,000 tQTC |
| Medium | 25,000 tQTC |
| Low | 5,000 tQTC |

Report: github.com/quantumchain-core/qc-node/issues

---

## Stay Updated

- GitHub: github.com/quantumchain-core
- Email: touqeerahmadofficial896@gmail.com
- X: @quantumchain
