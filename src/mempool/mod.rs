pub fn add(&mut self, tx: Transaction) -> Result<(), MempoolError> {
    // 1. Duplicate check
    if self.by_hash.contains_key(&tx.hash) {
        return Err(MempoolError::AlreadyKnown);
    }

    // 2. Base fee check
    if tx.base_fee < self.config.base_fee {
        return Err(MempoolError::FeeTooLow);
    }

    // 3. Gas sanity
    if tx.gas_limit == 0 || tx.gas_limit > BLOCK_GAS_LIMIT {
        return Err(MempoolError::InvalidGas);
    }

    // 4. Check nonce + per-sender cap WITHOUT holding the borrow
    {
        let sender_queue = self.by_sender.get(&tx.from);
        if let Some(queue) = sender_queue {
            if queue.contains_key(&tx.nonce) {
                return Err(MempoolError::NonceTooLow);
            }
            if queue.len() >= self.config.per_sender_max {
                return Err(MempoolError::SenderQueueFull);
            }
        }
    } // borrow drops here

    // 5. Global cap — evict lowest-fee tx if needed
    if self.by_hash.len() >= self.config.global_max {
        let new_effective = tx.effective_fee(self.config.base_fee);
        if !self.try_evict_for(new_effective) {
            return Err(MempoolError::PoolFull);
        }
    }

    // 6. TODO: signature verification (hook into src/crypto/)
    // self.verify_signature(&tx)?;

    // 7. Insert - now safe to mutably borrow again
    let effective_fee = tx.effective_fee(self.config.base_fee);
    let hash = tx.hash;
    let fee_key = (u64::MAX - effective_fee, tx.received_at);

    self.by_sender
        .entry(tx.from)
        .or_default()
        .insert(tx.nonce, hash);
    self.fee_index.insert(fee_key, hash);
    self.by_hash.insert(hash, PendingTx { tx, effective_fee });

    Ok(())
                                              }
