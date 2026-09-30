use std::error::Error;
use std::fmt::{Display, Formatter};
use wc3::model::{ConversionOptions, DynamicModel, Model, V1800};

#[derive(Debug)]
pub struct ModelError(pub(crate) String);

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ModelError {}

/// A version-independent source model. Conversion is strict: unsupported data
/// is reported rather than silently discarded.
pub struct Wc3Model {
    pub source_version: u32,
    pub model: Model<V1800>,
}

impl Wc3Model {
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        let source =
            DynamicModel::decode_mdx(bytes, 800).map_err(|error| ModelError(error.to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_fixture_converts_to_runtime_version() {
        let bytes = include_bytes!("../../wc3/tests/fixtures/mdl/quad_model.mdx");
        let model = Wc3Model::decode(bytes).unwrap();
        assert_eq!(model.source_version, 800);
        assert_eq!(model.model.version(), 1800);
        assert!(!model.model.geosets().is_empty());
    }
}
