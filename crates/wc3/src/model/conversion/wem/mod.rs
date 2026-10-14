//! Direct editable-model adaptation over independent logical authoring domains.
mod metadata;
mod resources;
mod text;

mod attributes;

mod geometry;

mod materials;

mod forces;

mod shader_materials;

mod actors;

mod material_records;

mod animation;

mod appearance;

mod document;

mod hierarchy;

mod stages;

mod physics;

mod assembly;

/// Checks native declaration availability without encoding or reading a document.
pub(crate) fn supports_node_declarations(model: &Model<'_, Wem>) -> bool {
    hierarchy::supports(model)
}

pub(crate) fn supports_model_declarations(models: &[Model<'_, Wem>]) -> bool {
    assembly::model_records::supports(models)
}

use crate::model::conversion::{Asset, Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::Model;
use crate::model::wem::RawDocument;

/// Resolves every generation-three model and its shared dependencies into editable authoring values.
/// Historical generation-one and generation-two envelopes remain available through the raw codec;
/// their distinct authored schemas are not interpreted by this adapter.
pub fn import<'view>(
    source: &'view RawDocument<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Asset<'view, Wem>>, Error> {
    assembly::read(source, options)
}

/// Constructs a complete native document from its current authored collection.
pub fn plan<'view, 'data: 'view>(
    asset: &'view Asset<'data, Wem>,
    options: &ExportOptions,
) -> Result<Plan<RawDocument<'view>>, Error> {
    assembly::write(asset, options)
}
