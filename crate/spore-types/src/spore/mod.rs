pub mod action;
mod spore_v1;
mod spore_v2;

pub use spore_v1::*;
pub use spore_v2::*;

impl From<&[u8]> for spore_v1::Bytes {
    fn from(value: &[u8]) -> Self {
        use molecule::prelude::*;
        BytesBuilder::default()
            .set(value.iter().map(|f| (*f).into()).collect())
            .build()
    }
}

extern crate alloc;
use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct NativeNFTData {
    pub content_type: String,
    pub content: Vec<u8>,
    pub cluster_id: Option<Vec<u8>>,
}

use molecule::prelude::{Builder, Entity};
impl From<NativeNFTData> for SporeData {
    fn from(data: NativeNFTData) -> Self {
        let content: Bytes = data.content.as_slice().into();
        let content_type: Bytes = data.content_type.as_bytes().into();
        let cluster_id = match data.cluster_id {
            Some(cluster) => BytesOpt::new_builder()
                .set(Some(cluster.as_slice().into()))
                .build(),
            None => BytesOpt::default(),
        };
        SporeData::new_builder()
            .content(content)
            .content_type(content_type)
            .cluster_id(cluster_id)
            .build()
    }
}
