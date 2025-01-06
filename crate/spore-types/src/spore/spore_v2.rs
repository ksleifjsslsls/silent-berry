// Generated by Molecule 0.8.0
#![allow(
    clippy::needless_lifetimes,
    clippy::needless_borrow,
    clippy::derivable_impls,
    clippy::clone_on_copy,
    clippy::write_literal,
    clippy::if_same_then_else,
    clippy::useless_conversion,
    clippy::redundant_slicing
)]

use super::spore_v1::*;
use molecule::prelude::*;
#[derive(Clone)]
pub struct ClusterDataV2(molecule::bytes::Bytes);
impl ::core::fmt::LowerHex for ClusterDataV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl ::core::fmt::Debug for ClusterDataV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl ::core::fmt::Display for ClusterDataV2 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "name", self.name())?;
        write!(f, ", {}: {}", "description", self.description())?;
        write!(f, ", {}: {}", "mutant_id", self.mutant_id())?;
        let extra_count = self.count_extra_fields();
        if extra_count != 0 {
            write!(f, ", .. ({} fields)", extra_count)?;
        }
        write!(f, " }}")
    }
}
impl ::core::default::Default for ClusterDataV2 {
    fn default() -> Self {
        let v = molecule::bytes::Bytes::from_static(&Self::DEFAULT_VALUE);
        ClusterDataV2::new_unchecked(v)
    }
}
impl ClusterDataV2 {
    const DEFAULT_VALUE: [u8; 24] = [
        24, 0, 0, 0, 16, 0, 0, 0, 20, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    pub const FIELD_COUNT: usize = 3;
    pub fn total_size(&self) -> usize {
        molecule::unpack_number(self.as_slice()) as usize
    }
    pub fn field_count(&self) -> usize {
        if self.total_size() == molecule::NUMBER_SIZE {
            0
        } else {
            (molecule::unpack_number(&self.as_slice()[molecule::NUMBER_SIZE..]) as usize / 4) - 1
        }
    }
    pub fn count_extra_fields(&self) -> usize {
        self.field_count() - Self::FIELD_COUNT
    }
    pub fn has_extra_fields(&self) -> bool {
        Self::FIELD_COUNT != self.field_count()
    }
    pub fn name(&self) -> Bytes {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[4..]) as usize;
        let end = molecule::unpack_number(&slice[8..]) as usize;
        Bytes::new_unchecked(self.0.slice(start..end))
    }
    pub fn description(&self) -> Bytes {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[8..]) as usize;
        let end = molecule::unpack_number(&slice[12..]) as usize;
        Bytes::new_unchecked(self.0.slice(start..end))
    }
    pub fn mutant_id(&self) -> BytesOpt {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[12..]) as usize;
        if self.has_extra_fields() {
            let end = molecule::unpack_number(&slice[16..]) as usize;
            BytesOpt::new_unchecked(self.0.slice(start..end))
        } else {
            BytesOpt::new_unchecked(self.0.slice(start..))
        }
    }
    pub fn as_reader<'r>(&'r self) -> ClusterDataV2Reader<'r> {
        ClusterDataV2Reader::new_unchecked(self.as_slice())
    }
}
impl molecule::prelude::Entity for ClusterDataV2 {
    type Builder = ClusterDataV2Builder;
    const NAME: &'static str = "ClusterDataV2";
    fn new_unchecked(data: molecule::bytes::Bytes) -> Self {
        ClusterDataV2(data)
    }
    fn as_bytes(&self) -> molecule::bytes::Bytes {
        self.0.clone()
    }
    fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
    fn from_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        ClusterDataV2Reader::from_slice(slice).map(|reader| reader.to_entity())
    }
    fn from_compatible_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        ClusterDataV2Reader::from_compatible_slice(slice).map(|reader| reader.to_entity())
    }
    fn new_builder() -> Self::Builder {
        ::core::default::Default::default()
    }
    fn as_builder(self) -> Self::Builder {
        Self::new_builder()
            .name(self.name())
            .description(self.description())
            .mutant_id(self.mutant_id())
    }
}
#[derive(Clone, Copy)]
pub struct ClusterDataV2Reader<'r>(&'r [u8]);
impl<'r> ::core::fmt::LowerHex for ClusterDataV2Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl<'r> ::core::fmt::Debug for ClusterDataV2Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl<'r> ::core::fmt::Display for ClusterDataV2Reader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "name", self.name())?;
        write!(f, ", {}: {}", "description", self.description())?;
        write!(f, ", {}: {}", "mutant_id", self.mutant_id())?;
        let extra_count = self.count_extra_fields();
        if extra_count != 0 {
            write!(f, ", .. ({} fields)", extra_count)?;
        }
        write!(f, " }}")
    }
}
impl<'r> ClusterDataV2Reader<'r> {
    pub const FIELD_COUNT: usize = 3;
    pub fn total_size(&self) -> usize {
        molecule::unpack_number(self.as_slice()) as usize
    }
    pub fn field_count(&self) -> usize {
        if self.total_size() == molecule::NUMBER_SIZE {
            0
        } else {
            (molecule::unpack_number(&self.as_slice()[molecule::NUMBER_SIZE..]) as usize / 4) - 1
        }
    }
    pub fn count_extra_fields(&self) -> usize {
        self.field_count() - Self::FIELD_COUNT
    }
    pub fn has_extra_fields(&self) -> bool {
        Self::FIELD_COUNT != self.field_count()
    }
    pub fn name(&self) -> BytesReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[4..]) as usize;
        let end = molecule::unpack_number(&slice[8..]) as usize;
        BytesReader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn description(&self) -> BytesReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[8..]) as usize;
        let end = molecule::unpack_number(&slice[12..]) as usize;
        BytesReader::new_unchecked(&self.as_slice()[start..end])
    }
    pub fn mutant_id(&self) -> BytesOptReader<'r> {
        let slice = self.as_slice();
        let start = molecule::unpack_number(&slice[12..]) as usize;
        if self.has_extra_fields() {
            let end = molecule::unpack_number(&slice[16..]) as usize;
            BytesOptReader::new_unchecked(&self.as_slice()[start..end])
        } else {
            BytesOptReader::new_unchecked(&self.as_slice()[start..])
        }
    }
}
impl<'r> molecule::prelude::Reader<'r> for ClusterDataV2Reader<'r> {
    type Entity = ClusterDataV2;
    const NAME: &'static str = "ClusterDataV2Reader";
    fn to_entity(&self) -> Self::Entity {
        Self::Entity::new_unchecked(self.as_slice().to_owned().into())
    }
    fn new_unchecked(slice: &'r [u8]) -> Self {
        ClusterDataV2Reader(slice)
    }
    fn as_slice(&self) -> &'r [u8] {
        self.0
    }
    fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
        use molecule::verification_error as ve;
        let slice_len = slice.len();
        if slice_len < molecule::NUMBER_SIZE {
            return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
        }
        let total_size = molecule::unpack_number(slice) as usize;
        if slice_len != total_size {
            return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
        }
        if slice_len < molecule::NUMBER_SIZE * 2 {
            return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE * 2, slice_len);
        }
        let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
        if offset_first % molecule::NUMBER_SIZE != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
            return ve!(Self, OffsetsNotMatch);
        }
        if slice_len < offset_first {
            return ve!(Self, HeaderIsBroken, offset_first, slice_len);
        }
        let field_count = offset_first / molecule::NUMBER_SIZE - 1;
        if field_count < Self::FIELD_COUNT {
            return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
        } else if !compatible && field_count > Self::FIELD_COUNT {
            return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
        };
        let mut offsets: Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
            .chunks_exact(molecule::NUMBER_SIZE)
            .map(|x| molecule::unpack_number(x) as usize)
            .collect();
        offsets.push(total_size);
        if offsets.windows(2).any(|i| i[0] > i[1]) {
            return ve!(Self, OffsetsNotMatch);
        }
        BytesReader::verify(&slice[offsets[0]..offsets[1]], compatible)?;
        BytesReader::verify(&slice[offsets[1]..offsets[2]], compatible)?;
        BytesOptReader::verify(&slice[offsets[2]..offsets[3]], compatible)?;
        Ok(())
    }
}
#[derive(Clone, Debug, Default)]
pub struct ClusterDataV2Builder {
    pub(crate) name: Bytes,
    pub(crate) description: Bytes,
    pub(crate) mutant_id: BytesOpt,
}
impl ClusterDataV2Builder {
    pub const FIELD_COUNT: usize = 3;
    pub fn name(mut self, v: Bytes) -> Self {
        self.name = v;
        self
    }
    pub fn description(mut self, v: Bytes) -> Self {
        self.description = v;
        self
    }
    pub fn mutant_id(mut self, v: BytesOpt) -> Self {
        self.mutant_id = v;
        self
    }
}
impl molecule::prelude::Builder for ClusterDataV2Builder {
    type Entity = ClusterDataV2;
    const NAME: &'static str = "ClusterDataV2Builder";
    fn expected_length(&self) -> usize {
        molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1)
            + self.name.as_slice().len()
            + self.description.as_slice().len()
            + self.mutant_id.as_slice().len()
    }
    fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
        let mut total_size = molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1);
        let mut offsets = Vec::with_capacity(Self::FIELD_COUNT);
        offsets.push(total_size);
        total_size += self.name.as_slice().len();
        offsets.push(total_size);
        total_size += self.description.as_slice().len();
        offsets.push(total_size);
        total_size += self.mutant_id.as_slice().len();
        writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
        for offset in offsets.into_iter() {
            writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
        }
        writer.write_all(self.name.as_slice())?;
        writer.write_all(self.description.as_slice())?;
        writer.write_all(self.mutant_id.as_slice())?;
        Ok(())
    }
    fn build(&self) -> Self::Entity {
        let mut inner = Vec::with_capacity(self.expected_length());
        self.write(&mut inner)
            .unwrap_or_else(|_| panic!("{} build should be ok", Self::NAME));
        ClusterDataV2::new_unchecked(inner.into())
    }
}
