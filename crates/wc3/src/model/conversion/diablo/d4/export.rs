use super::{
    count, error, geometry, materials, matrix, native_bounds, native_name_info, native_transform,
    signed, vertices::VertexLayout,
};
use crate::model::conversion::{
    Asset, Context, Error, ExportOptions, IssueKind, Plan, UnknownPolicy,
};
use crate::model::diablo::d4::{self, records as native};
use crate::model::semantic::profiles::diablo::d4::{NodeRole, D4};
use crate::model::semantic::{BindMatrices, BindingSchema as _, Bounds, MeshId};
use std::collections::BTreeMap;

pub(super) fn plan(
    asset: &Asset<'_, D4>,
    layout: &VertexLayout,
    options: &ExportOptions,
) -> Result<Plan<d4::Appearance<'static>>, Error> {
    let model = asset.model(options)?;
    let mut context = Context::export(options);
    if model.clips().len() != 0 {
        context.unsupported(
            "d4.animation",
            "compressed native animation curves require a separately established decoder",
        )?;
    }
    if !asset.opaque.is_empty() {
        match options.unknown {
            UnknownPolicy::Reject => {
                return Err(error(
                    "inactive source attachments require an explicit output policy",
                ))
            }
            UnknownPolicy::Archive => context.issue(
                "d4.opaque",
                IssueKind::Archived,
                "inactive source attachments remain outside rebuilt appearance output",
            ),
            UnknownPolicy::Drop => context.issue(
                "d4.opaque",
                IssueKind::Dropped,
                "inactive source attachments omitted from appearance output",
            ),
        }
    }
    if asset.models.textures().len() != 0 {
        context.unsupported("d4.resources.textures","appearance output cannot retain the collection image arena; texture inputs belong to separately planned material resources")?;
    }
    if options.look.is_some() {
        return Err(error(
            "Diablo IV appearance export preserves its authored alternatives",
        ));
    }
    if !model.name().is_empty() {
        context.unsupported(
            "d4.model.name",
            "native appearances store hashed identities without a model name string",
        )?;
    }
    for (_, node) in model.nodes() {
        if !node.name.is_empty() {
            context.unsupported(
                "d4.node.name",
                "native bone and hardpoint declarations store hashes without name strings",
            )?;
        }
    }
    for (_, material) in model.materials() {
        if !material.name.is_empty() {
            context.unsupported(
                "d4.material.name",
                "native material slots store hashes without per-surface name strings",
            )?;
        }
    }
    let controls = model.controls();
    let mut root = native::AppearanceRecord {
        sno_id: controls.sno_id,
        record_flags: controls.record_flags,
        flags: controls.flags,
        ..Default::default()
    };
    root.structure.flags = controls.structure_flags;
    root.structure.vertex_animation_radius = controls.vertex_animation_radius;
    root.structure.low_quality_lod = controls.low_quality_lod;
    root.lod_distance_multiplier = controls.lod_distance_multiplier;
    root.deformation_scale = controls.deformation_scale;
    root.fused_prefab = controls.fused_prefab;
    root.look_variant = controls.look_variant;
    check_aabb_radius(
        model
            .bounds()
            .ok_or_else(|| error("model bounds are required"))?,
        &mut context,
        "d4.bounds",
    )?;
    root.structure.aabb_bounds = native_bounds(
        model
            .bounds()
            .ok_or_else(|| error("model bounds are required"))?,
    );
    root.structure.bounds = native::Sphere {
        center: controls.structure_sphere_center,
        wd_radius: controls.structure_sphere_radius,
    };
    let bones: Vec<_> = model
        .nodes()
        .filter(|(_, node)| matches!(node.controls, NodeRole::Bone(_)))
        .collect();
    let bone_indices: BTreeMap<_, _> = bones
        .iter()
        .enumerate()
        .map(|(index, (id, _))| Ok((*id, count(index)?)))
        .collect::<Result<_, Error>>()?;
    let mut skeleton = native::BoneData::default();
    for (_, node) in &bones {
        let NodeRole::Bone(control) = &node.controls else {
            unreachable!()
        };
        let parent = match node.parent {
            None => u16::MAX,
            Some(id) => u16::try_from(
                *bone_indices
                    .get(&id)
                    .ok_or_else(|| error("bone parent is not a skeleton bone"))?,
            )
            .map_err(|_| error("bone parent exceeds u16"))?,
        };
        skeleton.bone_structure.push(native::BoneStructure {
            hash: control.hash,
            flags: control.flags,
            parent_index: parent,
            lod: control.lod,
            true_bone_lod: control.true_bone_lod,
            unk_b03b557: control.unclassified_control,
            particle_system: control.particle_system,
            bounds: native::Sphere {
                center: control.sphere_center,
                wd_radius: control.sphere_radius,
            },
            transform: native_transform(control.bind_pose)?,
            transform_inv: native_transform(control.inverse_bind_pose)?,
            transform_parent_rel: native_transform(node.transform)?,
            transform_skinning_inv: native_transform(control.inverse_skinning_pose)?,
            ..Default::default()
        });
    }
    skeleton.base_bone_count = signed(bones.len())?;
    for (_, node) in model.nodes() {
        if let NodeRole::Hardpoint {
            hash,
            hash_flags,
            ignore_parent_orientation,
        } = &node.controls
        {
            if node.transform.scale != [1.; 3] || node.transform.pivot != [0.; 3] {
                return Err(error("hardpoints require rigid transforms"));
            }
            let trs = native_transform(node.transform)?;
            skeleton.hardpoints.push(native::Hardpoint {
                info: native::HardpointInfo {
                    hash: *hash,
                    hash_flag: *hash_flags,
                },
                bone_index: match node.parent {
                    None => -1,
                    Some(id) => signed(
                        *bone_indices
                            .get(&id)
                            .ok_or_else(|| error("hardpoint parent is not a bone"))?
                            as usize,
                    )?,
                },
                ignore_parent_orientation: *ignore_parent_orientation,
                transform: native::PRTransform {
                    q: trs.q,
                    wp: trs.wp,
                },
            });
        }
    }
    if !skeleton.bone_structure.is_empty() || !skeleton.hardpoints.is_empty() {
        root.structure.bone_data.push(skeleton);
    }
    let chunks: Vec<_> = controls.chunks().collect();
    let chunk_indices: BTreeMap<_, _> = chunks
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();
    for (_, chunk) in &chunks {
        check_aabb_radius(chunk.bounds, &mut context, "d4.chunk.bounds")?;
        let native_chunk = native::GeoChunk {
            aabb_bounds: native_bounds(chunk.bounds),
            lods: chunk
                .lod_distances
                .iter()
                .map(|&distance| native::GeoSet {
                    lod_distance: distance,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        root.structure.chunks.push(native_chunk);
    }
    let appearance = model.appearance();
    let slots: Vec<_> = appearance.slots().collect();
    let slot_indices: BTreeMap<_, _> = slots
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();
    let alternatives: Vec<_> = appearance.alternatives().collect();
    for (_, alternative) in &alternatives {
        root.appearance_looks.push(native::AppearanceLook {
            look_name: alternative.hash,
        });
    }
    for (slot_id, slot) in &slots {
        let mut selected = Vec::new();
        let mut variants = Vec::new();
        for (_, alternative) in &alternatives {
            let variant = alternative
                .bindings()
                .find_map(|(owner, variant)| (owner == *slot_id).then_some(variant))
                .ok_or_else(|| {
                    error("appearance alternative does not select every material slot")
                })?;
            let material = slot
                .variants()
                .find_map(|(id, material)| (id == variant).then_some(material))
                .ok_or_else(|| error("alternative selects a variant outside its slot"))?;
            variants.push(materials::write_appearance(
                &model
                    .material(material)
                    .ok_or_else(|| error("slot material is absent"))?
                    .surface,
                &mut context,
            )?);
            selected.push(variant);
        }
        for (variant, material) in slot.variants() {
            if !selected.contains(&variant) {
                variants.push(materials::write_appearance(
                    &model
                        .material(material)
                        .ok_or_else(|| error("slot material is absent"))?
                        .surface,
                    &mut context,
                )?);
            }
        }
        root.appearance_materials.push(native::AppearanceMaterial {
            material_hash: slot.hash,
            persona_material: slot.persona_material,
            persona: slot.persona,
            so_as: variants,
            ..Default::default()
        });
    }
    let mut locations: BTreeMap<MeshId, (usize, usize, usize)> = BTreeMap::new();
    let mut pending_bases = Vec::new();
    let mut used_materials = Vec::new();
    for (_, instance) in model.instances() {
        let mesh = model
            .mesh(instance.mesh)
            .ok_or_else(|| error("placement mesh is absent"))?;
        if !mesh.name.is_empty() {
            context.unsupported(
                "d4.mesh.name",
                "native subobjects store hashes without mesh name strings",
            )?;
        }
        if locations.contains_key(&instance.mesh) {
            return Err(error(
                "native appearance requires one placement per logical subobject",
            ));
        }
        let node = model
            .node(instance.node)
            .ok_or_else(|| error("placement frame is absent"))?;
        if !matches!(node.controls, NodeRole::ModelSpace)
            || !instance.draws.is_empty()
            || instance.selection_group != 0
        {
            return Err(error("native subobjects require the implicit model frame and appearance-owned material slots"));
        }
        let chunk = mesh
            .controls
            .chunk
            .and_then(|id| chunk_indices.get(&id).copied())
            .ok_or_else(|| error("mesh chunk is absent"))?;
        let lod = instance.lod.unwrap_or(0) as usize;
        if lod >= root.structure.chunks[chunk].lods.len() {
            return Err(error(
                "placement detail level exceeds its chunk's declarations",
            ));
        }
        let material = mesh
            .controls
            .material_slot
            .and_then(|id| slot_indices.get(&id).copied())
            .ok_or_else(|| error("mesh material slot is absent"))?;
        for (_, id) in slots[material].1.variants() {
            used_materials.push(id);
        }
        let mut palette = Vec::new();
        if let Some(skin_id) = instance.skin {
            let skin = model
                .skin(skin_id)
                .ok_or_else(|| error("placement skin is absent"))?;
            let BindMatrices::Explicit(matrices) = skin.binding.matrices(skin.joints.len()) else {
                return Err(error(
                    "native skeleton output requires explicit inverse skinning matrices",
                ));
            };
            for (index, id) in skin.joints.iter().enumerate() {
                let NodeRole::Bone(control) = &model
                    .node(*id)
                    .ok_or_else(|| error("skin bone is absent"))?
                    .controls
                else {
                    return Err(error("skin selects a non-bone node"));
                };
                if matrices.get(index) != Some(matrix(control.inverse_skinning_pose)) {
                    return Err(error(
                        "skin matrix contradicts its bone's authored inverse skinning pose",
                    ));
                }
                palette.push(*id);
            }
        } else if !mesh.geometry.data().influences.is_empty() {
            return Err(error("weighted geometry requires a placement skin"));
        }
        let encoded = geometry::write(
            &mesh.geometry,
            &mesh.controls.segments,
            layout,
            &bone_indices,
            &palette,
            &mut context,
        )?;
        check_aabb_radius(mesh.bounds, &mut context, "d4.mesh.bounds")?;
        let c = &mesh.controls;
        if c.unclassified_controls[6] != 0 {
            context.unsupported(
                "d4.mesh.control",
                "the seventh extension control has no established native field",
            )?;
        }
        let vertex_buffer = signed(root.structure.chunks[chunk].chunk_vertex_buffers.len())?;
        let index_buffer = signed(root.structure.chunks[chunk].chunk_index_buffers.len())?;
        root.structure.chunks[chunk]
            .chunk_vertex_buffers
            .push(encoded.vertex_buffer);
        root.structure.chunks[chunk]
            .chunk_index_buffers
            .push(encoded.index_buffer);
        let sub = native::SubObject {
            flags: c.flags,
            unk_334eb2d: c.unclassified_controls[0] as i32,
            ba_segment: c.unclassified_controls[1],
            unk_87b4c64: c.unclassified_controls[2],
            ba_size: c.unclassified_controls[3],
            unk_4c43adc: c.unclassified_controls[4],
            name_info: native_name_info(&c.name_info),
            material_index: signed(material)?,
            sub_object_hash: c.subobject_hash,
            vert_buffer_index: vertex_buffer,
            index_buffer_index: index_buffer,
            base_lod_sub_object_index: -1,
            unk_a1b71f5: c.unclassified_controls[5] as i32,
            sub_object_max_lod: c.max_lod,
            shader_map_override: c.shader_map_override,
            campaign_visibility_condition: c.campaign_visibility_condition,
            aabb_bounds: native_bounds(mesh.bounds),
            segments: encoded.segments,
            fixed_point_pos_offset: c.fixed_point_position_offset,
            fixed_point_pos_scale: c.fixed_point_position_scale,
            ..Default::default()
        };
        let index = root.structure.chunks[chunk].lods[lod].sub_objects.len();
        root.structure.chunks[chunk].lods[lod].sub_objects.push(sub);
        locations.insert(instance.mesh, (chunk, lod, index));
        if let Some(base) = c.base_lod_subobject {
            pending_bases.push((instance.mesh, base));
        }
    }
    for (mesh, base) in pending_bases {
        let &(chunk, lod, index) = locations.get(&mesh).expect("exported subobject");
        let &(base_chunk, base_lod, base_index) = locations
            .get(&base)
            .ok_or_else(|| error("base-detail mesh has no native placement"))?;
        if chunk != base_chunk || base_lod != 0 {
            return Err(error(
                "base-detail mesh must belong to the same chunk at detail level zero",
            ));
        }
        root.structure.chunks[chunk].lods[lod].sub_objects[index].base_lod_sub_object_index =
            signed(base_index)?;
    }
    if model.meshes().any(|(id, _)| !locations.contains_key(&id)) {
        context.unsupported(
            "d4.geometry.unplaced",
            "native appearance cannot retain a mesh without a placement",
        )?;
    }
    if model.materials().any(|(id, _)| {
        !slots
            .iter()
            .any(|(_, slot)| slot.variants().any(|(_, material)| material == id))
    }) {
        context.unsupported(
            "d4.material.unassigned",
            "material has no authored slot variant",
        )?;
    }
    Ok(Plan::new(d4::Appearance::new(root), context.report))
}

fn check_aabb_radius(bounds: Bounds, context: &mut Context, path: &str) -> Result<(), Error> {
    let native = native_bounds(bounds);
    let radius = native
        .ext
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if (radius - bounds.sphere_radius).abs() > f32::EPSILON * radius.max(1.) * 8. {
        context.unsupported(path,"independent semantic sphere radius has no native AABB field; structure sphere declarations remain separate")?;
    }
    Ok(())
}
