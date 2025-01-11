extern crate alloc;

use crate::Hash;
use alloc::vec::Vec;
pub use sparse_merkle_tree::traits::Value;
pub use sparse_merkle_tree::{blake2b::Blake2bHasher, CompiledMerkleProof};
use sparse_merkle_tree::{default_store::DefaultStore, SparseMerkleTree, H256};

type SMTTree = SparseMerkleTree<Blake2bHasher, SmtValue, DefaultStore<SmtValue>>;
#[derive(Default)]
pub struct Smt {
    tree: SMTTree,
}
impl Smt {
    pub fn update(&mut self, key: SmtKey, value: SmtValue) {
        self.tree
            .update(key.get_key(), value)
            .expect("Update SMT Failed");
    }
    pub fn root_hash(&self) -> Hash {
        self.tree.root().as_slice().try_into().unwrap()
    }
    pub fn proof(&self, k: Vec<SmtKey>) -> Vec<u8> {
        let buf = self
            .tree
            .merkle_proof(k.iter().map(|k| k.get_key()).collect())
            .unwrap()
            .compile(k.iter().map(|k| k.get_key()).collect())
            .unwrap()
            .0;
        buf
    }
}

#[derive(Clone)]
pub enum SmtKey {
    Total,
    Platform,
    Auther,
    Member(crate::Hash),
}
impl SmtKey {
    pub fn get_key(&self) -> H256 {
        crate::Hash::ckb_hash(match self {
            Self::Total => "Total".as_bytes(),
            Self::Platform => "Platform".as_bytes(),
            Self::Auther => "Auther".as_bytes(),
            Self::Member(hash) => hash.as_slice(),
        })
        .into()
    }
}

#[derive(Default, Clone)]
pub struct SmtValue {
    pub amount: u128,
}
impl Value for SmtValue {
    fn to_h256(&self) -> H256 {
        let mut hasher = blake2b_ref::Blake2bBuilder::new(crate::HASH_SIZE)
            .personal(crate::CKB_HASH_PERSONALIZATION)
            .build();

        hasher.update(&self.amount.to_le_bytes());

        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);

        hash.into()
    }
    fn zero() -> Self {
        Default::default()
    }
}
impl SmtValue {
    pub fn new(a: u128) -> Self {
        Self { amount: a }
    }
}
