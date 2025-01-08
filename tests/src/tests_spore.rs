use ckb_testtool::{
    ckb_types::{
        bytes::Bytes,
        core::{ScriptHashType, TransactionBuilder, TransactionView},
        packed::*,
        prelude::*,
    },
    context::Context,
};
use molecule::prelude::*;
use spore_types::spore::action::SporeActionUnion;
use spore_types::spore::SporeData;

const MAX_CYCLES: u64 = 10_000_000;

fn build_always_success_script(context: &mut Context, args: Bytes) -> Script {
    let always_success_out_point = context.deploy_cell_by_name("always_success");
    context
        .build_script_with_hash_type(&always_success_out_point, ScriptHashType::Data1, args)
        .unwrap()
}

fn build_xudt_script(context: &mut Context, owner_hash: Byte32, ext_data: Bytes) -> Script {
    let xudt_out_point = context.deploy_cell_by_name("xudt_rce");
    let args = if ext_data.is_empty() {
        owner_hash.as_slice().to_vec()
    } else {
        vec![
            owner_hash.as_slice().to_vec(),
            0u64.to_le_bytes().to_vec(),
            ext_data.to_vec(),
        ]
        .concat()
    };
    context
        .build_script_with_hash_type(&xudt_out_point, ScriptHashType::Data2, args.into())
        .expect("create xudt script")
}

fn build_spore_type_id(input: &CellInput, index: usize) -> [u8; 32] {
    let mut hasher = ckb_testtool::ckb_hash::new_blake2b();
    hasher.update(input.as_slice());
    hasher.update(&index.to_le_bytes());
    let mut spore_type_id = [0; 32];
    hasher.finalize(&mut spore_type_id);
    spore_type_id
}

fn build_spore_script(
    context: &mut Context,
    input: &CellInput,
    index: usize,
) -> (Script, [u8; 32]) {
    let spore_out_point: OutPoint = context.deploy_cell_by_name("spore");
    let spore_type_id = build_spore_type_id(input, index);

    (
        context
            .build_script_with_hash_type(
                &spore_out_point,
                ScriptHashType::Data1,
                spore_type_id.to_vec().into(),
            )
            .unwrap(),
        spore_type_id,
    )
}

#[test]
fn test_xudt_tx() {
    let mut context = Context::default();
    context.add_contract_dir("../build/release");
    context.add_contract_dir("../build/3rd-bin");

    let lock_script1 = build_always_success_script(&mut context, [1u8; 32].to_vec().into());
    let lock_script2 = build_always_success_script(&mut context, [2u8; 32].to_vec().into());
    // let lock_script3 = build_always_success_script(&mut context, [3u8; 32].to_vec().into());
    let type_script = build_xudt_script(
        &mut context,
        Byte32::from_slice(&[0; 32]).unwrap(),
        Default::default(),
    );

    let inputs = vec![
        CellInput::new_builder()
            .previous_output(
                context.create_cell(
                    CellOutput::new_builder()
                        .capacity(1000u64.pack())
                        .lock(lock_script1.clone())
                        .type_(Some(type_script.clone()).pack())
                        .build(),
                    Bytes::from(
                        vec![(1000 as u128).to_le_bytes().to_vec(), vec![0x2; 22]].concat(),
                    ),
                ),
            )
            .build(),
        CellInput::new_builder()
            .previous_output(
                context.create_cell(
                    CellOutput::new_builder()
                        .capacity(1000u64.pack())
                        .lock(lock_script2.clone())
                        .type_(Some(type_script.clone()).pack())
                        .build(),
                    Bytes::from(
                        vec![(1000 as u128).to_le_bytes().to_vec(), vec![0x2; 22]].concat(),
                    ),
                ),
            )
            .build(),
    ];

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script2.clone())
            .type_(Some(type_script.clone()).pack())
            .build(),
        // CellOutput::new_builder()
        //     .capacity(1000u64.pack())
        //     .lock(lock_script3)
        //     .type_(Some(type_script.clone()).pack())
        //     .build(),
    ];

    let outputs_data = vec![
        Bytes::from((800 as u128).to_le_bytes().to_vec()),
        // Bytes::from(vec![(200 as u128).to_le_bytes().to_vec(), vec![0x7; 66]].concat()),
    ];

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();

    let tx = context.complete_tx(tx);
    let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
    println!("Cycles: {}", cycles);
}

