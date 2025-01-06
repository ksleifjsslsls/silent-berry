#![no_std]

pub const HASH_SIZE: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn ckb_hash(data: &[u8]) -> [u8; HASH_SIZE] {
    let mut hasher = blake2b_ref::Blake2bBuilder::new(HASH_SIZE)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    hasher.update(data);
    let mut hash = [0u8; HASH_SIZE];
    hasher.finalize(&mut hash);

    hash
}
