extern crate alloc;

#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum MemberType {
    Platform = 1,
    Auther,
    Golden,
    Silver,
    Bronze,
    Blue,
}

#[derive(Clone)]
pub struct MemberInfo {
    pub spore_id: [u8; 32],
    pub withdrawn_amount: u128,
    pub member_type: MemberType,
}
impl Default for MemberInfo {
    fn default() -> Self {
        Self {
            spore_id: [0u8; 32],
            withdrawn_amount: 0,
            member_type: MemberType::Blue,
        }
    }
}

#[cfg(feature = "smt")]
pub mod smt {
    use super::*;

    impl MemberInfo {
        pub fn get_key(&self) -> H256 {
            crate::Hash::ckb_hash(&self.spore_id).into()
        }
    }

    use sparse_merkle_tree::{
        blake2b::Blake2bHasher, default_store::DefaultStore, traits::Value, SparseMerkleTree, H256,
    };
    type SMTTree = SparseMerkleTree<Blake2bHasher, MemberInfo, DefaultStore<MemberInfo>>;
    impl Value for MemberInfo {
        fn to_h256(&self) -> H256 {
            let mut hasher = blake2b_ref::Blake2bBuilder::new(crate::HASH_SIZE)
                .personal(crate::CKB_HASH_PERSONALIZATION)
                .build();

            hasher.update(&self.withdrawn_amount.to_le_bytes());
            hasher.update(&[self.member_type.clone() as u8]);

            let mut hash = [0u8; 32];
            hasher.finalize(&mut hash);

            hash.into()
        }
        fn zero() -> Self {
            Default::default()
        }
    }

    #[derive(Default)]
    pub struct SMT {
        tree: SMTTree,
        platform: Option<MemberInfo>,
        auther: Option<MemberInfo>,
    }

    impl SMT {
        pub fn update(&mut self, info: MemberInfo) {
            match info.member_type {
                MemberType::Platform => self.platform = Some(info.clone()),
                MemberType::Auther => self.auther = Some(info.clone()),
                _ => {}
            }
            self.tree
                .update(info.get_key(), info)
                .expect("Update SMT failed");
        }

        pub fn root(&self) -> [u8; 32] {
            self.tree.root().as_slice().try_into().unwrap()
        }

        pub fn proof(&self, k: Option<H256>) -> alloc::vec::Vec<u8> {
            assert!(self.platform.is_some());
            assert!(self.auther.is_some());

            let mut keys = alloc::vec![
                self.platform.as_ref().unwrap().get_key(),
                self.auther.as_ref().unwrap().get_key(),
            ];
            if k.is_some() {
                keys.push(k.unwrap());
            }

            self.tree
                .merkle_proof(keys.clone())
                .unwrap()
                .compile(keys)
                .unwrap()
                .0
        }
    }
}

#[cfg(feature = "smt")]
pub use smt::*;
