use num_enum::IntoPrimitive;

extern crate alloc;

#[repr(u8)]
#[derive(Debug, IntoPrimitive, PartialEq)]
pub enum SilentBerryError {
    Unknow = 1,
    SysError,
    MolVerificationError,
    TypeConversion,
    TxStructure,
    VerifiedData,
    VerifiedDataLen,
    DobSellingScriptHash,
    AccountBookScriptHash,
    SporeDataHash,
    OnwerScriptHash,
    BuyIntentCodeHash,
    XudtNotFound,
    XudtIncorrect,
    PaymentAmount,
    CapacityError,
    ExpireSince,
}

impl From<ckb_std::error::SysError> for SilentBerryError {
    fn from(value: ckb_std::error::SysError) -> Self {
        ckb_std::log::warn!("CKB SysError ({:?}) to SilentBerryError", value);
        Self::SysError
    }
}

impl From<molecule::error::VerificationError> for SilentBerryError {
    fn from(value: molecule::error::VerificationError) -> Self {
        ckb_std::log::warn!("MolVerificationError ({:?}) to SilentBerryError", value);
        Self::MolVerificationError
    }
}
