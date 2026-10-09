//! Deterministic model-space snapshots without an ECS world or renderer.
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::image::ImageSampler;
use bevy::math::Vec3A;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::array::from_fn;
use std::collections::{HashMap, HashSet};
use wc3::model::animation::AnimationTime;
use wc3::model::materials::{LayerFilterMode, ShaderType};

use super::mesh::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use super::PreparedModel;
use crate::animation::clocks::SamplingTime;
use crate::animation::pose::evaluation::{resolve_pose, NodeSample, PoseInput};
use crate::assets::ModelError;
use crate::materials::layers::AnimatedLayer;
use crate::materials::textures::{LinearImages, Wc3TextureBindings};
use crate::materials::Wc3LayerMaterial;

/// Source indices identifying a mesh pass within its owning model.
/// Empty geosets and shared mesh variants do not renumber these indices.
#[derive(Component, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Wc3PartId {
    /// Index in the normalized source geoset list.
    pub geoset: usize,
    /// Index in the normalized source material list.
    pub material: usize,
    /// Index within that material's normalized layers.
    pub layer: usize,
}

/// A prepared mesh pass and its original source identity.
#[derive(Clone, Copy)]
pub struct PreparedPart<'a> {
    /// Indices within the owning normalized model.
    pub id: Wc3PartId,
    /// Shared mesh asset, including the layer's selected UV set.
    pub mesh: &'a Handle<Mesh>,
    /// Authored geometry level; `None` denotes common geometry.
    pub lod: Option<u32>,
}

/// Explicit sampling clocks and optional camera for an offline pose.
/// Times are elapsed milliseconds, with the model clock looping or clamping
/// according to the selected sequence. No transitions or simulation are run.
#[derive(Clone, Copy, Debug, Default)]
pub struct Wc3PoseOptions {
    /// Selected sequence index; `None` samples static and global tracks only.
    pub sequence: Option<usize>,
    /// Elapsed milliseconds from the selected sequence's start.
    pub elapsed_ms: f64,
    /// Independent elapsed milliseconds for global sequence tracks.
    pub global_elapsed_ms: f64,
    /// Driving camera in model space, required for camera-dependent node flags.
    pub camera: Option<Transform>,
}

/// Texture role in a normalized material layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum Wc3TextureRole {
    /// Diffuse RGB and alpha.
    Diffuse = 0,
    /// Tangent-space normal data.
    Normal = 1,
    /// Occlusion, roughness, metallic, and team mask data.
    Orm = 2,
    /// Emissive color.
    Emissive = 3,
    /// Team color.
    Team = 4,
    /// Environment reflection color.
    Environment = 5,
}

/// Interpretation of a texture's RGB channels for this material use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wc3TextureColorSpace {
    /// Color data interpreted as sRGB.
    Srgb,
    /// Linear data such as normals and ORM.
    Linear,
}

/// Evaluated texture use, retaining provenance before linear image conversion.
#[derive(Clone, Debug)]
pub struct Wc3TextureReference {
    /// Index in the source bitmap list after evaluating texture animation.
    pub bitmap: usize,
    /// Literal source path, when present; lookup policy remains caller-owned.
    pub authored_path: Option<String>,
    /// Replaceable resource ID, when nonzero.
    pub replaceable_id: Option<u32>,
    /// Original resolved or overridden image, before any private linear clone.
    pub image: Option<Handle<Image>>,
    /// Sampler of the original image, or Bevy's default when unresolved.
    pub sampler: ImageSampler,
    /// Authored wrap flags for U and V. Resolved sampler overrides take precedence.
    pub authored_wrap: [bool; 2],
    /// Expected RGB interpretation for this use.
    pub color_space: Wc3TextureColorSpace,
}

/// Evaluated material state with source texture provenance for custom renderers.
#[derive(Clone)]
pub struct Wc3MaterialSnapshot {
    /// Bevy material, including evaluated tint, opacity, UVs, and image bindings.
    pub material: Wc3LayerMaterial,
    /// Diffuse, normal, ORM, emissive, team, and environment uses in role order.
    pub textures: [Option<Wc3TextureReference>; 6],
    /// Source shader identifier; unsupported shaders retain renderer fallback rules.
    pub shader: ShaderType,
    /// WC3 blend/filter mode.
    pub filter: LayerFilterMode,
    /// Whether the WC3 pass disables depth testing.
    pub no_depth_test: bool,
    /// Whether the WC3 pass disables depth writes.
    pub no_depth_write: bool,
    /// Evaluated linear Fresnel color.
    pub fresnel_color: [f32; 3],
    /// Evaluated Fresnel opacity.
    pub fresnel_opacity: f32,
    /// Evaluated Fresnel team-color contribution.
    pub fresnel_team_color: f32,
}

