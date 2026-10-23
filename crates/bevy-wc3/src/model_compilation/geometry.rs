//! Selected draw geometry, shared compact skin palettes and material passes.
use super::CompileOptions;
use crate::compiled::{RuntimeDraw, RuntimeSkin};
use crate::diagnostics::{RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSource};
use crate::geometry::{apply_skin, compile_geometry_with_color};
use crate::material::{mesh_alpha_mode, SurfacePass};
use crate::mesh_animation::{multiply, state as mesh_state, MeshAnimation, MeshMaterialState};
use crate::mesh_interner::MeshInterner;
use crate::skin_bounds::compile_skin_bounds;
use crate::world_lighting::{illumination, ColorAdjustment};
use crate::{DrawPurpose, DrawSource, ModelError, ModelMaterial};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::profiles::wmo::Pass as WorldPass;
use wc3::model::profiles::Universal;
use wc3::model::properties::DrawProperties;
use wc3::model::{MaterialId, Model, NodeId, SkinId};

pub(super) struct DrawGeometry {
    pub draws: Vec<RuntimeDraw<Arc<Mesh>>>,
    pub skins: Vec<RuntimeSkin<Arc<[Mat4]>>>,
}

pub(super) fn compile_draw_geometry(
    model: &Model<'_, Universal>,
    options: &CompileOptions,
    node_map: &HashMap<NodeId, usize>,
    materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    mesh_interner: &mut MeshInterner,
    diagnostics: &mut Vec<RenderDiagnostic>,
) -> Result<DrawGeometry, ModelError> {
    let mut skins = Vec::new();
    let mut skin_cache: HashMap<(SkinId, Vec<usize>), usize> = HashMap::new();
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
                let coordinates = surface.coordinates_for_mesh(
                    source
                        .texture_coordinates()
                        .get(1)
                        .is_some_and(|uv| !uv.is_empty()),
                );
                let key = (
                    instance.mesh,
                    subset_id,
                    surface.coordinate,
                    coordinates[0],
                    coordinates[1],
                    coordinates[2],
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
                        coordinates,
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
    Ok(DrawGeometry { draws, skins })
}
