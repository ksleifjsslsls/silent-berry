#![cfg_attr(not(any(feature = "native-simulator", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "native-simulator", test))]
extern crate alloc;

#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "native-simulator", test)))]
ckb_std::default_alloc!();

use core::panic;

use alloc::vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::{Builder, Entity, Pack, Reader, Unpack},
    error::SysError,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_type, load_cell_type_hash, load_script,
        load_witness_args, QueryIter,
    },
    log,
};
use spore_types::spore::{SporeData, SporeDataReader};
use types::AccountBookData;
use types::{error::SilentBerryError as Error, AccountBookCellData, AccountBookCellDataReader};
use utils::{Hash, UDTInfo};

fn load_verified_data() -> Result<AccountBookData, Error> {
    let args = load_script()?.args().raw_data();
    if args.len() != utils::HASH_SIZE {
        log::error!("Args len is not {} {}", utils::HASH_SIZE, args.len());
        return Err(Error::VerifiedData);
    }
    let witness = load_witness_args(0, Source::GroupOutput)?;
    let witness = witness
        .output_type()
        .to_opt()
        .ok_or_else(|| {
            log::error!("Load witnesses failed, output type is None");
            Error::ParseWitness
        })?
        .raw_data();

    types::AccountBookDataReader::verify(witness.to_vec().as_slice(), false)?;
    let data = AccountBookData::new_unchecked(witness);

    let data2 = data.clone().as_builder().proof(Default::default()).build();
    let hash = Hash::ckb_hash(data2.as_slice());
    let intent_data_hash: Hash = args.try_into()?;

    if hash != intent_data_hash {
        log::error!("Witness data Hash != Args");
        return Err(Error::VerifiedData);
    }

    Ok(data)
}

fn load_verified_cell_data(is_selling: bool) -> Result<(AccountBookCellData, Hash), Error> {
    let old_data = load_cell_data(0, Source::GroupInput)?;
    let new_data = load_cell_data(0, Source::GroupOutput)?;

    AccountBookCellDataReader::verify(&old_data, true)?;
    AccountBookCellDataReader::verify(&new_data, true)?;

    let old_data = AccountBookCellData::new_unchecked(old_data.into());
    let new_data = AccountBookCellData::new_unchecked(new_data.into());

    {
        let tmp_old = old_data
            .clone()
            .as_builder()
            .smt_root_hash(Default::default())
            .member_count(0u32.pack())
            .build();
        let tmp_new = new_data
            .clone()
            .as_builder()
            .smt_root_hash(Default::default())
            .member_count(0u32.pack())
            .build();

        if tmp_old.as_slice() != tmp_new.as_slice() {
            log::error!("Modification of CellData is not allowed");
            return Err(Error::VerifiedData);
        }
    }

    let old_member_count: u32 = old_data.member_count().unpack();
    let new_member_count: u32 = new_data.member_count().unpack();
    if is_selling {
        if old_member_count + 1 != new_member_count {
            log::error!(
                "CellData member count incorrect: {}, {}",
                old_member_count,
                new_member_count
            );
            return Err(Error::AccountBookModified);
        }
    } else {
        if old_member_count != new_member_count {
            log::error!("Withdrawal does not allow update member_count");
            return Err(Error::AccountBookModified);
        }
    }

    Ok((new_data, old_data.smt_root_hash().into()))
}

fn get_spore(source: Source) -> Result<(SporeData, Hash), Error> {
    let mut spore_data = None;
    let posion = QueryIter::new(load_cell_data, source).position(|cell_data| {
        let r = SporeDataReader::verify(&cell_data, true).is_ok();
        spore_data = Some(SporeData::new_unchecked(cell_data.into()));
        r
    });

    if posion.is_some() && spore_data.is_some() {
        let type_script_args = load_cell_type(posion.unwrap(), source)?
            .ok_or_else(|| {
                log::error!("Load Spore script is none");
                Error::Spore
            })?
            .args();

        Ok((spore_data.unwrap(), type_script_args.try_into()?))
    } else {
        log::error!("Spore Cell not found in {:?}", source);
        Err(Error::Spore)
    }
}

fn check_script_code_hash(data: &AccountBookData) -> Result<bool, Error> {
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
            log::error!("BuyIntent Script not found in Inputs");
            return Err(Error::CheckScript);
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
            log::error!("WithdrawalIntent Script not found in Inputs");
            return Err(Error::CheckScript);
        }

        Ok(false)
    }
}

fn check_account_book() -> Result<Hash, Error> {
    let hash = load_cell_type_hash(0, Source::GroupInput)?.ok_or_else(|| Error::CheckScript)?;
    load_cell_type_hash(0, Source::GroupOutput)?.ok_or_else(|| Error::CheckScript)?;

    // There is only one Input and Output
    let ret = load_cell_type_hash(1, Source::GroupInput);
    if ret.is_ok() || ret.unwrap_err() != SysError::IndexOutOfBound {
        log::error!("Multiple AccountBook found in Input");
        return Err(Error::TxStructure);
    }
    let ret = load_cell_type_hash(1, Source::GroupOutput);
    if ret.is_ok() || ret.unwrap_err() != SysError::IndexOutOfBound {
        log::error!("Multiple AccountBook found in Output");
        return Err(Error::TxStructure);
    }

    Ok(hash.into())
}

