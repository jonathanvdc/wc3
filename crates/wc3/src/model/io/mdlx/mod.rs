//! Warcraft III adapters for independent, borrowed semantic model graphs.
use super::{Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::formats::mdlx::mdl::Read as _;
use crate::model::formats::mdlx::{
    visit_document, Chunk, Dialect, Document as NativeDocument, DynamicDocument, NoExtensions,
};
use crate::model::profiles::Mdlx;
use crate::model::Model;
mod animation;
#[cfg(test)]
#[path = "billboard_tests.rs"]
mod billboard_tests;
mod curves;
mod encode_animation;
mod encode_geometry;
mod encode_helpers;
mod encode_lights;
mod encode_materials;
mod encode_scene;
mod export;
mod geometry;
mod import;
mod materials;
#[cfg(test)]
#[path = "ribbon_axis_tests.rs"]
mod ribbon_axis_tests;
mod scene;
mod scene_lights;
mod state;
mod storage;
mod text;
/// Decodes MDX directly into an independent semantic model.
/// Files without a `VERS` chunk use format version 800.
pub fn decode_mdx(
    bytes: &[u8],
    options: &ImportOptions,
) -> Result<Conversion<Model<'static, Mdlx>>, Error> {
    let document = DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800)
        .map_err(|error| Error::new("mdx", error.to_string()))?;
    import_dynamic_owned(&document, options)
}
/// Decodes MDL directly into an independent semantic model.
/// The native text codec selects the declared format version.
pub fn decode_mdl(
    text: &str,
    options: &ImportOptions,
) -> Result<Conversion<Model<'static, Mdlx>>, Error> {
    let document = DynamicDocument::<NoExtensions>::decode_mdl(text)
        .map_err(|error| Error::new("mdl", error.to_string()))?;
    import_dynamic_owned(&document, options)
}
/// Imports a decoded document in its original dialect into an independent model.
/// Consumers can inspect native metadata before importing without dispatching
/// the document variants or first converting them to a common file version.
pub fn import_dynamic_owned(
    source: &DynamicDocument<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'static, Mdlx>>, Error> {
    visit_document!(source, |document| import_owned(document, options))
}
/// Borrows native geometry and text values into a checked authoring graph.
pub fn import<'a, D: Dialect>(
    source: &'a NativeDocument<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Mdlx>>, Error>
where
    D::Extension: Chunk,
{
    import::read::<_, storage::Borrowed>(source, options)
}
/// Imports an independent model, copying native buffers that the borrowed importer retains.
/// The returned graph remains valid after both the document and its input bytes are dropped.
pub fn import_owned<D: Dialect>(
    source: &NativeDocument<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'static, Mdlx>>, Error>
where
    D::Extension: Chunk,
{
    import::read::<_, storage::Owned>(source, options)
}
/// Constructs a native model for a selected file dialect and reports encoding losses.
///
/// The input vocabulary must be Warcraft III. Translating a broader graph is a
/// separate operation; this planner does not implicitly approximate its payloads.
///
/// ```compile_fail
/// use wc3::model::{conversion::ExportOptions, io, formats::mdlx::V800, profiles::Universal, Model, ModelMetadata};
/// let universal = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
/// io::mdlx::plan::<V800>(&universal, &ExportOptions::default());
/// ```
pub fn plan<'a, 'data, T: Dialect>(
    asset: &'a Model<'data, Mdlx>,
    options: &ExportOptions,
) -> Result<Plan<NativeDocument<'a, T>>, Error> {
    export::write(asset, options)
}
