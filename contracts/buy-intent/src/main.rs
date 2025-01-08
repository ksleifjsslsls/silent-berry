#![cfg_attr(not(any(feature = "native-simulator", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "native-simulator", test))]
extern crate alloc;

#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::default_alloc!();

// Args: AccountBookScriptHash | Intent Data Hash

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::{Entity, Reader, Unpack},
    error::SysError,
    high_level::{
        load_cell_capacity, load_cell_lock_hash, load_script, load_witness_args, QueryIter,
    },
    log,
};
use types::{AccountBookData, BuyIntentData};
use utils::HASH_SIZE;

use types::error::SilentBerryError as Error;

fn is_input() -> Result<bool, Error> {
    let input = match load_cell_capacity(0, Source::GroupInput) {
        Ok(_) => true,
        Err(err) => {
            if err == SysError::IndexOutOfBound {
                false
            } else {
                log::error!("Load GroupInput Capacity failed: {:?}", err);
                return Err(err.into());
            }
        }
    };
    let output = match load_cell_capacity(0, Source::GroupOutput) {
        Ok(_) => true,
        Err(err) => {
            if err == SysError::IndexOutOfBound {
                false
            } else {
                log::error!("Load GroupOutput Capacity failed: {:?}", err);
                return Err(err.into());
            }
        }
    };

    if load_cell_capacity(1, Source::GroupInput).is_ok() {
        log::error!("There can be only one GroupInput");
        return Err(Error::TxStructure);
    }
    if load_cell_capacity(1, Source::GroupOutput).is_ok() {
        log::error!("There can be only one GroupOutput");
        return Err(Error::TxStructure);
    }

    if input && !output {
        Ok(true)
    } else if !input && output {
        Ok(false)
    } else {
        log::error!("Both Inputs and Outputs has But Intent");
        Err(Error::TxStructure)
    }
}

fn load_verified_data(is_input: bool) -> Result<(BuyIntentData, [u8; HASH_SIZE]), Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != HASH_SIZE * 2 {
        log::error!("Args len is incorrect: {}", args.len());
        return Err(Error::VerifiedDataLen);
    }

    let source = if is_input {
        Source::GroupInput
    } else {
        Source::GroupOutput
    };

    let witness = load_witness_args(0, source)?;
    let witness = if is_input {
        witness.input_type().to_opt()
    } else {
        witness.output_type().to_opt()
    }
    .ok_or_else(|| {
        log::error!("load witnesses failed");
        Error::TxStructure
    })?
    .raw_data();

    types::BuyIntentDataReader::verify(witness.to_vec().as_slice(), false)?;
    let data = BuyIntentData::new_unchecked(witness);

    let hash = utils::ckb_hash(data.as_slice());
    let intent_data_hash = &args[HASH_SIZE..];

    if hash != intent_data_hash {
        log::error!("Check intent data hash failed");
        return Err(Error::VerifiedData);
    }

    Ok((data, args[..HASH_SIZE].try_into().unwrap()))
}

fn check_xudt_script_hash(script_hash: &[u8], index: usize, source: Source) -> Result<(), Error> {
    let xudt_script_hash = ckb_std::high_level::load_cell_type_hash(index, source)?;

    if xudt_script_hash.is_none() {
        log::error!("xUDT Not Found, index: {}, source: {:?}", index, source);
        return Err(Error::XudtNotFound);
    }

    if script_hash == xudt_script_hash.as_ref().unwrap() {
        Ok(())
    } else {
        log::error!(
            "xUDT Script Hash, index: {}, source: {:?}\n{:02x?}\n{:02x?}",
            index,
            source,
            script_hash,
            xudt_script_hash.as_ref().unwrap()
        );
        Err(Error::XudtIncorrect)
    }
}

fn check_udt(asset_amount: u128) -> Result<(), Error> {
    let in_udt = u128::from_le_bytes(
        ckb_std::high_level::load_cell_data(0, Source::Input)?
            .try_into()
            .map_err(|d| {
                log::error!("Parse input xudt data failed: {:02x?}", d);
                Error::XudtIncorrect
            })?,
    );

    let out_udt0 = u128::from_le_bytes(
        ckb_std::high_level::load_cell_data(0, Source::Output)?
            .try_into()
            .map_err(|d| {
                log::error!("Parse output0 xudt data failed: {:02x?}", d);
                Error::XudtIncorrect
            })?,
    );
    let out_udt1 = u128::from_le_bytes(
        ckb_std::high_level::load_cell_data(1, Source::Output)?
            .try_into()
            .map_err(|d| {
                log::error!("Parse output1 xudt data failed: {:02x?}", d);
                Error::XudtIncorrect
            })?,
    );

    if in_udt
        != out_udt0.checked_add(out_udt1).ok_or_else(|| {
            log::error!("Output xudt addition error, {} + {}", out_udt0, out_udt1);
            Error::XudtIncorrect
        })?
    {
        log::error!(
            "xUDT The total number before and after is different, input: {}, output: {} {}",
            in_udt,
            out_udt0,
            out_udt1
        );
        return Err(Error::XudtIncorrect);
    }

    if asset_amount != out_udt1 {
        log::error!(
            "Not paid in full, {} should be paid, {} actually paid",
            asset_amount,
            out_udt1
        );
        return Err(Error::PaymentAmount);
    }

    Ok(())
}