pub fn complete_co_build_message_with_actions(
    tx: TransactionView,
    actions: &[(Option<Script>, SporeActionUnion)],
) -> TransactionView {
    use spore_types::{
        cobuild::{
            basic::{Action, ActionVec, Message, SighashAll},
            top_level::{WitnessLayout, WitnessLayoutUnion},
        },
        spore::action::SporeAction,
    };

    let action_value_vec: Vec<Action> = actions
        .to_owned()
        .into_iter()
        .map(|(script_hash, action)| {
            let script_hash = if let Some(script_hash) = script_hash {
                script_hash.calc_script_hash()
            } else {
                Byte32::default()
            };
            let spore_action = SporeAction::new_builder().set(action).build();

            Action::new_builder()
                .script_hash(script_hash)
                .data(spore_action.as_slice().pack())
                .build()
        })
        .collect();
    let action_vec = ActionVec::new_builder().set(action_value_vec).build();
    let message = Message::new_builder().actions(action_vec).build();
    let sighash_all = SighashAll::new_builder()
        .message(message)
        .seal(Bytes::new().pack())
        .build();
    let witness_layout = WitnessLayout::new_builder()
        .set(WitnessLayoutUnion::SighashAll(sighash_all))
        .build();

    tx.as_advanced_builder()
        .witnesses(vec![witness_layout.as_slice().pack(), Default::default()])
        .build()
}

fn script_to_address(script: Script) -> spore_types::spore::action::Address {
    let code_hash: [u8; 32] = script.code_hash().unpack();
    let hash_type = script.hash_type();
    let args: Bytes = script.args().raw_data();

    let args = spore_types::spore::action::Bytes::new_builder()
        .set(args.into_iter().map(Byte::new).collect())
        .build();

    let script = spore_types::spore::action::Script::new_builder()
        .code_hash(code_hash.into())
        .hash_type(hash_type)
        .args(args)
        .build();

    spore_types::spore::action::Address::new_builder()
        .set(spore_types::spore::action::AddressUnion::Script(script))
        .build()
}

pub fn build_mint_spore_action(
    context: &mut Context,
    nft_id: [u8; 32],
    content: &[u8],
) -> SporeActionUnion {
    use ckb_testtool::ckb_hash::blake2b_256;
    use spore_types::spore::action::MintSpore;

    let to = build_always_success_script(context, Default::default());
    let mint = MintSpore::new_builder()
        .spore_id(nft_id.into())
        .data_hash(blake2b_256(content).into())
        .to(script_to_address(to))
        .build();
    SporeActionUnion::MintSpore(mint)
}

#[test]
fn test_spore_mint() {
    let mut context = Context::default();
    context.add_contract_dir("../build/release");
    context.add_contract_dir("../build/3rd-bin");

    let lock_script = build_always_success_script(&mut context, Default::default());

    let input1 = {
        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(lock_script.clone())
                .build(),
            Bytes::new(),
        );
        CellInput::new_builder()
            .previous_output(input_out_point)
            .build()
    };

    let input2 = {
        // let ls = build_always_success_script(&mut context, vec![3u8; 5].into());
        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(lock_script.clone())
                .build(),
            Bytes::new(),
        );
        CellInput::new_builder()
            .previous_output(input_out_point)
            .build()
    };

    let (spore_script, spore_type_id) = build_spore_script(&mut context, &input1, 0);

    let output_spore = {
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .type_(Some(spore_script.clone()).pack())
            .build()
    };
    let output_data = {
        SporeData::new_builder()
            .content_type("text/html".as_bytes().into())
            .content("aaaabbbb".as_bytes().into())
            .build()
    };

    let output_t1 = {
        // let ls = build_always_success_script(&mut context, vec![4u8; 5].into());
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build()
    };

    let tx = TransactionBuilder::default()
        .inputs(vec![input1, input2])
        .outputs(vec![output_spore, output_t1])
        .outputs_data(vec![output_data.as_slice().pack(), Default::default()])
        .build();

    let tx = context.complete_tx(tx);
    let action = build_mint_spore_action(&mut context, spore_type_id, &output_data.as_slice());
    let tx = complete_co_build_message_with_actions(tx, &[(Some(spore_script), action)]);

    let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
    println!("Cycles: {}", cycles);
}

#[test]
fn test_spore_mint2() {
    let mut context = Context::default();
    context.add_contract_dir("../build/release");
    context.add_contract_dir("../build/3rd-bin");
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
    let tx: ckb_testtool::ckb_types::core::TransactionView = context.complete_tx(tx);

    println!(
        "Cluter Spore: {}",
        serde_json::to_string(&context.dump_tx(&tx).unwrap()).unwrap()
    );

    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("test spore mint from lock proxy");
}
