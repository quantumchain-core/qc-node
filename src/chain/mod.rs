pub mod rewards;
#[cfg(test)]
mod m3_tests {
    use super::*;
    use crate::crypto::generate_keypair;
    #
    fn m3_block_sign_verify() {
        let (pk, sk) = generate_keypair();
        let block = create_block(1, b"genesis".to_vec(), &sk, &pk);
        assert!(verify_block(&block));
        let mut bad_block = block.clone();
        bad_block.data[0] ^= 1;
        assert!(!verify_block(&bad_block));
    }
}