fn check_input_type_proxy_lock(
    hash: &Hash,
    udt_info: &UDTInfo,
    amount: u128,
) -> Result<(u128, u128), Error> {
    let self_script_hash: Hash = load_cell_type_hash(0, Source::GroupInput)?
        .ok_or_else(|| {
            log::error!("Unknow Error: load cell type hash (Group Input)");
            Error::Unknow
        })?
        .into();

    let mut input_amount = None;
    for (amount, index) in &udt_info.inputs {
        let script = load_cell_lock(*index, Source::Input)?;
        if *hash != script.code_hash() {
            continue;
        }
        let account_book_script_hash: Hash = script.args().raw_data().try_into()?;
        if self_script_hash == account_book_script_hash {
            if input_amount.is_some() {
                log::error!("Multiple input_type_proxy_locks found in Inputs");
                return Err(Error::TxStructure);
            } else {
                input_amount = Some(*amount);
            }
        }
    }
    let input_amount = input_amount.ok_or_else(|| {
        log::error!("Multiple input_type_proxy_locks not found in Inputs");
        Error::TxStructure
    })?;

    let mut output_amount: Option<u128> = None;
    for (amount, index) in &udt_info.outputs {
        let script = load_cell_lock(*index, Source::Output)?;
        if *hash != script.code_hash() {
            continue;
        }
        let account_book_script_hash: Hash = script.args().raw_data().try_into()?;
        if self_script_hash == account_book_script_hash {
            if output_amount.is_some() {
                log::error!("Multiple input_type_proxy_locks found in Outputs");
                return Err(Error::TxStructure);
            } else {
                output_amount = Some(*amount);
            }
        }
    }
    let output_amount = output_amount.ok_or_else(|| {
        log::error!("Multiple input_type_proxy_locks not found in Outputs");
        Error::TxStructure
    })?;

    if input_amount + amount != output_amount {
        log::error!(
            "In and Out Error: input: {}, output: {}, asset amount: {}",
            input_amount,
            output_amount,
            amount
        );
        return Err(Error::CheckXUDT);
    }

    Ok((input_amount, output_amount))
}

fn is_creation() -> Result<bool, Error> {
    Ok(false)
}

fn creation(_data: AccountBookData) -> Result<(), Error> {
    panic!("Unsuppore");
}

fn selling(
    data: AccountBookData,
    cell_data: AccountBookCellData,
    old_smt_hash: Hash,
) -> Result<(), Error> {
    let (spore_data, spore_id) = get_spore(Source::Output)?;

    // check cluster id
    if spore_data
        .cluster_id()
        .to_opt()
        .ok_or_else(|| {
            log::error!("Cluster ID is None in Spore Data");
            Error::Spore
        })?
        .raw_data()
        != data.cluster_id().as_slice()
    {
        log::error!("The cluster id does not match");
        return Err(Error::VerifiedData);
    }

    let udt_info = utils::UDTInfo::new(data.xudt_script_hash().into())?;
    udt_info.check_udt()?;

    let (input_total, output_total) = check_input_type_proxy_lock(
        &data.input_type_proxy_lock_code_hash().into(),
        &udt_info,
        cell_data.asset_amount().unpack(),
    )?;

    // SMT
    use utils::smt::{Blake2bHasher, CompiledMerkleProof, SmtKey, SmtValue, Value};

    let proof = CompiledMerkleProof(data.proof().unpack());
    if !proof
        .verify::<Blake2bHasher>(
            &old_smt_hash.into(),
            vec![
                (
                    SmtKey::Total.get_key(),
                    SmtValue::new(input_total).to_h256(),
                ),
                (
                    SmtKey::Member(spore_id.clone()).get_key(),
                    Default::default(),
                ),
            ],
        )
        .map_err(|e| {
            log::error!("Verify Inputs Smt Error: {:?}", e);
            Error::Smt
        })?
    {
        log::error!("Verify Inputs SMT failed");
        return Err(Error::Smt);
    }

    let new_smt_hash: Hash = cell_data.smt_root_hash().into();
    if !proof
        .verify::<Blake2bHasher>(
            &new_smt_hash.into(),
            vec![
                (
                    SmtKey::Total.get_key(),
                    SmtValue::new(output_total).to_h256(),
                ),
                (
                    SmtKey::Member(spore_id).get_key(),
                    SmtValue::new(0).to_h256(),
                ),
            ],
        )
        .map_err(|e| {
            log::error!("Verify Output Smt Error: {:?}", e);
            Error::Smt
        })?
    {
        log::error!("Verify Output SMT failed");
        return Err(Error::Smt);
    }

    Ok(())
}

fn withdrawal(_data: AccountBookData) -> Result<(), Error> {
    panic!("Unsuppore");
}

fn program_entry2() -> Result<(), Error> {
    let data = load_verified_data()?;
    if is_creation()? {
        return creation(data);
    }

    check_account_book()?;
    let is_selling = check_script_code_hash(&data)?;
    let (cell_data, old_smt_hash) = load_verified_cell_data(is_selling)?;
    if is_selling {
        selling(data, cell_data, old_smt_hash)?;
    } else {
        withdrawal(data)?;
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
