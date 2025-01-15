use crate::Hash;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_data, load_cell_type_hash},
    log,
};
use types::error::SilentBerryError;

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
            i = i.checked_add(u.0).ok_or_else(|| {
                log::error!("CheckUDT Failed, udt overflow");
                SilentBerryError::CheckXUDT
            })?;
        }

        let mut o = 0u128;
        for u in &self.inputs {
            o = o.checked_add(u.0).ok_or_else(|| {
                log::error!("CheckUDT Failed, udt overflow");
                SilentBerryError::CheckXUDT
            })?;
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
