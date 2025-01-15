// #![no_std]
#![cfg_attr(not(feature = "std",), no_std)]
extern crate alloc;

#[cfg(feature = "smt")]
pub mod account_book_proof;

mod hash;
pub use hash::{Hash, HASH_SIZE};

mod udt_info;
pub use udt_info::UDTInfo;

mod level;
pub use level::*;

use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_lock, load_cell_type},
    log,
};
use types::error::SilentBerryError as Error;

pub fn get_index_by_code_hash(
    hash: Hash,
    is_lock: bool,
    source: Source,
) -> Result<Vec<usize>, Error> {
    let mut indexs = Vec::new();
    let mut index = 0;
    loop {
        let ret = if is_lock {
            load_cell_lock(index, source).map(Some)
        } else {
            load_cell_type(index, source)
        };
        index += 1;

        match ret {
            Ok(script) => {
                if script.is_none() {
                    continue;
                }
                if hash == script.unwrap().code_hash() {
                    indexs.push(index - 1);
                }
            }
            Err(ckb_std::error::SysError::IndexOutOfBound) => {
                break;
            }
            Err(e) => {
                log::error!("Load cell script failed: {:?}", e);
                return Err(e.into());
            }
        }
    }
    Ok(indexs)
}
