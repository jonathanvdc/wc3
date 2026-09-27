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

macro_rules! version_capabilities {
    ($(
        $(#[$meta:meta])*
        $vis:vis trait $name:ident for $($version:ident),+;
    )+) => {
        $(
            $(#[$meta])*
            $vis trait $name: ModelVersion {}
            $(impl $name for $version {})+
        )+
    };
}

version_capabilities! {
    /// A model version supporting the `BPOS`, `FAFX`, and `CORN` chunks.
    ///
    /// ```compile_fail
    /// use wc3_mdx::{Model, V800};
    /// Model::<V800>::new().bind_poses();
    /// ```
    pub trait SupportsReforgedChunks for V900, V1000, V1100, V1200, V1800;

    /// Versions with a material shader path.
    pub trait SupportsMaterialShaderPath for V900, V1000;

    /// Versions with layer emissive gain.
    pub trait SupportsEmissiveGain for V900, V1000, V1100, V1200, V1800;

    /// Versions with layer Fresnel fields.
    pub trait SupportsFresnel for V1000, V1100, V1200, V1800;

    /// Versions with layer shader type IDs.
    pub trait SupportsLayerShaderTypeId for V1100, V1200, V1800;

    /// Versions with layer texture slots.
    pub trait SupportsLayerTextureSlots for V1100, V1200, V1800;

    /// Versions with the extra light record words.
    pub trait SupportsLightExtendedWords for V1200, V1800;

    /// Versions with geoset skin bone indices.
    pub trait SupportsSkinBoneIndices for V1200, V1800;
}
