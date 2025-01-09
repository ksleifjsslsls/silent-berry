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
        load_cell_data, load_cell_data_hash, load_cell_lock_hash, load_cell_type,
        load_cell_type_hash, load_script, load_witness_args, QueryIter,
    },
    log,
};
use types::error::SilentBerryError as Error;
use types::DobSellingData;
use utils::Hash;

fn load_verified_data() -> Result<DobSellingData, Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != utils::HASH_SIZE {
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

    let hash = Hash::ckb_hash(data.as_slice());
    let intent_data_hash: Hash = args.try_into()?;

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

fn check_buy_intent_code_hash(hash: [u8; 32]) -> Result<(), Error> {
    QueryIter::new(load_cell_type, Source::Input).any(|f| {
        if f.is_some() {
            let h: [u8; 32] = f.unwrap().code_hash().unpack();
            h == hash
        } else {
            false
        }
    });

    Ok(())
}

fn revocation(data: DobSellingData) -> Result<(), Error> {
    let hash = load_cell_lock_hash(0, Source::Output)?;
    let owner_script_hash: [u8; 32] = data.owner_script_hash().unpack();
    if hash != owner_script_hash {
        log::error!("Revocation failed, owner hash");
        return Err(Error::OnwerScriptHash);
    }

    let amount1 = u128::from_le_bytes(load_cell_data(0, Source::Input)?.try_into().unwrap());
    let amount2 = u128::from_le_bytes(load_cell_data(0, Source::Output)?.try_into().unwrap());
    if amount1 != amount2 {
        log::error!("Revocation failed, input: {}, output: {}", amount1, amount2);
        return Err(Error::XudtIncorrect);
    }

    let buy_intent_code_hash: [u8; 32] = data.buy_intent_code_hash().unpack();
    let lock_code_hash: [u8; 32] = load_cell_type(1, Source::Input)?
        .ok_or_else(|| Error::TxStructure)?
        .code_hash()
        .unpack();
    if buy_intent_code_hash != lock_code_hash {
        log::error!("Revocation failed, Buy Intent not fount in Input 1");
        return Err(Error::BuyIntentCodeHash);
    }

    Ok(())
}

fn program_entry2() -> Result<(), Error> {
    let data = load_verified_data()?;
    let ret = check_spore_data(data.spore_data_hash().unpack());
    if ret.is_err() && ret.unwrap_err() == Error::SporeDataHash {
        revocation(data)?;
    } else {
        check_account_book(data.account_book_script_hash().unpack())?;
        check_buy_intent_code_hash(data.buy_intent_code_hash().unpack())?;
    }
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
