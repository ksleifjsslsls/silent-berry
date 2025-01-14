use crate::{build_tx::*, *};
use ckb_testtool::ckb_types::{
    core::TransactionBuilder,
    packed::{CellDep, CellInput, CellOutput, Script, WitnessArgs},
    prelude::{Builder, Entity, Pack, PackVec},
};
use spore_types::spore::SporeData;
use types::{
    AccountBookCellData, AccountBookData, BuyIntentData, DobSellingData, WithdrawalIntentData,
};
use utils::Hash;

const DATA_ASSET_AMOUNT: u128 = 200;
const DATA_MIN_CAPACITY: u64 = 1000;

fn def_dob_selling_data(_context: &mut Context, spore_data: &SporeData) -> DobSellingData {
    DobSellingData::new_builder()
        .spore_data_hash(ckb_hash(spore_data.as_slice()).pack())
        .buy_intent_code_hash((*BuyIntentCodeHash).pack())
        .build()
}
fn def_buy_intent_data(context: &mut Context, dob_data: &DobSellingData) -> BuyIntentData {
    BuyIntentData::new_builder()
        .xudt_script_hash(get_opt_script_hash(&build_xudt_script(context)).pack())
        .dob_selling_script_hash(
            get_script_hash(&build_dob_selling_script(context, dob_data)).pack(),
        )
        .asset_amount(DATA_ASSET_AMOUNT.pack())
        .min_capacity(DATA_MIN_CAPACITY.pack())
        .change_script_hash([0u8; 32].pack())
        .expire_since(1000u64.pack())
        .owner_script_hash([0u8; 32].pack())
        .build()
}
fn def_withdrawal_intent_data(context: &mut Context) -> WithdrawalIntentData {
    WithdrawalIntentData::new_builder()
        .xudt_script_hash(get_opt_script_hash(&build_xudt_script(context)).pack())
        .spore_id([0u8; 32].pack())
        .cluster_id([0u8; 32].pack())
        .expire_since(1000u64.pack())
        .owner_script_hash([0u8; 32].pack())
        .build()
}
fn def_account_book_data(context: &mut Context) -> AccountBookData {
    AccountBookData::new_builder()
        .dob_selling_code_hash((*DOBSellingCodeHash).pack())
        .buy_intent_code_hash((*BuyIntentCodeHash).pack())
        .withdrawal_intent_code_hash((*WithdrawalIntentCodeHash).pack())
        .xudt_script_hash(get_opt_script_hash(&build_xudt_script(context)).pack())
        .input_type_proxy_lock_code_hash((*InputTypeProxyLockCodeHash).pack())
        .cluster_id([3u8; 32].pack())
        .build()
}
fn def_account_book_cell_data(_context: &mut Context) -> AccountBookCellData {
    AccountBookCellData::new_builder()
        .auther_id([1u8; 32].pack())
        .platform_id([2u8; 32].pack())
        .asset_amount(DATA_ASSET_AMOUNT.pack())
        .a_num(3u32.pack())
        .b_num(17u32.pack())
        .c_num(25u32.pack())
        .a_profit(
            types::AProfit::new_builder()
                .set([20u8.into(), 80u8.into()])
                .build(),
        )
        .b_profit(
            types::BProfit::new_builder()
                .set([20u8.into(), 20u8.into(), 60u8.into()])
                .build(),
        )
        .c_profit(
            types::CProfit::new_builder()
                .set([20u8.into(), 20u8.into(), 36u8.into(), 24u8.into()])
                .build(),
        )
        .d_profit(
            types::DProfit::new_builder()
                .set([
                    20u8.into(),
                    20u8.into(),
                    20u8.into(),
                    20u8.into(),
                    20u8.into(),
                ])
                .build(),
        )
        .build()
}

fn def_spore(context: &mut Context) -> (SporeData, CellDep) {
    let (cluster_id, cluster_deps) = build_cluster(context, ("Spore Cluster", "Test Cluster"));
    let spore_data = crate::spore::build_serialized_spore_data(
        "{\"dna\":\"4000000000002\"}".as_bytes().to_vec(),
        "dob/1",
        Some(cluster_id.to_vec()),
    );
    (spore_data, cluster_deps)
}

