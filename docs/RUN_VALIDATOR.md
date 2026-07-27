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

You will see:
```
================================================
  QTC NODE -- network: TESTNET
  (testnet tokens have NO monetary value)
================================================
generated new keypair, saved to ./qc-keystore.json
validator address: 0x<your-64-char-address>
```

**Copy your validator address immediately.**
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
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x<your-address>","latest"],"id":1}'
```

---

## Firewall Rules (Oracle Cloud)

Add these ingress rules in Oracle Cloud console:

| Port | Protocol | Purpose |
|---|---|---|
| 8545 | TCP | JSON-RPC API |
| 30333 | TCP | P2P gossip |

---

## Troubleshooting

**"loaded keypair from..."** — Good, identity is stable

**"generated new keypair..."** — New identity. If registered, re-register.

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
