//! Deterministic mesh/material snapshots without an ECS world or effect simulation.
use super::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use super::pose::{evaluate_pose, EvaluatedPose};
use super::{
    ModelDraw, ModelMaterial, Playback, PoseOverrides, PreparedModel, WorldLiquidSurface,
    WorldSelection,
};
use crate::error::ModelError;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, VertexAttributeValues};
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::{skin_matrices, DeformableGeometry};

/// Borrowed geometry-pass inputs retained by a prepared model.
pub struct PreparedPart<'a> {
    /// Stable semantic surface identity.
    pub source: ModelDraw,
    /// Undeformed shared source mesh.
    pub mesh: &'a Handle<Mesh>,
    /// Authored material defaults with resolved image handles.
    pub material: &'a ModelMaterial,
}
impl PreparedModel {
    /// Inspect shared source surfaces without exposing compiler-owned storage.
    pub fn parts(&self) -> impl ExactSizeIterator<Item = PreparedPart<'_>> {
        self.draws.iter().map(|draw| PreparedPart {
            source: ModelDraw {
                instance: draw.instance,
                source: draw.source,
                mesh: draw.mesh_id,
                material: draw.material_id,
                pass: draw.pass,
                lod: draw.lod,
            },
            mesh: &draw.mesh,
            material: &draw.material,
        })
    }
}
/// Coordinate space of baked geometry and bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakeSpace {
    /// Include the authored-to-Bevy coordinate frame. Apply only the desired
    /// application root transform when spawning the resulting meshes.
    #[default]
    Bevy,
    /// Keep the model's authored coordinate frame and physical units.
    Authored,
}
/// Explicit clocks and application pose inputs for an offline snapshot.
#[derive(Clone, Copy, Default)]
pub struct PoseOptions<'a> {
    /// Playback clocks and selected clip/controllers; absent samples rest clocks.
    pub playback: Option<&'a Playback>,
    /// Explicit clocks for independently mounted parts, keyed by their zero-based
    /// ordinal path through `PreparedModel::mounts`. Without an entry, synchronized
    /// parts follow their clip mapping; independent parts use host elapsed seconds.
    /// Supply live child playback here after independent seeks or clock edits.
    pub mounted_playback: Option<&'a HashMap<Vec<usize>, Playback>>,
    /// Application local-pose overrides, applied after authored animation.
    pub overrides: Option<&'a PoseOverrides>,
    /// Driving camera in authored model coordinates. Required for camera-sensitive nodes.
    pub camera: Option<Transform>,
    /// Select one LOD for visibility. Absent retains every authored level.
    pub lod: Option<u32>,
    /// World-group selection; absent uses the runtime's conservative default.
    pub world_selection: Option<&'a WorldSelection>,
    /// Coordinate frame of output meshes and bounds.
    pub space: BakeSpace,
}
/// Complete semantic provenance of a baked surface.
#[derive(Clone, Copy, Debug)]
pub enum BakedSource {
    /// Authored draw or implicit whole-instance geometry selection.
    Geometry(ModelDraw),
    /// Generated authored liquid grid, including its external liquid type.
    Liquid(WorldLiquidSurface),
}
/// A surface frozen into the selected coordinate frame, including hidden surfaces.
pub struct BakedPart {
    /// Source identities independent of mesh-sharing or output order.
    pub source: BakedSource,
    /// Posed geometry without joint attributes or skin bounds.
    pub mesh: Mesh,
    /// Evaluated PBR material, retaining its resolved image handles and sampling.
    /// A liquid requiring an application material provider retains `None`.
    pub material: Option<ModelMaterial>,
    /// Combined authored node visibility, material opacity and selected LOD.
    pub visible: bool,
    /// Finite posed bounds; absent for an empty mesh.
    pub bounds: Option<Aabb>,
}
/// Offline snapshot. Events, transitions, effects and external child models are not simulated.
pub struct BakedPose {
    /// All mesh passes and liquid surfaces in prepared order.
    pub parts: Vec<BakedPart>,
    /// Union of visible surface bounds in the selected coordinate frame.
    pub bounds: Option<Aabb>,
    /// Clocks and programme selection used for this snapshot.
    pub playback: Playback,
    /// Optional LOD visibility selection used for this snapshot.
    pub lod: Option<u32>,
    /// Coordinate frame of the snapshot.
    pub space: BakeSpace,
}
/// Reusable CPU evaluator. Source mesh/bind assets are borrowed and never mutated;
/// role-specific image interpretation already belongs to prepared materials.
#[derive(Default)]
pub struct PoseBaker {
    poses: Vec<EvaluatedPose>,
    visible: Vec<bool>,
    joints: Vec<Mat4>,
    transforms: Vec<Mat4>,
}
impl PoseBaker {
    /// Sample animation, deform up to eight influences, and snapshot materials.
    /// Returns an error for invalid clocks/selections, missing source assets,
    /// malformed skin streams or camera-dependent poses without a finite camera.
    pub fn bake(
        &mut self,
        prepared: &PreparedModel,
        meshes: &Assets<Mesh>,
        binds: &Assets<SkinnedMeshInverseBindposes>,
        options: PoseOptions<'_>,
    ) -> Result<BakedPose, ModelError> {
        self.bake_at(prepared, meshes, binds, options, &[])
    }
    fn bake_at(
        &mut self,
        prepared: &PreparedModel,
        meshes: &Assets<Mesh>,
        binds: &Assets<SkinnedMeshInverseBindposes>,
        options: PoseOptions<'_>,
        path: &[usize],
    ) -> Result<BakedPose, ModelError> {
        let default_playback = Playback::default();
        let playback = options.playback.unwrap_or(&default_playback);
        let selection = playback
            .selection(prepared)
            .map_err(|error| ModelError(error.to_string()))?;
        let sample = prepared
            .animation
            .sample(selection)
            .map_err(|error| ModelError(error.to_string()))?;
        if prepared.camera_sensitive && options.camera.is_none() {
            return Err(ModelError(
                "pose requires a driving camera in authored coordinates".into(),
            ));
        }
        if options.camera.is_some_and(|camera| {
            !camera.translation.is_finite()
                || !camera.rotation.is_finite()
                || camera.rotation.length_squared() <= 1e-12
                || !camera.scale.is_finite()
                || camera.scale.abs().min_element() <= 1e-12
        }) {
            return Err(ModelError(
                "pose camera must have a finite nonsingular transform".into(),
            ));
        }
        let camera = options.camera.map(|camera| camera.compute_affine());
        self.poses
            .resize(prepared.nodes.len(), EvaluatedPose::default());
        self.visible.resize(prepared.nodes.len(), true);
        for &index in prepared.order.iter() {
            let node = &prepared.nodes[index];
            let pose = options
                .overrides
                .and_then(|overrides| overrides.0.get(&node.id))
                .copied()
                .unwrap_or_else(|| node.sample(&sample));
            self.poses[index] = evaluate_pose(
                node,
                pose,
                node.parent
                    .map_or_else(EvaluatedPose::default, |parent| self.poses[parent]),
                camera,
            );
            self.visible[index] = node.sample_visibility(&sample)
                && node.parent.is_none_or(|parent| self.visible[parent]);
        }
        let default_selection = WorldSelection::default();
        let world_visibility = prepared
            .world
            .visibility(options.world_selection.unwrap_or(&default_selection));
        let basis = match options.space {
            BakeSpace::Bevy => prepared.basis.to_matrix(),
            BakeSpace::Authored => Mat4::IDENTITY,
        };
        let mut result = BakedPose {
            parts: Vec::with_capacity(prepared.draws.len() + prepared.liquids.len()),
            bounds: None,
            playback: playback.clone(),
            lod: options.lod,
            space: options.space,
        };
        for draw in prepared.draws.iter() {
            let source = meshes
                .get(&draw.mesh)
                .ok_or_else(|| ModelError("baking source mesh is missing".into()))?;
            self.joints.clear();
            if let Some(index) = draw.skin {
                let skin = &prepared.skins[index];
                let palette = binds
                    .get(&skin.binds)
                    .ok_or_else(|| ModelError("baking inverse bind palette is missing".into()))?;
                if palette.len() != skin.joints.len() {
                    return Err(ModelError(
                        "baking bind palette has an inconsistent joint count".into(),
                    ));
                }
                self.joints.extend(
                    skin.joints
                        .iter()
                        .zip(palette.iter())
                        .map(|(joint, bind)| basis * self.poses[*joint].matrix * *bind),
                );
            }
            let mesh = bake_mesh(
                source,
                &self.joints,
                basis * self.poses[draw.node].matrix,
                &mut self.transforms,
            )?;
            let mut material = draw.material.clone();
            draw.animation.apply_to_material(&sample, &mut material);
            let visible = self.visible[draw.node]
                && prepared
                    .world
                    .instance_visible(draw.instance, &world_visibility)
                && material.base.base_color.alpha() > 0.
                && options
                    .lod
                    .is_none_or(|lod| draw.lod.is_none_or(|level| level == lod));
            let bounds = mesh.compute_aabb();
            include_bounds(&mut result.bounds, bounds.filter(|_| visible));
            result.parts.push(BakedPart {
                source: BakedSource::Geometry(ModelDraw {
                    instance: draw.instance,
                    source: draw.source,
                    mesh: draw.mesh_id,
                    material: draw.material_id,
                    pass: draw.pass,
                    lod: draw.lod,
                }),
                mesh,
                material: Some(material),
                visible,
                bounds,
            });
        }
        for liquid in prepared.liquids.iter() {
            let source = meshes
                .get(&liquid.mesh)
                .ok_or_else(|| ModelError("baking liquid mesh is missing".into()))?;
            let mesh = bake_mesh(
                source,
                &[],
                basis * self.poses[liquid.node].matrix,
                &mut self.transforms,
            )?;
            let material = liquid.surface.as_ref().map(|(material, animation)| {
                let mut material = material.clone();
                animation.apply_to_material(&sample, &mut material);
                material
            });
            let visible = self.visible[liquid.node]
                && prepared
                    .world
                    .instance_visible(liquid.instance, &world_visibility)
                && options.lod.is_none_or(|lod| {
                    prepared
                        .world
                        .instance_lod(liquid.instance)
                        .is_none_or(|level| level == lod)
                })
                && material
                    .as_ref()
                    .is_none_or(|material| material.base.base_color.alpha() > 0.);
            let bounds = mesh.compute_aabb();
            include_bounds(&mut result.bounds, bounds.filter(|_| visible));
            result.parts.push(BakedPart {
                source: BakedSource::Liquid(WorldLiquidSurface {
                    instance: liquid.instance,
                    ordinal: liquid.ordinal,
                    liquid_type: liquid.liquid_type,
                    material: liquid.material_id,
                }),
                mesh,
                material,
                visible,
                bounds,
            });
        }
        for (ordinal, mount) in prepared.mounts.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(ordinal);
            let frame = self.poses[mount.parent].matrix * mount.transform.to_matrix();
            let mut child_playback = playback.clone();
            child_playback.controllers = Default::default();
            child_playback.clip = playback
                .clip
                .and_then(|clip| mount.clips.get(&clip).copied())
                .or_else(|| mount.model.clips.first().map(|(clip, _)| *clip));
            if let Some(explicit) = options
                .mounted_playback
                .and_then(|values| values.get(&child_path))
            {
                child_playback.clone_from(explicit);
            }
            let mut child = PoseBaker::default().bake_at(
                &mount.model,
                meshes,
                binds,
                PoseOptions {
                    playback: Some(&child_playback),
                    camera: options
                        .camera
                        .map(|camera| Transform::from_matrix(frame.inverse() * camera.to_matrix())),
                    space: BakeSpace::Authored,
                    lod: options.lod,
                    mounted_playback: options.mounted_playback,
                    ..Default::default()
                },
                &child_path,
            )?;
            for mut part in child.parts.drain(..) {
                part.mesh = bake_mesh(&part.mesh, &[], basis * frame, &mut self.transforms)?;
                part.visible &= self.visible[mount.parent]
                    && mount
                        .socket
                        .is_none_or(|point| prepared.features.attachment_active(point, &sample));
                part.bounds = part.mesh.compute_aabb();
                include_bounds(&mut result.bounds, part.bounds.filter(|_| part.visible));
                result.parts.push(part);
            }
        }
        Ok(result)
    }
}
fn include_bounds(bounds: &mut Option<Aabb>, next: Option<Aabb>) {
    if let Some(next) = next {
        *bounds = Some(match *bounds {
            Some(current) => Aabb::from_min_max(
                current.min().min(next.min()).into(),
                current.max().max(next.max()).into(),
            ),
            None => next,
        });
    }
}

