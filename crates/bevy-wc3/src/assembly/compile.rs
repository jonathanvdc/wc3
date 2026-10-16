use super::images::ImageCompiler;
use super::plan::plan;
use super::{Assembly, Models, PendingAssembly, PendingPart, RgbaImage, Source};
use crate::{CompiledModel, ModelError, ModelPart, PartBinding};
use bevy::prelude::{Assets, Image};
use std::collections::HashMap;
use wc3::model::{ImageResource, Model};

/// Compile a checked resource assembly with shared source decoding and typed replacements.
/// Compatible repeated skinned sources are compiled once with united geometry.
/// Native image decoding belongs to the caller. Failed decoding may leave
/// reusable images in the asset store, but never returns a partial assembly.
pub fn compile_assembly(
    assembly: &Assembly,
    models: &Models<'_>,
    images: &mut Assets<Image>,
    resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    let mut pending = PendingAssembly::new(assembly.root.clone());
    pending.parts = assembly
        .parts
        .iter()
        .map(|part| {
            let mut pending = PendingPart::new(part.source.clone(), part.binding.clone());
            pending.clips = part.clips.clone();
            pending
        })
        .collect();
    compile_with_bindings(
        &pending,
        models,
        images,
        resolve_image,
        |b| matches!(b, PartBinding::Skinned { .. }),
        |_, _, _, binding| Ok(binding.clone()),
    )
}

pub(crate) fn compile_with_bindings<B: Clone + PartialEq>(
    assembly: &PendingAssembly<B>,
    models: &Models<'_>,
    images: &mut Assets<Image>,
    mut resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
    is_skinned: impl Fn(&B) -> bool,
    mut resolve_binding: impl FnMut(
        &Model<'_>,
        &Model<'_>,
        &CompiledModel,
        &B,
    ) -> Result<PartBinding, ModelError>,
) -> Result<CompiledModel, ModelError> {
    let parts = plan(assembly, models, is_skinned)?;
    let root = &models[&assembly.root.model];
    let mut decoded = HashMap::new();
    let mut compiler = ImageCompiler {
        images,
        resolve: &mut resolve_image,
        decoded: &mut decoded,
    };
    let compiled_root =
        compiler.compile(root, assembly.root.draws.clone(), &assembly.root.textures)?;
    let mut compiled_parts = Vec::with_capacity(parts.len());
    let mut bindings = Vec::with_capacity(parts.len());
    let mut sources = Vec::new();
    for part in &parts {
        let model = &models[&part.source.model];
        let compiled = if let Some(index) = sources.iter().position(|source: &&Source| {
            source.model == part.source.model
                && source.draws == part.source.draws
                && source.textures == part.source.textures
        }) {
            CompiledModel::clone(&compiled_parts[index])
        } else {
            compiler.compile(model, part.source.draws.clone(), &part.source.textures)?
        };
        sources.push(&part.source);
        bindings.push(resolve_binding(root, model, &compiled, &part.binding)?);
        compiled_parts.push(compiled);
    }
    compiled_root.compose(compiled_parts.iter().zip(bindings).zip(&parts).map(
        |((model, binding), part)| ModelPart {
            model,
            binding,
            clips: part.clips.iter().map(|(&a, &b)| (a, b)).collect(),
        },
    ))
}