impl Wc3MaterialSnapshot {
    /// Returns evaluated provenance for the requested role, if that role exists.
    pub fn texture(&self, role: Wc3TextureRole) -> Option<&Wc3TextureReference> {
        self.textures[role as usize].as_ref()
    }
}

/// One mesh pass frozen into model space, including passes hidden by animation.
pub struct BakedModelPart {
    /// Original normalized source indices.
    pub id: Wc3PartId,
    /// Authored geometry level; `None` denotes common geometry.
    pub lod: Option<u32>,
    /// Posed geometry without joint attributes or skin bounds.
    pub mesh: Mesh,
    /// Evaluated material and original texture uses.
    pub material: Wc3MaterialSnapshot,
    /// Whether combined layer/geoset opacity is positive at the sample time.
    pub visible: bool,
    /// Bounds of this posed mesh in model space.
    pub bounds: Aabb,
}

/// A deterministic snapshot of all prepared mesh passes and authored levels.
/// Attachments, lights, cameras, events, and simulated effects are not baked.
pub struct BakedModelPose {
    /// Parts in source geoset/layer order, including invisible parts.
    pub parts: Vec<BakedModelPart>,
    /// Union of visible part bounds across every authored LOD, or `None` when empty.
    pub bounds: Option<Aabb>,
}

/// Reusable CPU pose baker, retaining private linear image variants across samples.
/// Original images must remain available in `Assets<Image>`. Call
/// [`Self::clear_image_cache`] after modifying source images in place.
#[derive(Default)]
pub struct Wc3PoseBaker {
    linear_images: LinearImages,
}

impl Wc3PoseBaker {
    /// Discards cached variant handles after image edits; existing snapshots keep
    /// their old image handles alive. Removed source images are pruned on baking.
    pub fn clear_image_cache(&mut self) {
        self.linear_images = LinearImages::default();
    }

