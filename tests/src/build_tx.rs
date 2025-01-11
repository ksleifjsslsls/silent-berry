use ckb_testtool::{
    ckb_types::{
        core::{ScriptHashType, TransactionView},
        packed::{Bytes, CellDep, CellInput, CellOutput, OutPoint, Script, WitnessArgs},
        prelude::*,
    },
    context::Context,
};
use types::{AccountBookCellData, AccountBookData, DobSellingData};
use utils::Hash;

use crate::*;

pub const XUDT_OWNER_SCRIPT_HASH: [u8; 32] = [0xAA; 32];

pub fn get_script_hash(s: &Script) -> [u8; 32] {
    s.calc_script_hash().as_slice().try_into().unwrap()
}
pub fn get_opt_script_hash(s: &Option<Script>) -> [u8; 32] {
    s.as_ref()
        .unwrap()
        .calc_script_hash()
        .as_slice()
        .try_into()
        .unwrap()
}

pub fn build_input(outpoint: OutPoint) -> CellInput {
    CellInput::new_builder().previous_output(outpoint).build()
}

pub fn build_out_point1(context: &mut Context, lock_script: Script) -> OutPoint {
    context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script)
            .build(),
        Default::default(),
    )
}
pub fn build_out_point2(
    context: &mut Context,
    lock_script: Script,
    type_script: Option<Script>,
) -> OutPoint {
    context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script)
            .type_(type_script.pack())
            .build(),
        Default::default(),
    )
}
pub fn build_out_point3(
    context: &mut Context,
    lock_script: Script,
    type_script: Option<Script>,
    data: ckb_testtool::bytes::Bytes,
) -> OutPoint {
    context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script)
            .type_(type_script.pack())
            .build(),
        data,
    )
}

pub fn build_always_suc_script(context: &mut Context, args: &[u8]) -> Script {
    let out_point = context.deploy_cell_by_name(ALWAYS_SUC_NAME);

    context
        .build_script_with_hash_type(&out_point, ScriptHashType::Data1, args.to_vec().into())
        .expect("always success")
}
pub fn build_user1_script(context: &mut Context) -> Script {
    build_always_suc_script(context, &[1u8; 32])
}
pub fn build_user2_script(context: &mut Context) -> Script {
    build_always_suc_script(context, &[2u8; 32])
}

pub fn build_xudt_script(context: &mut Context) -> Option<Script> {
    let out_point = context.deploy_cell_by_name(XUDT_NAME);
    Some(
        context
            .build_script_with_hash_type(
                &out_point,
                ScriptHashType::Data1,
                [XUDT_OWNER_SCRIPT_HASH].concat().into(),
            )
            .expect("build xudt"),
    )
}

pub fn build_xudt_cell(context: &mut Context, lock_script: Script) -> CellOutput {
    let xudt_script: Option<Script> = build_xudt_script(context);

    CellOutput::new_builder()
        .capacity(16u64.pack())
        .lock(lock_script)
        .type_(xudt_script.pack())
        .build()
}

fn build_input_proxy_script(context: &mut Context, type_script_hash: [u8; 32]) -> Script {
    let out_point = context.deploy_cell_by_name(INPUT_TYPE_PROXY_LOCK_NAME);
    context
        .build_script_with_hash_type(
            &out_point,
            ScriptHashType::Data1,
            type_script_hash.to_vec().into(),
        )
        .expect("build input-proxy-lock")
}

