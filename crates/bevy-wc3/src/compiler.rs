//! Compile semantic graphs into owned immutable render definitions.
use super::bounds::rest_bounds;
use super::coordinates::{axis, basis};
use super::effects::{ParticleEffect, RuntimeParticles};
use super::events::RuntimeEvents;
use super::features::RuntimeFeatures;
use super::geometry::{apply_skin, compile_geometry_with_color};
use super::lod::LodDefinition;
use super::material::compile_materials_with_images;
use super::material::mesh_alpha_mode;
use super::mesh_animation::{multiply, state as mesh_state, MeshAnimation, MeshMaterialState};
use super::mesh_interner::MeshInterner;
use super::skin_bounds::compile_skin_bounds;
use super::textures::TextureImages;
use super::world::WorldRuntime;
use super::world_lighting::{illumination, ColorAdjustment};
use super::{
    CompiledModel, DrawPurpose, DrawSource, ModelMaterial, RenderDiagnostic, TextureInput,
    TextureRequest,
};
use crate::animation_targets::AnimationTargets;
use crate::child_models::{validate as validate_child_models, ChildModelBinding, ChildModelKey};
use crate::compiled::{ClipTiming, RuntimeDraw, RuntimeLiquid, RuntimeNode, RuntimeSkin};
use crate::composition::DrawSelection;
use crate::diagnostics::{RenderDiagnosticKind, RenderDiagnosticSource};
use crate::image_replacements::ImageBinding;
use crate::physics::{PhysicsDefinition, DEFAULT_CONTACT_PAIR_BUDGET};
use crate::ModelError;
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::profiles::wmo::Pass as WorldPass;
use wc3::model::properties::DrawProperties;

use wc3::model::profiles::Universal;
use wc3::model::systems::ModelMetadata;
use wc3::model::systems::System;
use wc3::model::{
    AnimationProgram, Attachment, Billboard, Camera as ModelCamera, ClipProperties, FeatureId,
    Light as SemanticLight, Link, MaterialId, MaterialSelection, Model, NodeFeature,
    ParticleEmitter, PoseLayout, RibbonEmitter as SemanticRibbonEmitter, SkinId, Texture,
    ValidateLinks, ValueSlot,
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
        .map_err(|error| ModelError(error.to_string()))
}

/// Application-selected authored state used during immutable model compilation.
#[derive(Clone, Debug)]
pub struct CompileOptions {
    /// Material appearance. Ambiguous defaults require an explicit selection.
    pub material: MaterialSelection,
    /// Authored geometry selection, independent of automatic LOD.
    pub draws: DrawSelection,
    /// Resolved optional models, keyed by their source feature and authored role.
    pub child_models: HashMap<ChildModelKey, ChildModelBinding>,
    /// Prepared external particle programmes, keyed by their authored Universal programme feature.
    /// Absent, foreign or wrong-family keys fail before image resolution, even for hidden geometry.
    pub particle_effects: HashMap<FeatureId, ParticleEffect>,
    /// Maximum potential unconnected-dynamic body pairs prepared across source physics systems.
    /// Sources remain isolated; their shared bound is checked before joint exclusions or quadratic enumeration.
    /// Zero disables preparation when any potential pairs exist. Composition also caps retained remapped pairs.
    pub physics_contact_pair_budget: usize,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            material: Default::default(),
            draws: Default::default(),
            child_models: Default::default(),
            particle_effects: Default::default(),
            physics_contact_pair_budget: DEFAULT_CONTACT_PAIR_BUDGET,
        }
    }
}

fn validate_particle_effects(
    model: &Model<'_, Universal>,
    bindings: &HashMap<FeatureId, ParticleEffect>,
) -> Result<(), ModelError> {
    for id in bindings.keys() {
        let feature = model.feature(*id).ok_or_else(|| {
            ModelError::new(format!(
                "particle programme binding {id:?} belongs to an absent or foreign feature"
            ))
        })?;
        if !matches!(feature.payload, NodeFeature::ParticleProgram(_)) {
            return Err(ModelError::new(format!(
                "particle programme binding {id:?} does not match an authored particle programme"
            )));
        }
    }
    Ok(())
}

/// Check source-scoped prepared dependency ownership before resolving images.
pub(crate) fn validate_dependencies(
    model: &Model<'_, Universal>,
    child_models: &HashMap<ChildModelKey, ChildModelBinding>,
    particle_effects: &HashMap<FeatureId, ParticleEffect>,
) -> Result<(), ModelError> {
    validate_child_models(model, child_models)?;
    validate_particle_effects(model, particle_effects)
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
    compile_with_geometry(model, options, resolve_image, &mut MeshInterner::default())
}

