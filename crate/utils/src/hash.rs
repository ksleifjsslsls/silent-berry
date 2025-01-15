use ckb_std::{ckb_types::prelude::Pack, log};
use types::error::SilentBerryError as Error;

pub const HASH_SIZE: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

#[derive(PartialEq, Eq, Clone)]
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
impl From<Hash> for sparse_merkle_tree::H256 {
    fn from(value: Hash) -> Self {
        value.0.into()
    }
}
impl From<Hash> for types::blockchain::Byte32 {
    fn from(value: Hash) -> Self {
        value.0.pack()
    }
}
impl From<Hash> for [u8; 32] {
    fn from(value: Hash) -> Self {
        value.0
    }
}
impl From<Hash> for types::blockchain::Bytes {
    fn from(value: Hash) -> Self {
        use ckb_std::ckb_types::prelude::Pack;
        value.0.to_vec().pack()
    }
}
impl From<Hash> for ckb_std::ckb_types::bytes::Bytes {
    fn from(value: Hash) -> Self {
        value.0.to_vec().into()
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Error> {
        let v: [u8; 32] = value.try_into().map_err(|e| {
            ckb_std::log::warn!("Type conversion failed, Error: {:?}", e);
            Error::TypeConversion
        })?;

        Ok(Self(v))
    }
}
impl TryFrom<ckb_std::ckb_types::bytes::Bytes> for Hash {
    type Error = Error;
    fn try_from(value: ckb_std::ckb_types::bytes::Bytes) -> Result<Self, Error> {
        let v: [u8; 32] = value.to_vec().try_into().map_err(|e| {
            ckb_std::log::warn!("Type conversion failed, Error: {:?}", e);
            Error::TypeConversion
        })?;

        Ok(Self(v))
    }
}
impl TryFrom<types::blockchain::Bytes> for Hash {
    type Error = Error;
    fn try_from(value: types::blockchain::Bytes) -> Result<Self, Self::Error> {
        value.raw_data().to_vec().as_slice().try_into()
    }
}
impl TryFrom<types::blockchain::BytesOpt> for Hash {
    type Error = Error;
    fn try_from(value: types::blockchain::BytesOpt) -> Result<Self, Self::Error> {
        value
            .to_opt()
            .ok_or_else(|| {
                log::error!("BytesOpt to Hash failed, BytesOpt is None");
                Error::TypeConversion
            })?
            .try_into()
    }
}
impl TryFrom<spore_types::spore::BytesOpt> for Hash {
    type Error = Error;
    fn try_from(value: spore_types::spore::BytesOpt) -> Result<Self, Self::Error> {
        value
            .to_opt()
            .ok_or_else(|| {
                log::error!("BytesOpt to Hash failed, BytesOpt is None");
                Error::TypeConversion
            })?
            .raw_data()
            .to_vec()
            .as_slice()
            .try_into()
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
impl PartialEq<types::blockchain::Byte32> for Hash {
    fn eq(&self, other: &types::blockchain::Byte32) -> bool {
        self.0 == other.raw_data().to_vec().as_slice()
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
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}
