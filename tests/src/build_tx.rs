use ckb_testtool::{
    ckb_types::{
        bytes::Bytes,
        core::{ScriptHashType, TransactionView},
        packed::{CellDep, CellInput, CellOutput, Script, WitnessArgs},
        prelude::*,
    },
    context::Context,
};

use crate::*;

pub fn build_always_suc_script(context: &mut Context, args: &[u8]) -> Script {
    let out_point = context.deploy_cell_by_name(ALWAYS_SUC_NAME);

    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data1, args.to_vec().into())
        .expect("always success")
}

pub fn build_xudt_script(
    context: &mut Context,
    owner_script: [u8; 32],
    other_args: &[u8],
) -> Option<Script> {
    let out_point = context.deploy_cell_by_name(XUDT_NAME);
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
    let out_point = context.deploy_cell_by_name(DOB_SELLING_NAME);

    let dob_hash = ckb_hash(dob_selling_data.as_slice());
    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data2, dob_hash.to_vec().into())
        .expect("build dob-selling script")
}

pub fn build_buy_intent_script(context: &mut Context, args: &[u8]) -> Script {
    let out_point = context.deploy_cell_by_name(BUY_INTENT_NAME);

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

pub fn build_account_book_script(context: &mut Context, data: types::AccountBookData) -> Script {
    let args = ckb_hash(
        data.as_builder()
            .proof(Default::default())
            .build()
            .as_slice(),
    );
    let out_point = context.deploy_cell_by_name(ACCOUNT_BOOK_NAME);
    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data1, args.to_vec().into())
        .expect("build xudt")
}

pub fn build_account_book_cell(
    context: &mut Context,
    account_book_data: types::AccountBookData,
    smt_root_hash: [u8; 32],
    member_count: u32,
) -> (CellInput, CellOutput, Bytes) {
    let account_book_script = build_account_book_script(context, account_book_data);

    let xudt_script = build_xudt_script(context, [0u8; 32], &[]);

    let cell_output = CellOutput::new_builder()
        .capacity(16.pack())
        .lock(account_book_script)
        .type_(xudt_script.pack())
        .build();

    let cell_data: Bytes = vec![
        10000u128.to_le_bytes().to_vec(),
        smt_root_hash.to_vec(),
        member_count.to_le_bytes().to_vec(),
    ]
    .concat()
    .into();

    let cell_input = CellInput::new_builder()
        .previous_output(context.create_cell(cell_output.clone(), cell_data.clone()))
        .build();

    (cell_input, cell_output, cell_data)
}

pub fn build_account_book(
    context: &mut Context,
    tx: TransactionView,
    data: types::AccountBookData,
    udt: (u128, u128),
    smt_hash: ([u8; 32], [u8; 32]),
    member_count: (u32, u32),
) -> TransactionView {
    let account_book_script = build_account_book_script(context, data.clone());
    let xudt_script = build_xudt_script(context, [0u8; 32], &[]);

    let cell_output = CellOutput::new_builder()
        .capacity(1000.pack())
        .lock(account_book_script)
        .type_(xudt_script.pack())
        .build();

    let cell_input = CellInput::new_builder()
        .previous_output(
            context.create_cell(
                cell_output.clone(),
                vec![
                    udt.0.to_le_bytes().to_vec(),
                    smt_hash.0.to_vec(),
                    member_count.0.to_le_bytes().to_vec(),
                ]
                .concat()
                .into(),
            ),
        )
        .build();

    tx.as_advanced_builder()
        .input(cell_input)
        .output(cell_output)
        .output_data(
            vec![
                udt.1.to_le_bytes().to_vec(),
                smt_hash.1.to_vec(),
                member_count.1.to_le_bytes().to_vec(),
            ]
            .concat()
            .pack(),
        )
        .witness(
            WitnessArgs::new_builder()
                .lock(Some(data.as_bytes()).pack())
                .build()
                .as_bytes()
                .pack(),
        )
        .build()
}

pub fn build_cluster(context: &mut Context, cluster: (&str, &str)) -> ([u8; 32], CellDep) {
    let (cluster_out_point, _) =
        crate::spore::build_spore_contract_materials(context, CLUSTER_NAME);
    let cluster = crate::spore::build_serialized_cluster_data(cluster.0, cluster.1);
    let (cluster_id, _, _, _, cluster_dep) =
        crate::spore::build_cluster_materials(context, &cluster_out_point, cluster, 0, &[]);

    (cluster_id, cluster_dep)
}

pub fn build_spore(
    context: &mut Context,
    tx: TransactionView,
    cluster: ([u8; 32], CellDep),
    spore_data: spore_types::spore::SporeData,
) -> TransactionView {
    let (spore_out_point, spore_script_dep) =
        crate::spore::build_spore_contract_materials(context, "spore");

    let first_input = tx.inputs().get(0).unwrap();
    let output_index = tx.outputs().len();
    let type_id = crate::spore::build_type_id(&first_input, output_index);
    let spore_type =
        crate::spore::build_spore_type_script(context, &spore_out_point, type_id.to_vec().into());
    let spore_output =
        crate::spore::build_normal_output_cell_with_type(context, spore_type.clone());

    let tx = tx
        .as_advanced_builder()
        .output(spore_output)
        .output_data(spore_data.as_slice().pack())
        .cell_dep(spore_script_dep)
        .build();

    let action =
        crate::spore::co_build::build_mint_spore_action(context, type_id, spore_data.as_slice());
    let actions = vec![(spore_type, action)];

    let tx = crate::spore::co_build::complete_co_build_message_with_actions(tx, &actions);
    let tx = tx.as_advanced_builder().cell_dep(cluster.1).build();

    tx
}

pub fn get_account_script_hash(data: types::AccountBookData) -> [u8; 32] {
    build_account_book_script(&mut new_context(), data)
        .calc_script_hash()
        .as_slice()
        .try_into()
        .unwrap()
}
