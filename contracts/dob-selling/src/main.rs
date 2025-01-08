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
    ckb_types::prelude::{Entity, Reader, Unpack},
    high_level::{
        load_cell_data_hash, load_cell_type_hash, load_script, load_witness_args, QueryIter,
    },
    log,
};
use types::error::SilentBerryError as Error;
use types::DobSellingData;
use utils::HASH_SIZE;

fn load_verified_data() -> Result<DobSellingData, Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != HASH_SIZE {
        log::error!("Args len is incorrect: {}", args.len());
        return Err(Error::VerifiedDataLen);
    }

    let witness = load_witness_args(0, Source::GroupInput)?;
    let witness = witness
        .lock()
        .to_opt()
        .ok_or_else(|| {
            log::error!("load witnesses failed");
            Error::TxStructure
        })?
        .raw_data();

    types::DobSellingDataReader::verify(witness.to_vec().as_slice(), false)?;
    let data = DobSellingData::new_unchecked(witness);

    let hash = utils::ckb_hash(data.as_slice());
    let intent_data_hash: [u8; 32] = args.to_vec().try_into().unwrap();

    if hash != intent_data_hash {
        log::error!("Check intent data hash failed");
        return Err(Error::VerifiedData);
    }

    Ok(data)
}

fn check_spore_data(hash: [u8; 32]) -> Result<(), Error> {
    if QueryIter::new(load_cell_data_hash, Source::Output).all(|f| f != hash) {
        Err(Error::SporeDataHash)
    } else {
        Ok(())
    }
}

fn check_account_book(account_book_hash: [u8; 32]) -> Result<(), Error> {
    if !QueryIter::new(load_cell_type_hash, Source::Input)
        .any(|f| f.is_some() && f.unwrap() == account_book_hash)
    {
        log::error!("AccountBook not found");
        return Err(Error::AccountBookScriptHash);
    }
    if !QueryIter::new(load_cell_type_hash, Source::Output)
        .any(|f| f.is_some() && f.unwrap() == account_book_hash)
    {
        log::error!("AccountBook not found");
        return Err(Error::AccountBookScriptHash);
    }

    Ok(())
}

fn program_entry2() -> Result<(), Error> {
    let data = load_verified_data()?;
    check_spore_data(data.spore_data_hash().unpack())?;
    check_account_book(data.account_book_script_hash().unpack())?;

    Ok(())
}

pub fn program_entry() -> i8 {
    ckb_std::logger::init().expect("Init Logger Failed");
    log::debug!("Begin DobSelling");
    let res = program_entry2();
    match res {
        Ok(()) => {
            log::debug!("End DobSelling!");
            0
        }
        Err(error) => {
            log::error!("DobSelling Failed: {:?}", error);
            u8::from(error) as i8
        }
    }
}
