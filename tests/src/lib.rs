use ckb_testtool::{
    ckb_error::Error,
    ckb_types::{
        core::{Cycle, TransactionView},
        packed::CellOutput,
        prelude::Entity,
    },
    context::Context,
};

pub mod build_tx;
pub mod spore;

#[cfg(test)]
mod tests;

pub const MAX_CYCLES: u64 = 10_000_000;

pub const ALWAYS_SUC_NAME: &str = "always_success";
pub const XUDT_NAME: &str = "xudt_rce";
pub const SPORE_NAME: &str = "spore";
pub const CLUSTER_NAME: &str = "cluster";
pub const BUY_INTENT_NAME: &str = "buy-intent";
pub const DOB_SELLING_NAME: &str = "dob-selling";
pub const ACCOUNT_BOOK_NAME: &str = "account-book";
pub const WITHDRAWAL_INTENT_NAME: &str = "withdrawal-intent";
pub const INPUT_TYPE_PROXY_LOCK_NAME: &str = "input-type-proxy-lock";

lazy_static::lazy_static! {
    static ref BuyIntentCodeHash: [u8; 32] = get_code_hash(BUY_INTENT_NAME);
    static ref DOBSellingCodeHash: [u8; 32] = get_code_hash(DOB_SELLING_NAME);
    static ref AccountBookCodeHash: [u8; 32] = get_code_hash(ACCOUNT_BOOK_NAME);
    static ref WithdrawalIntentCodeHash: [u8; 32] = get_code_hash(WITHDRAWAL_INTENT_NAME);
    static ref InputTypeProxyLockCodeHash: [u8; 32] = get_code_hash(INPUT_TYPE_PROXY_LOCK_NAME);
    static ref SporeCodeHash: [u8; 32] = get_code_hash(SPORE_NAME);
}

fn get_code_hash(n: &str) -> [u8; 32] {
    let mut context = new_context();
    let out_point = context.deploy_cell_by_name(n);
    let (_, contract_data) = context.cells.get(&out_point).unwrap();
    CellOutput::calc_data_hash(contract_data)
        .as_slice()
        .try_into()
        .unwrap()
}

pub fn print_tx_info(context: &Context, tx: &TransactionView) {
    use std::collections::HashMap;
    fn update_hash(bins: &mut HashMap<[u8; 32], String>, name: &str) {
        let mut c2 = new_context();
        let op = c2.deploy_cell_by_name(name);
        let (_, d) = c2.get_cell(&op).unwrap();
        let h = ckb_hash(&d);
        bins.insert(h, name.to_string());
    }
    let mut bins = HashMap::new();
    update_hash(&mut bins, "always_success");
    update_hash(&mut bins, "xudt_rce");
    update_hash(&mut bins, "spore");
    update_hash(&mut bins, "cluster");
    update_hash(&mut bins, INPUT_TYPE_PROXY_LOCK_NAME);

    update_hash(&mut bins, "buy-intent");
    update_hash(&mut bins, "dob-selling");
    update_hash(&mut bins, "withdrawal-intent");
    update_hash(&mut bins, "account-book");

    let mut d: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&context.dump_tx(tx).expect("dump tx info"))
            .expect("tx format json"),
    )
    .unwrap();

    d.get_mut("mock_info")
        .unwrap()
        .get_mut("cell_deps")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .all(|f| {
            let f_data = f.get_mut("data").unwrap();
            let hash = ckb_hash(&hex::decode(&f_data.as_str().unwrap()[2..]).unwrap());

            let name = bins.get(&hash);
            if name.is_some() {
                *f_data = serde_json::to_value(format!("-- {} --", &name.unwrap())).unwrap();
            }
            true
        });

    fn add_contract_name(output: &mut serde_json::Value, bins: &HashMap<[u8; 32], String>) {
        let script = output.get_mut("lock").unwrap();
        let code_hash: [u8; 32] =
            hex::decode(&script.get("code_hash").unwrap().as_str().unwrap()[2..])
                .unwrap()
                .try_into()
                .unwrap();
        let hash_type = script.get("hash_type").unwrap().as_str().unwrap();

        if hash_type != "type" {
            let n = bins.get(&code_hash);
            script.as_object_mut().unwrap().insert(
                "name".to_string(),
                serde_json::to_value(n.unwrap()).unwrap(),
            );
        }

        let script = output.get_mut("type");
        if script.is_none() {
            return;
        }
        let script = script.unwrap();
        let code_hash = script.get("code_hash");
        if code_hash.is_none() {
            return;
        }

        let code_hash: [u8; 32] = hex::decode(&code_hash.unwrap().as_str().unwrap()[2..])
            .unwrap()
            .try_into()
            .unwrap();
        let hash_type = script.get("hash_type").unwrap().as_str().unwrap();

        if hash_type != "type" {
            let n = bins.get(&code_hash);
            script.as_object_mut().unwrap().insert(
                "name".to_string(),
                serde_json::to_value(n.unwrap()).unwrap(),
            );
        }
    }

    d.get_mut("mock_info")
        .unwrap()
        .get_mut("inputs")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .all(|f| {
            let output = f.get_mut("output").unwrap();
            add_contract_name(output, &bins);
            true
        });
    d.get_mut("tx")
        .unwrap()
        .get_mut("outputs")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .all(|f| {
            add_contract_name(f, &bins);
            true
        });

    println!("tx info: \n{}", d);
}

// This helper method runs Context::verify_tx, but in case error happens,
// it also dumps current transaction to failed_txs folder.
pub fn verify_and_dump_failed_tx(
    context: &Context,
    tx: &TransactionView,
    max_cycles: u64,
) -> Result<Cycle, Error> {
    let result = context.verify_tx(tx, max_cycles);
    if result.is_err() {
        // let mut path = env::current_dir().expect("current dir");
        // path.push("failed_txs");
        // std::fs::create_dir_all(&path).expect("create failed_txs dir");
        // let mock_tx = context.dump_tx(tx).expect("dump failed tx");
        // let json = serde_json::to_string_pretty(&mock_tx).expect("json");
        // path.push(format!("0x{:x}.json", tx.hash()));
        // println!("Failed tx written to {:?}", path);
        // std::fs::write(path, json).expect("write");

        print_tx_info(context, tx);
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

pub fn ckb_hash(data: &[u8]) -> [u8; 32] {
    ckb_testtool::ckb_hash::blake2b_256(data)
}

pub fn new_smt_tree() -> utils::smt::Smt {
    use utils::smt::*;
    use utils::Hash;

    let mut smt = Smt::default();

    let mut c: u8 = 0;
    fn new_hash(count: &mut u8) -> Hash {
        *count += 1;
        [*count; 32].into()
    }

    smt.update(SmtKey::Auther, SmtValue::new(2001));
    smt.update(SmtKey::Platform, SmtValue::new(0));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(123));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(4324));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(4444));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(555));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(0));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(300000));

    smt
}
