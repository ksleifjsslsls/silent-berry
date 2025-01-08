use num_enum::IntoPrimitive;

extern crate alloc;

#[repr(u8)]
#[derive(Debug, IntoPrimitive)]
pub enum SilentBerryError {
    Unknow = 1,
    TxStructure,
    VerifiedData,
    VerifiedDataLen,
    SysError,
    MolVerificationError,
    DobSellingScriptHash,
    AccountBookScriptHash,
    SporeDataHash,
    XudtNotFound,
    XudtIncorrect,
    PaymentAmount,
    CapacityError,
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
