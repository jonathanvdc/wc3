use bevy_wc3::Wc3Model;
use std::error::Error as StdError;
use wc3::model::conversion::{Error, ImportOptions};
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::{
    ConversionOptions, Document, DynamicDocument, NoExtensions, V1800,
};
use wc3::model::io::mdlx::import_owned;

/// A native fixture editor; production preparation always receives the checked graph.
pub struct NativeFixture {
    pub model: Document<'static, V1800>,
    source_version: u32,
}

impl NativeFixture {
    pub fn decode_mdl(text: &str) -> Result<Self, Box<dyn StdError>> {
        let source = DynamicDocument::<NoExtensions>::decode_mdl(text)?;
        let source_version = source.version();
        let model = source
            .into_convert::<V1800>(&ConversionOptions::strict())?
            .model
            .into_owned();
        Ok(Self {
            model,
            source_version,
        })
    }

    pub fn checked(&self) -> Result<Wc3Model, Error> {
        Ok(Wc3Model {
            source_version: self.source_version,
            model: import_owned(&self.model, &ImportOptions::default())?
                .value
                .into(),
        })
    }
}
