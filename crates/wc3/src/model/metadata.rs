//! Profile-owned model declarations and universal coordinate interpretation.
use super::profiles::{m2, mdlx};
use super::profiles::{Mdlx, Universal, Wmo, M2};
use super::{Basis, Bounds, LengthUnit, Model, ModelMetadata};
use std::borrow::Cow;

macro_rules! named_metadata {
    ($module:ident) => {
        impl<'a> From<Cow<'a, str>> for $module::ModelMetadata<'a> {
            fn from(name: Cow<'a, str>) -> Self {
                Self {
                    name,
                    ..Self::default()
                }
            }
        }
        impl<'a> From<&'a str> for $module::ModelMetadata<'a> {
            fn from(name: &'a str) -> Self {
                Cow::Borrowed(name).into()
            }
        }
        impl<'a> From<String> for $module::ModelMetadata<'a> {
            fn from(name: String) -> Self {
                Cow::<str>::Owned(name).into()
            }
        }
    };
}
named_metadata!(mdlx);
named_metadata!(m2);

// Accessors are implemented only for the built-in spatial profiles, rather than
// imposing metadata capabilities on application-defined profiles.
macro_rules! native_space {
    ($($profile:ty),*) => { $(impl Model<'_, $profile> {
        /// Fixed coordinate convention of the native format.
        pub fn basis(&self) -> Basis { Basis::default() }
        /// Native formats do not author a physical length scale.
        pub fn units(&self) -> LengthUnit { LengthUnit::Unspecified }
    })* }
}
native_space!(Mdlx, M2, Wmo);
macro_rules! named_model {
    ($($profile:ty),*) => { $(impl Model<'_, $profile> {
        /// Authored model name.
        pub fn name(&self) -> &str { &self.metadata().name }
    })* }
}
named_model!(Mdlx, M2);
macro_rules! required_bounds {
    ($($profile:ty),*) => { $(impl Model<'_, $profile> {
        /// Authored model bounding volume.
        pub fn bounds(&self) -> Bounds { self.metadata().bounds }
    })* }
}
required_bounds!(M2);
macro_rules! optional_bounds {
    ($($profile:ty),*) => { $(impl Model<'_, $profile> {
        /// Authored model bounds, when the corresponding native declaration exists.
        pub fn bounds(&self) -> Option<Bounds> { self.metadata().bounds }
    })* }
}
optional_bounds!(Mdlx, Wmo);
impl Model<'_, Universal> {
    /// Authored name when the selected native declarations supply one.
    pub fn name(&self) -> &str {
        match self.metadata() {
            ModelMetadata::Mdlx(value) => &value.name,
            ModelMetadata::M2(value) => &value.name,

            _ => "",
        }
    }
    /// Interpretation of authored coordinates.
    pub fn basis(&self) -> Basis {
        self.metadata().basis()
    }
    /// Physical length scale, when known.
    pub fn units(&self) -> LengthUnit {
        self.metadata().units()
    }
    /// Bounding volume supplied by the model declarations.
    pub fn bounds(&self) -> Option<Bounds> {
        match self.metadata() {
            ModelMetadata::Mdlx(value) => value.bounds,
            ModelMetadata::Wmo(value) => value.bounds,
            ModelMetadata::M2(value) => Some(value.bounds),
        }
    }
}

impl ModelMetadata<'_> {
    /// Fixed coordinate convention of the selected native declaration family.
    pub fn basis(&self) -> Basis {
        Basis::default()
    }
    /// Native declaration families do not author a physical length scale.
    pub fn units(&self) -> LengthUnit {
        LengthUnit::Unspecified
    }
}
