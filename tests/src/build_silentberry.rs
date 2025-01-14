// #[derive(Default)]
// struct TxWithdrawalIntentBuilder {
//     pub is_input: bool,
// }

// impl TxWithdrawalIntentBuilder {
//     fn _create_intent(mut self) -> Self {
//         self.is_input = false;
//         self
//     }
//     fn release_intent(mut self) -> Self {
//         self.is_input = true;
//         self
//     }

//     fn build(self, context: &mut Context, tx: TransactionView) -> TransactionView {
//         tx
//     }
// }

// #[derive(Clone)]
// struct TxXudtBuilder {
//     amount: u128,
//     owner_script_hash: Hash,
//     outpoint: OutPoint,
// }
// impl TxXudtBuilder {
//     fn new(a: u128, context: &mut Context) -> Self {
//         Self {
//             amount: a,
//             owner_script_hash: [0xAA; 32].into(),
//             outpoint: context.deploy_cell_by_name(XUDT_NAME),
//         }
//     }
//     fn build_script(&self, context: &mut Context) -> Script {
//         context
//             .build_script_with_hash_type(
//                 &self.outpoint,
//                 ckb_testtool::ckb_types::core::ScriptHashType::Data2,
//                 self.owner_script_hash.clone().into(),
//             )
//             .unwrap()
//     }
//     fn build_cell_data(&self) -> ckb_testtool::bytes::Bytes {
//         self.amount.to_le_bytes().to_vec().into()
//     }
//     fn get_script_hash(&self, context: &mut Context) -> Hash {
//         context
//             .build_script_with_hash_type(
//                 &self.outpoint,
//                 ScriptHashType::Data1,
//                 self.owner_script_hash.clone().into(),
//             )
//             .unwrap()
//             .calc_script_hash()
//             .into()
//     }
// }

// struct TxAccountBookBuilder {
//     is_selling: bool,
//     account_book_capacity: u64,
//     xudt_capacity: u64,
//     lock_script: Script,

//     cluster_id: Hash,
// }
// impl TxAccountBookBuilder {
//     fn new(context: &mut Context) -> Self {
//         Self {
//             is_selling: false,
//             account_book_capacity: 64,
//             xudt_capacity: 16,
//             lock_script: build_always_suc_script(context, &[0x10; 32]),

//             cluster_id: [0u8; 32].into(),
//         }
//     }

//     fn _selling(mut self) -> Self {
//         self.is_selling = true;
//         self
//     }
//     fn withdrawal(mut self) -> Self {
//         self.is_selling = false;
//         self
//     }

//     fn build(
//         self,
//         context: &mut Context,
//         tx: TransactionView,
//         input_xudt: TxXudtBuilder,
//         output_xudt: TxXudtBuilder,
//     ) -> TransactionView {
//         let input_len = tx.inputs().len();
//         assert_eq!(input_len, tx.outputs().len());
//         assert_eq!(input_len, tx.witnesses().len());

//         let account_book_data = AccountBookData::new_builder()
//             .dob_selling_code_hash((*DOBSellingCodeHash).pack())
//             .buy_intent_code_hash((*BuyIntentCodeHash).pack())
//             .withdrawal_intent_code_hash((*WithdrawalIntentCodeHash).pack())
//             .xudt_script_hash(input_xudt.get_script_hash(context).into())
//             .input_type_proxy_lock_code_hash((*InputTypeProxyLockCodeHash).pack())
//             .cluster_id(self.cluster_id.into())
//             .build();

//         let account_book_cell_data = AccountBookCellData::new_builder().build();

//         let account_book_script = build_account_book_script(context, account_book_data.clone());
//         let input_proxy_script = build_input_proxy_script(
//             context,
//             account_book_script
//                 .as_ref()
//                 .unwrap()
//                 .calc_script_hash()
//                 .into(),
//         );
//         let input_cell1 = {
//             let input_xudt_script = input_xudt.build_script(context);
//             context.create_cell(
//                 CellOutput::new_builder()
//                     .capacity(self.xudt_capacity.pack())
//                     .lock(input_proxy_script.clone())
//                     .type_(Some(input_xudt_script.clone()).pack())
//                     .build(),
//                 input_xudt.build_cell_data(),
//             )
//         };
//         let output_cell1 = {
//             let output_xudt_script = output_xudt.build_script(context);
//             CellOutput::new_builder()
//                 .capacity(self.xudt_capacity.pack())
//                 .lock(input_proxy_script.clone())
//                 .type_(Some(output_xudt_script).pack())
//                 .build()
//         };

//         let input_cell2 = context.create_cell(
//             CellOutput::new_builder()
//                 .capacity(self.xudt_capacity.pack())
//                 .lock(self.lock_script.clone())
//                 .type_(account_book_script.clone().pack())
//                 .build(),
//             input_xudt.build_cell_data(),
//         );
//         let output_cell2 = {
//             CellOutput::new_builder()
//                 .capacity(self.account_book_capacity.pack())
//                 .lock(self.lock_script.clone())
//                 .type_(account_book_script.clone().pack())
//                 .build()
//         };

//         tx.as_advanced_builder()
//             .input(build_input(input_cell1))
//             .output(output_cell1)
//             .output_data(output_xudt.build_cell_data().pack())
//             .witness(Default::default())
//             .input(build_input(input_cell2))
//             .output(output_cell2)
//             .output_data(account_book_cell_data.as_bytes().pack())
//             .witness(
//                 WitnessArgs::new_builder()
//                     .output_type(Some(account_book_data.as_bytes()).pack())
//                     .build()
//                     .as_bytes()
//                     .pack(),
//             )
//             .build()
//     }
// }