fn check_input_dob_selling(dob_selling_hash: [u8; 32]) -> Result<(), Error> {
    if !QueryIter::new(load_cell_lock_hash, Source::Input).all(|f| f != dob_selling_hash) {
        Ok(())
    } else {
        Err(Error::DobSellingScriptHash)
    }
}

fn check_account_book(account_book_hash: [u8; 32], amount: u128) -> Result<(), Error> {
    let mut count = 0;
    QueryIter::new(load_cell_lock_hash, Source::Input).all(|f| {
        if f == account_book_hash {
            count += 1;
        }
        true
    });
    if count != 1 {
        log::error!(
            "AccountBook quantity error in Input, Need 1, Found {}",
            count
        );
        return Err(Error::AccountBookScriptHash);
    }

    let mut query_iter = QueryIter::new(load_cell_lock_hash, Source::Output);
    let pos = query_iter.position(|f| f == account_book_hash);
    if pos.is_none() {
        log::error!("AccountBook not found in Output");
        return Err(Error::AccountBookScriptHash);
    }

    if query_iter.position(|f| f == account_book_hash).is_some() {
        log::error!("AccountBook not found in Output");
        return Err(Error::AccountBookScriptHash);
    }

    let accountbook_asset_amount: u128 = AccountBookData::new_unchecked(
        load_witness_args(pos.unwrap(), Source::Output)?
            .lock()
            .to_opt()
            .ok_or_else(|| Error::Unknow)?
            .raw_data(),
    )
    .asset_amount()
    .unpack();

    if accountbook_asset_amount != amount {
        log::error!(
            "Does not match asset_amount in AccountBook, {}, {}",
            amount,
            accountbook_asset_amount
        );
        return Err(Error::VerifiedData);
    }

    Ok(())
}

fn program_entry2() -> Result<(), Error> {
    let is_input = is_input()?;
    let (data, accountbook_hash) = load_verified_data(is_input)?;

    // Check xUDT Script Hash
    let xudt_script_hash = data.xudt_script_hash().as_slice().to_vec();

    if !is_input {
        let dob_selling = ckb_std::high_level::load_cell_lock_hash(1, Source::Output)?;

        if dob_selling != data.dob_selling_script_hash().as_slice() {
            log::error!("Dob Selling Script Hash failed");
            return Err(Error::DobSellingScriptHash);
        }

        check_xudt_script_hash(&xudt_script_hash, 0, Source::Input)?;
        check_xudt_script_hash(&xudt_script_hash, 0, Source::Output)?;
        check_xudt_script_hash(&xudt_script_hash, 1, Source::Output)?;

        let asset_amount = data.asset_amount().unpack();

        check_udt(asset_amount)?;

        let capacity = load_cell_capacity(2, Source::Output)?;

        let buy_intent_capacity =
            u64::from_le_bytes(data.min_capacity().as_slice().try_into().map_err(|e| {
                log::error!("Parse BuyIntentData failed, {:?}", e);
                Error::Unknow
            })?);

        if capacity > buy_intent_capacity {
            log::error!(
                "Capacity does not meet transaction needs, required: {}, actual: {}",
                buy_intent_capacity,
                capacity
            );
            return Err(Error::CapacityError);
        }
        Ok(())
    } else {
        let ret = check_input_dob_selling(data.dob_selling_script_hash().unpack());
        if ret.is_ok() {
            check_account_book(accountbook_hash, data.asset_amount().unpack())?;

            Ok(())
        } else {
            // TODO Since
            Ok(())
        }
    }
}

pub fn program_entry() -> i8 {
    ckb_std::logger::init().expect("Init Logger Failed");
    log::debug!("Begin BuyIntent!");

    let res = program_entry2();
    match res {
        Ok(()) => {
            log::debug!("End BuyIntent!");
            0
        }
        Err(error) => {
            log::error!("BuyIntent Failed: {:?}", error);
            u8::from(error) as i8
        }
    }
}
