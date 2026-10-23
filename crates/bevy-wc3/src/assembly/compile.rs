use super::images::ImageCompiler;
use super::plan::plan;
use super::{
    Assembly, ImageSink, Models, PendingAssembly, PendingPart, RgbaImage, Source,
    SourceDependencies,
};
use crate::mesh_interner::MeshInterner;
use crate::model_compilation::validate_dependencies;
use crate::{CompiledModel, ModelError, ModelPart, PartBinding};
use std::collections::{hash_map::Entry, HashMap};
use std::iter::once;
use wc3::model::{ImageResource, Model, ResourceLocator};

/// Compile a checked resource assembly with shared source decoding and typed replacements.
/// Compatible repeated skinned sources are compiled once with united geometry.
/// Native image decoding belongs to the caller. Images are emitted through the
/// sink during compilation, including before a later failure. A staging sink can
/// defer their publication until the complete assembly succeeds.
pub fn compile_assembly(
    assembly: &Assembly,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    mut resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    compile_assembly_impl(
        assembly,
        models,
        images,
        &mut |_, _| Ok(SourceDependencies::default()),
        &mut resolve_image,
    )
}

/// Compile an assembly with prepared dependencies resolved once per source resource.
///
/// Resolution follows root then first occurrence order, after plan validation.
/// Every binding is checked against its owning Universal model before images are
/// decoded. Repeated occurrences share the prepared definitions, but retain
/// independent runtime state. Different occurrence-specific bindings for one
/// resource are not supported by this entrypoint.
///
/// Source material, draw and image choices retain their ordinary owners. Programme
/// images and finite limits remain in the supplied [`ParticleEffect`](crate::ParticleEffect).
/// Other compilation settings, including the physics pair budget, keep defaults.
/// A failed resolver or binding check returns no partial assembly; callback side
/// effects remain application-owned.
pub fn compile_assembly_with_dependencies(
    assembly: &Assembly,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    mut resolve_dependencies: impl FnMut(
        &ResourceLocator<'static>,
        &Model<'_>,
    ) -> Result<SourceDependencies, ModelError>,
    mut resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    compile_assembly_impl(
        assembly,
        models,
        images,
        &mut resolve_dependencies,
        &mut resolve_image,
    )
}

// Keep preparation code generation here; only resource preparation and image I/O cross callbacks.
#[inline(never)]
fn compile_assembly_impl(
    assembly: &Assembly,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    resolve_dependencies: &mut dyn FnMut(
        &ResourceLocator<'static>,
        &Model<'_>,
    ) -> Result<SourceDependencies, ModelError>,
    resolve_image: &mut dyn FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
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
    compile_with_dependencies(
        &pending,
        models,
        images,
        resolve_dependencies,
        resolve_image,
        |b| matches!(b, PartBinding::Skinned { .. }),
        |_, _, _, binding| Ok(binding.clone()),
    )
}

pub(crate) fn compile_with_bindings<B: Clone + PartialEq>(
    assembly: &PendingAssembly<B>,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
    is_skinned: impl Fn(&B) -> bool,
    resolve_binding: impl FnMut(
        &Model<'_>,
        &Model<'_>,
        &CompiledModel,
        &B,
    ) -> Result<PartBinding, ModelError>,
) -> Result<CompiledModel, ModelError> {
    compile_with_dependencies(
        assembly,
        models,
        images,
        |_, _| Ok(SourceDependencies::default()),
        resolve_image,
        is_skinned,
        resolve_binding,
    )
}

fn compile_with_dependencies<B: Clone + PartialEq>(
    assembly: &PendingAssembly<B>,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    mut resolve_dependencies: impl FnMut(
        &ResourceLocator<'static>,
        &Model<'_>,
    ) -> Result<SourceDependencies, ModelError>,
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
    let mut dependencies = HashMap::new();
    for source in once(&assembly.root).chain(assembly.parts.iter().map(|part| &part.source)) {
        if let Entry::Vacant(entry) = dependencies.entry(source.model.clone()) {
            let model = &models[&source.model];
            let prepared = resolve_dependencies(&source.model, model)?;
            validate_dependencies(model, &prepared.child_models, &prepared.particle_effects)?;
            entry.insert(prepared);
        }
    }
    let root = &models[&assembly.root.model];
    let mut decoded = HashMap::new();
    let mut compiler = ImageCompiler {
        images,
        resolve: &mut resolve_image,
        decoded: &mut decoded,
        handles: Vec::new(),
    };
    let mut geometry = MeshInterner::default();
    let compiled_root = compiler.compile(
        root,
        &assembly.root,
        &dependencies[&assembly.root.model],
        &mut geometry,
    )?;
    let mut compiled_parts = Vec::with_capacity(parts.len());
    let mut bindings = Vec::with_capacity(parts.len());
    let mut sources = Vec::new();
    for part in &parts {
        let model = &models[&part.source.model];
        let compiled = if let Some(index) = sources
            .iter()
            .position(|source: &&Source| *source == &part.source)
        {
            CompiledModel::clone(&compiled_parts[index])
        } else {
            compiler.compile(
                model,
                &part.source,
                &dependencies[&part.source.model],
                &mut geometry,
            )?
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
