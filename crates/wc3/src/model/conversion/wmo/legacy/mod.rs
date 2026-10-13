//! Alpha and beta layouts translated through native record values.
use super::*;
use native::legacy as old;

/// Monolithic alpha version 14 output layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V14;
/// Split early-beta version 16 output layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V16;
/// A native source whose complete companion graph can be imported.
pub trait Source {
    /// Converts native record values into active editable WEM records.
    fn import(&self, options: &ImportOptions) -> Result<Conversion<Asset>, Error>;
}
impl<D: Dialect> Source for Bundle<'_, D> {
    fn import(&self, options: &ImportOptions) -> Result<Conversion<Asset>, Error> {
        import::modern(self, options)
    }
}
/// An output layout whose native graph is rebuilt from WEM values.
pub trait Destination {
    /// Native root or bundle produced by the selected layout.
    type Output;
    /// Constructs and validates an inspectable output draft.
    fn plan(asset: &Asset, options: &ExportOptions) -> Result<Plan<Self::Output>, Error>;
}
impl<D: Dialect> Destination for D {
    type Output = Bundle<'static, D>;
    fn plan(asset: &Asset, options: &ExportOptions) -> Result<Plan<Self::Output>, Error> {
        export::modern::<D>(asset, options)
    }
}
/// Imports a modern bundle or a historical native root/bundle.
pub fn import<S: Source>(source: &S, options: &ImportOptions) -> Result<Conversion<Asset>, Error> {
    source.import(options)
}
/// Plans modern or historical native output through a statically selected layout.
pub fn plan<D: Destination>(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Plan<D::Output>, Error> {
    D::plan(asset, options)
}

mod from_native;
mod metadata_fields;
mod to_native;
use metadata_fields::{legacy_metadata14, profile_fields, set_version};
