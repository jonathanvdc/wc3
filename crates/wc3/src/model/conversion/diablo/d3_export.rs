//! Authoritative logical relationships determine native array indices and look tables.
use super::{
    geometry, i32_count, materials, matrix, name, native, native_bounds, native_transform,
};
use crate::model::conversion::select_model;
use crate::model::conversion::{Context, Error, ExportOptions, Plan};
use crate::model::diablo::d3::{Appearance, V260};
use crate::model::semantic::profiles::diablo::d3::{LookAtTarget, ModelControls, NodeRole, D3};
use crate::model::semantic::Asset;
use crate::model::semantic::BindMatrices;
use std::collections::BTreeMap;
use zerovec::ZeroVec;
pub(super) fn plan(
    asset: &Asset<'_, D3>,
    options: &ExportOptions,
) -> Result<Plan<Appearance<'static, V260>>, Error> {
    plan_with_animations(asset, options, false)
}
pub(super) fn plan_with_animations(
    asset: &Asset<'_, D3>,
    options: &ExportOptions,
    animations: bool,
) -> Result<Plan<Appearance<'static, V260>>, Error> {
    let model = select_model(asset, options)?;
    let mut context = Context::export(options);
    if !model.name().is_empty() {
        context.unsupported(
            "d3.model.name",
            "native appearance resources do not declare a model display name",
        )?;
    }
    if !animations && model.clips().len() != 0 {
        context.unsupported(
            "d3.animation",
            "appearance output requires separately planned native animation documents",
        )?;
    }

    let mut root = native::AppearanceRecord::default();

    let look_at_name = match &model.controls().look_at.target {
        LookAtTarget::Bone(id) => {
            let node = model
                .node(*id)
                .ok_or_else(|| Error::new("d3.look_at.target", "look-at bone is absent"))?;
            if !matches!(node.controls, NodeRole::Bone(_))
                || model
                    .nodes()
                    .filter(|(_, candidate)| {
                        matches!(candidate.controls, NodeRole::Bone(_))
                            && candidate.name == node.name
                    })
                    .count()
                    != 1
            {
                return Err(Error::new(
                    "d3.look_at.target",
                    "look-at target must be a uniquely named actual skeleton bone",
                ));
            }
            node.name.as_ref()
        }
        LookAtTarget::Unresolved(name) => {
            if model.controls().look_at.enabled != 0 {
                return Err(Error::new(
                    "d3.look_at.target",
                    "active look-at target must resolve to an actual skeleton bone",
                ));
            }
            name.as_str()
        }
    };
    write_controls(model.controls(), &mut root, look_at_name)?;
    root.bounds = native_bounds(
        model
            .bounds()
            .ok_or_else(|| Error::new("d3.bounds", "model bounds are required"))?,
    );
    if model
        .bounds()
        .is_some_and(|bounds| bounds.sphere_radius != 0.)
    {
        context.unsupported("d3.bounds.sphere_radius","native AABB has no independent conservative sphere radius; collision sphere remains separately authored")?;
    }
    let bones: Vec<_> = model
        .nodes()
        .filter(|(_, n)| matches!(n.controls, NodeRole::Bone(_)))
        .collect();
    let bone_indices: BTreeMap<_, _> = bones
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i as i32))
        .collect();
    for (_, node) in &bones {
        let NodeRole::Bone(c) = &node.controls else {
            unreachable!()
        };
        if c.bounds.sphere_radius != 0. {
            context.unsupported("d3.bone.bounds.sphere_radius","native bone AABB has no independent conservative radius; native sphere remains separately authored")?;
        }
        root.bones.push(native::BoneStructure {
            name: name(&node.name)?,
            parent_index: match node.parent {
                None => -1,
                Some(id) => *bone_indices.get(&id).ok_or_else(|| {
                    Error::new(
                        "d3.bone.parent",
                        "bone parent must be another skeleton bone",
                    )
                })?,
            },
            bounds: native_bounds(c.bounds),
            sphere: native::Sphere {
                center: c.sphere_center,
                radius: c.sphere_radius,
            },
            transform0: native_transform(c.bind_pose)?,
            transform1: native_transform(c.inverse_bind_pose)?,
            transform2: native_transform(node.transform)?,
            transform3: native_transform(c.alternate_bind_pose)?,
            transform4: native_transform(c.inverse_alternate_bind_pose)?,
            particle: c.particle,
            ..Default::default()
        });
    }
    root.bone_count = i32_count(root.bones.len())?;
    let mut hardpoints = Vec::new();
    for (_, node) in model.nodes() {
        if let NodeRole::Hardpoint {
            name: original_name,
        } = &node.controls
        {
            if node.transform.scale != [1.; 3] || node.transform.pivot != [0.; 3] {
                return Err(Error::new(
                    "d3.hardpoint",
                    "native hardpoint requires rigid transform",
                ));
            }
            if node.name != *original_name {
                return Err(Error::new(
                    "d3.hardpoint.name",
                    "attachment role name and node name must agree",
                ));
            }
            hardpoints.push(native::Hardpoint {
                name: name(&node.name)?,
                bone_index: match node.parent {
                    None => -1,
                    Some(id) => *bone_indices.get(&id).ok_or_else(|| {
                        Error::new(
                            "d3.hardpoint.parent",
                            "hardpoint parent must be a skeleton bone",
                        )
                    })?,
                },
                transform: native::RigidTransform {
                    rotation: node.transform.rotation.0,
                    translation: node.transform.translation,
                },
            });
        }
    }
    root.hardpoint_count = i32_count(hardpoints.len())?;
    root.hardpoints = ZeroVec::alloc_from_slice(&hardpoints);
    let appearance = model.appearance();
    let mut slot_names = Vec::new();
    let mut used_materials = Vec::new();
    for (_, slot) in appearance.slots() {
        if slot_names.contains(&slot.name.as_ref()) {
            return Err(Error::new(
                "d3.material.slot.name",
                "native slot names must be unique",
            ));
        }
        slot_names.push(slot.name.as_ref());
        let mut variants = Vec::new();
        for (_, material_id) in slot.variants() {
            let material = model
                .material(material_id)
                .ok_or_else(|| Error::new("d3.material.variant", "slot material is absent"))?;
            if !material.name.is_empty() {
                context.unsupported(
                    "d3.material.name",
                    "native variant records have no independent material display label",
                )?;
            }
            variants.push(materials::write(&material.surface, &mut context)?);
            used_materials.push(material_id);
        }
        root.materials.push(native::AppearanceMaterial {
            name: name(&slot.name)?,
            variants,
            ..Default::default()
        });
    }
    for (id, _) in model.materials() {
        if !used_materials.contains(&id) {
            context.unsupported(
                "d3.material.unassigned",
                "material definition is not owned by a native slot variant",
            )?;
        }
    }
    if asset.resources().textures().len() != 0 {
        context.unsupported("d3.resources.textures","native D3 materials own external SNO references; the shared resource arena has no native appearance table")?;
    }
    let mut used_meshes = Vec::new();
    for (_, instance) in model.instances() {
        let mesh = model
            .mesh(instance.mesh)
            .ok_or_else(|| Error::new("d3.mesh", "placement geometry is absent"))?;
        let frame = model
            .node(instance.node)
            .ok_or_else(|| Error::new("d3.instance.node", "placement frame is absent"))?;
        if !matches!(frame.controls, NodeRole::ModelSpace) || instance.selection_group != 0 {
            return Err(Error::new(
                "d3.instance",
                "native subobjects use the implicit model frame and no independent selection group",
            ));
        }
        let lod = instance.lod.unwrap_or(0);
        if lod > 1 {
            return Err(Error::new(
                "d3.lod",
                "appearance supports two geometry sets",
            ));
        }
        if !instance.draws.is_empty() {
            return Err(Error::new(
                "d3.instance.draws",
                "material assignments belong to actual slot variants and alternatives",
            ));
        }
        let slot = mesh
            .controls
            .material_slot
            .and_then(|id| appearance.slot(id))
            .ok_or_else(|| {
                Error::new(
                    "d3.material.slot",
                    "native subobject requires an actual material-slot relationship",
                )
            })?;
        let mut palette = Vec::new();
        if let Some(skin_id) = instance.skin {
            let skin = model
                .skin(skin_id)
                .ok_or_else(|| Error::new("d3.skin", "placement skin is absent"))?;
            let BindMatrices::Explicit(matrices) = skin.inverse_bind() else {
                return Err(Error::new(
                    "d3.skin",
                    "explicit inverse binding is required",
                ));
            };
            for (i, id) in skin.joints.iter().enumerate() {
                let index = *bone_indices.get(id).ok_or_else(|| {
                    Error::new("d3.skin.joint", "skin joint is not a skeleton bone")
                })?;
                let NodeRole::Bone(c) = &model.node(*id).unwrap().controls else {
                    unreachable!()
                };
                if matrices.get(i) != Some(matrix(c.inverse_alternate_bind_pose)) {
                    return Err(Error::new(
                        "d3.skin.binding",
                        "skin binding must agree with independently authored inverse B pose",
                    ));
                }
                palette.push(index as u32);
            }
        } else if !mesh.geometry.influence_rows().is_empty() {
            return Err(Error::new(
                "d3.skin",
                "weighted geometry requires a placement skin",
            ));
        }
        let sub = geometry::write(mesh, &slot.name, &palette, &bone_indices, &mut context)?;
        if lod == 0 {
            root.geo_set0.sub_objects.push(sub);
        } else {
            root.geo_set1.sub_objects.push(sub);
        }
        used_meshes.push(instance.mesh);
    }
    if model.meshes().any(|(id, _)| !used_meshes.contains(&id)) {
        context.unsupported(
            "d3.geometry.unplaced",
            "native appearance cannot retain geometry with no placement",
        )?;
    }
    if options.look.is_some() {
        return Err(Error::new("d3.look","native appearance output preserves its actual named alternatives; generic draw-look selection is unavailable"));
    }
    let mut native_looks = Vec::new();
    for (look_index, (_, look)) in appearance.alternatives().enumerate() {
        for (slot_id, slot) in appearance.slots() {
            let expected = slot.variants().nth(look_index).map(|(id, _)| id);
            let selected = look
                .bindings()
                .find(|(id, _)| *id == slot_id)
                .map(|(_, id)| id);
            if expected.is_none() || selected != expected {
                return Err(Error::new("d3.look.binding","native named alternatives select each slot's actual variant at the same ordinal"));
            }
        }
        native_looks.push(native::AppearanceLook {
            name: name(&look.name)?,
        });
    }
    root.looks = ZeroVec::alloc_from_slice(&native_looks);
    root.look_count = i32_count(native_looks.len())?;
    root.material_count = i32_count(root.materials.len())?;
    root.geo_set0.sub_object_count = i32_count(root.geo_set0.sub_objects.len())?;
    root.geo_set1.sub_object_count = i32_count(root.geo_set1.sub_objects.len())?;
    Ok(Plan::new(Appearance::<V260>::new(root), context.report))
}
fn write_controls(
    c: &ModelControls,
    v: &mut native::AppearanceRecord<'_>,
    look_at_name: &str,
) -> Result<(), Error> {
    v.sno_id = c.sno_id;
    v.object_type = c.object_type;
    v.collision_sphere = native::Sphere {
        center: c.collision_center,
        radius: c.collision_radius,
    };
    v.look_at = native::LookAtData {
        enabled: c.look_at.enabled,
        bone_name: name(look_at_name)?,
        yaw_min: c.look_at.yaw[0],
        yaw_max: c.look_at.yaw[1],
        pitch_min: c.look_at.pitch[0],
        pitch_max: c.look_at.pitch[1],
        angular_velocity: c.look_at.angular_velocity,
        unknown58: c.look_at.unclassified_scalar,
    };
    v.unknown108 = c.unclassified_vector;
    [
        v.unknown190,
        v.unknown1_e0,
        v.unknown1_e4,
        v.unknown1_e8,
        v.unknown1_ec,
        v.unknown1_f0,
        v.unknown1_f4,
        v.unknown1_f8,
        v.unknown200,
        v.unknown204,
        v.unknown210,
    ] = c.unclassified_integers;
    [v.unknown198, v.unknown208] = c.unclassified_wide_integers;
    v.unknown1_fc = c.unclassified_scalar;
    Ok(())
}
