use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// All amounts in microQTC. 1 QTC = 1_000_000 microQTC
pub const BLOCK_REWARD: u64 = 10_000_000; // 10 QTC per block
pub const UPTIME_REWARD_PER_HOUR: u64 = 1_000_000; // 1 QTC/hour online

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payout {
    pub address: String, // Dilithium3 pubkey hex
    pub amount: u64,     // microQTC
    pub reason: String,  // "block_proposal" or "uptime"
}

pub struct RewardState {
    pub pending_payouts: HashMap<String, u64>,
}

impl RewardState {
    pub fn new() -> Self {
        Self {
            pending_payouts: HashMap::new(),
        }
    }
    
    // Called every block. Proposer gets paid immediately.
    pub fn reward_block_proposer(&mut self, proposer: String) {
        *self.pending_payouts.entry(proposer).or_insert(0) += BLOCK_REWARD;
    }
    
    // Called every hour by node. Pays for staying online.
    pub fn reward_uptime(&mut self, node: String) {
        *self.pending_payouts.entry(node).or_insert(0) += UPTIME_REWARD_PER_HOUR;
    }
    
    // M5: Called every 3 hours to create payout transaction
    pub fn batch_payout(&mut self) -> Vec<Payout> {
        let payouts: Vec<Payout> = self.pending_payouts
            .drain()
            .map(|(address, amount)| Payout {
                address,
                amount,
                reason: "batch_uptime+blocks".into(),
            })
            .collect();
        payouts
    }
}