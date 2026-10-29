//! Compile Universal model declarations into reusable Bevy render definitions.
mod dependencies;
mod geometry;
mod options;
mod selection;

pub(crate) use dependencies::validate_dependencies;
use geometry::DrawGeometry;
pub use options::CompileOptions;

use crate::animation_targets::AnimationTargets;
use crate::bounds::rest_bounds;
use crate::child_models::ChildModelKey;
use crate::compiled::{ClipTiming, RuntimeLiquid, RuntimeNode};
use crate::coordinates::{axis, basis};
use crate::diagnostics::{RenderDiagnosticKind, RenderDiagnosticSource};
use crate::effects::RuntimeParticles;
use crate::events::RuntimeEvents;
use crate::features::RuntimeFeatures;
use crate::image_replacements::ImageBinding;
use crate::lod::LodDefinition;
use crate::material::compile_materials_with_images;
use crate::mesh_interner::MeshInterner;
use crate::physics::PhysicsDefinition;
use crate::textures::TextureImages;
use crate::world::WorldRuntime;
use crate::{
    CompiledModel, ModelError, ModelMaterial, PreparationContext, PreparationOperation,
    RenderDiagnostic, TextureInput, TextureRequest,
};
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::profiles::Universal;
use wc3::model::systems::{ModelMetadata, System};
use wc3::model::{
    AnimationProgram, Attachment, Billboard, Camera as ModelCamera, ClipProperties,
    Light as SemanticLight, Model, NodeFeature, ParticleEmitter, PoseLayout,
    RibbonEmitter as SemanticRibbonEmitter,
};

