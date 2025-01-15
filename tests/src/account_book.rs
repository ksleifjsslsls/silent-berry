use std::collections::HashMap;
use utils::{
    account_book_proof::{SMTTree, SmtKey, SmtValue, TotalAmounts, H256},
    Hash,
};

#[derive(Default)]
pub struct AccountBook {
    tree: SMTTree,
    bk_items: HashMap<[u8; 32], SmtValue>,
}
impl AccountBook {
    pub fn update(&mut self, key: SmtKey, value: SmtValue) {
        self.bk_items.insert(key.get_key().into(), value.clone());

        self.tree
            .update(key.get_key(), value)
            .expect("Update SMT Failed");
    }
    pub fn root_hash(&self) -> Hash {
        self.tree.root().as_slice().try_into().unwrap()
    }
    pub fn proof(&self, k: SmtKey) -> Vec<u8> {
        let ks: Vec<H256> = [
            SmtKey::TotalA,
            SmtKey::TotalB,
            SmtKey::TotalC,
            SmtKey::TotalD,
            k,
        ]
        .iter()
        .map(|k| k.get_key())
        .collect();

        self.tree
            .merkle_proof(ks.clone())
            .unwrap()
            .compile(ks)
            .unwrap()
            .0
    }
}

impl AccountBook {
    pub fn new_test() -> Self {
        let mut smt: AccountBook = Default::default();

        smt.update(SmtKey::TotalA, SmtValue::new(10000));
        smt.update(SmtKey::TotalB, SmtValue::new(20000));
        smt.update(SmtKey::TotalC, SmtValue::new(5000));
        smt.update(SmtKey::TotalD, SmtValue::new(0));

        let mut c: u8 = 0;
        fn new_hash(count: &mut u8) -> Hash {
            *count += 1;
            [*count; 32].into()
        }

        smt.update(SmtKey::Auther, SmtValue::new(122));
        smt.update(SmtKey::Platform, SmtValue::new(0));

        for _ in 0..100 {
            smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(0));
        }

        smt.update(SmtKey::Member(new_hash(&mut 2)), SmtValue::new(21313));
        smt.update(SmtKey::Member(new_hash(&mut 3)), SmtValue::new(4324));
        smt.update(SmtKey::Member(new_hash(&mut 4)), SmtValue::new(4444));
        smt.update(SmtKey::Member(new_hash(&mut 5)), SmtValue::new(555));

        smt
    }

    pub fn update_total(&mut self, total: TotalAmounts) {
        self.update(SmtKey::TotalA, SmtValue::new(total.a));
        self.update(SmtKey::TotalB, SmtValue::new(total.b));
        self.update(SmtKey::TotalC, SmtValue::new(total.c));
        self.update(SmtKey::TotalD, SmtValue::new(total.d));
    }

    pub fn get_item(&self, k: SmtKey) -> u128 {
        let k: Hash = k.get_key().into();
        let k: [u8; 32] = k.into();
        self.bk_items.get(&k).unwrap().clone().amount
    }

    pub fn get_total(&self) -> TotalAmounts {
        TotalAmounts {
            a: self.get_item(SmtKey::TotalA),
            b: self.get_item(SmtKey::TotalB),
            c: self.get_item(SmtKey::TotalC),
            d: self.get_item(SmtKey::TotalD),
        }
    }
}

#[test]
fn test_smt() {
    let mut smt = AccountBook::new_test();

    let mut c: u8 = 200;
    fn new_hash(count: &mut u8) -> Hash {
        *count += 1;
        [*count; 32].into()
    }

    smt.update(SmtKey::TotalA, SmtValue::new(80000));
    smt.update(SmtKey::Auther, SmtValue::new(2001));
    smt.update(SmtKey::Platform, SmtValue::new(0));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(123));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(4324));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(4444));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(555));
    smt.update(SmtKey::Member(new_hash(&mut c)), SmtValue::new(0));

    println!("c it: {}", c);
    let k = SmtKey::Member(new_hash(&mut c));

    let proof = smt.proof(k.clone());
    let root_hash_1 = smt.root_hash();
    let total_1 = smt.get_total();

    smt.update(k.clone(), SmtValue::new(200));
    let root_hash_2 = smt.root_hash();

    smt.update(SmtKey::TotalA, SmtValue::new(79800));
    let root_hash_3 = smt.root_hash();
    let total_3 = smt.get_total();

    let cproof = utils::account_book_proof::AccountBookProof::new(proof);

    assert!(cproof
        .verify(root_hash_1, total_1.clone(), (k.clone(), None))
        .unwrap());
    assert!(cproof
        .verify(root_hash_2, total_1, (k.clone(), Some(200)))
        .unwrap());
    assert!(cproof
        .verify(root_hash_3, total_3, (k.clone(), Some(200)))
        .unwrap());
}