fn get_cluster_id(d: &SporeData) -> [u8; 32] {
    d.cluster_id()
        .to_opt()
        .unwrap()
        .raw_data()
        .to_vec()
        .try_into()
        .unwrap()
}

#[test]
fn test_simple_buy_intent() {
    let mut context = new_context();

    let lock_script = build_user1_script(&mut context);
    let udt_cell = build_xudt_cell(&mut context, lock_script.clone());

    let inputs = vec![
        build_input(context.create_cell(udt_cell.clone(), 1000u128.to_le_bytes().to_vec().into())),
        build_input(build_out_point1(&mut context, lock_script.clone())),
    ];

    let (spore_data, _) = def_spore(&mut context);
    let dob_selling_data = def_dob_selling_data(&mut context, &spore_data);
    let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
    let dob_selling_udt = build_xudt_cell(&mut context, dob_selling.clone());

    let buy_intent_data = def_buy_intent_data(&mut context, &dob_selling_data);

    let buy_intent_script = build_buy_intent_cell(
        &mut context,
        1000,
        lock_script,
        &[[0u8; 32], ckb_hash(buy_intent_data.as_slice())].concat(),
    );

    let outputs = vec![
        udt_cell.clone(),
        dob_selling_udt.clone(),
        buy_intent_script.clone(),
    ];

    let outputs_data: Vec<ckb_testtool::ckb_types::packed::Bytes> = vec![
        800u128.to_le_bytes().to_vec().pack(),
        DATA_ASSET_AMOUNT.to_le_bytes().to_vec().pack(),
        Default::default(),
    ];

    let witnesses = vec![
        Default::default(),
        Default::default(),
        WitnessArgs::new_builder()
            .output_type(Some(buy_intent_data.as_bytes()).pack())
            .build()
            .as_slice()
            .pack(),
    ];

    let tx = context.complete_tx(
        TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .witnesses(witnesses)
            .build(),
    );
    // print_tx_info(&context, &tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_revocation_buy_intent() {
    let mut context = new_context();
    let def_lock_script: Script = build_always_suc_script(&mut context, &[]);
    let (spore_data, _cluster_deps) = def_spore(&mut context);
    let account_book_script_hash = [0u8; 32];

    // DOB Selling
    let dob_selling_data = def_dob_selling_data(&mut context, &spore_data)
        .as_builder()
        .owner_script_hash(def_lock_script.calc_script_hash())
        .build();
    let cell_input_dob_selling = {
        let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
        let dob_selling_udt = build_xudt_cell(&mut context, dob_selling.clone());

        CellInput::new_builder()
            .previous_output(context.create_cell(
                dob_selling_udt.clone(),
                DATA_ASSET_AMOUNT.to_le_bytes().to_vec().into(),
            ))
            .build()
    };
    let tx = TransactionBuilder::default()
        .input(cell_input_dob_selling)
        .output(build_xudt_cell(&mut context, def_lock_script.clone()))
        .output_data(DATA_ASSET_AMOUNT.to_le_bytes().to_vec().pack())
        .witness(
            WitnessArgs::new_builder()
                .lock(Some(dob_selling_data.as_bytes()).pack())
                .build()
                .as_bytes()
                .pack(),
        )
        .build();

    // Buy Intent
    let buy_intent_data = def_buy_intent_data(&mut context, &dob_selling_data)
        .as_builder()
        .owner_script_hash(def_lock_script.calc_script_hash())
        .build();
    let cell_input_buy_intent = {
        let buy_intent_script = build_buy_intent_cell(
            &mut context,
            1000,
            def_lock_script.clone(),
            &[
                account_book_script_hash,
                ckb_hash(buy_intent_data.as_slice()),
            ]
            .concat(),
        );

        CellInput::new_builder()
            .previous_output(context.create_cell(buy_intent_script.clone(), Default::default()))
            .since(10000.pack())
            .build()
    };

    let tx = tx
        .as_advanced_builder()
        .input(cell_input_buy_intent)
        .output(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(def_lock_script.clone())
                .build(),
        )
        .output_data(Default::default())
        .witness(
            WitnessArgs::new_builder()
                .input_type(Some(buy_intent_data.as_bytes()).pack())
                .build()
                .as_bytes()
                .pack(),
        )
        .build();

    let tx = context.complete_tx(tx);
    // print_tx_info(&context, &tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_simple_selling() {
    let mut context = new_context();
    let def_lock_script: Script = build_always_suc_script(&mut context, &[]);
    let (spore_data, cluster_deps) = def_spore(&mut context);

    let tx = TransactionBuilder::default().build();

    // Account Book
    let account_book_data = def_account_book_data(&mut context);
    let account_book_data = account_book_data
        .as_builder()
        .cluster_id(get_cluster_id(&spore_data).pack())
        // .proof(smt_proof.pack())
        .build();
    let ab_cell_data = def_account_book_cell_data(&mut context)
        .as_builder()
        // .smt_root_hash(old_smt_hash.into())
        .member_count(35u32.pack())
        .build();
    let ab_cell_data_new = ab_cell_data
        .clone()
        .as_builder()
        // .smt_root_hash(new_smt_hash.into())
        .member_count(36u32.pack())
        .build();

    let tx = build_account_book(
        &mut context,
        tx,
        account_book_data.clone(),
        (ab_cell_data, ab_cell_data_new),
        (10000, 10000 + DATA_ASSET_AMOUNT),
    );
    let account_book_script_hash = get_account_script_hash(account_book_data);

    // DOB Selling
    let dob_selling_data = def_dob_selling_data(&mut context, &spore_data)
        .as_builder()
        .account_book_script_hash(account_book_script_hash.pack())
        .build();
    let cell_input_dob_selling = {
        let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
        let dob_selling_udt = build_xudt_cell(&mut context, dob_selling.clone());

        CellInput::new_builder()
            .previous_output(context.create_cell(
                dob_selling_udt.clone(),
                DATA_ASSET_AMOUNT.to_le_bytes().to_vec().into(),
            ))
            .build()
    };
    let tx = tx
        .as_advanced_builder()
        .input(cell_input_dob_selling)
        .output(
            CellOutput::new_builder()
                .lock(def_lock_script.clone())
                .capacity(1000.pack())
                .build(),
        )
        .output_data(Default::default())
        .witness(
            WitnessArgs::new_builder()
                .lock(Some(dob_selling_data.as_bytes()).pack())
                .build()
                .as_bytes()
                .pack(),
        )
        .build();

    // Buy Intent
    let buy_intent_data = def_buy_intent_data(&mut context, &dob_selling_data);
    let cell_input_buy_intent = {
        let buy_intent_script = build_buy_intent_cell(
            &mut context,
            1000,
            def_lock_script.clone(),
            &[
                account_book_script_hash,
                ckb_hash(buy_intent_data.as_slice()),
            ]
            .concat(),
        );

        CellInput::new_builder()
            .previous_output(context.create_cell(buy_intent_script.clone(), Default::default()))
            .build()
    };

    let tx = tx
        .as_advanced_builder()
        .input(cell_input_buy_intent)
        .witness(
            WitnessArgs::new_builder()
                .input_type(Some(buy_intent_data.as_bytes()).pack())
                .build()
                .as_bytes()
                .pack(),
        )
        .build();

    // Spore
    let tx = build_mint_spore(&mut context, tx, cluster_deps, spore_data);

    let tx = update_accountbook(&mut context, tx, DATA_ASSET_AMOUNT, (5000, 5000, 0, 0));
    let tx = context.complete_tx(tx);
    // print_tx_info(&context, &tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_simple_withdrawal_intent() {
    // Add Spore
    let mut context = new_context();
    let tx = TransactionBuilder::default().build();
    let def_lock_script = build_always_suc_script(&mut context, &[]);

    let (spore_data, _cluster_dep) = def_spore(&mut context);
    let tx = build_transfer_spore(&mut context, tx, &spore_data);
    let tx = context.complete_tx(tx);

    let withdrawal_intent_data = def_withdrawal_intent_data(&mut context)
        .as_builder()
        .spore_id(get_spore_id(&tx).pack())
        .cluster_id(get_cluster_id(&spore_data).pack())
        .build();
    let withdrawal_intent_script =
        build_withdrawal_intent_script(&mut context, &withdrawal_intent_data, [0u8; 32].into());
    // Inputs: CKB + Spore
    // Output: Withdrawal intent + Spore
    let tx = tx
        .as_advanced_builder()
        .input(build_input(build_out_point1(
            &mut context,
            def_lock_script.clone(),
        )))
        .output(
            CellOutput::new_builder()
                .lock(def_lock_script)
                .type_(withdrawal_intent_script.pack())
                .capacity(1000.pack())
                .build(),
        )
        .output_data(Default::default())
        .witness(
            WitnessArgs::new_builder()
                .output_type(Some(withdrawal_intent_data.as_bytes()).pack())
                .build()
                .as_slice()
                .pack(),
        )
        .build();

    let tx = context.complete_tx(tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_simple_withdrawal_suc() {
    let mut context = new_context();
    let tx = TransactionBuilder::default().build();
    let def_lock_script = build_always_suc_script(&mut context, &[0x11; 32]);
    let xudt_script = build_xudt_script(&mut context);

    let spore_id: Hash = [0x1B; 32].into();
    let cluster_id: Hash = [0x1A; 32].into();

    // Account Book
    let account_book_cell_data = def_account_book_cell_data(&mut context);
    let account_book_data = def_account_book_data(&mut context)
        .as_builder()
        .cluster_id(cluster_id.clone().into())
        .build();
    let account_book_script = build_account_book_script(&mut context, account_book_data.clone());
    let tx = {
        let input_proxy_script = build_input_proxy_script(
            &mut context,
            account_book_script
                .as_ref()
                .unwrap()
                .calc_script_hash()
                .into(),
        );

        let input_cell = {
            context.create_cell(
                CellOutput::new_builder()
                    .capacity(16.pack())
                    .lock(input_proxy_script.clone())
                    .type_(xudt_script.clone().pack())
                    .build(),
                10000u128.to_le_bytes().to_vec().into(),
            )
        };
        let output_cell = {
            CellOutput::new_builder()
                .capacity(16.pack())
                .lock(input_proxy_script.clone())
                .type_(xudt_script.pack())
                .build()
        };
        tx.as_advanced_builder()
            .input(build_input(input_cell))
            .output(output_cell)
            .output_data((10000u128 - 200).to_le_bytes().pack())
            .witness(Default::default())
            .build()
    };
    let tx = {
        // TODO Build SMT Info

        let input_cell = {
            context.create_cell(
                CellOutput::new_builder()
                    .capacity(1000.pack())
                    .lock(def_lock_script.clone())
                    .type_(account_book_script.clone().pack())
                    .build(),
                account_book_cell_data.as_bytes().into(),
            )
        };
        let output_cell = {
            CellOutput::new_builder()
                .capacity(1000.pack())
                .lock(def_lock_script.clone())
                .type_(account_book_script.pack())
                .build()
        };
        tx.as_advanced_builder()
            .input(build_input(input_cell))
            .output(output_cell)
            .output_data(account_book_cell_data.as_slice().pack())
            .witness(
                WitnessArgs::new_builder()
                    .output_type(Some(account_book_data.as_bytes()).pack())
                    .build()
                    .as_bytes()
                    .pack(),
            )
            .build()
    };

    // Withdrawal Intent
    let tx = {
        let withdrawal_intent_data = def_withdrawal_intent_data(&mut context)
            .as_builder()
            .spore_id(spore_id.into())
            .cluster_id(cluster_id.into())
            .build();
        let withdrawal_intent_script = build_withdrawal_intent_script(
            &mut context,
            &withdrawal_intent_data,
            account_book_script
                .as_ref()
                .unwrap()
                .calc_script_hash()
                .into(),
        );

        let input_cell = {
            context.create_cell(
                CellOutput::new_builder()
                    .capacity(16.pack())
                    .lock(def_lock_script.clone())
                    .type_(withdrawal_intent_script.pack())
                    .build(),
                Default::default(),
            )
        };
        let output_cell = {
            CellOutput::new_builder()
                .capacity(16.pack())
                .lock(def_lock_script.clone())
                .type_(xudt_script.pack())
                .build()
        };

        tx.as_advanced_builder()
            .input(build_input(input_cell))
            .output(output_cell)
            .output_data(200u128.to_le_bytes().pack())
            .witness(
                WitnessArgs::new_builder()
                    .input_type(Some(withdrawal_intent_data.as_bytes()).pack())
                    .build()
                    .as_bytes()
                    .pack(),
            )
            .build()
    };

    let tx = context.complete_tx(tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}
