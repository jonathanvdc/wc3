//! Decode Warcraft model fixtures into the same semantic vocabulary as other formats.
use std::error::Error;
use std::str::from_utf8;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::{DynamicDocument, NoExtensions};
use wc3::model::io::mdlx;
use wc3::model::Model;

pub fn decode(bytes: &[u8]) -> Result<Model<'static>, Box<dyn Error>> {
    let document = if bytes.starts_with(b"MDLX") {
        DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)?
    } else {
        DynamicDocument::<NoExtensions>::decode_mdl(from_utf8(bytes)?)?
    };
    Ok(
        mdlx::import_dynamic_owned(&document, &ImportOptions::default())?
            .value
            .widen(),
    )
}
