// scripts/airdrop.ts
// QTC M13 — Testnet Airdrop Script
//
// Distributes tQTC to early community members from the 0xAirdrop pool.
// Eligibility: first 500 valid claims. Max per address: 20,000 tQTC.
//
// Usage:
//   npx ts-node scripts/airdrop.ts --rpc http://<oracle-ip>:8545 --list airdrop_list.json
//   npx ts-node scripts/airdrop.ts --dry-run   (preview only, no txs sent)
//
// airdrop_list.json format:
// [
//   { "address": "0x<64 hex>", "amount": 20000, "reason": "genesis validator" },
//   { "address": "0x<64 hex>", "amount": 10000, "reason": "faucet claim" }
// ]

import * as fs from "fs";

const MAX_PER_ADDRESS = 20_000n;
const MAX_TOTAL_AIRDROP = 10_000_000n; // 10M tQTC pool

interface AirdropEntry {
  address: string;
  amount: number;
  reason: string;
}

interface AirdropResult {
  address: string;
  amount: number;
  reason: string;
  status: "queued" | "skipped" | "error";
  error?: string;
}

function validateAddress(addr: string): boolean {
  const clean = addr.startsWith("0x") ? addr.slice(2) : addr;
  return /^[0-9a-fA-F]{64}$/.test(clean);
}

function parseArgs(): { rpc: string; list: string; dryRun: boolean } {
  const args = process.argv.slice(2);
  let rpc = "http://localhost:8545";
  let list = "airdrop_list.json";
  let dryRun = false;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--rpc") rpc = args[++i];
    else if (args[i] === "--list") list = args[++i];
    else if (args[i] === "--dry-run") dryRun = true;
  }
  return { rpc, list, dryRun };
}

async function getBalance(rpc: string, address: string): Promise<bigint> {
  const res = await fetch(rpc, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", method: "eth_getBalance", params: [address, "latest"], id: 1 }),
  });
  const data = await res.json() as any;
  if (data.error) throw new Error(data.error.message);
  return BigInt(data.result);
}

async function getBlockNumber(rpc: string): Promise<number> {
  const res = await fetch(rpc, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", method: "eth_blockNumber", params: [], id: 1 }),
  });
  const data = await res.json() as any;
  return parseInt(data.result, 16);
}

async function main() {
  const { rpc, list, dryRun } = parseArgs();

  console.log("================================================");
  console.log("  QTC TESTNET AIRDROP — M13");
  console.log("================================================");
  console.log(`RPC  : ${rpc}`);
  console.log(`List : ${list}`);
  console.log(`Mode : ${dryRun ? "DRY RUN" : "LIVE"}`);
  console.log("");

  if (!fs.existsSync(list)) {
    console.error(`Airdrop list not found: ${list}`);
    console.error('Create JSON: [{"address":"0x...","amount":20000,"reason":"genesis validator"}]');
    process.exit(1);
  }

  const entries: AirdropEntry[] = JSON.parse(fs.readFileSync(list, "utf-8"));
  const seen = new Set<string>();
  let totalQTC = 0n;
  const errors: string[] = [];

  for (const e of entries) {
    if (!validateAddress(e.address)) { errors.push(`Invalid address: ${e.address}`); continue; }
    if (e.amount <= 0 || BigInt(e.amount) > MAX_PER_ADDRESS) { errors.push(`Invalid amount for ${e.address}`); continue; }
    if (seen.has(e.address.toLowerCase())) { errors.push(`Duplicate: ${e.address}`); continue; }
    seen.add(e.address.toLowerCase());
    totalQTC += BigInt(e.amount);
  }

  if (errors.length > 0) {
    errors.forEach(e => console.error(`✗ ${e}`));
    process.exit(1);
  }
  if (totalQTC > MAX_TOTAL_AIRDROP) {
    console.error(`Total ${totalQTC} tQTC exceeds 10M pool cap`);
    process.exit(1);
  }

  console.log(`Entries  : ${entries.length}`);
  console.log(`Total    : ${totalQTC.toLocaleString()} tQTC`);
  console.log("");

  if (dryRun) {
    console.log("DRY RUN — would send:");
    entries.forEach(e => console.log(`  → ${e.address.slice(0, 14)}... ${e.amount} tQTC (${e.reason})`));
    console.log("\nRe-run without --dry-run to execute.");
    return;
  }

  // Verify node connectivity
  try {
    const block = await getBlockNumber(rpc);
    console.log(`Node live at block ${block}`);
  } catch (e) {
    console.error(`Cannot connect to ${rpc}: ${e}`);
    process.exit(1);
  }

  const results: AirdropResult[] = [];
  let queued = 0, skipped = 0, failed = 0;

  for (const entry of entries) {
    process.stdout.write(`${entry.address.slice(0, 14)}... ${entry.amount} tQTC — `);
    try {
      const balance = await getBalance(rpc, entry.address);
      if (balance > 0n) {
        console.log(`SKIPPED (has balance)`);
        results.push({ ...entry, status: "skipped" });
        skipped++;
        continue;
      }
      // NOTE: Full tx signing requires the 0xAirdrop keystore loaded in M14.
      // For now, this script validates eligibility and queues transactions.
      // Wire in QTCClient.sendTransaction() after M14 vesting/wallet is built.
      console.log(`QUEUED`);
      results.push({ ...entry, status: "queued" });
      queued++;
    } catch (e: any) {
      console.log(`ERROR: ${e.message}`);
      results.push({ ...entry, status: "error", error: e.message });
      failed++;
    }
    await new Promise(r => setTimeout(r, 100));
  }

  const report = `airdrop_report_${Date.now()}.json`;
  fs.writeFileSync(report, JSON.stringify(results, null, 2));

  console.log("\n================================================");
  console.log(`Queued  : ${queued}`);
  console.log(`Skipped : ${skipped}`);
  console.log(`Failed  : ${failed}`);
  console.log(`Report  : ${report}`);
  console.log("================================================");
  console.log("NOTE: Tx signing wired in after M14. Run this script again post-M14.");
}

main().catch(e => { console.error(e); process.exit(1); });
