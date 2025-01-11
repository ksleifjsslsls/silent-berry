// #![no_std]
#![cfg_attr(not(feature = "std",), no_std)]
extern crate alloc;

#[cfg(feature = "smt")]
pub mod smt;

use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Pack,
    error::SysError,
    high_level::{load_cell_data, load_cell_type_hash},
    log,
};
use types::{blockchain::Byte32, error::SilentBerryError};

pub const HASH_SIZE: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

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
impl Into<sparse_merkle_tree::H256> for Hash {
    fn into(self) -> sparse_merkle_tree::H256 {
        self.0.into()
    }
}
impl Into<types::blockchain::Byte32> for Hash {
    fn into(self) -> types::blockchain::Byte32 {
        self.0.pack()
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
impl TryFrom<types::blockchain::Bytes> for Hash {
    type Error = SilentBerryError;
    fn try_from(value: types::blockchain::Bytes) -> Result<Self, Self::Error> {
        value.raw_data().to_vec().as_slice().try_into()
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
impl PartialEq<Byte32> for Hash {
    fn eq(&self, other: &Byte32) -> bool {
        &self.0 == other.raw_data().to_vec().as_slice()
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

pub struct UDTInfo {
    pub inputs: Vec<(u128, usize)>,
    pub outputs: Vec<(u128, usize)>,
}
impl UDTInfo {
    pub fn new(xudt_script_hash: Hash) -> Result<Self, SilentBerryError> {
        let inputs = Self::load_udt(Source::Input, &xudt_script_hash)?;
        let outputs = Self::load_udt(Source::Output, &xudt_script_hash)?;

        Ok(Self { inputs, outputs })
    }

    fn load_udt(
        source: Source,
        xudt_script_hash: &Hash,
    ) -> Result<Vec<(u128, usize)>, SilentBerryError> {
        let mut xudt_info = Vec::new();
        let mut index = 0usize;
        loop {
            match load_cell_type_hash(index, source) {
                Ok(type_hash) => {
                    if (*xudt_script_hash) == type_hash {
                        let udt = u128::from_le_bytes(
                            load_cell_data(index, source)?.try_into().map_err(|d| {
                                log::error!("Parse {:?} xudt data failed: {:02x?}", source, d);
                                SilentBerryError::CheckXUDT
                            })?,
                        );
                        xudt_info.push((udt, index));
                    }
                }
                Err(error) => match error {
                    SysError::IndexOutOfBound => break,
                    _ => return Err(error.into()),
                },
            }
            index += 1;
        }
        Ok(xudt_info)
    }

    pub fn check_udt(&self) -> Result<(), SilentBerryError> {
        let mut i = 0u128;
        for u in &self.inputs {
            i = i
                .checked_add(u.0)
                .ok_or_else(|| SilentBerryError::CheckXUDT)?;
        }

        let mut o = 0u128;
        for u in &self.inputs {
            o = o
                .checked_add(u.0)
                .ok_or_else(|| SilentBerryError::CheckXUDT)?;
        }

        if i != o {
            log::error!("Inputs and Outputs UDT is not equal");
            return Err(SilentBerryError::CheckXUDT);
        }

        Ok(())
    }

    pub fn input_total(&self) -> u128 {
        // Overflow is already checked in check_udt
        let mut total = 0;
        for (amount, _) in &self.inputs {
            total += amount;
        }
        total
    }
}