fn feature_name(feature: &NodeFeature<'_>) -> &'static str {
    match feature {
        NodeFeature::Light(_) => "Light",
        NodeFeature::Camera(_) => "Camera",
        NodeFeature::Attachment(_) => "Attachment",
        NodeFeature::Particle(_) => "Particle emitter",
        NodeFeature::Ribbon(_) => "Ribbon emitter",
        NodeFeature::GeometryAnimation(_) => "Geometry animation",
        NodeFeature::CollisionShape(_) => "Collision shape",
        NodeFeature::ParticleProgram(_) => "External particle programme",
        NodeFeature::FaceActor(_) => "Face animation",
        NodeFeature::LightProgram(_) => "Light programme",

        NodeFeature::Particle2(_) => "Atlas particles",
        NodeFeature::PointLight(_) => "Point light",
    }
}

pub(crate) fn node_order(nodes: &[RuntimeNode]) -> Result<Vec<usize>, ModelError> {
    PoseLayout::new(nodes.to_vec())
        .map(|layout| layout.into_parts().1)
        .map_err(ModelError::from_source)
}

/// Compile geometry, skin bindings, default appearances, surfaces, and animation
/// directly from a borrowed Universal model.
///
/// The image resolver receives the complete resource identity, including file-data
/// IDs, namespaces, replacement categories and non-UTF-8 paths. It owns sampler
/// and color-space variants. Limitations are available through diagnostics.
/// Invalid geometry and unsupported joint counts return an error atomically.
pub fn compile_model(
    model: &Model<'_, Universal>,
    resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
) -> Result<CompiledModel, ModelError> {
    compile_model_with_options(model, CompileOptions::default(), resolve_image)
}

/// Compile a model with an explicit authored appearance selection.
///
/// Resource handling and ownership are identical to [`compile_model`].
pub fn compile_model_with_options(
    model: &Model<'_, Universal>,
    options: CompileOptions,
    resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
) -> Result<CompiledModel, ModelError> {
    compile_with_geometry(model, options, resolve_image, &mut MeshInterner::default()).map_err(
        |error| error.with_context(PreparationContext::Operation(PreparationOperation::Compile)),
    )
}

/// Share fully compiled geometry within one preparation operation.
pub(crate) fn compile_with_geometry(
    model: &Model<'_, Universal>,
    options: CompileOptions,
    mut resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    mesh_interner: &mut MeshInterner,
) -> Result<CompiledModel, ModelError> {
    model.validate().map_err(|error| {
        ModelError::from_source(error).with_context(PreparationContext::Operation(
            PreparationOperation::Validate,
        ))
    })?;
    validate_dependencies(model, &options.child_models, &options.particle_effects)?;
    let targets = AnimationTargets::new(model);
    let node_map: HashMap<_, _> = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let layout = model.prepare_pose().map_err(|error| {
        ModelError::from_source(error).with_context(PreparationContext::Operation(
            PreparationOperation::Validate,
        ))
    })?;
    let (nodes, order) = layout.into_parts();
    let mut diagnostics = Vec::new();
    for (id, clip) in model.clips() {
        if let Some(ClipProperties::M2(controls)) = &clip.properties {
            if controls.blend_time != [0; 2] {
                diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unsupported, RenderDiagnosticSource::Clip(id), format!("M2 clip {id:?} ({}) declares incoming/outgoing blend durations {:?} milliseconds; native transition arbitration is not interpreted, so use Playback::play_with_blend for an explicit duration", clip.name(), controls.blend_time)));
            }
        }
    }

    let selected_materials = selection::materials(model, &options)?;
    let selected_textures = selection::textures(model, &selected_materials);
    let mut image_bindings = Vec::new();
    let mut resolve_symbol = |request: TextureRequest<'_, '_>| {
        let default = resolve_image(TextureRequest {
            input: request.input,
            texture: request.texture,
            resource: request.resource,
            role: request.role,
            color_space: request.color_space,
            sampler: request.sampler,
        });
        let symbol = Handle::from(Uuid::new_v4());
        image_bindings.push(ImageBinding {
            key: request.key(),
            resource: request.resource.clone().into_owned(),
            role: request.role,
            symbol: symbol.clone(),
            default,
        });
        Some(symbol)
    };
    let images = match selected_textures.as_ref() {
        Some(selected) => {
            TextureImages::resolve_selected(model, Some(selected), &mut resolve_symbol)
        }
        None => TextureImages::resolve(model, &mut resolve_symbol),
    };
    let materials = compile_materials_with_images(
        model,
        model.materials().iter().filter_map(|(id, _)| {
            selected_materials
                .as_ref()
                .is_none_or(|selected| selected.contains(&id))
                .then_some(id)
        }),
        &images,
        &targets,
    )?;
    for (&id, passes) in &materials {
        for pass in passes {
            diagnostics.extend(
                pass.diagnostics
                    .iter()
                    .cloned()
                    .map(|issue| issue.at(RenderDiagnosticSource::Material(id))),
            );
        }
    }
    let DrawGeometry { draws, skins } = geometry::compile_draw_geometry(
        model,
        &options,
        &node_map,
        &materials,
        mesh_interner,
        &mut diagnostics,
    )
    .map_err(|error| {
        error.with_context(PreparationContext::Operation(
            PreparationOperation::Geometry,
        ))
    })?;
    let particles = Arc::new(RuntimeParticles::compile(
        model,
        &node_map,
        &images,
        &materials,
        &options.child_models,
        &options.particle_effects,
    ));
    diagnostics.extend(
        particles
            .diagnostics()
            .iter()
            .map(|(feature, issue)| issue.clone().at(RenderDiagnosticSource::Feature(*feature))),
    );
    let features = Arc::new(RuntimeFeatures::compile(model, &options.child_models));
    diagnostics.extend(features.light_diagnostics().map(|(feature, message)| {
        RenderDiagnostic::new(
            RenderDiagnosticKind::Approximation,
            RenderDiagnosticSource::Feature(feature),
            message,
        )
    }));
    for (id, feature) in model.features() {
        if let NodeFeature::Attachment(attachment) = &feature.payload {
            if options
                .child_models
                .contains_key(&ChildModelKey::attachment(id))
                && attachment
                    .socket()
                    .is_some_and(|socket| targets.contains(&socket.visibility))
            {
                diagnostics.push(RenderDiagnostic::new(
                    RenderDiagnosticKind::Approximation,
                    RenderDiagnosticSource::Feature(id),
                    "particle-owned attachment startup with animated activation begins at readiness; its first subframe activation crossings are not replayed",
                ));
            }
        }
        let message = match &feature.payload {
            NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(_) | ParticleEmitter::MdlxEmitter2(_) | ParticleEmitter::M2(_))
            | NodeFeature::Ribbon(SemanticRibbonEmitter::Mdlx(_) | SemanticRibbonEmitter::M2(_))
            | NodeFeature::ParticleProgram(_) => continue,

            NodeFeature::GeometryAnimation(value) if value.mesh.is_some() && !value.drop_shadow => continue,
            NodeFeature::GeometryAnimation(_) => "geometry tint and opacity are rendered; orphan controllers and authored drop shadows need application interpretation".to_string(),
            NodeFeature::Camera(ModelCamera::Node(_) | ModelCamera::LookAt(_))
            | NodeFeature::Attachment(Attachment::Socket(_)) => continue,

            NodeFeature::Light(SemanticLight::SingleEmission(_) | SemanticLight::AmbientDiffuse(_)) =>
                "direct illumination uses Bevy scene lights; ambient terms remain available through ModelLight metadata".to_string(),
            NodeFeature::Light(SemanticLight::World(_) | SemanticLight::Point(_)) => continue,
            NodeFeature::Attachment(Attachment::Child(_)) if options.child_models.contains_key(&ChildModelKey::attachment(id)) => continue,
            NodeFeature::Attachment(Attachment::Child(_)) =>
                "external attachment identity and activation are exposed; application resource instantiation is required".to_string(),
            _ => format!("{} requires a visual interpreter or application integration", feature_name(&feature.payload)),
        };
        let kind = match feature.payload {
            NodeFeature::Attachment(Attachment::Child(_)) => RenderDiagnosticKind::Resource,
            NodeFeature::GeometryAnimation(_) | NodeFeature::Light(_) => {
                RenderDiagnosticKind::Approximation
            }
            _ => RenderDiagnosticKind::Unsupported,
        };
        diagnostics.push(RenderDiagnostic::new(
            kind,
            RenderDiagnosticSource::Feature(id),
            message,
        ));
    }
    let world = WorldRuntime::compile(model)?;
    diagnostics.extend(world.diagnostics);
    let mut liquids = Vec::new();
    for liquid in world.liquids {
        if options.draws.hidden_instances.contains(&liquid.instance) {
            continue;
        }
        let mesh = mesh_interner.intern(liquid.mesh);
        let surfaces = liquid.material.and_then(|id| materials.get(&id));
        for pass in 0..surfaces.map_or(1, Vec::len) {
            let surface = surfaces.map(|surfaces| {
                let surface = &surfaces[pass];
                (
                    ModelMaterial {
                        base: surface.material.clone(),
                        extension: surface.state.clone(),
                    },
                    Arc::new(surface.animation.clone()),
                )
            });
            liquids.push(RuntimeLiquid {
                instance: liquid.instance,
                node: node_map[&liquid.node],
                ordinal: liquid.ordinal,
                liquid_type: liquid.liquid_type,
                material_id: liquid.material,
                pass,
                mesh: mesh.clone(),
                surface,
            });
        }
    }
    for (id, system) in model.systems() {
        if matches!(system, System::Wmo(_) | System::Physics(_)) {
            continue;
        }
        diagnostics.push(RenderDiagnostic::new(
            RenderDiagnosticKind::Unsupported,
            RenderDiagnosticSource::System(id),
            "system remains in its authored/default state; application simulation is not active",
        ));
    }
    let lod = Arc::new(LodDefinition::compile(&draws, world.runtime.lod_levels()));
    let has_animation = draws.iter().any(|draw| draw.animation.uses_shared_time())
        || liquids.iter().any(|liquid| {
            liquid
                .surface
                .as_ref()
                .is_some_and(|(_, animation)| animation.uses_shared_time())
        })
        || model.tracks().next().is_some();
    let pose_animated = nodes.iter().any(|node| {
        node.channels.translation
            || node.channels.rotation
            || node.channels.scale
            || node.channels.visibility
    });
    let camera_sensitive = nodes
        .iter()
        .any(|node| node.camera_anchored || !matches!(node.billboard, Billboard::None));
    let physics = Arc::new(PhysicsDefinition::compile(
        model,
        &nodes,
        &mut diagnostics,
        options.physics_contact_pair_budget,
    ));
    let mut compiled = CompiledModel {
        image_template: None,
        image_bindings: image_bindings.into(),
        mounts: Vec::new(),
        physics,
        nodes: nodes.into(),
        blend_time: match model.metadata() {
            ModelMetadata::Mdlx(metadata) => metadata.blend_time,
            _ => 0.,
        },
        lod,
        has_animation,
        pose_animated,
        camera_sensitive,
        order: order.into(),
        animation: Arc::new(AnimationProgram::compile(model)),
        clips: model
            .clips()
            .iter()
            .map(|(id, clip)| (id, clip.name().to_string()))
            .collect::<Vec<_>>()
            .into(),
        skins,
        draws,
        diagnostics: diagnostics.into(),
        rest_bounds: None,
        basis: basis(model)?,
        up: axis(model.basis().up()),
        timing: Arc::new(
            model
                .clips()
                .iter()
                .map(|(id, _)| {
                    (
                        id,
                        ClipTiming {
                            blend_duration: None,
                        },
                    )
                })
                .collect(),
        ),
        features,
        events: Arc::new(RuntimeEvents::compile(model)),
        particles,
        world: Arc::new(world.runtime),
        liquids,
    };
    compiled.rest_bounds = rest_bounds(&compiled, model.bounds());
    let mut diagnostics = compiled.diagnostics.to_vec();
    for binding in compiled
        .image_bindings
        .iter()
        .filter(|binding| binding.default.is_none())
    {
        let source = match binding.key.input {
            Some(TextureInput::Material { material, .. }) => {
                RenderDiagnosticSource::Material(material)
            }
            Some(TextureInput::Particle { feature, .. } | TextureInput::Ribbon { feature, .. }) => {
                RenderDiagnosticSource::Feature(feature)
            }
            None => continue,
        };
        diagnostics.push(RenderDiagnostic::new(
            RenderDiagnosticKind::Resource,
            source,
            format!(
                "texture {:?} resource {:?} with role {:?} is unresolved",
                binding.key.resource, binding.resource, binding.role
            ),
        ));
    }
    compiled.diagnostics = diagnostics.into();
    let template = Arc::new(compiled.clone());
    let defaults: HashMap<_, _> = compiled
        .image_bindings
        .iter()
        .map(|binding| (binding.symbol.id(), binding.default.clone()))
        .collect();
    compiled.map_images(&mut |image| {
        defaults
            .get(&image.id())
            .cloned()
            .unwrap_or_else(|| Some(image.clone()))
    });
    compiled.image_template = Some(template);
    Ok(compiled)
}

#[cfg(test)]
mod tests;
