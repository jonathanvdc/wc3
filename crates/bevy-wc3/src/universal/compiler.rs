//! Compile semantic graphs into owned immutable render definitions.
use super::coordinates::{axis, basis};
use super::features::RuntimeFeatures;
use super::geometry::{apply_skin, compile_geometry_with_uvs};
use super::material::compile_material_with_images;
use super::material::mesh_alpha_mode;
use super::mesh_animation::{MeshAnimation, MeshMaterialState};
use super::particles::RuntimeParticles;
use super::textures::TextureImages;
use super::{
    ClipTiming, CompiledUniversalModel, RenderDiagnostic, RuntimeDraw, RuntimeNode, RuntimeSkin,
    TextureRequest, UniversalMaterial,
};
use crate::ModelError;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimationProgram, Attachment, Billboard, Camera as ModelCamera, ClipControls,
    Light as ModelLight, Model, NodeFeature, ParticleDetails, ParticleEmitter, PropertyTarget,
    SkinId,
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

/// Compile geometry, skin bindings, default appearances, surfaces, and animation
/// directly from a borrowed Universal model.
///
/// The image resolver receives the complete resource identity, including file-data
/// IDs, namespaces, replacement categories and non-UTF-8 paths. It owns sampler
/// and color-space variants. Limitations are available through diagnostics.
/// Invalid geometry and unsupported joint counts return an error atomically.
pub fn compile_universal_model(
    model: &Model<'_, Universal>,
    mut resolve_image: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
) -> Result<CompiledUniversalModel, ModelError> {
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
    let mut draws = Vec::new();
    for (instance_id, instance) in model.instances() {
        let source = model
            .mesh(instance.mesh)
            .ok_or_else(|| ModelError("instance geometry is absent".into()))?;
        let skin_id = instance.skin.or(source.skin());
        for draw in &instance.draws {
            let material_id = model
                .material_for_draw(instance_id, draw.id(), None)
                .map_err(|error| ModelError(error.to_string()))?;
            let subset = source
                .subset(draw.subset)
                .ok_or_else(|| ModelError("draw subset is absent".into()))?;
            for (pass, surface) in materials[&material_id].iter().enumerate() {
                let key = (
                    instance.mesh,
                    draw.subset,
                    surface.coordinate,
                    surface.secondary_coordinate,
                    skin_id,
                );
                let meshes = if let Some(meshes) = geometry_cache.get(&key) {
                    meshes
                } else {
                    let mut parts = compile_geometry_with_uvs(
                        source,
                        subset,
                        surface.coordinate,
                        surface.secondary_coordinate,
                    )?;
                    if let Some(id) = skin_id {
                        let skin = model
                            .skin(id)
                            .ok_or_else(|| ModelError("instance skin is absent".into()))?;
                        for part in &mut parts {
                            apply_skin(part, source, &skin.joints)?;
                        }
                    }
                    let parts = parts
                        .into_iter()
                        .map(|mut part| {
                            part.mesh.merge_duplicate_vertices().map_err(|e| {
                                ModelError(format!("cannot index compiled mesh: {e}"))
                            })?;
                            Ok(Arc::new(part.mesh))
                        })
                        .collect::<Result<Vec<_>, ModelError>>()?;
                    geometry_cache.entry(key).or_insert(parts)
                };
                for mesh in meshes {
                    draws.push(RuntimeDraw {
                        instance: instance_id,
                        draw: draw.id(),
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
                        material: UniversalMaterial {
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
    for (id, feature) in model.features() {
        let message = match &feature.payload {
            NodeFeature::Particle(ParticleEmitter::Simulated(definition)) if matches!(definition.details,ParticleDetails::AgeDriven(_)|ParticleDetails::Atlas(_)) => continue,
            NodeFeature::GeometryAnimation(value) if value.mesh.is_some() && !value.drop_shadow && value.unclassified.bits() == 0 => continue,
            NodeFeature::GeometryAnimation(_) => "geometry tint and opacity are rendered; orphan controllers, authored drop shadows, and unclassified geometry flags need application interpretation".to_string(),
            NodeFeature::Camera(ModelCamera::Portable(_) | ModelCamera::Aimed(_))
            | NodeFeature::Attachment(Attachment::Point(_)) => continue,
            NodeFeature::Light(ModelLight::Portable(_) | ModelLight::AmbientDiffuse(_)) =>
                "direct illumination uses Bevy scene lights; ambient terms remain available through UniversalLight metadata".to_string(),
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
    for (id, _) in model.systems() {
        diagnostics.push(RenderDiagnostic { feature: None, material: None, message: format!("system {id:?} remains in its authored/default state; application simulation is not active") });
    }
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
    Ok(CompiledUniversalModel {
        nodes: nodes.into(),
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
                        },
                    ))
                })
                .collect(),
        ),
        features: Arc::new(RuntimeFeatures::compile(model)),
        particles,
    })
}
