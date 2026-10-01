use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::from_utf8;
use wc3::model::mdl::Read as _;
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
    /// Detects binary MDX by its `MDLX` header; otherwise reads UTF-8 MDL.
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
    pub fn decode_mdx(bytes: &[u8]) -> Result<Self, ModelError> {
        let source =
            DynamicModel::decode_mdx(bytes, 800).map_err(|error| ModelError(error.to_string()))?;
        Self::normalize(source)
    }

    /// Reads MDL text and applies the same strict conversion as MDX loading.
    pub fn decode_mdl(source: &str) -> Result<Self, ModelError> {
        let source =
            DynamicModel::decode_mdl(source).map_err(|error| ModelError(error.to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::mdl::Write as _;

    #[test]
    fn classic_fixture_converts_to_runtime_version() {
        let bytes = include_bytes!("../../wc3/tests/fixtures/mdl/quad_model.mdx");
        let model = Wc3Model::decode(bytes).unwrap();
        assert_eq!(model.source_version, 800);
        assert_eq!(model.model.version(), 1800);
        assert!(!model.model.geosets().is_empty());
    }

    #[test]
    fn mdl_and_mdx_normalize_to_the_same_model() {
        let text = include_str!("../../wc3/tests/fixtures/mdl/quad_model.mdl");
        let mdl = Wc3Model::decode_mdl(text).unwrap();
        let mdx = Wc3Model::decode_mdx(include_bytes!(
            "../../wc3/tests/fixtures/mdl/quad_model.mdx"
        ))
        .unwrap();
        assert_eq!(mdl.source_version, 800);
        assert_eq!(
            mdl.model.encode_mdl().unwrap(),
            mdx.model.encode_mdl().unwrap()
        );
        assert_eq!(
            Wc3Model::decode(text.as_bytes())
                .unwrap()
                .model
                .encode_mdl()
                .unwrap(),
            mdl.model.encode_mdl().unwrap()
        );
        assert_eq!(
            Wc3Model::decode_mdl(&format!("\u{feff}{text}"))
                .unwrap()
                .model
                .encode_mdl()
                .unwrap(),
            mdl.model.encode_mdl().unwrap()
        );
    }

    #[test]
    fn malformed_mdl_and_non_utf8_source_report_errors() {
        assert!(Wc3Model::decode_mdl("Version { FormatVersion").is_err());
        let error = Wc3Model::decode(&[0xff]).err().unwrap();
        assert!(error.to_string().contains("UTF-8"));
    }

    #[test]
    fn synthetic_capture_fixture_contains_live_particle_emitters() {
        let model =
            Wc3Model::decode_mdl(include_str!("../tests/fixtures/particle_capture.mdl")).unwrap();
        let emitters = model.model.particle_emitters2();
        assert_eq!(emitters.len(), 2);
        assert!(emitters.iter().all(|emitter| emitter.life_span > 0.0));
        assert!(!emitters[0].node.flags.model_space());
        assert!(emitters[1].node.flags.model_space());
    }
}
