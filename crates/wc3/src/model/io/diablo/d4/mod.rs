//! Diablo IV appearance conversion with explicit vertex interpretation.
//!
//! Native numeric vertex codes are not guessed. Supply checked `BufferLayout`
//! declarations for geometry import and a `VertexLayout` for output. Without
//! declarations, strict import rejects populated geometry while explicit loss
//! policy can retain the supported skeleton and material-reference graph.
mod vertices;
pub use vertices::{BufferLayout, ElementEncoding, ElementSemantic, VertexElement, VertexLayout};
mod export;
mod geometry;
mod materials;
use crate::model::conversion::{Context, Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::formats::diablo::d4::{self, records as native};
use crate::model::profiles::diablo::d4::{
    Appearance as LogicalAppearance, BoneControls, Chunk, Geometry, MeshControls, ModelMetadata,
    NameInfo, NodeRole, Slot, D4,
};
use crate::model::{
    Bounds, ExplicitBinding, Material, Mesh, MeshInstance, Model, Node, Quaternion, Skin, Transform,
};
use std::{array::from_fn, borrow::Cow, collections::BTreeMap};

/// Imports supported appearance declarations while requiring explicit loss for unmapped geometry.
pub fn import<'a>(
    source: &'a d4::Appearance<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, D4>>, Error> {
    import_with_layouts(source, &[], options)
}
/// Imports appearance geometry only through caller-established native vertex declarations.
pub fn import_with_layouts<'a>(
    source: &'a d4::Appearance<'_>,
    layouts: &[BufferLayout],
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, D4>>, Error> {
    let root = &source.root;
    let mut context = Context::import(options);
    unsupported(root, &mut context)?;
    let mut builder = Model::<D4>::builder("");
    builder.set_bounds(Some(bounds(&root.structure.aabb_bounds)));
    let mut controls = ModelMetadata::default();
    controls.sno_id = root.sno_id;
    controls.record_flags = root.record_flags;
    controls.flags = root.flags;
    controls.structure_flags = root.structure.flags;
    controls.vertex_animation_radius = root.structure.vertex_animation_radius;
    controls.low_quality_lod = root.structure.low_quality_lod;
    controls.lod_distance_multiplier = root.lod_distance_multiplier;
    controls.deformation_scale = root.deformation_scale;
    controls.fused_prefab = root.fused_prefab;
    controls.look_variant = root.look_variant;
    controls.structure_sphere_center = root.structure.bounds.center;
    controls.structure_sphere_radius = root.structure.bounds.wd_radius;
    let chunk_ids: Vec<_> = root
        .structure
        .chunks
        .iter()
        .map(|chunk| {
            controls.add_chunk(Chunk {
                bounds: bounds(&chunk.aabb_bounds),
                lod_distances: chunk.lods.iter().map(|lod| lod.lod_distance).collect(),
            })
        })
        .collect();
    let frame = builder.add_node(Node::named("").with_controls(NodeRole::ModelSpace))?;
    for chunk in &root.structure.chunks {
        if !chunk.lod_distance_per_sub_object.is_empty() {
            context.unsupported(
                "d4.geometry.subobject_lod_distances",
                "per-subobject LOD distance declarations have no semantic mapping",
            )?;
        }
    }
    let skeleton = root.structure.bone_data.first();
    let bones = skeleton.map(|s| s.bone_structure.as_slice()).unwrap_or(&[]);
    let bone_ids: Vec<_> = bones.iter().map(|_| builder.reserve_node()).collect();
    for (index, bone) in bones.iter().enumerate() {
        if !bone.shapes.is_empty() || !bone.constraint.is_empty() {
            context.unsupported(
                "d4.bones.physics",
                "bone collision and constraints have no supported semantic physics declaration",
            )?;
        }
        let control = BoneControls {
            hash: bone.hash,
            flags: bone.flags,
            lod: bone.lod,
            true_bone_lod: bone.true_bone_lod,
            unclassified_control: bone.unk_b03b557,
            particle_system: bone.particle_system,
            sphere_center: bone.bounds.center,
            sphere_radius: bone.bounds.wd_radius,
            bind_pose: transform(&bone.transform),
            inverse_bind_pose: transform(&bone.transform_inv),
            inverse_skinning_pose: transform(&bone.transform_skinning_inv),
        };
        let mut node = Node::named("")
            .with_controls(NodeRole::Bone(Box::new(control)))
            .with_transform(transform(&bone.transform_parent_rel))?;
        node.parent = if bone.parent_index == u16::MAX {
            None
        } else {
            Some(
                *bone_ids
                    .get(usize::from(bone.parent_index))
                    .ok_or_else(|| error("bone parent exceeds skeleton"))?,
            )
        };
        builder.set_node(bone_ids[index], node)?;
    }
    if let Some(skeleton) = skeleton {
        for hardpoint in &skeleton.hardpoints {
            let mut node = Node::named("")
                .with_controls(NodeRole::Hardpoint {
                    hash: hardpoint.info.hash,
                    hash_flags: hardpoint.info.hash_flag,
                    ignore_parent_orientation: hardpoint.ignore_parent_orientation,
                })
                .with_transform(Transform {
                    translation: hardpoint.transform.wp,
                    rotation: quaternion(&hardpoint.transform.q),
                    ..Default::default()
                })?;
            if hardpoint.bone_index < -1 {
                return Err(error(
                    "hardpoint parent uses an unsupported negative sentinel",
                ));
            }
            node.parent = if hardpoint.bone_index < 0 {
                None
            } else {
                Some(
                    *bone_ids
                        .get(nonnegative(hardpoint.bone_index, "hardpoint bone")?)
                        .ok_or_else(|| error("hardpoint bone exceeds skeleton"))?,
                )
            };
            builder.add_node(node)?;
        }
    }
    let skin = if bones.is_empty() {
        None
    } else {
        Some(
            builder.add_skin(Skin::new(
                bone_ids.clone(),
                ExplicitBinding::new(
                    bones
                        .iter()
                        .map(|b| matrix(transform(&b.transform_skinning_inv)))
                        .collect::<Vec<_>>(),
                )?,
            ))?,
        )
    };
    let mut appearance = LogicalAppearance::default();
    let mut material_sets = Vec::new();
    for slot in &root.appearance_materials {
        let slot_id = appearance.add_slot(Slot::new(
            slot.material_hash,
            slot.persona_material,
            slot.persona,
        ));
        let mut ids = Vec::new();
        for variant in &slot.so_as {
            let material = builder.add_material(Material {
                name: Cow::Borrowed(""),
                surface: materials::read_appearance(variant, &mut context)?,
            })?;
            ids.push(appearance.add_variant(slot_id, material)?);
        }
        if ids.is_empty() {
            return Err(error("appearance material slot has no alternatives"));
        }
        material_sets.push((slot_id, ids));
    }
    let mut mesh_ids = BTreeMap::new();
    for (chunk_index, chunk) in root.structure.chunks.iter().enumerate() {
        for (lod_index, lod) in chunk.lods.iter().enumerate() {
            for (sub_index, sub) in lod.sub_objects.iter().enumerate() {
                if layouts.iter().any(|l| {
                    l.chunk as usize == chunk_index && l.buffer as i32 == sub.vert_buffer_index
                }) {
                    let id = builder.add_mesh(Mesh::new(Geometry::default()))?;
                    mesh_ids.insert((chunk_index, lod_index, sub_index), id);
                } else {
                    context.unsupported("d4.geometry.layout","populated native subobject has no caller-established vertex interpretation")?;
                }
            }
        }
    }
    for (chunk_index, chunk) in root.structure.chunks.iter().enumerate() {
        for (lod_index, lod) in chunk.lods.iter().enumerate() {
            if !lod.base_sub_object_index_map.is_empty() {
                context.unsupported(
                    "d4.geometry.base_map",
                    "native base-subobject index map is not interpreted",
                )?;
            }
            for (sub_index, sub) in lod.sub_objects.iter().enumerate() {
                let Some(&id) = mesh_ids.get(&(chunk_index, lod_index, sub_index)) else {
                    continue;
                };
                if !sub.cloth_data.is_empty()
                    || !sub.unk_26f39c1.is_empty()
                    || !sub.shapes.is_empty()
                    || !sub.base_bone_influences.is_empty()
                {
                    context.unsupported(
                        "d4.geometry.physics",
                        "cloth, collision, and undeclared base-bone influences cannot be projected",
                    )?;
                }
                let layout = &layouts
                    .iter()
                    .find(|l| {
                        l.chunk as usize == chunk_index && l.buffer as i32 == sub.vert_buffer_index
                    })
                    .expect("selected source layout")
                    .layout;
                let (geometry, segments) =
                    geometry::read(sub, chunk, layout, &bone_ids, &mut context)?;
                let slot = nonnegative(sub.material_index, "material slot")?;
                let &(slot_id, ref _variants) = material_sets
                    .get(slot)
                    .ok_or_else(|| error("subobject material slot exceeds appearance materials"))?;
                let base = if sub.base_lod_sub_object_index < 0 {
                    None
                } else {
                    Some(
                        *mesh_ids
                            .get(&(chunk_index, 0, sub.base_lod_sub_object_index as usize))
                            .ok_or_else(|| {
                                error("base-detail subobject is missing or was not imported")
                            })?,
                    )
                };
                let control = MeshControls {
                    chunk: Some(chunk_ids[chunk_index]),
                    subobject_hash: sub.sub_object_hash,
                    material_slot: Some(slot_id),
                    flags: sub.flags,
                    name_info: name_info(&sub.name_info),
                    base_lod_subobject: base,
                    max_lod: sub.sub_object_max_lod,
                    shader_map_override: sub.shader_map_override,
                    campaign_visibility_condition: sub.campaign_visibility_condition,
                    fixed_point_position_offset: sub.fixed_point_pos_offset,
                    fixed_point_position_scale: sub.fixed_point_pos_scale,
                    unclassified_controls: [
                        sub.unk_334eb2d as u32,
                        sub.ba_segment,
                        sub.unk_87b4c64,
                        sub.ba_size,
                        sub.unk_4c43adc,
                        sub.unk_a1b71f5 as u32,
                        0,
                    ],
                    segments,
                };
                let weighted = !geometry.data().influences.is_empty();
                let mut mesh = Mesh::new(geometry).with_controls(control);
                mesh.bounds = bounds(&sub.aabb_bounds);
                builder.set_mesh(id, mesh)?;
                let mut instance = MeshInstance::new(frame, id);
                instance.skin = if weighted { skin } else { None };
                instance.lod = Some(count(lod_index)?);
                builder.add_instance(instance)?;
            }
        }
    }
    for (look_index, look) in root.appearance_looks.iter().enumerate() {
        let bindings = material_sets
            .iter()
            .map(|(slot, variants)| {
                variants
                    .get(look_index)
                    .copied()
                    .map(|variant| (*slot, variant))
                    .ok_or_else(|| {
                        error("material slot has fewer alternatives than declared looks")
                    })
            })
            .collect::<Result<_, _>>()?;
        appearance.add_look(look.look_name, bindings)?;
    }
    builder.set_appearance(appearance);
    builder.set_metadata(controls);
    Ok(Conversion {
        value: builder.finish()?,
        report: context.report,
    })
}
/// Plans native appearance output using an explicit complete vertex interpretation.
pub fn plan(
    asset: &Model<'_, D4>,
    layout: &VertexLayout,
    options: &ExportOptions,
) -> Result<Plan<d4::Appearance<'static>>, Error> {
    export::plan(asset, layout, options)
}
fn unsupported(root: &native::AppearanceRecord<'_>, context: &mut Context) -> Result<(), Error> {
    if root.structure.bone_data.len() > 1 {
        context.unsupported("d4.skeleton","multiple native bone-data blocks have no established logical skeleton-selection semantics")?;
    }
    if !root.structure.collision_meshes.is_empty()
        || !root.structure.collision_capsules.is_empty()
        || !root.structure.scene_data.is_empty()
    {
        context.unsupported(
            "d4.structure.physics",
            "structure collision and scene declarations have no supported semantic projection",
        )?;
    }
    if !root.look_variant_map.is_empty()
        || !root.speed_tree_wind.is_empty()
        || !root.base_appearance_texture_overrides.is_empty()
        || root.foliage_settings != Default::default()
    {
        context.unsupported("d4.appearance.extensions","look-variant maps, foliage, wind, and texture overrides have no supported semantic projection")?;
    }
    if let Some(skeleton) = root.structure.bone_data.first() {
        if skeleton.base_bone_count as usize != skeleton.bone_structure.len()
            || skeleton.cloth_bone_count != 0
        {
            context.unsupported("d4.skeleton.partition","only a complete base skeleton without a separate cloth-bone partition is supported")?;
        }
        let mut extra = skeleton.clone();
        extra.bone_structure.clear();
        extra.hardpoints.clear();
        extra.base_bone_count = 0;
        extra.cloth_bone_count = 0;
        if extra != Default::default() {
            context.unsupported("d4.skeleton.extensions","skeleton procedural, masking, cloth, and constraint declarations have no supported semantic projection")?;
        }
    }
    Ok(())
}
fn quaternion(value: &native::BcQuat) -> Quaternion {
    Quaternion([value.x, value.y, value.z, value.w])
}
fn transform(value: &native::PRSTransform) -> Transform {
    Transform {
        translation: value.wp,
        rotation: quaternion(&value.q),
        scale: value.scale,
        ..Default::default()
    }
}
fn native_transform(value: Transform) -> Result<native::PRSTransform, Error> {
    if value.pivot != [0.; 3] {
        return Err(error("native transforms do not store pivots"));
    }
    Ok(native::PRSTransform {
        q: native::BcQuat {
            x: value.rotation.0[0],
            y: value.rotation.0[1],
            z: value.rotation.0[2],
            w: value.rotation.0[3],
        },
        wp: value.translation,
        scale: value.scale,
    })
}
fn bounds(value: &native::Bounds) -> Bounds {
    Bounds {
        minimum: from_fn(|i| value.wp[i] - value.ext[i]),
        maximum: from_fn(|i| value.wp[i] + value.ext[i]),
        sphere_radius: value.ext.iter().map(|v| v * v).sum::<f32>().sqrt(),
    }
}
fn native_bounds(value: Bounds) -> native::Bounds {
    native::Bounds {
        wp: from_fn(|i| (value.minimum[i] + value.maximum[i]) * 0.5),
        ext: from_fn(|i| (value.maximum[i] - value.minimum[i]) * 0.5),
    }
}
fn matrix(transform: Transform) -> [[f32; 4]; 4] {
    let [x, y, z, w] = transform.rotation.0;
    let [sx, sy, sz] = transform.scale;
    let flat = [
        sx * (1. - 2. * (y * y + z * z)),
        sx * 2. * (x * y + z * w),
        sx * 2. * (x * z - y * w),
        0.,
        sy * 2. * (x * y - z * w),
        sy * (1. - 2. * (x * x + z * z)),
        sy * 2. * (y * z + x * w),
        0.,
        sz * 2. * (x * z + y * w),
        sz * 2. * (y * z - x * w),
        sz * (1. - 2. * (x * x + y * y)),
        0.,
        transform.translation[0],
        transform.translation[1],
        transform.translation[2],
        1.,
    ];
    from_fn(|col| from_fn(|row| flat[col * 4 + row]))
}
fn name_info(v: &native::SubObjectNameInfo) -> NameInfo {
    NameInfo {
        kind: v.type_,
        flags: v.flags,
        persona: v.persona,
        state: v.state,
        style: v.style,
        slot_hash: v.slot_hash,
        npc_component_hash: v.npc_component_name_hash,
        detail_hash: v.detail_hash,
        unclassified_controls: [v.unk_c15f7d2, v.unk_92fdd14, v.unk_b0b7d46],
    }
}
fn native_name_info(v: &NameInfo) -> native::SubObjectNameInfo {
    native::SubObjectNameInfo {
        type_: v.kind,
        flags: v.flags,
        persona: v.persona,
        state: v.state,
        style: v.style,
        slot_hash: v.slot_hash,
        npc_component_name_hash: v.npc_component_hash,
        detail_hash: v.detail_hash,
        unk_c15f7d2: v.unclassified_controls[0],
        unk_92fdd14: v.unclassified_controls[1],
        unk_b0b7d46: v.unclassified_controls[2],
        ..Default::default()
    }
}
fn count(value: usize) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| error("count exceeds u32"))
}
fn signed(value: usize) -> Result<i32, Error> {
    i32::try_from(value).map_err(|_| error("count exceeds native signed width"))
}
fn nonnegative(value: i32, field: &str) -> Result<usize, Error> {
    usize::try_from(value)
        .map_err(|_| Error::new(format!("d4.{field}"), "negative native index or size"))
}
fn error(message: &str) -> Error {
    Error::new("d4.appearance", message)
}

mod material_asset;
pub use material_asset::{import_material, plan_material};
