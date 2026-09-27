//! Supported MDX versions as types.
use std::fmt::Debug;

use crate::geometry::GeosetLayout;
use crate::materials::MaterialLayout;
use crate::scene::{CameraLayout, LightLayout};
use crate::Version;

mod sealed {
    pub trait Sealed {}
}

/// A version whose record layout is known by this library.
pub trait ModelVersion:
    sealed::Sealed + MaterialLayout + GeosetLayout + CameraLayout + LightLayout + Clone + Debug + Eq
{
    const NUMBER: Version;
}

macro_rules! versions {
    ($($name:ident = $number:literal),* $(,)?) => {$ (
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl ModelVersion for $name {
            const NUMBER: Version = $number;
        }
    )* };
}

versions!(
    V800 = 800,
    V900 = 900,
    V1000 = 1000,
    V1100 = 1100,
    V1200 = 1200,
    V1800 = 1800
);

/// A model version supporting the `BPOS`, `FAFX`, and `CORN` chunks.
///
/// ```compile_fail
/// use wc3_mdx::{Model, V800};
/// Model::<V800>::new().bind_poses();
/// ```
pub trait SupportsReforgedChunks: ModelVersion {}

impl SupportsReforgedChunks for V900 {}
impl SupportsReforgedChunks for V1000 {}
impl SupportsReforgedChunks for V1100 {}
impl SupportsReforgedChunks for V1200 {}
impl SupportsReforgedChunks for V1800 {}

/// Versions with a material shader path.
pub trait SupportsMaterialShaderPath: ModelVersion {}
impl SupportsMaterialShaderPath for V900 {}
impl SupportsMaterialShaderPath for V1000 {}

/// Versions with layer emissive gain.
pub trait SupportsEmissiveGain: ModelVersion {}
impl SupportsEmissiveGain for V900 {}
impl SupportsEmissiveGain for V1000 {}
impl SupportsEmissiveGain for V1100 {}
impl SupportsEmissiveGain for V1200 {}
impl SupportsEmissiveGain for V1800 {}

/// Versions with layer Fresnel fields.
pub trait SupportsFresnel: ModelVersion {}
impl SupportsFresnel for V1000 {}
impl SupportsFresnel for V1100 {}
impl SupportsFresnel for V1200 {}
impl SupportsFresnel for V1800 {}

/// Versions with layer shader type IDs.
pub trait SupportsLayerShaderTypeId: ModelVersion {}
impl SupportsLayerShaderTypeId for V1100 {}
impl SupportsLayerShaderTypeId for V1200 {}
impl SupportsLayerShaderTypeId for V1800 {}

/// Versions with layer texture slots.
pub trait SupportsLayerTextureSlots: ModelVersion {}
impl SupportsLayerTextureSlots for V1100 {}
impl SupportsLayerTextureSlots for V1200 {}
impl SupportsLayerTextureSlots for V1800 {}

/// Versions with the extra light record words.
pub trait SupportsLightExtendedWords: ModelVersion {}
impl SupportsLightExtendedWords for V1200 {}
impl SupportsLightExtendedWords for V1800 {}

/// Versions with geoset skin bone indices.
pub trait SupportsSkinBoneIndices: ModelVersion {}
impl SupportsSkinBoneIndices for V1200 {}
impl SupportsSkinBoneIndices for V1800 {}