    /// Evaluate animation and skin all prepared mesh passes without spawning.
    /// Source and global clocks are independent. Meshes, materials, texture-role
    /// animation, and node flags follow the live renderer's evaluation rules.
    ///
    /// # Errors
    /// Returns an error for invalid sequence indices, nonfinite clocks, malformed
    /// node hierarchies, missing meshes, or camera-dependent nodes without a camera.
    /// Degenerate skin transforms keep normals and tangents finite.
    pub fn bake(
        &mut self,
        prepared: &PreparedModel,
        meshes: &Assets<Mesh>,
        images: &mut Assets<Image>,
        bindings: &Wc3TextureBindings,
        options: Wc3PoseOptions,
    ) -> Result<BakedModelPose, ModelError> {
        if !options.elapsed_ms.is_finite() || !options.global_elapsed_ms.is_finite() {
            return Err(ModelError("pose clocks must be finite".into()));
        }
        if options.camera.is_some_and(|camera| {
            !camera.translation.is_finite()
                || !camera.rotation.is_finite()
                || camera.rotation.length_squared() <= 1e-12
                || !camera.scale.is_finite()
        }) {
            return Err(ModelError(
                "pose camera must have a finite transform and nonzero rotation".into(),
            ));
        }
        let sequences = prepared.model.sequences();
        let sequence = options
            .sequence
            .map(|index| {
                sequences
                    .get(index)
                    .ok_or_else(|| ModelError(format!("invalid pose sequence {index}")))
            })
            .transpose()?;
        let globals = prepared.model.global_sequences();
        let time = SamplingTime {
            model: AnimationTime {
                sequence,
                elapsed_ms: options.elapsed_ms,
                global_sequences: &globals,
            },
            global_elapsed_ms: options.global_elapsed_ms,
        };
        let joints = sample_joints(prepared, time, options.camera)?;
        let bindings = bindings.clone().with_defaults(prepared.textures.clone());
        self.linear_images.retain_loaded(images);
        let mut parts = Vec::new();
        let mut bounds: Option<Aabb> = None;
        let source_textures = prepared.model.textures();
        let source_materials = prepared.model.materials();
        for part in prepared.parts() {
            let layer = &prepared.layers[part.id.material][part.id.layer];
            let definition = &source_materials[part.id.material].layers[part.id.layer];
            let geoset = prepared.geoset_animations[part.id.geoset].as_ref();
            let mut material = layer.material.clone();
            material.extension.hd.maps.x = u32::from(layer.surface.hd);
            let animated_layer = AnimatedLayer {
                root: Entity::PLACEHOLDER,
                alpha: layer.alpha.clone(),
                geoset_alpha: geoset.map(|value| value.alpha.clone()),
                geoset_color: geoset
                    .filter(|value| value.flags.color())
                    .map(|value| value.color.clone()),
                texture_id: layer.texture_id.clone(),
            };
            let visible = animated_layer.evaluate(&mut material, Some(&bindings), time);
            layer.surface.evaluate(
                &mut material,
                &bindings,
                time,
                images,
                &mut self.linear_images,
            );
            let textures = from_fn(|role| {
                let id = if role == 0 {
                    Some(time.sample(&layer.texture_id).unwrap_or(0))
                } else if layer.surface.hd {
                    layer.surface.slots[role]
                        .as_ref()
                        .map(|value| time.sample(value).unwrap_or(0))
                } else {
                    None
                }? as usize;
                let bitmap = source_textures.get(id)?;
                let image = bindings.bitmap(id);
                Some(Wc3TextureReference {
                    bitmap: id,
                    authored_path: (!bitmap.path.text().is_empty())
                        .then(|| bitmap.path.text().into_owned()),
                    replaceable_id: (bitmap.replaceable_id != 0).then_some(bitmap.replaceable_id),
                    sampler: image
                        .as_ref()
                        .and_then(|handle| images.get(handle))
                        .map(|image| image.sampler.clone())
                        .unwrap_or_default(),
                    image,
                    authored_wrap: [bitmap.flags.wrap_width(), bitmap.flags.wrap_height()],
                    color_space: if role == 1 || role == 2 {
                        Wc3TextureColorSpace::Linear
                    } else {
                        Wc3TextureColorSpace::Srgb
                    },
                })
            });
            let mesh = meshes
                .get(part.mesh)
                .ok_or_else(|| ModelError(format!("missing prepared mesh for {:?}", part.id)))?;
            let mesh = bake_mesh(mesh, &joints)?;
            let part_bounds = mesh
                .compute_aabb()
                .ok_or_else(|| ModelError(format!("missing posed bounds for {:?}", part.id)))?;
            if visible {
                bounds = Some(match bounds {
                    None => part_bounds,
                    Some(previous) => {
                        let min = (previous.center - previous.half_extents)
                            .min(part_bounds.center - part_bounds.half_extents);
                        let max = (previous.center + previous.half_extents)
                            .max(part_bounds.center + part_bounds.half_extents);
                        Aabb {
                            center: (min + max) * 0.5,
                            half_extents: (max - min) * 0.5,
                        }
                    }
                });
            }
            let snapshot = Wc3MaterialSnapshot {
                shader: definition.shader_type(),
                filter: material.extension.filter,
                no_depth_test: material.extension.no_depth_test,
                no_depth_write: material.extension.no_depth_set,
                fresnel_color: material.extension.hd.fresnel_color.truncate().to_array(),
                fresnel_opacity: material.extension.hd.fresnel.x,
                fresnel_team_color: material.extension.hd.fresnel.y,
                material,
                textures,
            };
            parts.push(BakedModelPart {
                id: part.id,
                lod: part.lod,
                mesh,
                material: snapshot,
                visible,
                bounds: part_bounds,
            });
        }
        Ok(BakedModelPose { parts, bounds })
    }
}