/// Share fully compiled geometry within one preparation operation.
pub(crate) fn compile_with_geometry(
    model: &Model<'_, Universal>,
    options: CompileOptions,
    mut resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    mesh_interner: &mut MeshInterner,
) -> Result<CompiledModel, ModelError> {
    model.validate().map_err(|e| ModelError(e.to_string()))?;
    validate_dependencies(model, &options.child_models, &options.particle_effects)?;
    let targets = AnimationTargets::new(model);
    let node_map: HashMap<_, _> = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let layout = model
        .prepare_pose()
        .map_err(|error| ModelError(error.to_string()))?;
    let (nodes, order) = layout.into_parts();
    let mut skins = Vec::new();
    let mut skin_cache: HashMap<(SkinId, Vec<usize>), usize> = HashMap::new();
    let mut diagnostics = Vec::new();
    for (id, clip) in model.clips() {
        if let Some(ClipProperties::M2(controls)) = &clip.properties {
            if controls.blend_time != [0; 2] {
                diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unsupported, RenderDiagnosticSource::Clip(id), format!("M2 clip {id:?} ({}) declares incoming/outgoing blend durations {:?} milliseconds; native transition arbitration is not interpreted, so use Playback::play_with_blend for an explicit duration", clip.name(), controls.blend_time)));
            }
        }
    }

    let selected_materials =
        if options.draws.draws.is_some() || !options.draws.hidden_instances.is_empty() {
            let mut selected = HashSet::new();
            for (id, instance) in model.instances() {
                if options.draws.hidden_instances.contains(&id) {
                    continue;
                }
                for draw in instance
                    .draws
                    .iter()
                    .filter(|draw| options.draws.includes(draw.id()))
                {
                    selected.insert(
                        model
                            .resolve_material(id, draw.id(), options.material)
                            .map_err(|e| ModelError(e.to_string()))?,
                    );
                }
            }
            for (_, feature) in model.features() {
                feature.payload.visit_links(&mut |link| {
                    if let Link::Material(id) = link {
                        selected.insert(id);
                    }
                });
            }
            for (_, system) in model.systems() {
                system.visit_links(&mut |link| {
                    if let Link::Material(id) = link {
                        selected.insert(id);
                    }
                });
            }
            Some(selected)
        } else {
            None
        };
    let selected_textures = selected_materials
        .as_ref()
        .map(|materials: &HashSet<MaterialId>| {
            let mut selected = HashSet::<Texture>::new();
            let mut animated = false;
            let mut visit = |link| match link {
                Link::SampledImage(id) => {
                    selected.insert(id);
                }
                Link::ImageResource(id) => {
                    selected.insert(Texture::new(id));
                }
                Link::Value(ValueSlot::Texture(_) | ValueSlot::OptionalTexture(_)) => {
                    animated = true;
                }
                _ => {}
            };
            for (_, feature) in model.features() {
                feature.payload.visit_links(&mut visit);
            }
            for (_, system) in model.systems() {
                system.visit_links(&mut visit);
            }
            for (id, material) in model.materials() {
                if materials.contains(&id) {
                    material.surface.visit_links(&mut visit);
                }
            }
            if animated {
                selected.extend(model.sampled_images());
            }
            selected
        });
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
    let images = Arc::new(match selected_textures.as_ref() {
        Some(selected) => {
            TextureImages::resolve_selected(model, Some(selected), &mut resolve_symbol)
        }
        None => TextureImages::resolve(model, &mut resolve_symbol),
    });
    let materials = compile_materials_with_images(
        model,
        model.materials().iter().filter_map(|(id, _)| {
            selected_materials
                .as_ref()
                .is_none_or(|selected| selected.contains(&id))
                .then_some(id)
        }),
        images.clone(),
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
    let mesh_animation = MeshAnimation::compile(model);
    let mut geometry_cache = HashMap::new();
    let mut draws = Vec::new();
    for (instance_id, instance) in model.instances() {
        if options.draws.hidden_instances.contains(&instance_id) {
            continue;
        }
        let source = model
            .mesh(instance.mesh)
            .ok_or_else(|| ModelError("instance geometry is absent".into()))?;
        let skin_id = instance.skin.or(source.skin());
        let color_adjustment = ColorAdjustment::new(model, instance);
        if let Some(message) = color_adjustment
            .as_ref()
            .and_then(ColorAdjustment::diagnostic)
        {
            diagnostics.push(RenderDiagnostic::new(
                RenderDiagnosticKind::Approximation,
                RenderDiagnosticSource::Instance(instance_id),
                format!("world region {instance_id:?}: {message}"),
            ));
        }
        let selections = instance
            .draws
            .iter()
            .filter(|draw| options.draws.includes(draw.id()))
            .map(|draw| {
                let material = model
                    .resolve_material(instance_id, draw.id(), options.material)
                    .map_err(|error| ModelError(error.to_string()))?;
                let subset = source
                    .subset(draw.subset)
                    .ok_or_else(|| ModelError("draw subset is absent".into()))?
                    .clone();
                Ok((
                    DrawSource::Draw(draw.id()),
                    Some(draw.subset),
                    material,
                    subset,
                    draw.properties.as_ref(),
                ))
            })
            .collect::<Result<Vec<_>, ModelError>>()?;
        for (draw_source, subset_id, material_id, subset, draw_controls) in selections {
            for (pass, surface) in materials[&material_id].iter().enumerate() {
                let key = (
                    instance.mesh,
                    subset_id,
                    surface.coordinate,
                    surface.secondary_coordinate,
                    surface.third_coordinate,
                    surface.fourth_coordinate,
                    skin_id,
                    color_adjustment.as_ref().map(|_| instance_id),
                );
                let meshes = if let Some(meshes) = geometry_cache.get(&key) {
                    meshes
                } else {
                    let parts = compile_geometry_with_color(
                        source,
                        &subset,
                        surface.coordinate,
                        [
                            surface.secondary_coordinate,
                            surface.third_coordinate,
                            surface.fourth_coordinate,
                        ],
                        |vertex, color| {
                            color_adjustment
                                .as_ref()
                                .map_or(color, |adjustment| adjustment.color(vertex, color))
                        },
                    )?;
                    let parts = parts
                        .into_iter()
                        .map(|mut part| {
                            let compact_skin = if let Some(id) = skin_id {
                                let skin = model
                                    .skin(id)
                                    .ok_or_else(|| ModelError("instance skin is absent".into()))?;
                                let retained = apply_skin(&mut part, source, &skin.joints)?;
                                let next = skins.len();
                                let index = *skin_cache
                                    .entry((id, retained.clone()))
                                    .or_insert_with(|| {
                                        let binds = skin.inverse_bind();
                                        skins.push(RuntimeSkin {
                                            joints: retained
                                                .iter()
                                                .map(|&slot| node_map[&skin.joints[slot]])
                                                .collect(),
                                            binds: retained
                                                .iter()
                                                .map(|&slot| {
                                                    binds.get(slot).expect("validated bind slot")
                                                })
                                                .collect::<Vec<_>>()
                                                .into(),
                                        });
                                        next
                                    });
                                compile_skin_bounds(&mut part.mesh)?;
                                Some(index)
                            } else {
                                None
                            };
                            part.mesh.merge_duplicate_vertices().map_err(|e| {
                                ModelError(format!("cannot index compiled mesh: {e}"))
                            })?;
                            Ok((mesh_interner.intern(part.mesh), compact_skin))
                        })
                        .collect::<Result<Vec<_>, ModelError>>()?;
                    geometry_cache.entry(key).or_insert(parts)
                };
                let mut extension = surface.state.clone();
                let (ambient, lighting) =
                    illumination(model, instance, draw_controls, surface.exterior_lit);
                let world_state = extension.set_world_illumination(ambient, lighting);
                for (mesh, skin) in meshes {
                    let mut material = ModelMaterial {
                        base: surface.material.clone(),
                        extension: extension.clone(),
                    };
                    if let Some(value) = mesh_animation.get(&instance.mesh) {
                        material.base.base_color =
                            multiply(material.base.base_color, value.defaults());
                        material.base.alpha_mode =
                            mesh_alpha_mode(material.base.alpha_mode, value.default_opacity());
                    }
                    material.extension.apply_opacity(&mut material.base);
                    draws.push(RuntimeDraw {
                        instance: instance_id,
                        source: draw_source,
                        mesh_id: instance.mesh,
                        material_id,
                        world_state,
                        mesh_state: mesh_animation
                            .get(&instance.mesh)
                            .map_or(MeshMaterialState::Plain, |value| {
                                mesh_state(value, instance.mesh)
                            }),
                        pass,
                        purpose: if matches!(draw_controls, Some(DrawProperties::Wmo(controls)) if matches!(&controls.pass, WorldPass::Shadow { .. })) {
                            DrawPurpose::ShadowOnly
                        } else { DrawPurpose::Surface },
                        node: node_map[&instance.node],
                        skin: *skin,
                        lod: instance.lod,
                        mesh: mesh.clone(),
                        material,
                        animation: Arc::new(surface.animation.clone().with_mesh(
                            surface.material.base_color,
                            surface.material.alpha_mode,
                            mesh_animation.get(&instance.mesh),
                        )),
                    });
                }
            }
        }
    }
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
