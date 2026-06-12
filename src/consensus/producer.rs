use crate::state::{StateDB, Executor, Storage, Address};
use crate::mempool::Mempool;
use crate::chain::{Block, BlockHeader};
use std::time::{SystemTime, UNIX_EPOCH};

impl Producer {
    pub fn produce_block(
        &self,
        mempool: &mut Mempool,
        state: &mut StateDB,
        storage: &Storage,
        coinbase: Address, // your validator address
        parent: &Block,
    ) -> Result<Block, String> {
        
        // 1. Pull txs from mempool
        let txs = mempool.take_batch(1000); // M6: batch size 1000
        let base_fee = 1u128; // M6: hardcoded. M8 will make this dynamic

        // 2. Build header skeleton
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut header = BlockHeader {
            parent_hash: parent.header.hash(),
            number: parent.header.number + 1,
            timestamp,
            state_root: [0u8; 32], // temp, will fill after exec
            gas_limit: 10_000_000, // M6: 10M gas per block
            gas_used: 0, // temp
            base_fee,
            signature: [0u8; 2420], // temp
        };

        let mut block = Block {
            header: header.clone(),
            transactions: txs,
        };

        // 3. M6: EXECUTE BLOCK - this updates state in-place
        let gas_used = Executor::execute_block(state, &block, &coinbase)
            .map_err(|e| format!("exec failed: {:?}", e))?;

        // 4. Update header with results from execution
        header.gas_used = gas_used;
        header.state_root = state.state_root();
        block.header = header;

        // 5. Sign block with Dilithium2 - from M5
        block.header.signature = self.sign_block(&block.header);

        // 6. Save to disk
        storage.put_block(&block).map_err(|e| format!("storage failed: {:?}", e))?;
        storage.put_state(state).map_err(|e| format!("state save failed: {:?}", e))?;

        Ok(block)
    }
        }
