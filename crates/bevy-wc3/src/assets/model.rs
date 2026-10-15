use crate::preparation::shared::Shared;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::from_utf8;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::DynamicDocument;
use wc3::model::formats::mdlx::NoExtensions;
#[cfg(test)]
use wc3::model::formats::mdlx::{ConversionOptions, Document, V1800};
use wc3::model::io::mdlx as semantic_mdlx;
use wc3::model::{profiles::Mdlx, Model};

/// A model decoding, strict conversion, or geometry preparation failure.
#[derive(Debug)]
pub struct ModelError(pub(crate) String);

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Error for ModelError {}

/// A normalized model whose MDX buffers may borrow the encoded input.
/// Consume it with `compile_document` before releasing that input.
#[cfg(test)]
pub(crate) struct DecodedDocument<'data> {
    /// Normalized model, borrowing compatible MDX arrays from the input.
    pub model: Document<'data, V1800>,
}

#[cfg(test)]
impl<'data> DecodedDocument<'data> {
    /// Detect MDX by its header, otherwise decode UTF-8 MDL. Strict conversion
    /// preserves compatible borrowed arrays and moves owned conversion results.
    pub fn decode(bytes: &'data [u8]) -> Result<Self, ModelError> {
        let source = if bytes.starts_with(b"MDLX") {
            DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
                .map_err(|error| ModelError(error.to_string()))?
        } else {
            let text = from_utf8(bytes)
                .map_err(|error| ModelError(format!("MDL source is not UTF-8: {error}")))?;
            DynamicDocument::<NoExtensions>::decode_mdl(text)
                .map_err(|error| ModelError(error.to_string()))?
        };
        let model = source
            .into_convert::<V1800>(&ConversionOptions::strict())
            .map_err(|error| ModelError(error.to_string()))?
            .model;
        Ok(Self { model })
    }
}

/// An owned decoded model for source inspection and editing. Conversion is strict: unsupported data
/// is reported rather than silently discarded.
#[cfg(test)]
pub(crate) struct Wc3Document {
    /// Format version recorded before normalization.
    pub source_version: u32,
    /// Source data normalized to the common runtime model version.
    pub model: Shared<Document<'static, V1800>>,
}

#[cfg(test)]
impl Wc3Document {
    /// Detects binary MDX by its `MDLX` header; otherwise reads UTF-8 MDL.
    ///
    /// Returns an error for invalid UTF-8, malformed model data, or data that
    /// cannot be strictly converted to the runtime model version.
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        if bytes.starts_with(b"MDLX") {
            Self::decode_mdx(bytes)
        } else {
            let source = from_utf8(bytes)
                .map_err(|error| ModelError(format!("MDL source is not UTF-8: {error}")))?;
            Self::decode_mdl(source)
        }
    }

    /// Reads binary MDX, using version 800 when there is no `VERS` chunk.
    /// Returns an error for malformed data or a failed strict conversion.
    pub fn decode_mdx(bytes: &[u8]) -> Result<Self, ModelError> {
        let source = DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
            .map_err(|error| ModelError(error.to_string()))?;
        Self::normalize(source)
    }

    /// Reads MDL text and applies the same strict conversion as MDX loading.
    /// Returns an error for malformed text or a failed strict conversion.
    pub fn decode_mdl(source: &str) -> Result<Self, ModelError> {
        let source = DynamicDocument::<NoExtensions>::decode_mdl(source)
            .map_err(|error| ModelError(error.to_string()))?;
        Self::normalize(source)
    }

    fn normalize(source: DynamicDocument<'_>) -> Result<Self, ModelError> {
        let source_version = source.version();
        let model = source
            .into_convert::<V1800>(&ConversionOptions::strict())
            .map_err(|error| ModelError(error.to_string()))?
            .model
            .into_owned();
        Ok(Self {
            source_version,
            model: model.into(),
        })
    }
}

/// An owned Warcraft III semantic model, independent of the source file dialect.
/// Native document inspection belongs to `wc3::model::formats::mdlx`.
pub struct Wc3Model {
    /// Format version recorded in the input document.
    pub source_version: u32,
    /// Checked semantic graph used by default preparation APIs.
    pub model: Shared<Model<'static, Mdlx>>,
}

/// A decoded semantic graph ready for consuming compilation.
pub struct DecodedModel {
    /// Format version recorded in the input document.
    pub source_version: u32,
    /// Owned semantic graph. The input bytes can be released after decoding.
    pub model: Model<'static, Mdlx>,
}

impl DecodedModel {
    /// Decode MDX or UTF-8 MDL and import the original dialect with strict policies.
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        let source = if bytes.starts_with(b"MDLX") {
            DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
                .map_err(|error| ModelError(error.to_string()))?
        } else {
            let text = from_utf8(bytes)
                .map_err(|error| ModelError(format!("MDL source is not UTF-8: {error}")))?;
            DynamicDocument::<NoExtensions>::decode_mdl(text)
                .map_err(|error| ModelError(error.to_string()))?
        };
        Self::from_native(source)
    }

    fn from_native(source: DynamicDocument<'_>) -> Result<Self, ModelError> {
        let source_version = source.version();
        let model = semantic_mdlx::import_dynamic_owned(&source, &ImportOptions::default())
            .map_err(|error| ModelError(error.to_string()))?
            .value;
        Ok(Self {
            source_version,
            model,
        })
    }
}

impl Wc3Model {
    /// Decode MDX or UTF-8 MDL into an independently owned semantic graph.
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        Ok(Self::from_decoded(DecodedModel::decode(bytes)?))
    }
    fn from_decoded(source: DecodedModel) -> Self {
        Self {
            source_version: source.source_version,
            model: source.model.into(),
        }
    }
    /// Decode owned input; no encoded bytes are retained.
    pub fn decode_owned(bytes: Vec<u8>) -> Result<Self, ModelError> {
        Self::decode(&bytes)
    }
    /// Decode a binary MDX document and import its semantics.
    pub fn decode_mdx(bytes: &[u8]) -> Result<Self, ModelError> {
        let source = DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
            .map_err(|error| ModelError(error.to_string()))?;
        Ok(Self::from_decoded(DecodedModel::from_native(source)?))
    }
    /// Decode an MDL document and import its semantics.
    pub fn decode_mdl(source: &str) -> Result<Self, ModelError> {
        let source = DynamicDocument::<NoExtensions>::decode_mdl(source)
            .map_err(|error| ModelError(error.to_string()))?;
        Ok(Self::from_decoded(DecodedModel::from_native(source)?))
    }
}
