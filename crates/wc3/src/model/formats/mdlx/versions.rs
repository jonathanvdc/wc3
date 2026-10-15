//! Supported MDX versions as types.
use std::fmt::Debug;

use crate::model::formats::mdlx::geometry::GeosetLayout;
use crate::model::formats::mdlx::materials::MaterialLayout;
use crate::model::formats::mdlx::scene::{CameraLayout, LightLayout};
use crate::model::formats::mdlx::Version;

mod sealed {
    pub trait Sealed {}
}

/// A version whose record layout is known by this library.
pub trait FormatVersion:
    'static
    + sealed::Sealed
    + MaterialLayout
    + GeosetLayout
    + CameraLayout
    + LightLayout
    + Clone
    + Debug
    + Eq
{
    /// Numeric format version written in the `VERS` chunk.
    const NUMBER: Version;
}

macro_rules! versions {
    ($($name:ident = $number:literal),* $(,)?) => {$ (
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[doc = concat!("A marker selecting model format version ", stringify!($number), " and its record layout.")]
        pub struct $name;
        impl sealed::Sealed for $name {}
        impl FormatVersion for $name {
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
    V1300 = 1300,
    V1400 = 1400,
    V1600 = 1600,
    V1800 = 1800
);

macro_rules! version_capabilities {
    ($(
        $(#[$meta:meta])*
        $vis:vis trait $name:ident for $($version:ident),+;
    )+) => {
        $(
            $(#[$meta])*
            $vis trait $name: FormatVersion {}
            $(impl $name for $version {})+
        )+
    };
}

version_capabilities! {
    /// A model version supporting the `BPOS`, `FAFX`, and `CORN` chunks.
    ///
    /// ```compile_fail
    /// use wc3::model::formats::mdlx::{Document, V800};
    /// Document::<V800>::new().bind_poses();
    /// ```
    pub trait SupportsReforgedChunks for V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800;

    /// Versions with a material shader path.
    pub trait SupportsMaterialShaderPath for V900, V1000;

    /// Versions with layer emissive gain.
    pub trait SupportsEmissiveGain for V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800;

    /// Versions with layer Fresnel fields.
    pub trait SupportsFresnel for V1000, V1100, V1200, V1300, V1400, V1600, V1800;

    /// Versions with layer shader type IDs.
    pub trait SupportsLayerShaderTypeId for V1100, V1200, V1300, V1400, V1600, V1800;

    /// Versions with layer texture slots.
    pub trait SupportsLayerTextureSlots for V1100, V1200, V1300, V1400, V1600, V1800;

    /// Versions with a serialized light shadow intensity.
    ///
    /// ```compile_fail
    /// use wc3::model::formats::mdlx::{scene::{Light, LightType, Node}, V1100};
    /// Light::<V1100>::new(Node::new("Lamp", 1).unwrap(), LightType::Omnidirectional).set_shadow_intensity(0.5);
    /// ```
    pub trait SupportsLightShadowIntensity for V1200, V1300, V1400, V1600, V1800;

    /// Versions with light shadow casting and its range.
    ///
    /// ```compile_fail
    /// use wc3::model::formats::mdlx::{scene::{Light, LightType, Node}, V1200};
    /// Light::<V1200>::new(Node::new("Lamp", 1).unwrap(), LightType::Omnidirectional).set_shadow_casting(true);
    /// ```
    pub trait SupportsLightShadowCasting for V1300, V1400, V1600, V1800;

    /// Versions with serialized light falloff coefficients.
    ///
    /// ```compile_fail
    /// use wc3::model::formats::mdlx::{scene::{Light, LightType, LightFalloff, Node}, V1400};
    /// Light::<V1400>::new(Node::new("Lamp", 1).unwrap(), LightType::Omnidirectional).set_falloff(LightFalloff::default());
    /// ```
    pub trait SupportsLightFalloff for V1600, V1800;

}
