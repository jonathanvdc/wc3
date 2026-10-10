use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::from_utf8;
use wc3::model::mdl::Read as _;
use wc3::model::NoExtensions;
use wc3::model::{ConversionOptions, DynamicModel, Model, V1800};

/// A model decoding, strict conversion, or geometry preparation failure.
#[derive(Debug)]
pub struct ModelError(pub(crate) String);

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Error for ModelError {}

/// A version-independent source model. Conversion is strict: unsupported data
/// is reported rather than silently discarded.
pub struct Wc3Model {
    /// Format version recorded before normalization.
    pub source_version: u32,
    /// Source data normalized to the common runtime model version.
    pub model: Model<V1800>,
}

impl Wc3Model {
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
        let source = DynamicModel::<NoExtensions>::decode_mdx(bytes, 800)
            .map_err(|error| ModelError(error.to_string()))?;
        Self::normalize(source)
    }

    /// Reads MDL text and applies the same strict conversion as MDX loading.
    /// Returns an error for malformed text or a failed strict conversion.
    pub fn decode_mdl(source: &str) -> Result<Self, ModelError> {
        let source = DynamicModel::<NoExtensions>::decode_mdl(source)
            .map_err(|error| ModelError(error.to_string()))?;
        Self::normalize(source)
    }

    fn normalize(source: DynamicModel) -> Result<Self, ModelError> {
        let source_version = source.version();
        let model = source
            .convert::<V1800>(&ConversionOptions::strict())
            .map_err(|error| ModelError(error.to_string()))?
            .model;
        Ok(Self {
            source_version,
            model,
        })
    }
}