fn bake_mesh(
    mesh: &Mesh,
    joints: &[Mat4],
    placement: Mat4,
    transforms: &mut Vec<Mat4>,
) -> Result<Mesh, ModelError> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .ok_or_else(|| ModelError("pose mesh has no positions".into()))?;
    if !joints.is_empty() && !mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        return Err(ModelError(
            "skinned pose mesh is missing its base joint stream".into(),
        ));
    }
    for (attribute, valid) in [
        (
            Mesh::ATTRIBUTE_NORMAL,
            matches!(
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
                None | Some(VertexAttributeValues::Float32x3(_))
            ),
        ),
        (
            Mesh::ATTRIBUTE_TANGENT,
            matches!(
                mesh.attribute(Mesh::ATTRIBUTE_TANGENT),
                None | Some(VertexAttributeValues::Float32x4(_))
            ),
        ),
    ] {
        if !valid {
            return Err(ModelError(
                "unsupported pose normal or tangent format".into(),
            ));
        }
        if mesh
            .attribute(attribute)
            .is_some_and(|values| values.len() != positions.len())
        {
            return Err(ModelError(
                "pose attribute has inconsistent vertex count".into(),
            ));
        }
    }
    for (indices, weights) in [
        (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
        (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
    ] {
        match (mesh.attribute(indices), mesh.attribute(weights)) {
            (None, None) => (),
            (
                Some(VertexAttributeValues::Uint16x4(indices)),
                Some(VertexAttributeValues::Float32x4(weights)),
            ) if indices.len() == positions.len()
                && weights.len() == positions.len()
                && weights
                    .iter()
                    .flatten()
                    .all(|weight| weight.is_finite() && *weight >= 0.) => {}
            _ => return Err(ModelError("malformed pose skin attribute pair".into())),
        }
    }
    if mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        let rows = (0..positions.len()).map(|vertex| {
            [
                (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
                (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
            ]
            .into_iter()
            .flat_map(move |(indices, weights)| {
                match (mesh.attribute(indices), mesh.attribute(weights)) {
                    (
                        Some(VertexAttributeValues::Uint16x4(indices)),
                        Some(VertexAttributeValues::Float32x4(weights)),
                    ) => Some(
                        indices[vertex]
                            .into_iter()
                            .zip(weights[vertex])
                            .map(|(id, weight)| (usize::from(id), weight)),
                    ),
                    _ => None,
                }
                .into_iter()
                .flatten()
            })
        });
        skin_matrices(rows, joints, transforms).map_err(|error| ModelError(error.to_string()))?;
    } else {
        transforms.clear();
        transforms.resize(positions.len(), placement);
    }
    let mut baked = mesh.clone();
    let mut positions = positions.to_vec();
    let mut normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
        _ => Vec::new(),
    };
    let mut tangents = match mesh.attribute(Mesh::ATTRIBUTE_TANGENT) {
        Some(VertexAttributeValues::Float32x4(values)) => values.clone(),
        _ => Vec::new(),
    };
    DeformableGeometry {
        positions: &mut positions,
        normals: &mut normals,
        tangents: &mut tangents,
    }
    .deform(transforms)
    .map_err(|error| ModelError(error.to_string()))?;
    baked.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
        baked.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
    if mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT) {
        baked.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    }
    for attribute in [
        Mesh::ATTRIBUTE_JOINT_INDEX,
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        EXTRA_JOINT_INDEX,
        EXTRA_JOINT_WEIGHT,
    ] {
        baked.remove_attribute(attribute);
    }
    baked.set_skinned_mesh_bounds(None);
    Ok(baked)
}
