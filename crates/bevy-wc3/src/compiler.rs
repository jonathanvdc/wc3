//! Compile semantic graphs into owned immutable render definitions.
use super::bounds::rest_bounds;
use super::coordinates::{axis, basis};
use super::effects::RuntimeParticles;
use super::events::RuntimeEvents;
use super::features::RuntimeFeatures;
use super::geometry::{apply_skin, compile_geometry_with_uvs, whole_geometry};
use super::lod::LodDefinition;
use super::material::compile_material_with_images;
use super::material::mesh_alpha_mode;
use super::mesh_animation::{MeshAnimation, MeshMaterialState};
use super::mesh_interner::MeshInterner;
use super::placements::PlacementSpace;
use super::pose::BillboardConvention;
use super::skin_bounds::compile_skin_bounds;
use super::textures::TextureImages;
use super::world::WorldRuntime;
use super::{
    CompiledModel, DrawSource, ModelAsset, ModelMaterial, RenderDiagnostic, TextureRequest,
};
use crate::compiled::{ClipTiming, RuntimeDraw, RuntimeLiquid, RuntimeNode, RuntimeSkin};
use crate::ModelError;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::profiles::Universal;
use wc3::model::systems::ModelMetadata;
use wc3::model::systems::System;
use wc3::model::{
    AnimationProgram, Attachment, Billboard, Camera as ModelCamera, ClipControls, FeatureId,
    Light as SemanticLight, MaterialSelection, MeshControls, Model, NodeControls, NodeFeature,
    ParticleDetails, ParticleEmitter, PropertyTarget, RibbonEmitter, SkinId,
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
        NodeFeature::Extent(_) => "Animated extent",
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

fn node_order(nodes: &[RuntimeNode]) -> Result<Vec<usize>, ModelError> {
    let mut order = Vec::with_capacity(nodes.len());
    let mut children = vec![Vec::new(); nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            children[parent].push(index);
        } else {
            order.push(index);
        }
    }
    let mut cursor = 0;
    while cursor < order.len() {
        let parent = order[cursor];
        order.extend(&children[parent]);
        cursor += 1;
    }
    if order.len() != nodes.len() {
        return Err(ModelError("model hierarchy contains a cycle".into()));
    }
    Ok(order)
}

/// Application-selected authored state used during immutable model compilation.
#[derive(Clone, Debug, Default)]
pub struct CompileOptions {
    /// Material appearance. Ambiguous defaults require an explicit selection.
    pub material: MaterialSelection,
    /// Resolved optional child models, keyed by the authored attachment or emitter.
    pub child_models: HashMap<FeatureId, Handle<ModelAsset>>,
    /// External child coordinate-space overrides; omitted features use authored space.
    pub model_placements: HashMap<FeatureId, PlacementSpace>,
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
    let node_map: HashMap<_, _> = model
        .nodes()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let nodes: Vec<_> = model
        .nodes()
        .map(|(id, node)| RuntimeNode {
            id,
            name: node.name.to_string(),
            parent: node.parent.map(|id| node_map[&id]),
            rest: node.transform,
            inheritance: node.inheritance,
            billboard: node.billboard,
            billboard_convention: if matches!(node.controls, Some(NodeControls::Mdlx(_))) {
                BillboardConvention::Warcraft3
            } else {
                BillboardConvention::Generic
            },
            visible: node.visible,
            camera_anchored: node.camera_anchored,
        })
        .collect();
    let order = node_order(&nodes)?;
    let skin_map: HashMap<SkinId, _> = model
        .skins()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let skins = model
        .skins()
        .map(|(_, skin)| RuntimeSkin {
            joints: skin.joints.iter().map(|id| node_map[id]).collect(),
            binds: skin
                .inverse_bind()
                .iter()
                .map(|matrix| Mat4::from_cols_array_2d(&matrix))
                .collect::<Vec<_>>()
                .into(),
        })
        .collect();
    let mut diagnostics = Vec::new();
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

