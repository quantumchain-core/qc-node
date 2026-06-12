use crate::crypto::{PublicKey, verify};
use crate::chain::Block;

pub fn validate_block(block: &Block) -> Result<(), String> {
    let bytes = block.header.to_bytes_without_sig();
    let pk = PublicKey::from_bytes(&block.header.validator)
        .map_err(|_| "bad pubkey")?;
    
    if !verify(&bytes, &block.header.sig, &pk) {
        return Err("bad signature".into());
    }
    Ok(())
}