pub fn build_dob_selling_script(
    context: &mut Context,
    dob_selling_data: &DobSellingData,
) -> Script {
    let out_point: types::blockchain::OutPoint = context.deploy_cell_by_name(DOB_SELLING_NAME);

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

pub fn build_account_book_script(
    context: &mut Context,
    data: types::AccountBookData,
) -> Option<Script> {
    let args = ckb_hash(
        data.as_builder()
            .proof(Default::default())
            .build()
            .as_slice(),
    );
    let out_point = context.deploy_cell_by_name(ACCOUNT_BOOK_NAME);
    Some(
        context
            .build_script_with_hash_type(&out_point, ScriptHashType::Data2, args.to_vec().into())
            .expect("build xudt"),
    )
}

pub fn build_account_book(
    context: &mut Context,
    tx: TransactionView,
    data: AccountBookData,
    cell_data: (AccountBookCellData, AccountBookCellData),
    udt: (u128, u128),
) -> TransactionView {
    let account_book_script = build_account_book_script(context, data.clone());
    let xudt_script = build_xudt_script(context);
    let account_book_lock_script = build_always_suc_script(context, &[]);
    let input_proxy_script = build_input_proxy_script(
        context,
        account_book_script
            .as_ref()
            .unwrap()
            .calc_script_hash()
            .unpack(),
    );

    let cell_output = CellOutput::new_builder()
        .capacity(16.pack())
        .lock(input_proxy_script)
        .type_(xudt_script.pack())
        .build();
    let cell_output2 = CellOutput::new_builder()
        .capacity(16.pack())
        .lock(account_book_lock_script)
        .type_(account_book_script.pack())
        .build();

    let cell_input = CellInput::new_builder()
        .previous_output(
            context.create_cell(cell_output.clone(), udt.0.to_le_bytes().to_vec().into()),
        )
        .build();
    let cell_input2 = CellInput::new_builder()
        .previous_output(context.create_cell(cell_output2.clone(), cell_data.0.as_bytes()))
        .build();

    tx.as_advanced_builder()
        .input(cell_input)
        .input(cell_input2)
        .output(cell_output)
        .output(cell_output2)
        .output_data(udt.1.to_le_bytes().to_vec().pack())
        .output_data(cell_data.1.as_bytes().pack())
        .witness(Default::default())
        .witness(
            WitnessArgs::new_builder()
                .output_type(Some(data.as_bytes()).pack())
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
    cluster_deps: CellDep,
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
    tx.as_advanced_builder().cell_dep(cluster_deps).build()
}

pub fn get_spore_id(tx: &TransactionView) -> [u8; 32] {
    let spore_output = tx.outputs().into_iter().find(|f| {
        if let Some(t) = f.type_().to_opt() {
            t.code_hash().as_slice() == *SporeCodeHash
        } else {
            false
        }
    });

    spore_output
        .unwrap()
        .type_()
        .to_opt()
        .unwrap()
        .args()
        .raw_data()
        .to_vec()
        .try_into()
        .unwrap()
}

pub fn get_account_script_hash(data: types::AccountBookData) -> [u8; 32] {
    build_account_book_script(&mut new_context(), data)
        .as_ref()
        .unwrap()
        .calc_script_hash()
        .as_slice()
        .try_into()
        .unwrap()
}

pub fn update_accountbook(
    context: &mut Context,
    tx: TransactionView,
    smt_hash: (Hash, Hash),
    proof: Vec<u8>,
) -> TransactionView {
    let input_pos = tx
        .inputs()
        .into_iter()
        .position(|f| {
            if let Some((output, _)) = context.get_cell(&f.previous_output()) {
                if let Some(type_script) = output.type_().to_opt() {
                    let type_script_code_hash: Hash = type_script.code_hash().into();
                    type_script_code_hash == *AccountBookCodeHash
                } else {
                    false
                }
            } else {
                false
            }
        })
        .unwrap();
    let outpoint = tx.inputs().get(input_pos).unwrap().previous_output();
    let (_, cell_data) = context.cells.get_mut(&outpoint).unwrap();

    let abcd = AccountBookCellData::new_unchecked(cell_data.clone())
        .as_builder()
        .smt_root_hash(smt_hash.0.into())
        .build();
    *cell_data = abcd.as_slice().to_vec().into();

    let mut outputs_data: Vec<Bytes> = tx.outputs_data().into_iter().collect();
    let cell_data =
        AccountBookCellData::new_unchecked(outputs_data.get(input_pos).unwrap().clone().unpack())
            .as_builder()
            .smt_root_hash(smt_hash.1.into())
            .build();
    *outputs_data.get_mut(input_pos).unwrap() = cell_data.as_slice().to_vec().pack();
    let tx = tx
        .as_advanced_builder()
        .set_outputs_data(outputs_data)
        .build();

    let mut witnesses: Vec<Bytes> = tx.witnesses().into_iter().collect();
    let witness = WitnessArgs::new_unchecked(witnesses.get(input_pos).unwrap().unpack());
    let abd = AccountBookData::new_unchecked(witness.output_type().to_opt().unwrap().unpack())
        .as_builder()
        .proof(proof.pack())
        .build();
    let witness = witness
        .as_builder()
        .output_type(Some(abd.as_bytes()).pack())
        .build();
    *witnesses.get_mut(input_pos).unwrap() = witness.as_bytes().pack();

    tx.as_advanced_builder().set_witnesses(witnesses).build()
}
