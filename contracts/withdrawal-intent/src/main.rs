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
    ckb_types::prelude::{Entity, Reader},
    error::SysError,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_type, load_cell_type_hash, load_script,
        load_witness_args, QueryIter,
    },
    log,
};
use spore_types::spore::{SporeData, SporeDataReader};
use types::error::SilentBerryError as Error;
use types::WithdrawalIntentData;
use utils::{Hash, UDTInfo};

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

fn load_verified_data(is_input: bool) -> Result<(WithdrawalIntentData, Hash), Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != utils::HASH_SIZE * 2 {
        log::error!("Args len is not {} {}", utils::HASH_SIZE * 2, args.len());
        return Err(Error::VerifiedData);
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

    types::WithdrawalIntentDataReader::verify(witness.to_vec().as_slice(), false)?;
    let data = WithdrawalIntentData::new_unchecked(witness);

    let hash = Hash::ckb_hash(data.as_slice());
    let intent_data_hash: Hash = args[utils::HASH_SIZE..].try_into()?;

    if hash != intent_data_hash {
        log::error!("Check intent data hash failed");
        return Err(Error::VerifiedData);
    }

    Ok((data, args[..utils::HASH_SIZE].try_into()?))
}

fn check_spore(data: &WithdrawalIntentData) -> Result<(), Error> {
    let spore_data = {
        let spore_data1 = load_cell_data(0, Source::Input)?;
        let spore_data2 = load_cell_data(0, Source::Output)?;
        if spore_data1 != spore_data2 {
            log::error!("Input and output sporedata are different");
            return Err(Error::Spore);
        }
        SporeDataReader::verify(&spore_data1, true)?;
        SporeData::new_unchecked(spore_data1.into())
    };

    let cluster_id: Hash = spore_data.cluster_id().try_into()?;
    let cluster_id2: Hash = data.cluster_id().into();
    if cluster_id != cluster_id2 {
        log::error!("The cluster id in Spore is different from the one passed in");
        return Err(Error::Spore);
    }

    let spore_id: Hash = load_cell_type(0, Source::Input)?
        .ok_or_else(|| {
            log::error!("Load Cell type scripe failed, Type is None");
            Error::Spore
        })?
        .args()
        .try_into()?;
    let spore_id2: Hash = data.spore_id().into();
    if spore_id != spore_id2 {
        log::error!("The spore id in Spore is different from the one passed in");
        return Err(Error::Spore);
    }

    let spore_level: ckb_std::ckb_types::packed::Byte = utils::get_spore_level(&spore_data)?.into();
    if data.spore_level() != spore_level {
        log::error!("Check spore level failed");
        return Err(Error::Spore);
    }

    Ok(())
}

fn check_account_book(hash: Hash) -> Result<(), Error> {
    if !QueryIter::new(load_cell_type_hash, Source::Input).any(|script_hash| hash == script_hash) {
        log::error!("Account Book not found in Input");
        return Err(Error::TxStructure);
    }
    if !QueryIter::new(load_cell_type_hash, Source::Output).any(|script_hash| hash == script_hash) {
        log::error!("Account Book not found in Output");
        return Err(Error::TxStructure);
    }

    Ok(())
}

fn program_entry2() -> Result<(), Error> {
    let is_input = is_input()?;
    let (data, accountbook_hash) = load_verified_data(is_input)?;

    if is_input {
        check_account_book(accountbook_hash)?;

        let xudt_script_hash: Hash = data.xudt_script_hash().into();
        let udt_info = UDTInfo::new(xudt_script_hash)?;
        udt_info.check_udt()?;
        Ok(())
    } else {
        // check spore
        check_spore(&data)?;
        Ok(())
    }
}

pub fn program_entry() -> i8 {
    ckb_std::logger::init().expect("Init Logger Failed");
    log::debug!("Begin WithdrawalIntent");
    let res = program_entry2();
    match res {
        Ok(()) => {
            log::debug!("End WithdrawalIntent!");
            0
        }
        Err(error) => {
            log::error!("WithdrawalIntent Failed: {:?}", error);
            u8::from(error) as i8
        }
    }
}
