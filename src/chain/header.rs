use crate::crypto::DILITHIUM2_SIG_BYTES; // or wherever your crypto const is

pub struct Header {
    pub parent_hash: [u8; 32],
    pub height: u64,
    pub timestamp: u64,
    pub base_fee: u128,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub tx_root: [u8; 32],
    pub state_root: [u8; 32],
    pub validator: [u8; 32],
    pub sig: [u8; DILITHIUM2_SIG_BYTES], // ADD THIS
}

impl Header {
    pub fn to_bytes_without_sig(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend(&self.parent_hash);
        v.extend(&self.height.to_le_bytes());
        v.extend(&self.timestamp.to_le_bytes());
        v.extend(&self.base_fee.to_le_bytes());
        v.extend(&self.gas_limit.to_le_bytes());
        v.extend(&self.gas_used.to_le_bytes());
        v.extend(&self.tx_root);
        v.extend(&self.state_root);
        v.extend(&self.validator);
        v // don't add sig
    }
}
