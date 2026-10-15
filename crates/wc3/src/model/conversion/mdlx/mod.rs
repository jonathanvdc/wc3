//! Warcraft III adapters for independent, borrowed semantic model graphs.
use super::{Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::mdlx::{Chunk, Model as NativeModel, ModelDialect};
use crate::model::semantic::profiles::Mdlx;
use crate::model::semantic::Model;
mod animation;
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
mod scene;
mod scene_lights;
mod state;
mod text;
/// Borrows native geometry and animation values into a checked authoring graph.
pub fn import<'a, D: ModelDialect>(
    source: &'a NativeModel<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Mdlx>>, Error>
where
    D::Extension: Chunk,
{
    import::read(source, options)
}
/// Constructs a native model for a selected file dialect and reports encoding losses.
///
/// The input vocabulary must be Warcraft III. Translating a broader graph is a
/// separate operation; this planner does not implicitly approximate its payloads.
///
/// ```compile_fail
/// use wc3::model::{conversion::{self, ExportOptions}, mdlx::V800, semantic::Model};
/// let universal = Model::standalone("Example");
/// conversion::mdlx::plan::<V800>(&universal, &ExportOptions::default());
/// ```
pub fn plan<'a, 'data, T: ModelDialect>(
    asset: &'a Model<'data, Mdlx>,
    options: &ExportOptions,
) -> Result<Plan<NativeModel<'a, T>>, Error> {
    export::write(asset, options)
}
