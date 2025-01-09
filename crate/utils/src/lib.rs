#![no_std]

mod member_info;
pub use member_info::*;

use types::error::SilentBerryError;

pub const HASH_SIZE: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

#[derive(PartialEq, Eq)]
pub struct Hash(pub [u8; HASH_SIZE]);
impl From<[u8; 32]> for Hash {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}
impl From<types::blockchain::Byte32> for Hash {
    fn from(value: types::blockchain::Byte32) -> Self {
        Self(value.raw_data().to_vec().try_into().unwrap())
    }
}
#[cfg(feature = "smt")]
impl From<sparse_merkle_tree::H256> for Hash {
    fn from(value: sparse_merkle_tree::H256) -> Self {
        Self(value.into())
    }
}
#[cfg(feature = "smt")]
impl Into<sparse_merkle_tree::H256> for Hash {
    fn into(self) -> sparse_merkle_tree::H256 {
        self.0.into()
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = SilentBerryError;
    fn try_from(value: &[u8]) -> Result<Self, SilentBerryError> {
        let v: [u8; 32] = value.try_into().map_err(|e| {
            ckb_std::log::warn!("Type conversion failed, Error: {:?}", e);
            SilentBerryError::TypeConversion
        })?;

        Ok(Self(v))
    }
}
impl TryFrom<ckb_std::ckb_types::bytes::Bytes> for Hash {
    type Error = SilentBerryError;
    fn try_from(value: ckb_std::ckb_types::bytes::Bytes) -> Result<Self, SilentBerryError> {
        let v: [u8; 32] = value.to_vec().try_into().map_err(|e| {
            ckb_std::log::warn!("Type conversion failed, Error: {:?}", e);
            SilentBerryError::TypeConversion
        })?;

        Ok(Self(v))
    }
}

impl PartialEq<&[u8]> for Hash {
    fn eq(&self, other: &&[u8]) -> bool {
        &self.0 == other
    }
}
impl PartialEq<[u8; 32]> for Hash {
    fn eq(&self, other: &[u8; 32]) -> bool {
        &self.0 == other
    }
}
impl PartialEq<Option<[u8; 32]>> for Hash {
    fn eq(&self, other: &Option<[u8; 32]>) -> bool {
        if let Some(v) = other {
            &self.0 == v
        } else {
            false
        }
    }
}

impl Hash {
    pub fn ckb_hash(data: &[u8]) -> Self {
        let mut hasher = blake2b_ref::Blake2bBuilder::new(HASH_SIZE)
            .personal(CKB_HASH_PERSONALIZATION)
            .build();
        hasher.update(data);
        let mut hash = [0u8; HASH_SIZE];
        hasher.finalize(&mut hash);

        hash.into()
    }
}
