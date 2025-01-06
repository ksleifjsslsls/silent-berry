use ckb_testtool::{
    ckb_error::Error,
    ckb_types::{
        core::{Cycle, ScriptHashType, TransactionView},
        packed::{CellOutput, Script},
        prelude::*,
    },
    context::Context,
};
use std::env;

#[cfg(test)]
mod tests;

pub mod spore;

pub const MAX_CYCLES: u64 = 10_000_000;

pub const NAME_ALWAYS_SUC: &str = "always_success";
pub const NAME_XUDT: &str = "xudt_rce";
pub const NAME_BUY_INTENT: &str = "buy-intent";
pub const NAME_DOB_SELLING: &str = "dob-selling";
pub const NAME_ACCOUNT_BOOK: &str = "account-book";
pub const NAME_WITHDRAWAL_INTENT: &str = "withdrawal-intent";

// This helper method runs Context::verify_tx, but in case error happens,
// it also dumps current transaction to failed_txs folder.
pub fn verify_and_dump_failed_tx(
    context: &Context,
    tx: &TransactionView,
    max_cycles: u64,
) -> Result<Cycle, Error> {
    let result = context.verify_tx(tx, max_cycles);
    if result.is_err() {
        let mut path = env::current_dir().expect("current dir");
        path.push("failed_txs");
        std::fs::create_dir_all(&path).expect("create failed_txs dir");
        let mock_tx = context.dump_tx(tx).expect("dump failed tx");
        let json = serde_json::to_string_pretty(&mock_tx).expect("json");
        path.push(format!("0x{:x}.json", tx.hash()));
        println!("Failed tx written to {:?}", path);
        std::fs::write(path, json).expect("write");
    } else {
        println!("Cycles: {}", result.as_ref().unwrap());
    }
    result
}

pub fn new_context() -> Context {
    let mut context = Context::default();
    context.add_contract_dir("../build/release");
    context.add_contract_dir("../build/3rd-bin");
    context
}

pub fn build_always_suc_script(context: &mut Context, args: &[u8]) -> Script {
    let out_point = context.deploy_cell_by_name(NAME_ALWAYS_SUC);

    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data1, args.to_vec().into())
        .expect("always success")
}

pub fn build_xudt_script(
    context: &mut Context,
    owner_script: [u8; 32],
    other_args: &[u8],
) -> Option<Script> {
    let out_point = context.deploy_cell_by_name(NAME_XUDT);
    Some(
        context
            .build_script_with_hash_type(
                &out_point,
                ScriptHashType::Data1,
                [owner_script.to_vec(), other_args.to_vec()].concat().into(),
            )
            .expect("build xudt"),
    )
}

pub fn build_xudt_cell(context: &mut Context, capacity: u64, lock_script: Script) -> CellOutput {
    let xudt_script: Option<Script> = build_xudt_script(context, [0u8; 32], &[]);

    CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(xudt_script.pack())
        .build()
}

pub fn build_dob_selling_script(
    context: &mut Context,
    dob_selling_data: &types::DobSellingData,
) -> Script {
    let out_point = context.deploy_cell_by_name(NAME_DOB_SELLING);

    let dob_hash = ckb_hash(dob_selling_data.as_slice());
    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data2, dob_hash.to_vec().into())
        .expect("build dob-selling script")
}

pub fn build_buy_intent_script(context: &mut Context, args: &[u8]) -> Script {
    let out_point = context.deploy_cell_by_name(NAME_BUY_INTENT);

    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data2, args.to_vec().into())
        .expect("build buy-intent script")
}

pub fn build_buy_intent_cell(
    context: &mut Context,
    capacity: u64,
    lock_script: Script,
    buy_intent_args: &[u8],
) -> CellOutput {
    let t = build_buy_intent_script(context, buy_intent_args);

    CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(Some(t).pack())
        .build()
}

pub fn build_account_book_script(context: &mut Context, args: [u8; 32]) -> Script {
    let out_point = context.deploy_cell_by_name(NAME_ACCOUNT_BOOK);
    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data1, args.to_vec().into())
        .expect("build xudt")
}

pub fn build_account_book_cell(
    context: &mut Context,
    account_book_data: types::AccountBookData,
) -> CellOutput {
    let account_book_script =
        build_account_book_script(context, ckb_hash(account_book_data.as_slice()));

    let xudt_script = build_xudt_script(context, [0u8; 32], &[]);

    CellOutput::new_builder()
        .capacity(16.pack())
        .lock(account_book_script)
        .type_(xudt_script.pack())
        .build()
}

pub fn ckb_hash(data: &[u8]) -> [u8; 32] {
    ckb_testtool::ckb_hash::blake2b_256(data)
}

pub fn to_fixed32(d: &[u8]) -> [u8; 32] {
    d.try_into().unwrap()
}
