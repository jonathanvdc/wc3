//! Compile semantic graphs into owned immutable render definitions.
use super::bounds::rest_bounds;
use super::child_coordinates::CoordinatePolicy;
use super::coordinates::{axis, basis};
use super::effects::RuntimeParticles;
use super::events::RuntimeEvents;
use super::features::RuntimeFeatures;
use super::geometry::{apply_skin, compile_geometry_with_color, whole_geometry};
use super::lod::LodDefinition;
use super::material::compile_materials_with_images;
use super::material::mesh_alpha_mode;
use super::mesh_animation::{multiply, state as mesh_state, MeshAnimation, MeshMaterialState};
use super::mesh_interner::MeshInterner;
use super::pose::BillboardConvention;
use super::skin_bounds::compile_skin_bounds;
use super::textures::TextureImages;
use super::world::WorldRuntime;
use super::world_lighting::{illumination, ColorAdjustment};
use super::{
    CompiledModel, DrawSource, ModelAsset, ModelMaterial, RenderDiagnostic, TextureRequest,
};
use crate::animation_targets::AnimationTargets;
use crate::compiled::{ClipTiming, RuntimeDraw, RuntimeLiquid, RuntimeNode, RuntimeSkin};
use crate::composition::DrawSelection;
use crate::physics::PhysicsDefinition;
use crate::ModelError;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::profiles::m3::NodeRole as M3NodeRole;
use wc3::model::profiles::Universal;
use wc3::model::systems::ModelMetadata;
use wc3::model::systems::System;
use wc3::model::{
    AnimationProgram, Attachment, Billboard, Camera as ModelCamera, ClipControls, FeatureId,
    Light as SemanticLight, Link, MaterialId, MaterialSelection, MeshControls, Model, NodeControls,
    NodeFeature, ParticleDetails, ParticleEmitter, PoseLayout, RibbonEmitter, SkinId, Texture,
    ValidateLinks, ValueSlot,
};

