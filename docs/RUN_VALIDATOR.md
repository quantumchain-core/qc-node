# Run a QTC Testnet Validator

**Network:** QTC Testnet
**Token:** tQTC (test tokens — zero monetary value)
**Purpose:** Validate blocks, earn community emissions, help find bugs before mainnet

---

## Requirements

| Item | Minimum | Recommended |
|---|---|---|
| CPU | 1 vCPU | 2 vCPU |
| RAM | 512 MB | 1 GB |
| Disk | 10 GB | 20 GB |
| OS | Ubuntu 22.04 | Ubuntu 22.04 |
| Network | 1 Mbps | 10 Mbps |
| Cost | $0 (Oracle Cloud Always Free) | $0 |

**Oracle Cloud Always Free tier covers all requirements perfectly.**
Sign up at cloud.oracle.com — no credit card required for Always Free VMs.

---

## Step 1 — Get a Free Server

1. Go to **cloud.oracle.com** → Sign up (free, no credit card needed for Always Free)
2. Create a VM instance:
   - Shape: **VM.Standard.A1.Flex** (Always Free)
   - OCPU: 1, RAM: 1 GB
   - OS: **Ubuntu 22.04**
   - Enable public IP
3. Note your public IP address — you'll need it

---

## Step 2 — Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version   # should show stable 1.7x+
```

---

## Step 3 — Clone and Build qc-node

```bash
git clone https://github.com/quantumchain-core/qc-node
cd qc-node
cargo build --release
```

Build takes 3-5 minutes on first run. The binary is at `target/release/node`.

---

## Step 4 — Generate Your Validator Identity

Run the node once to generate your Dilithium2 keypair and see your validator address:

```bash
QC_KEYSTORE_PATH=./qc-keystore.json \
QC_DB_PATH=./qc-data \
QC_NETWORK=testnet \
./target/release/node
```

You will see output like:
```
================================================
  QTC NODE -- network: TESTNET
  (testnet tokens have NO monetary value)
================================================
generated new keypair, saved to ./qc-keystore.json
validator address: 0x<your-64-char-address>
```

**Copy your validator address** — you need it to register and receive the airdrop.

Stop the node with `Ctrl+C` after copying the address.

---

## Step 5 — Register as a Testnet Validator

Submit your validator address in one of two ways:

**Option A — GitHub Issue (recommended):**
Open an issue at github.com/quantumchain-core/qc-node with title:
`[Validator Registration] 0x<your-address>`

Include:
- Your validator address (0x + 64 hex chars)
- Your server's public IP and port (default: 0.0.0.0:30333)
- Your country/region (helps us track global distribution)

**Option B — Email:**
Send your address to touqeerahmadofficial896@gmail.com
Subject: `QTC Testnet Validator Registration`

You will be added to the next genesis update within 48 hours.

---

## Step 6 — Download the Testnet Genesis File

Once registered, download the latest genesis config:

```bash
curl -o testnet-genesis.json \
  https://raw.githubusercontent.com/quantumchain-core/qtc-mainnet/main/genesis/testnet.json
```

---

## Step 7 — Run Your Validator

```bash
QC_KEYSTORE_PATH=./qc-keystore.json \
QC_DB_PATH=./qc-data \
QC_NETWORK=testnet \
QC_GENESIS_PATH=./testnet-genesis.json \
QC_RPC_ADDR=0.0.0.0:8545 \
./target/release/node
```

You should see:
```
================================================
  QTC NODE -- network: TESTNET
  (testnet tokens have NO monetary value)
================================================
loaded keypair from ./qc-keystore.json
validator address: 0x<your-address>
coinbase: 0x<your-address> (derived from validator pubkey)
loading validator registry from ./testnet-genesis.json
validator registry: N validator(s)
QTC node RPC listening on 0.0.0.0:8545
```

---

## Step 8 — Keep It Running (Optional but Recommended)

Create a systemd service so your validator restarts automatically:

```bash
sudo tee /etc/systemd/system/qtc-validator.service << 'UNIT'
[Unit]
Description=QTC Testnet Validator
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/qc-node
Environment="QC_KEYSTORE_PATH=/home/ubuntu/qc-node/qc-keystore.json"
Environment="QC_DB_PATH=/home/ubuntu/qc-node/qc-data"
Environment="QC_NETWORK=testnet"
Environment="QC_GENESIS_PATH=/home/ubuntu/qc-node/testnet-genesis.json"
Environment="QC_RPC_ADDR=0.0.0.0:8545"
ExecStart=/home/ubuntu/qc-node/target/release/node
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

sudo systemctl daemon-reload
sudo systemctl enable qtc-validator
sudo systemctl start qtc-validator
sudo journalctl -u qtc-validator -f   # view logs
```

---

## Verify Your Node is Working

```bash
# Check your block number
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check your balance (replace with your address)
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x<your-address>","latest"],"id":1}'
```

---

## Earn tQTC

As a registered testnet validator you earn:
- **Community emissions** — proportional to blocks produced + uptime
- **Genesis airdrop** — 20,000 tQTC for early validators (first 500 claims)
- **Bug bounty** — up to 1,000,000 tQTC for finding critical bugs

Testnet tokens are converted to mainnet QTC at a 1:1 ratio for genesis validators
(subject to DAO vote before mainnet launch).

---

## Firewall Rules (Oracle Cloud)

In the Oracle Cloud console, add these ingress rules to your security list:

| Port | Protocol | Purpose |
|---|---|---|
| 8545 | TCP | JSON-RPC API |
| 30333 | TCP | P2P gossip |

---

## Troubleshooting

**"loaded keypair from..."** — ✅ Good, your identity is stable

**"generated new keypair..."** — ⚠️ New identity created. If you were registered, re-register.

**Node not producing blocks** — You may not be in the genesis validator set yet. Register via GitHub Issue.

**"unknown parent" errors** — Your chain is out of sync. Stop node, delete `qc-data/`, restart.

---

## Bug Bounty

Found a bug? You earn tQTC:

| Severity | Reward |
|---|---|
| Critical | 500,000 tQTC |
| High | 100,000 tQTC |
| Medium | 25,000 tQTC |
| Low | 5,000 tQTC |

Report bugs as GitHub Issues with title `[BUG] <brief description>`.
Include: steps to reproduce, expected vs actual behavior, your node version.

---

## Stay Updated

- GitHub: github.com/quantumchain-core
- X/Twitter: @quantumchain (follow for airdrop announcements)
- Email: touqeerahmadofficial896@gmail.com
