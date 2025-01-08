use crate::{build_tx::*, *};
use ckb_testtool::ckb_types::{
    core::TransactionBuilder,
    packed::{CellInput, CellOutput, Script, WitnessArgs},
    prelude::{Builder, Entity, Pack, PackVec},
};
use utils::MemberInfo;

mod test_data {
    pub const G_ASSET_AMOUNT: u128 = 200;
    pub const G_MIN_CAPACITY: u64 = 1000;

    use crate::{build_tx::*, *};
    use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
    use lazy_static::lazy_static;
    use types::*;

    lazy_static! {
        pub static ref G_DefaultLockScriptHash: [u8; 32] = {
            build_always_suc_script(&mut new_context(), &[0u8; 16])
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_XUdtLockScriptHash: [u8; 32] = {
            build_always_suc_script(&mut new_context(), &[1u8; 16])
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_UserLockScriptHash: [u8; 32] = {
            build_always_suc_script(&mut new_context(), &[2u8; 16])
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_XUdtScriptHash: [u8; 32] = {
            build_xudt_script(&mut new_context(), [0u8; 32], &[])
                .unwrap()
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_AccountBookDataBuilder: AccountBookDataBuilder = {
            AccountBookDataBuilder::default()
                .dob_selling_code_hash((*DOBSellingCodeHash).pack())
                .buy_intent_code_hash((*BuyIntentCodeHash).pack())
                .withdrawal_intent_code_hash((*WithdrawalIntentCodeHash).pack())
                .auther_id([1u8; 32].pack())
                .platform_id([2u8; 32].pack())
                .cluster_id([3u8; 32].pack())
                .asset_amount(G_ASSET_AMOUNT.pack())
                .a_num(3u32.pack())
                .b_num(17u32.pack())
                .c_num(25u32.pack())
                .a_profit(
                    AProfit::new_builder()
                        .set([20u8.into(), 80u8.into()])
                        .build(),
                )
                .b_profit(
                    BProfit::new_builder()
                        .set([20u8.into(), 20u8.into(), 60u8.into()])
                        .build(),
                )
                .c_profit(
                    CProfit::new_builder()
                        .set([20u8.into(), 20u8.into(), 36u8.into(), 24u8.into()])
                        .build(),
                )
                .d_profit(
                    DProfit::new_builder()
                        .set([
                            20u8.into(),
                            20u8.into(),
                            20u8.into(),
                            20u8.into(),
                            20u8.into(),
                        ])
                        .build(),
                )
        };
        pub static ref G_DobSellingDataBuilder: DobSellingDataBuilder =
            DobSellingData::new_builder();
        pub static ref G_BuyIntentDataBuilder: BuyIntentDataBuilder = {
            BuyIntentDataBuilder::default()
                .xudt_script_hash((*G_XUdtScriptHash).pack())
                .asset_amount(G_ASSET_AMOUNT.pack())
                .min_capacity(G_MIN_CAPACITY.pack())
        };
    }
}

#[test]
fn test_simple_buy_intent() {
    let mut context = new_context();

    let udt_lock_script = build_always_suc_script(&mut context, &[1u8; 16]);
    let udt_cell = build_xudt_cell(&mut context, 16, udt_lock_script.clone());

    let def_lock_script = build_always_suc_script(&mut context, &[]);
    let def_output = CellOutput::new_builder()
        .capacity(1000u64.pack())
        .lock(def_lock_script.clone())
        .build();

    let inputs = vec![
        CellInput::new_builder()
            .previous_output(
                context.create_cell(udt_cell.clone(), 1000u128.to_le_bytes().to_vec().into()),
            )
            .build(),
        CellInput::new_builder()
            .previous_output(context.create_cell(def_output.clone(), Default::default()))
            .build(),
    ];
    let spore_data = crate::spore::build_serialized_spore_data(
        "{\"dna\":\"4000000000002\"}".as_bytes().to_vec(),
        "dob/1",
        Some(vec![0u8; 32]),
    );
    let dob_selling_data = test_data::G_DobSellingDataBuilder
        .clone()
        .spore_data_hash(ckb_hash(spore_data.as_slice()).pack())
        .build();
    let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
    let dob_selling_udt = build_xudt_cell(&mut context, 16, dob_selling.clone());

    let buy_intent_data = test_data::G_BuyIntentDataBuilder
        .clone()
        .dob_selling_script_hash(dob_selling.calc_script_hash())
        .change_script_hash([0u8; 32].pack())
        .expire_since(1000u64.pack())
        .owner_script_hash([0u8; 32].pack())
        .build();

    let buy_intent_script = build_buy_intent_cell(
        &mut context,
        1000,
        def_lock_script.clone(),
        &[[0u8; 32], ckb_hash(buy_intent_data.as_slice())].concat(),
    );

    let outputs = vec![
        udt_cell.clone(),
        dob_selling_udt.clone(),
        buy_intent_script.clone(),
    ];

    let outputs_data: Vec<ckb_testtool::ckb_types::packed::Bytes> = vec![
        800u128.to_le_bytes().to_vec().pack(),
        test_data::G_ASSET_AMOUNT.to_le_bytes().to_vec().pack(),
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
fn test_simple_selling() {
    let mut context = new_context();
    let def_lock_script: Script = build_always_suc_script(&mut context, &[]);
    let (cluster_id, cluster_deps) = build_cluster(&mut context, ("Spore Cluster", "Test Cluster"));
    let spore_data = crate::spore::build_serialized_spore_data(
        "{\"dna\":\"4000000000002\"}".as_bytes().to_vec(),
        "dob/1",
        Some(cluster_id.to_vec()),
    );

    let tx = TransactionBuilder::default().build();

    // generate smt tree
    let mut smt = new_smt_tree();
    let old_smt_hash: [u8; 32] = smt.root();

    let sport_id = [0u8; 32];
    let member_info = MemberInfo {
        spore_id: sport_id,
        withdrawn_amount: 100,
        member_type: utils::MemberType::Silver,
    };
    smt.update(member_info.clone());
    let new_smt_hash = smt.root();
    let smt_proof = smt.proof(Some(member_info.get_key()));

    // Account Book
    let account_book_data = test_data::G_AccountBookDataBuilder.clone();
    let account_book_data = account_book_data
        .cluster_id(cluster_id.pack())
        .proof(smt_proof.pack())
        .build();
    let tx = build_account_book(
        &mut context,
        tx,
        account_book_data.clone(),
        (10000, 10200),
        (old_smt_hash, new_smt_hash),
        (123, 124),
    );
    let account_book_script_hash = get_account_script_hash(account_book_data);

    // DOB Selling
    let dob_selling_data = test_data::G_DobSellingDataBuilder
        .clone()
        .spore_data_hash(ckb_hash(spore_data.as_slice()).pack())
        .account_book_script_hash(account_book_script_hash.pack())
        .build();
    let cell_input_dob_selling = {
        let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
        let dob_selling_udt = build_xudt_cell(&mut context, 16, dob_selling.clone());
        CellInput::new_builder()
            .previous_output(context.create_cell(
                dob_selling_udt.clone(),
                test_data::G_ASSET_AMOUNT.to_le_bytes().to_vec().into(),
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
    let buy_intent_data = test_data::G_BuyIntentDataBuilder
        .clone()
        .change_script_hash([0u8; 32].pack())
        .expire_since(1000u64.pack())
        .owner_script_hash([0u8; 32].pack())
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
    let tx = build_spore(&mut context, tx, cluster_deps, spore_data);

    let tx = context.complete_tx(tx);
    print_tx_info(&context, &tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_simple_withdrawal_intent() {}

#[test]
fn testsimple_withdrawal() {}
