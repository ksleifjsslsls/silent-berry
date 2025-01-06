use crate::*;
use ckb_testtool::ckb_types::{core::TransactionBuilder, packed::*};

mod test_data {
    pub const G_ASSET_AMOUNT: u128 = 200;
    pub const G_MIN_CAPACITY: u64 = 1000;

    use crate::*;
    use lazy_static::lazy_static;
    use molecule::prelude::{Builder, Entity};
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
        static ref G_BuyIntentCodeHash: [u8; 32] = {
            let mut context = new_context();
            let out_point = context.deploy_cell_by_name(NAME_BUY_INTENT);
            let (_, contract_data) = context.cells.get(&out_point).unwrap();
            CellOutput::calc_data_hash(contract_data)
                .as_slice()
                .try_into()
                .unwrap()
        };
        static ref G_DOBSellingCodeHash: [u8; 32] = {
            let mut context = new_context();
            let out_point = context.deploy_cell_by_name(NAME_DOB_SELLING);
            let (_, contract_data) = context.cells.get(&out_point).unwrap();
            CellOutput::calc_data_hash(contract_data)
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_AccountBookData: AccountBookData = {
            AccountBookData::new_builder()
                .asset_amount(G_ASSET_AMOUNT.to_le_bytes().into())
                .dob_selling_code_hash((*G_DOBSellingCodeHash).into())
                .buy_intent_code_hash((*G_BuyIntentCodeHash).into())
                .build()
        };
        pub static ref G_AccountBookScriptHash: [u8; 32] = {
            let hash = ckb_hash(G_AccountBookData.as_slice())
                .as_slice()
                .try_into()
                .unwrap();
            build_account_book_script(&mut new_context(), hash)
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_DobSellingData: DobSellingData = {
            DobSellingData::new_builder()
                .asset_amount(G_ASSET_AMOUNT.to_le_bytes().into())
                .account_book_script_hash((*G_AccountBookScriptHash).into())
                .build()
        };
        pub static ref G_DobSellingScriptHash: [u8; 32] = {
            build_dob_selling_script(&mut new_context(), &*G_DobSellingData)
                .calc_script_hash()
                .as_slice()
                .try_into()
                .unwrap()
        };
        pub static ref G_BuyIntentDataBuilder: BuyIntentDataBuilder = {
            BuyIntentData::new_builder()
                .dob_selling_script_hash((*G_DobSellingScriptHash).into())
                .account_book_script_hash((*G_AccountBookScriptHash).into())
                .xudt_script_hash((*G_XUdtScriptHash).into())
                .asset_amount(G_ASSET_AMOUNT.to_le_bytes().into())
                .min_capacity(G_MIN_CAPACITY.to_le_bytes().into())
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

    let dob_selling_data = test_data::G_DobSellingData.clone();
    let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
    let dob_selling_udt = build_xudt_cell(&mut context, 16, dob_selling.clone());

    let buy_intent_data = test_data::G_BuyIntentDataBuilder
        .clone()
        .change_script_hash([0u8; 32].into())
        .expire_since(1000u64.to_le_bytes().into())
        .owner_script_hash([0u8; 32].into())
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

    let outputs_data: Vec<Bytes> = vec![
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
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_simple_selling() {
    let mut context = new_context();
    let (cluster_out_point, _) =
        crate::spore::build_spore_contract_materials(&mut context, "cluster");
    let cluster = crate::spore::build_serialized_cluster_data("Spore Cluster", "Test Cluster");
    let (cluster_id, _, _, _, cluster_dep) =
        crate::spore::build_cluster_materials(&mut context, &cluster_out_point, cluster, 0, &[]);

    let mut tx = crate::spore::build_single_spore_mint_tx(
        &mut context,
        "Hello Spore!".as_bytes().to_vec(),
        "plain/text",
        None,
        Some(cluster_id),
    );
    tx = tx.as_advanced_builder().cell_dep(cluster_dep).build();

    let account_cell = build_account_book_cell(&mut context, test_data::G_AccountBookData.clone());

    let smt_hash = [0u8; 32];
    let member_count = 12u32;

    let dob_selling_data = test_data::G_DobSellingData.clone();
    let dob_selling = build_dob_selling_script(&mut context, &dob_selling_data);
    let dob_selling_udt = build_xudt_cell(&mut context, 16, dob_selling.clone());

    let def_lock_script: Script = build_always_suc_script(&mut context, &[]);
    let buy_intent_data = test_data::G_BuyIntentDataBuilder
        .clone()
        .change_script_hash([0u8; 32].into())
        .expire_since(1000u64.to_le_bytes().into())
        .owner_script_hash([0u8; 32].into())
        .build();

    let buy_intent_script = build_buy_intent_cell(
        &mut context,
        1000,
        def_lock_script.clone(),
        &[[0u8; 32], ckb_hash(buy_intent_data.as_slice())].concat(),
    );

    tx = tx
        .as_advanced_builder()
        .input(
            CellInput::new_builder()
                .previous_output(
                    context.create_cell(
                        account_cell.clone(),
                        vec![
                            10000u128.to_le_bytes().to_vec(),
                            smt_hash.to_vec(),
                            member_count.to_le_bytes().to_vec(),
                        ]
                        .concat()
                        .into(),
                    ),
                )
                .build(),
        )
        .input(
            CellInput::new_builder()
                .previous_output(context.create_cell(
                    dob_selling_udt.clone(),
                    test_data::G_ASSET_AMOUNT.to_le_bytes().to_vec().into(),
                ))
                .build(),
        )
        .input(
            CellInput::new_builder()
                .previous_output(context.create_cell(buy_intent_script.clone(), Default::default()))
                .build(),
        )
        .build();

    let change_cell = CellOutput::new_builder()
        .capacity(100.pack())
        .lock(build_always_suc_script(&mut context, &[2u8; 32]))
        .build();

    tx = tx
        .as_advanced_builder()
        .output(account_cell.clone())
        .output(change_cell)
        .output_data(
            vec![
                10200u128.to_le_bytes().to_vec(),
                smt_hash.to_vec(),
                member_count.to_le_bytes().to_vec(),
            ]
            .concat()
            .pack(),
        )
        .output_data(Default::default())
        .witness(Default::default())
        .witness(Default::default())
        .witness(
            WitnessArgs::new_builder()
                .input_type(Some(buy_intent_data.as_bytes()).pack())
                .build()
                .as_slice()
                .pack(),
        )
        .build();

    let tx = context.complete_tx(tx);
    verify_and_dump_failed_tx(&context, &tx, MAX_CYCLES).expect("pass");
}

#[test]
fn test_create_account_book() {}

#[test]
fn test_simple_withdrawal_intent() {}

#[test]
fn testsimple_withdrawal() {}
