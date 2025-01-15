use ckb_std::log;
use spore_types::spore::SporeData;
use types::error::SilentBerryError as Error;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
pub enum Level {
    A = 1,
    B,
    C,
    D,
    Platform = 0x11,
    Auther = 0x12,
}
impl TryFrom<SporeData> for Level {
    type Error = Error;
    fn try_from(data: SporeData) -> Result<Self, Self::Error> {
        use alloc::string::String;
        let content = String::from_utf8(data.content().raw_data().to_vec()).map_err(|e| {
            log::error!("Parse Spore Content to string Failed, error: {:?}", e);
            Error::Spore
        })?;

        let v = content
            .chars()
            .rev()
            .find(|c| c.is_ascii_hexdigit())
            .ok_or_else(|| {
                log::error!("Spore Content format error, unable to find level");
                Error::Spore
            })?
            .to_digit(16)
            .ok_or_else(|| {
                log::error!("Spore Content format error, unable to find level");
                Error::Spore
            })? as u8;

        v.try_into()
    }
}
impl TryFrom<ckb_std::ckb_types::packed::Byte> for Level {
    type Error = Error;
    fn try_from(value: ckb_std::ckb_types::packed::Byte) -> Result<Self, Self::Error> {
        let value: u8 = value.into();
        value.try_into()
    }
}
impl TryFrom<u8> for Level {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::B),
            3 => Ok(Self::C),
            4 => Ok(Self::D),
            _ => {
                log::error!(
                    "Spore level error, the value should be 1~4, but the actual value is {}",
                    value
                );
                Err(Error::Spore)
            }
        }
    }
}