    let images = Arc::new(TextureImages::resolve(model, &mut resolve_image));
    let mut materials = HashMap::new();
    for (id, _) in model.materials() {
        let passes = compile_material_with_images(model, id, images.clone())?;
        for pass in &passes {
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
        materials.insert(id, passes);
    }
    let mesh_animation = MeshAnimation::compile(model);
    let mut geometry_cache = HashMap::new();
    let mut mesh_interner = MeshInterner::default();
    let mut draws = Vec::new();
    for (instance_id, instance) in model.instances() {
        let source = model
            .mesh(instance.mesh)
            .ok_or_else(|| ModelError("instance geometry is absent".into()))?;
        let skin_id = instance.skin.or(source.skin());
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
            )]
        } else {
            instance
                .draws
                .iter()
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
                    ))
                })
                .collect::<Result<Vec<_>, ModelError>>()?
        };
        for (draw_source, subset_id, material_id, subset) in selections {
            for (pass, surface) in materials[&material_id].iter().enumerate() {
                let key = (
                    instance.mesh,
                    subset_id,
                    surface.coordinate,
                    surface.secondary_coordinate,
                    skin_id,
                );
                let meshes = if let Some(meshes) = geometry_cache.get(&key) {
                    meshes
                } else {
                    let mut parts = compile_geometry_with_uvs(
                        source,
                        &subset,
                        surface.coordinate,
                        surface.secondary_coordinate,
                    )?;
                    if let Some(id) = skin_id {
                        let skin = model
                            .skin(id)
                            .ok_or_else(|| ModelError("instance skin is absent".into()))?;
                        for part in &mut parts {
                            apply_skin(part, source, &skin.joints)?;
                            compile_skin_bounds(&mut part.mesh)?;
                        }
                    }
                    let parts = parts
                        .into_iter()
                        .map(|mut part| {
                            part.mesh.merge_duplicate_vertices().map_err(|e| {
                                ModelError(format!("cannot index compiled mesh: {e}"))
                            })?;
                            Ok(mesh_interner.intern(part.mesh))
                        })
                        .collect::<Result<Vec<_>, ModelError>>()?;
                    geometry_cache.entry(key).or_insert(parts)
                };
                for mesh in meshes {
                    draws.push(RuntimeDraw {
                        instance: instance_id,
                        source: draw_source,
                        mesh_id: instance.mesh,
                        material_id,
                        mesh_state: mesh_animation
                            .get(&instance.mesh)
                            .map_or(MeshMaterialState::Plain, |value| value.state(instance.mesh)),
                        pass,
                        node: node_map[&instance.node],
                        skin: skin_id.map(|id| skin_map[&id]),
                        lod: instance.lod,
                        mesh: mesh.clone(),
                        material: ModelMaterial {
                            base: {
                                let mut material = surface.material.clone();
                                if let Some(value) = mesh_animation.get(&instance.mesh) {
                                    material.base_color = value.apply_defaults(material.base_color);
                                    material.alpha_mode = mesh_alpha_mode(
                                        material.alpha_mode,
                                        value.default_opacity(),
                                    );
                                }
                                material
                            },
                            extension: surface.state.clone(),
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
        images.color_images().as_ref(),
        &materials,
        &options.child_models,
        &options.model_placements,
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
        &options.model_placements,
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
            | NodeFeature::Attachment(Attachment::Point(_)) => continue,
            NodeFeature::Light(SemanticLight::World(_) | SemanticLight::Specular(_) | SemanticLight::Point(_)) | NodeFeature::PointLight(_) => continue,
            NodeFeature::Light(SemanticLight::Portable(_) | SemanticLight::AmbientDiffuse(_)) =>
                "direct illumination uses Bevy scene lights; ambient terms remain available through ModelLight metadata".to_string(),
            NodeFeature::Attachment(Attachment::External(_)) if options.child_models.contains_key(&id) => continue,
            NodeFeature::Attachment(Attachment::External(_)) =>
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
        if matches!(system, System::Wmo(_)) {
            continue;
        }
        diagnostics.push(RenderDiagnostic { feature: None, material: None, message: format!("system {id:?} remains in its authored/default state; application simulation is not active") });
    }
    let lod = Arc::new(LodDefinition::compile(&draws, world.runtime.lod_levels()));
    let has_animation = !model.timeline_channels().is_empty()
        || !model.shared_channels().is_empty()
        || model.clips().any(|(_, clip)| !clip.channels().is_empty())
        || model
            .controllers()
            .any(|(_, controller)| !controller.channels.is_empty());
    let pose_animated = model
        .timeline_channels()
        .iter()
        .chain(model.shared_channels())
        .chain(model.clips().flat_map(|(_, clip)| clip.channels()))
        .chain(
            model
                .controllers()
                .flat_map(|(_, controller)| &controller.channels),
        )
        .any(|channel| {
            matches!(
                channel.target(),
                PropertyTarget::Translation(_)
                    | PropertyTarget::Rotation(_)
                    | PropertyTarget::Scale(_)
                    | PropertyTarget::Visibility(_)
            )
        });
    let camera_sensitive = nodes.iter().any(|node| {
        node.camera_anchored
            || !matches!(
                node.billboard,
                Billboard::None | Billboard::Locked { enabled: false, .. }
            )
    });
    let mut compiled = CompiledModel {
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
                .filter_map(|(id, clip)| {
                    let origin = match &clip.controls {
                        Some(ClipControls::Mdlx(controls)) => controls.start.as_seconds(),
                        Some(ClipControls::M3(controls)) => controls.timeline_origin.as_seconds(),
                        _ => return None,
                    };
                    Some((
                        id,
                        ClipTiming {
                            origin,
                            duration: clip.duration().as_seconds(),
                            looping: clip.is_looping(),
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
                    ))
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