fn sample_joints(
    prepared: &PreparedModel,
    time: SamplingTime<'_>,
    camera: Option<Transform>,
) -> Result<Vec<Mat4>, ModelError> {
    let nodes = prepared.model.nodes();
    let pivots = prepared.model.pivot_points();
    let mut inputs = HashMap::new();
    for node in &nodes {
        if node.object_id == u32::MAX || inputs.contains_key(&node.object_id) {
            return Err(ModelError(format!(
                "invalid or duplicate node ID {}",
                node.object_id
            )));
        }
        if node.flags.bits() & 0xf8 != 0 && camera.is_none() {
            return Err(ModelError(format!(
                "node {} requires an explicit pose camera",
                node.object_id
            )));
        }
        let pivot = Vec3::from_array(
            pivots
                .get(node.object_id as usize)
                .copied()
                .unwrap_or([0.0; 3]),
        );
        let parent_pivot = Vec3::from_array(
            pivots
                .get(node.parent_id as usize)
                .copied()
                .unwrap_or([0.0; 3]),
        );
        let offset = pivot - parent_pivot;
        let local = Transform {
            translation: offset
                + node
                    .translation
                    .as_ref()
                    .and_then(|track| time.track(track))
                    .map(Vec3::from_array)
                    .unwrap_or(Vec3::ZERO),
            rotation: node
                .rotation
                .as_ref()
                .and_then(|track| time.track(track))
                .map(Quat::from_array)
                .unwrap_or(Quat::IDENTITY),
            scale: node
                .scaling
                .as_ref()
                .and_then(|track| time.track(track))
                .map(Vec3::from_array)
                .unwrap_or(Vec3::ONE),
        };
        inputs.insert(node.object_id, (local, node.parent_id, node.flags, offset));
    }
    let root = u32::MAX;
    let lookup = |id| {
        if id == root {
            return Some(PoseInput {
                local: Transform::IDENTITY,
                parent: None,
                node: None,
                world: None,
                anchor: None,
            });
        }
        let &(local, parent, flags, offset) = inputs.get(&id)?;
        Some(PoseInput {
            local,
            parent: Some(parent),
            world: None,
            anchor: None,
            node: Some(NodeSample {
                root,
                flags,
                pivot_offset: offset,
                camera,
            }),
        })
    };
    let mut poses = HashMap::new();
    // Validate and resolve every node, including helpers not used directly as joints.
    for node in &nodes {
        resolve_pose(node.object_id, &lookup, &mut poses, &mut HashSet::new()).ok_or_else(
            || {
                ModelError(format!(
                    "invalid parent hierarchy at node {}",
                    node.object_id
                ))
            },
        )?;
    }
    Ok(prepared
        .joint_bindings
        .iter()
        .map(|(id, bind)| Mat4::from(poses[id].affine) * *bind)
        .collect())
}

fn bake_mesh(mesh: &Mesh, joints: &[Mat4]) -> Result<Mesh, ModelError> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .ok_or_else(|| ModelError("pose mesh has no positions".into()))?;
    let mut transforms = Vec::with_capacity(positions.len());
    for vertex in 0..positions.len() {
        let mut matrix = Mat4::ZERO;
        for (indices, weights) in [
            (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
            (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
        ] {
            if let (
                Some(VertexAttributeValues::Uint16x4(indices)),
                Some(VertexAttributeValues::Float32x4(weights)),
            ) = (mesh.attribute(indices), mesh.attribute(weights))
            {
                let indices = indices
                    .get(vertex)
                    .ok_or_else(|| ModelError("short pose skin indices".into()))?;
                let weights = weights
                    .get(vertex)
                    .ok_or_else(|| ModelError("short pose skin weights".into()))?;
                for (&index, &weight) in indices.iter().zip(weights) {
                    if weight > 0.0 {
                        matrix += *joints
                            .get(index as usize)
                            .ok_or_else(|| ModelError("invalid pose joint index".into()))?
                            * weight;
                    }
                }
            }
        }
        // Use the same weighted matrix sum as the GPU skinning shader.
        transforms.push(if mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
            matrix
        } else {
            Mat4::IDENTITY
        });
    }
    let mut baked = mesh.clone();
    let positions: Vec<_> = positions
        .iter()
        .zip(&transforms)
        .map(|(position, matrix)| {
            matrix
                .transform_point3(Vec3::from_array(*position))
                .to_array()
        })
        .collect();
    if positions
        .iter()
        .any(|position| !Vec3A::from_array(*position).is_finite())
    {
        return Err(ModelError("nonfinite posed positions".into()));
    }
    baked.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    if let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        baked.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            normals
                .iter()
                .zip(&transforms)
                .map(|(normal, matrix)| {
                    if matrix.determinant().abs() <= 1e-12 {
                        Vec3::ZERO
                    } else {
                        matrix
                            .inverse()
                            .transpose()
                            .transform_vector3(Vec3::from_array(*normal))
                            .normalize_or_zero()
                    }
                    .to_array()
                })
                .collect::<Vec<_>>(),
        );
    }
    if let Some(VertexAttributeValues::Float32x4(tangents)) =
        mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
    {
        baked.insert_attribute(
            Mesh::ATTRIBUTE_TANGENT,
            tangents
                .iter()
                .zip(&transforms)
                .map(|(tangent, matrix)| {
                    let sign = if matrix.determinant() < 0.0 {
                        -1.0
                    } else {
                        1.0
                    };
                    matrix
                        .transform_vector3(Vec3::from_slice(&tangent[..3]))
                        .normalize_or_zero()
                        .extend(tangent[3] * sign)
                        .to_array()
                })
                .collect::<Vec<_>>(),
        );
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
