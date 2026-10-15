//! Direct codecs between world-model files and the typed world authoring vocabulary.
use super::{Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::Model;
use crate::model::wmo::legacy::{Bundle16, Root14};
use crate::model::wmo::{Bundle, Dialect};
mod graph;

/// Monolithic alpha world-model output layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V14;
/// Split early-beta world-model output layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V16;
/// A native source with resolved companion geometry.
pub trait Source {
    /// Borrows authored fields into a checked world model without an encoded intermediate.
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, Wmo>>, Error>;
}
impl<D: Dialect> Source for Bundle<'_, D> {
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, Wmo>>, Error> {
        graph::import(self, options)
    }
}
impl Source for Root14<'_> {
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, Wmo>>, Error> {
        graph::import_alpha(self, options)
    }
}
impl Source for Bundle16<'_> {
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, Wmo>>, Error> {
        graph::import_beta(self, options)
    }
}
/// A selected native output layout.
pub trait Destination {
    /// Inspectable native root and supporting documents.
    type Output<'a>;
    /// Constructs output records from typed world authoring data.
    fn plan<'a>(
        asset: &'a Model<'_, Wmo>,
        options: &ExportOptions,
    ) -> Result<Plan<Self::Output<'a>>, Error>;
}
impl<D: Dialect> Destination for D {
    type Output<'a> = Bundle<'a, D>;
    fn plan<'a>(
        asset: &'a Model<'_, Wmo>,
        options: &ExportOptions,
    ) -> Result<Plan<Self::Output<'a>>, Error> {
        graph::plan(asset, options)
    }
}
impl Destination for V14 {
    type Output<'a> = Root14<'a>;
    fn plan<'a>(
        asset: &'a Model<'_, Wmo>,
        options: &ExportOptions,
    ) -> Result<Plan<Self::Output<'a>>, Error> {
        graph::plan_alpha(asset, options)
    }
}
impl Destination for V16 {
    type Output<'a> = Bundle16<'a>;
    fn plan<'a>(
        asset: &'a Model<'_, Wmo>,
        options: &ExportOptions,
    ) -> Result<Plan<Self::Output<'a>>, Error> {
        graph::plan_beta(asset, options)
    }
}
/// Imports a native source into its own family vocabulary.
pub fn import<'a, S: Source>(
    source: &'a S,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Wmo>>, Error> {
    source.import(options)
}
/// Plans native output from its own family vocabulary.
pub fn plan<'a, D: Destination>(
    asset: &'a Model<'_, Wmo>,
    options: &ExportOptions,
) -> Result<Plan<D::Output<'a>>, Error> {
    D::plan(asset, options)
}