fn feature_name(feature: &NodeFeature<'_>) -> &'static str {
    match feature {
        NodeFeature::Light(_) => "Light",
        NodeFeature::Camera(_) => "Camera",
        NodeFeature::Attachment(_) => "Attachment",
        NodeFeature::Particle(_) => "Particle emitter",
        NodeFeature::Ribbon(_) => "Ribbon emitter",
        NodeFeature::Event(_) => "Event source",
        NodeFeature::GeometryAnimation(_) => "Geometry animation",
        NodeFeature::CollisionShape(_) => "Collision shape",
        NodeFeature::ParticleProgram(_) => "External particle programme",
        NodeFeature::FaceActor(_) => "Face animation",
        NodeFeature::LightProgram(_) => "Light programme",
        NodeFeature::Extent(_) => "Property extent",
        NodeFeature::ParticleCopy(_) => "Particle copy",
        NodeFeature::Projector(_) => "Projector",
        NodeFeature::Force(_) => "Force",
        NodeFeature::Warp(_) => "Warp",
        NodeFeature::HitTest(_) => "Hit-test shape",
        NodeFeature::AttachmentVolume(_) => "Attachment volume",
        NodeFeature::Billboard(_) => "Billboard programme",
        NodeFeature::Turret(_) => "Turret",
        NodeFeature::IkJoint(_) => "IK joint",
        NodeFeature::IkTwoJoint(_) => "Two-joint IK solver",
        NodeFeature::IkCcd(_) => "CCD IK solver",
        NodeFeature::OneBoneSolver(_) => "One-bone solver",
        NodeFeature::ShadowBox(_) => "Shadow box",
        NodeFeature::ViewVolume(_) => "View volume",
        NodeFeature::AtlasParticles(_) => "Atlas particles",
        NodeFeature::PointLight(_) => "Point light",
    }
}

pub(crate) fn node_order(nodes: &[RuntimeNode]) -> Result<Vec<usize>, ModelError> {
    PoseLayout::new(nodes.to_vec())
        .map(|layout| layout.into_parts().1)
        .map_err(|error| ModelError(error.to_string()))
}

/// Application-selected authored state used during immutable model compilation.
#[derive(Clone, Debug, Default)]
pub struct CompileOptions {
    /// Material appearance. Ambiguous defaults require an explicit selection.
    pub material: MaterialSelection,
    /// Authored geometry selection, independent of automatic LOD.
    pub draws: DrawSelection,
    /// Resolved optional child models, keyed by the authored attachment or emitter.
    pub child_models: HashMap<FeatureId, Handle<ModelAsset>>,
    /// Child coordinate policies; omitted features use the host authored frame.
    pub child_coordinates: HashMap<FeatureId, CoordinatePolicy>,
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
    mut resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
) -> Result<CompiledModel, ModelError> {
    model.validate().map_err(|e| ModelError(e.to_string()))?;
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
    for (id, node) in model.nodes() {
        if let Some(NodeControls::M3(M3NodeRole::Bone(controls))) = &node.controls {
            if controls.extra_flags.bits() & 7 != 0 {
                diagnostics.push(RenderDiagnostic { feature: None, material: None, message: format!("M3 node {id:?} ({}) retains unresolved native inheritance flags; rendering uses ordinary parent transforms", node.name) });
            }
        }
    }
    for node in &nodes {
        if node.billboard_convention == BillboardConvention::M2
            && matches!(node.billboard, Billboard::Locked { enabled: true, .. })
        {
            diagnostics.push(RenderDiagnostic {
                feature: None,
                material: None,
                message: format!("M2 node {:?} ({}) uses axis-locked billboarding; generic rotation locks approximate its native camera-frame behavior", node.id, node.name),
            });
        }
    }
    for (id, clip) in model.clips() {
        if let Some(ClipControls::M2(controls)) = &clip.controls {
            if controls.blend_time != [0; 2] {
                diagnostics.push(RenderDiagnostic {
                    feature: None,
                    material: None,
                    message: format!("M2 clip {id:?} ({}) declares incoming/outgoing blend durations {:?} milliseconds; native transition arbitration is not interpreted, so use Playback::play_with_blend for an explicit duration", clip.name(), controls.blend_time),
                });
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
                if instance.draws.is_empty() {
                    if let Ok(material) = model.resolve_instance_material(id, options.material) {
                        selected.insert(material);
                    }
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
    let images = Arc::new(match selected_textures.as_ref() {
        Some(selected) => {
            TextureImages::resolve_selected(model, Some(selected), &mut resolve_image)
        }
        None => TextureImages::resolve(model, &mut resolve_image),
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
                    .map(|message| RenderDiagnostic {
                        feature: None,
                        material: Some(id),
                        message,
                    }),
            );
        }
    }
    let mesh_animation = MeshAnimation::compile(model);
    let mut geometry_cache = HashMap::new();
    let mut mesh_interner = MeshInterner::default();
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
            diagnostics.push(RenderDiagnostic {
                feature: None,
                material: None,
                message: format!("world region {instance_id:?}: {message}"),
            });
        }
        let implicit = instance.draws.is_empty()
            && match source.controls.as_ref() {
                Some(MeshControls::D3(value)) => value.material_slot.is_some(),
                Some(MeshControls::D4(value)) => value.material_slot.is_some(),
                _ => false,
            };
        let selections = if implicit {
            vec![(
                DrawSource::WholeInstance,
                None,
                model
                    .resolve_instance_material(instance_id, options.material)
                    .map_err(|error| ModelError(error.to_string()))?,
                whole_geometry(source)?,
                None,
            )]
        } else {
            instance
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
                        draw.controls.as_ref(),
                    ))
                })
                .collect::<Result<Vec<_>, ModelError>>()?
        };
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
                                                    Mat4::from_cols_array_2d(
                                                        &binds
                                                            .get(slot)
                                                            .expect("validated bind slot"),
                                                    )
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
                        node: node_map[&instance.node],
                        skin: *skin,
                        lod: instance.lod,
                        mesh: mesh.clone(),
                        material: ModelMaterial {
                            base: {
                                let mut material = surface.material.clone();
                                if let Some(value) = mesh_animation.get(&instance.mesh) {
                                    material.base_color =
                                        multiply(material.base_color, value.defaults());
                                    material.alpha_mode = mesh_alpha_mode(
                                        material.alpha_mode,
                                        value.default_opacity(),
                                    );
                                }
                                material
                            },
                            extension: extension.clone(),
                        },
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
        &options.child_coordinates,
    ));
    diagnostics.extend(
        particles
            .diagnostics()
            .iter()
            .map(|(feature, message)| RenderDiagnostic {
                feature: Some(*feature),
                material: None,
                message: message.clone(),
            }),
    );
    let features = Arc::new(RuntimeFeatures::compile(
        model,
        &options.child_models,
        &options.child_coordinates,
    ));
    diagnostics.extend(
        features
            .light_diagnostics()
            .map(|(feature, message)| RenderDiagnostic {
                feature: Some(feature),
                material: None,
                message: message.to_owned(),
            }),
    );
    for (id, feature) in model.features() {
        let message = match &feature.payload {
            NodeFeature::Particle(ParticleEmitter::Simulated(definition)) if matches!(definition.details,ParticleDetails::AgeDriven(_)|ParticleDetails::Atlas(_)|ParticleDetails::Model(_)|ParticleDetails::Programmed(_)) => continue,
            NodeFeature::Ribbon(RibbonEmitter::Trail(_) | RibbonEmitter::Programmed(_)) | NodeFeature::Event(_) => continue,
            NodeFeature::GeometryAnimation(value) if value.mesh.is_some() && !value.drop_shadow && value.unclassified.bits() == 0 => continue,
            NodeFeature::GeometryAnimation(_) => "geometry tint and opacity are rendered; orphan controllers, authored drop shadows, and unclassified geometry flags need application interpretation".to_string(),
            NodeFeature::Camera(ModelCamera::Portable(_) | ModelCamera::Aimed(_))
            | NodeFeature::Attachment(Attachment::Socket(_)) => continue,
            NodeFeature::Light(SemanticLight::World(_) | SemanticLight::Specular(_) | SemanticLight::Point(_)) | NodeFeature::PointLight(_) => continue,
            NodeFeature::Light(SemanticLight::Portable(_) | SemanticLight::AmbientDiffuse(_)) =>
                "direct illumination uses Bevy scene lights; ambient terms remain available through ModelLight metadata".to_string(),
            NodeFeature::Attachment(Attachment::Child(_)) if options.child_models.contains_key(&id) => continue,
            NodeFeature::Attachment(Attachment::Child(_)) =>
                "external attachment identity and activation are exposed; application resource instantiation is required".to_string(),
            _ => format!("{} requires a visual interpreter or application integration", feature_name(&feature.payload)),
        };
        diagnostics.push(RenderDiagnostic {
            feature: Some(id),
            material: None,
            message,
        });
    }
    let world = WorldRuntime::compile(model)?;
    diagnostics.extend(
        world
            .diagnostics
            .into_iter()
            .map(|message| RenderDiagnostic {
                feature: None,
                material: None,
                message,
            }),
    );
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
        diagnostics.push(RenderDiagnostic { feature: None, material: None, message: format!("system {id:?} remains in its authored/default state; application simulation is not active") });
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
    let camera_sensitive = nodes.iter().any(|node| {
        node.camera_anchored
            || !matches!(
                node.billboard,
                Billboard::None | Billboard::Locked { enabled: false, .. }
            )
    });
    let physics = Arc::new(PhysicsDefinition::compile(model, &nodes, &mut diagnostics));
    let mut compiled = CompiledModel {
        mounts: Vec::new(),
        physics,
        nodes: nodes.into(),
        blend_time: match model.metadata() {
            Some(ModelMetadata::Mdlx(metadata)) => metadata.blend_time,
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
                .map(|(id, clip)| {
                    let origin = clip.source_start.as_seconds();
                    (
                        id,
                        ClipTiming {
                            origin,
                            controllers: match &clip.controls {
                                Some(ClipControls::M3(controls)) => controls
                                    .selection
                                    .as_ref()
                                    .map(|selection| selection.controllers.clone())
                                    .unwrap_or_default()
                                    .into(),
                                _ => Arc::from([]),
                            },
                            blend_duration: match &clip.controls {
                                Some(ClipControls::M3(controls)) => {
                                    Some(controls.blend_duration.as_seconds())
                                }
                                _ => None,
                            },
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
    Ok(compiled)
}

#[cfg(test)]
#[path = "compiler_tests.rs"]
mod tests;
