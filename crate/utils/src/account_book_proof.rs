extern crate alloc;

use crate::Hash;
use alloc::vec::Vec;
pub use sparse_merkle_tree::traits::Value;
pub use sparse_merkle_tree::{blake2b::Blake2bHasher, CompiledMerkleProof, H256};
use types::error::SilentBerryError;

#[cfg(feature = "std")]
use sparse_merkle_tree::{default_store::DefaultStore, SparseMerkleTree};

#[cfg(feature = "std")]
pub type SMTTree = SparseMerkleTree<Blake2bHasher, SmtValue, DefaultStore<SmtValue>>;

#[derive(Clone)]
pub enum SmtKey {
    TotalA,
    TotalB,
    TotalC,
    TotalD,
    Platform,
    Auther,
    Member(crate::Hash),
}
impl SmtKey {
    pub fn get_key(&self) -> H256 {
        crate::Hash::ckb_hash(match self {
            Self::TotalA => "Total-A".as_bytes(),
            Self::TotalB => "Total-B".as_bytes(),
            Self::TotalC => "Total-C".as_bytes(),
            Self::TotalD => "Total-D".as_bytes(),
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

pub struct AccountBookProof {
    proof: Vec<u8>,
}
impl AccountBookProof {
    pub fn new(proof: Vec<u8>) -> Self {
        Self { proof }
    }

    pub fn verify(
        &self,
        root: Hash,
        total: (u128, u128, u128, u128),
        member: (SmtKey, Option<u128>),
    ) -> Result<bool, SilentBerryError> {
        use alloc::vec;
        let proof = CompiledMerkleProof(self.proof.clone());

        proof
            .verify::<Blake2bHasher>(
                &root.into(),
                vec![
                    (SmtKey::TotalA.get_key(), SmtValue::new(total.0).to_h256()),
                    (SmtKey::TotalB.get_key(), SmtValue::new(total.1).to_h256()),
                    (SmtKey::TotalC.get_key(), SmtValue::new(total.2).to_h256()),
                    (SmtKey::TotalD.get_key(), SmtValue::new(total.3).to_h256()),
                    (
                        member.0.get_key(),
                        if let Some(a) = member.1 {
                            SmtValue::new(a).to_h256()
                        } else {
                            Default::default()
                        },
                    ),
                ],
            )
            .map_err(|e| {
                ckb_std::log::error!("Verify Inputs Smt Error: {:?}", e);
                SilentBerryError::Smt
            })
    }
}
