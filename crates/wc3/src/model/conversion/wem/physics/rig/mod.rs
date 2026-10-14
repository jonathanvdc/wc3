//! Body groups resolve stable producer identities into simulation-owned rig identities.
mod choices;
use choices::{read_joint_range, write_joint_range};
mod fracture;
use super::super::text;
use super::codec::{allocate, append, mapped, read_error, read_words, revision, words};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::*;
use crate::model::semantic::{MaterialSlotId, MeshId, NodeId};
use crate::model::wem::{records, Bool, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;
fn read_rigstart(value: records::RigStart) -> RigStart {
    match value {
        records::RigStart::Animated => RigStart::Animated,
        records::RigStart::OnDeath => RigStart::OnDeath,
        records::RigStart::Always => RigStart::Always,
        records::RigStart::Never => RigStart::Never,
        records::RigStart::Count => RigStart::Count,
        records::RigStart::Unknown(value) => RigStart::Unknown(value),
    }
}
fn write_rigstart(value: RigStart) -> records::RigStart {
    match value {
        RigStart::Animated => records::RigStart::Animated,
        RigStart::OnDeath => records::RigStart::OnDeath,
        RigStart::Always => records::RigStart::Always,
        RigStart::Never => records::RigStart::Never,
        RigStart::Count => records::RigStart::Count,
        RigStart::Unknown(value) => records::RigStart::Unknown(value),
    }
}
fn read_wowphysicskind(value: records::WowPhysicsKind) -> WowPhysicsKind {
    match value {
        records::WowPhysicsKind::WornItem => WowPhysicsKind::WornItem,
        records::WowPhysicsKind::Vegetation => WowPhysicsKind::Vegetation,
        records::WowPhysicsKind::Ragdoll => WowPhysicsKind::Ragdoll,
        records::WowPhysicsKind::PrivateWorld => WowPhysicsKind::PrivateWorld,
        records::WowPhysicsKind::Count => WowPhysicsKind::Count,
        records::WowPhysicsKind::Unknown(value) => WowPhysicsKind::Unknown(value),
    }
}
fn write_wowphysicskind(value: WowPhysicsKind) -> records::WowPhysicsKind {
    match value {
        WowPhysicsKind::WornItem => records::WowPhysicsKind::WornItem,
        WowPhysicsKind::Vegetation => records::WowPhysicsKind::Vegetation,
        WowPhysicsKind::Ragdoll => records::WowPhysicsKind::Ragdoll,
        WowPhysicsKind::PrivateWorld => records::WowPhysicsKind::PrivateWorld,
        WowPhysicsKind::Count => records::WowPhysicsKind::Count,
        WowPhysicsKind::Unknown(value) => records::WowPhysicsKind::Unknown(value),
    }
}
fn read_shapekind(value: records::PhysicsShapeKind) -> ShapeKind {
    match value {
        records::PhysicsShapeKind::Box => ShapeKind::Box,
        records::PhysicsShapeKind::Sphere => ShapeKind::Sphere,
        records::PhysicsShapeKind::Capsule => ShapeKind::Capsule,
        records::PhysicsShapeKind::Cylinder => ShapeKind::Cylinder,
        records::PhysicsShapeKind::ConvexHull => ShapeKind::ConvexHull,
        records::PhysicsShapeKind::TriangleMesh => ShapeKind::TriangleMesh,
        records::PhysicsShapeKind::Count => ShapeKind::Count,
        records::PhysicsShapeKind::Unknown(value) => ShapeKind::Unknown(value),
    }
}
fn write_shapekind(value: ShapeKind) -> records::PhysicsShapeKind {
    match value {
        ShapeKind::Box => records::PhysicsShapeKind::Box,
        ShapeKind::Sphere => records::PhysicsShapeKind::Sphere,
        ShapeKind::Capsule => records::PhysicsShapeKind::Capsule,
        ShapeKind::Cylinder => records::PhysicsShapeKind::Cylinder,
        ShapeKind::ConvexHull => records::PhysicsShapeKind::ConvexHull,
        ShapeKind::TriangleMesh => records::PhysicsShapeKind::TriangleMesh,
        ShapeKind::Count => records::PhysicsShapeKind::Count,
        ShapeKind::Unknown(value) => records::PhysicsShapeKind::Unknown(value),
    }
}
fn read_wowvegetation(row: &records::WowVegetation) -> WowVegetation {
    WowVegetation {
        pos_max_push: row.pos_max_push,
        pos_push_amt: row.pos_push_amt,
        pos_relax_speed: row.pos_relax_speed,
        vel_max_push: row.vel_max_push,
        vel_speed: row.vel_speed,
        min_push_dist: row.min_push_dist,
    }
}
fn write_wowvegetation(row: &WowVegetation) -> records::WowVegetation {
    records::WowVegetation {
        pos_max_push: row.pos_max_push,
        pos_push_amt: row.pos_push_amt,
        pos_relax_speed: row.pos_relax_speed,
        vel_max_push: row.vel_max_push,
        vel_speed: row.vel_speed,
        min_push_dist: row.min_push_dist,
    }
}
fn read_ragdollrecipe(row: &records::RagdollRecipe) -> RagdollRecipe {
    RagdollRecipe {
        torso: read_shapekind(row.torso),
        head: read_shapekind(row.head),
        limbs: read_shapekind(row.limbs),
        props: read_shapekind(row.props),
        hull_points: row.hull_points,
        tightness: row.tightness,
        thickness: row.thickness,
        range: read_joint_range(row.range),
        hinges: row.hinges.get(),
        collide_connected: row.collide_connected.get(),
        fold_short_links: row.fold_short_links.get(),
        weld_props: row.weld_props.get(),
        material: row.material,
        linear_damping: row.linear_damping,
        angular_damping: row.angular_damping,
    }
}
fn write_ragdollrecipe(row: &RagdollRecipe) -> records::RagdollRecipe {
    records::RagdollRecipe {
        torso: write_shapekind(row.torso),
        head: write_shapekind(row.head),
        limbs: write_shapekind(row.limbs),
        props: write_shapekind(row.props),
        hull_points: row.hull_points,
        tightness: row.tightness,
        thickness: row.thickness,
        range: write_joint_range(row.range),
        hinges: Bool::from(row.hinges),
        collide_connected: Bool::from(row.collide_connected),
        fold_short_links: Bool::from(row.fold_short_links),
        weld_props: Bool::from(row.weld_props),
        material: row.material,
        linear_damping: row.linear_damping,
        angular_damping: row.angular_damping,
    }
}
fn read_wow<'a>(
    doc: &'a RawDocument<'_>,
    row: &records::WowRigExtension,
) -> Result<WowRig<'a>, Error> {
    Ok(WowRig {
        kind: read_wowphysicskind(row.kind),
        vegetation: row.vegetation.value.as_ref().map(read_wowvegetation),
        allowed_skeleton_names: read_words(doc, row.allow_list)?,
    })
}
fn write_wow(
    doc: &mut RawDocument<'_>,
    row: &WowRig<'_>,
) -> Result<records::WowRigExtension, Error> {
    Ok(records::WowRigExtension {
        kind: write_wowphysicskind(row.kind),
        vegetation: InlineOption::from(row.vegetation.as_ref().map(write_wowvegetation)),
        allow_list: words(doc, row.allowed_skeleton_names.iter())?,
    })
}
pub(super) fn read<'a>(
    doc: &'a RawDocument<'_>,
    reference: Ref<records::targets::PhysicsRig>,
    physics: &mut Physics<'a>,
    bodies: &BTreeMap<u32, BodyId>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    slots: &[MaterialSlotId],
) -> Result<BTreeMap<u32, RigId>, Error> {
    let mut result = BTreeMap::new();
    let Some(rev) = revision(doc, reference)? else {
        return Ok(result);
    };
    match rev {
        1 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V1>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: None,
                    ragdoll: None,
                    fracture: None,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        2 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V2>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: None,
                    fracture: None,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        3 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V3>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: None,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        4 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V4>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: row
                        .fracture
                        .value
                        .as_ref()
                        .map(|value| fracture::read(doc, value, nodes, meshes, slots))
                        .transpose()?,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        5 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V5>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: row
                        .fracture
                        .value
                        .as_ref()
                        .map(|value| fracture::read(doc, value, nodes, meshes, slots))
                        .transpose()?,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        6 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V6>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: row
                        .fracture
                        .value
                        .as_ref()
                        .map(|value| fracture::read(doc, value, nodes, meshes, slots))
                        .transpose()?,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        7 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V7>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: row
                        .fracture
                        .value
                        .as_ref()
                        .map(|value| fracture::read(doc, value, nodes, meshes, slots))
                        .transpose()?,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        8 => {
            for row in doc
                .resolve::<records::PhysicsRig<records::V8>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let value = Rig {
                    name: text::read(doc, row.name)?,
                    start: read_rigstart(row.start),
                    bodies: read_words(doc, row.bodies)?
                        .iter()
                        .map(|id| {
                            bodies
                                .get(&id)
                                .copied()
                                .ok_or_else(|| read_error("rig body has no producer identity"))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    wow: row
                        .wow
                        .value
                        .as_ref()
                        .map(|value| read_wow(doc, value))
                        .transpose()?,
                    ragdoll: row.recipe.value.as_ref().map(read_ragdollrecipe),
                    fracture: row
                        .fracture
                        .value
                        .as_ref()
                        .map(|value| fracture::read(doc, value, nodes, meshes, slots))
                        .transpose()?,
                };
                if result.insert(row.id, physics.add_rig(value)).is_some() {
                    return Err(read_error("duplicate rig producer identity"));
                }
            }
        }
        _ => return Err(read_error("unsupported physics rig revision")),
    }
    Ok(result)
}
pub(super) fn write(
    doc: &mut RawDocument<'_>,
    physics: &Physics<'_>,
    bodies: &BTreeMap<BodyId, u32>,
    nodes: &BTreeMap<NodeId, u32>,
    meshes: &BTreeMap<MeshId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
    next_id: &mut u32,
) -> Result<(Ref<records::targets::PhysicsRig>, BTreeMap<RigId, u32>), Error> {
    let mut result = BTreeMap::new();
    let mut rows = Vec::new();
    for (id, value) in physics.rigs() {
        let producer = allocate(next_id)?;
        result.insert(id, producer);
        rows.push(records::PhysicsRig::<records::V8> {
            id: producer,
            name: text::write_bytes(doc, value.name.as_bytes())?,
            start: write_rigstart(value.start),
            bodies: words(
                doc,
                value
                    .bodies
                    .iter()
                    .copied()
                    .map(|id| mapped(bodies, id, "rig body"))
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
            wow: InlineOption::from(
                value
                    .wow
                    .as_ref()
                    .map(|value| write_wow(doc, value))
                    .transpose()?,
            ),
            recipe: InlineOption::from(value.ragdoll.as_ref().map(write_ragdollrecipe)),
            fracture: InlineOption::from(
                value
                    .fracture
                    .as_ref()
                    .map(|value| fracture::write(doc, value, nodes, meshes, slots))
                    .transpose()?,
            ),
        });
    }
    Ok((append(doc, &rows)?.cast(), result))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
