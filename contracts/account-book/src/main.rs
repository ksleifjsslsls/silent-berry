#![cfg_attr(not(any(feature = "native-simulator", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "native-simulator", test))]
extern crate alloc;

#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::default_alloc!();

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::{Builder, Entity, Reader},
    error::SysError,
    high_level::{
        load_cell_lock, load_cell_type, load_cell_type_hash, load_script, load_witness_args,
        QueryIter,
    },
    log,
};
use types::error::SilentBerryError as Error;
use types::AccountBookData;
use utils::Hash;

fn load_verified_data() -> Result<AccountBookData, Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != utils::HASH_SIZE {
        log::error!("Args len is incorrect: {}", args.len());
        return Err(Error::VerifiedDataLen);
    }
    let witness = load_witness_args(0, Source::GroupOutput)?;
    let witness = witness
        .output_type()
        .to_opt()
        .ok_or_else(|| {
            log::error!("load witnesses failed");
            Error::TxStructure
        })?
        .raw_data();

    types::AccountBookDataReader::verify(witness.to_vec().as_slice(), false)?;
    let data = AccountBookData::new_unchecked(witness);

    let data2 = data.clone().as_builder().proof(Default::default()).build();
    let hash = Hash::ckb_hash(data2.as_slice());
    let intent_data_hash: Hash = args.try_into()?;

    if hash != intent_data_hash {
        log::error!("Check intent data hash failed");
        return Err(Error::VerifiedData);
    }

    Ok(data)
}

fn check_silent_berry_code_hash(data: &AccountBookData) -> Result<bool, Error> {
    let dob_selling_code_hash = data.dob_selling_code_hash();
    if QueryIter::new(load_cell_lock, Source::Input).any(|f| f.code_hash() == dob_selling_code_hash)
    {
        let hash = data.buy_intent_code_hash();
        if !QueryIter::new(load_cell_type, Source::Input).any(|f| {
            if let Some(s) = f {
                s.code_hash() == hash
            } else {
                false
            }
        }) {
            return Err(Error::BuyIntentCodeHash);
        }

        Ok(true)
    } else {
        let hash = data.withdrawal_intent_code_hash();
        if !QueryIter::new(load_cell_type, Source::Input).any(|f| {
            if let Some(s) = f {
                s.code_hash() == hash
            } else {
                false
            }
        }) {
            return Err(Error::BuyIntentCodeHash);
        }

        Ok(false)
    }
}

fn is_creation() -> Result<bool, Error> {
    Ok(false)
}

fn creation(_data: AccountBookData) -> Result<(), Error> {
    Ok(())
}

fn check_account_book() -> Result<Hash, Error> {
    //
    let hash =
        load_cell_type_hash(0, Source::GroupInput)?.ok_or_else(|| Error::AccountBookScriptHash)?;
    load_cell_type_hash(0, Source::GroupOutput)?.ok_or_else(|| Error::AccountBookScriptHash)?;

    // There is only one Input and Output
    let ret = load_cell_type_hash(1, Source::GroupInput);
    if ret.is_ok() || ret.unwrap_err() != SysError::IndexOutOfBound {
        return Err(Error::TxStructure);
    }
    let ret = load_cell_type_hash(1, Source::GroupOutput);
    if ret.is_ok() || ret.unwrap_err() != SysError::IndexOutOfBound {
        return Err(Error::TxStructure);
    }

    Ok(hash.into())
}

fn program_entry2() -> Result<(), Error> {
    let data = load_verified_data()?;
    if is_creation()? {
        return creation(data);
    }

    check_account_book()?;
    let is_selling = check_silent_berry_code_hash(&data)?;

    if is_selling {
    } else {
        // TODO
    }

    Ok(())
}

pub fn program_entry() -> i8 {
    ckb_std::logger::init().expect("Init Logger Failed");
    log::debug!("Begin AccountBook");

    let res = program_entry2();
    match res {
        Ok(()) => {
            log::debug!("End AccountBook!");
            0
        }
        Err(error) => {
            log::error!("AccountBook Failed: {:?}", error);
            u8::from(error) as i8
        }
    }
}
