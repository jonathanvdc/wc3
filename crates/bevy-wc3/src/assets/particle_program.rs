//! Resolve baked effects at the asset boundary, preserving semantic feature ownership.
use super::images::ImageLoadCache;
use super::model::ResourceResolver;
use crate::diagnostics::{RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSource};
use crate::{CompileOptions, ModelError, ParticleEffect};
use bevy::asset::{io::AssetReaderError, AssetPath, LoadContext, ReadAssetBytesError};
use std::collections::HashMap;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{EffectProgram, MissingPayloadPolicy};
use wc3::model::{Model, NodeFeature};

pub(super) async fn resolve_programmes(
    model: &Model<'_>,
    options: &mut CompileOptions,
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    images: &mut ImageLoadCache,
) -> Result<Vec<RenderDiagnostic>, ModelError> {
    let mut diagnostics = Vec::new();
    let mut programs: HashMap<AssetPath<'static>, EffectProgram> = HashMap::new();
    for (feature, value) in model.features() {
        let NodeFeature::ParticleProgram(emitter) = &value.payload else {
            continue;
        };
        if options.particle_effects.contains_key(&feature) {
            continue;
        }
        let Some(resource) = emitter.resource.and_then(|id| model.effect_resource(id)) else {
            continue;
        };
        let candidates = resolver.particle_effect_candidates(feature, resource, owner);
        let mut resolved = None;
        let mut attempted = false;
        for path in &candidates {
            if let Some(program) = programs.get(path) {
                resolved = Some((path.clone(), program.clone()));
                break;
            }
            let bytes = match context.read_asset_bytes(path.clone()).await {
                Ok(bytes) => bytes,
                Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                    continue;
                }
                Err(error) => {
                    return Err(ModelError::new(format!(
                        "cannot read external particle programme at {path}: {error}"
                    )));
                }
            };
            attempted = true;
            let compiled = (|| {
                let document = Document::read(&bytes).map_err(|error| error.to_string())?;
                let mut roots = document.effect_roots();
                let (link, _) = roots
                    .next()
                    .ok_or("baked effect has no effect root")?
                    .map_err(|error| error.to_string())?;
                if roots.next().is_some() {
                    return Err(
                        "baked effect has multiple roots; select a programme explicitly".to_owned(),
                    );
                }
                document
                    .compile_effect(
                        link,
                        &EffectCompileOptions {
                            missing_payload: MissingPayloadPolicy::Zero,
                            ..Default::default()
                        },
                    )
                    .map_err(|error| error.to_string())
                    .map(|program| program.with_execution(options.particle_execution))
            })();
            match compiled {
                Ok(program) => {
                    programs.insert(path.clone(), program.clone());
                    resolved = Some((path.clone(), program));
                }
                Err(error) => diagnostics.push(RenderDiagnostic::new(
                    RenderDiagnosticKind::Unsupported,
                    RenderDiagnosticSource::Feature(feature),
                    format!("external particle programme at {path} cannot execute: {error}"),
                )),
            }
            break;
        }
        let Some((path, program)) = resolved else {
            if attempted {
                continue;
            }
            diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Resource,RenderDiagnosticSource::Feature(feature),format!("external particle programme {resource:?} could not be prepared; tried {candidates:?}")));
            continue;
        };
        let mut handles = HashMap::new();
        for request in ParticleEffect::texture_requests(&program) {
            let candidates = resolver.particle_effect_texture_candidates(feature, &request, &path);
            if let Some(handle) = images
                .resolve(context, candidates, request.color_space, request.sampler)
                .await?
            {
                handles.insert((request.renderer, request.binding), handle);
            }
        }
        let prepared = ParticleEffect::new(program, |request| {
            handles.get(&(request.renderer, request.binding)).cloned()
        });
        options.particle_effects.insert(feature, prepared);
    }
    Ok(diagnostics)
}
