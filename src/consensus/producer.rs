use crate::crypto::{SecretKey, sign};
use crate::chain::{Block, Header};
use crate::mempool::Mempool;

pub struct BlockProducer {
    pub sk: SecretKey,
}

impl BlockProducer {
    pub fn produce(&self, mempool: &mut Mempool, parent: &Block) -> Block {
        let txs = mempool.peek_best(100);
        
        let mut header = Header {
            parent_hash: parent.hash(),
            height: parent.height + 1,
            timestamp: 0, // fill this
            base_fee: parent.header.base_fee,
            gas_limit: 30_000_000,
            gas_used: txs.iter().map(|t| t.gas_limit).sum(),
            tx_root: [0; 32],
            state_root: [0; 32],
            validator: self.sk.public_key().to_bytes(),
            sig: [0; 0], // will fix size error, use DILITHIUM2_SIG_BYTES
        };

        header.sig = sign(&header.to_bytes_without_sig(), &self.sk);
        
        Block { header, transactions: txs }
    }
}
