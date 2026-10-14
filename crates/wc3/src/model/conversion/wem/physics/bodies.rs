//! Bodies retain actual hierarchy anchors and shared collider identities.
use super::codec::{append, mapped, node, read_error, revision, Optional, Scalar};
use super::shapes;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    Body, BodyId, ForceChannels, Motion, Physics, Sc2Body, Sc2BodyFlags, ShapeId, WowBody,
};
use crate::model::semantic::{Animated, NodeId};
use crate::model::wem::{records, Bool, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn read<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::PhysicsBody>,
    nodes: &[NodeId],
    target: &mut Physics<'a>,
) -> Result<BTreeMap<u32, BodyId>, Error> {
    let mut source = Vec::new();
    let mut cache = shapes::Cache::new();
    macro_rules! rows {
        ($revision:ty) => {{
            for row in document
                .resolve::<records::PhysicsBody<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                source.push((
                    row.id,
                    Optional::<records::WowBodyExtension>::optional(&row.wow)
                        .map(|value| value.parent)
                        .filter(|id| *id != u32::MAX),
                    body(document, &row, nodes, target, &mut cache)?,
                ));
            }
        }};
    }
    if reference.entries != 0 {
        match revision(document, reference)? {
            Some(1) => rows!(records::V1),
            Some(2) => rows!(records::V2),
            Some(3) => rows!(records::V3),
            _ => return Err(Error::new("wem.physics.body", "unsupported body revision")),
        }
    }
    let mut ids = BTreeMap::new();
    let mut parents = Vec::new();
    for (source_id, parent, value) in source {
        let id = target.add_body(value);
        if ids.insert(source_id, id).is_some() {
            return Err(Error::new("wem.physics.body", "duplicate body identity"));
        }
        parents.push((id, parent));
    }
    for (id, parent) in parents {
        if let Some(parent) = parent {
            let parent = *ids.get(&parent).ok_or_else(|| {
                Error::new(
                    "wem.physics.body.parent",
                    "parent body is outside the simulation domain",
                )
            })?;
            let mut body = target.body(id).unwrap().clone();
            body.wow.as_mut().unwrap().parent = Some(parent);
            target.set_body(id, body)?;
        }
    }
    Ok(ids)
}
fn body<'a, D: records::PhysicsBodyDialect>(
    document: &'a RawDocument<'_>,
    value: &records::PhysicsBody<D>,
    nodes: &[NodeId],
    target: &mut Physics<'a>,
    cache: &mut shapes::Cache,
) -> Result<Body, Error>
where
    D::Wow: Optional<records::WowBodyExtension>,
    D::Locked: Scalar<Bool>,
{
    Ok(Body {
        node: node(nodes, value.node)?,
        motion: match value.motion {
            records::BodyMotion::Dynamic => Motion::Dynamic,
            records::BodyMotion::Kinematic => Motion::Kinematic,
            records::BodyMotion::Static => Motion::Static,
            records::BodyMotion::Count => Motion::Count,
            records::BodyMotion::Unknown(code) => Motion::Unknown(code),
        },
        simulates: Animated::constant(value.simulates.get()),
        shapes: shapes::read(document, value.shapes, target, cache)?,
        damping: [value.linear_damping, value.angular_damping],
        inertia_scale: value.inertia_scale,
        gravity_scale: value.gravity_scale,
        inherit_dynamic: value.inherit_dynamic.get(),
        exempt_from_ragdoll: value.exempt_from_ragdoll.get(),
        force_channels: ForceChannels::from_bits_retain(value.force_channels.bits()),
        sc2: value.sc2.value.as_ref().map(|value| Sc2Body {
            material: value.physics_material,
            collidable: value.collidable.get(),
            walkable: value.walkable.get(),
            stackable: value.stackable.get(),
            simulate_collision: value.simulate_collision.get(),
            keeps_bone_driven: value.keeps_bone_driven.get(),
            flags: Sc2BodyFlags::from_bits_retain(value.untraced_flags.bits()),
        }),
        wow: Optional::optional(&value.wow).map(|value| WowBody {
            follow_factor: value.follow_factor,
            has_children: value.has_children.get(),
            ragdoll_root: value.ragdoll_root.get(),
            parent: None,
        }),
        locked: Scalar::scalar(&value.locked).is_some_and(Bool::get),
    })
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    source: &Physics<'_>,
    ids: &BTreeMap<BodyId, u32>,
    nodes: &BTreeMap<NodeId, u32>,
) -> Result<Ref<records::targets::PhysicsBody>, Error> {
    let mut rows = Vec::new();
    let mut cache = BTreeMap::<Vec<ShapeId>, Ref<records::targets::PhysicsShape>>::new();
    for (id, value) in source.bodies() {
        let shapes = if let Some(reference) = cache.get(&value.shapes) {
            *reference
        } else {
            let reference = shapes::write(document, source, &value.shapes)?;
            cache.insert(value.shapes.clone(), reference);
            reference
        };
        rows.push(records::PhysicsBody::<records::V3> {
            id: mapped(ids, id, "physics body")?,
            node: mapped(nodes, value.node, "physics node")?,
            motion: match value.motion {
                Motion::Dynamic => records::BodyMotion::Dynamic,
                Motion::Kinematic => records::BodyMotion::Kinematic,
                Motion::Static => records::BodyMotion::Static,
                Motion::Count => records::BodyMotion::Count,
                Motion::Unknown(code) => records::BodyMotion::Unknown(code),
            },
            simulates: value.simulates.default.into(),
            shapes,
            linear_damping: value.damping[0],
            angular_damping: value.damping[1],
            inertia_scale: value.inertia_scale,
            gravity_scale: value.gravity_scale,
            inherit_dynamic: value.inherit_dynamic.into(),
            exempt_from_ragdoll: value.exempt_from_ragdoll.into(),
            force_channels: records::ForceChannels::from_bits_retain(value.force_channels.bits()),
            sc2: InlineOption::from(value.sc2.as_ref().map(|value| records::Sc2BodyExtension {
                physics_material: value.material,
                collidable: value.collidable.into(),
                walkable: value.walkable.into(),
                stackable: value.stackable.into(),
                simulate_collision: value.simulate_collision.into(),
                keeps_bone_driven: value.keeps_bone_driven.into(),
                untraced_flags: records::Sc2BodyUntracedFlags::from_bits_retain(value.flags.bits()),
            })),
            wow: InlineOption::from(
                value
                    .wow
                    .as_ref()
                    .map(|value| {
                        Ok::<_, Error>(records::WowBodyExtension {
                            follow_factor: value.follow_factor,
                            has_children: value.has_children.into(),
                            ragdoll_root: value.ragdoll_root.into(),
                            parent: value
                                .parent
                                .map(|id| mapped(ids, id, "parent body"))
                                .transpose()?
                                .unwrap_or(u32::MAX),
                        })
                    })
                    .transpose()?,
            ),
            locked: value.locked.into(),
        });
    }
    Ok(append(document, &rows)?.cast())
}

#[cfg(test)]
mod tests {
    use super::super::codec::append;
    use super::*;
    use crate::model::semantic::profiles::Wem;
    use crate::model::semantic::{Model, Node};
    #[test]
    fn forward_parent_and_unnamed_force_bits_survive_rebuilding() {
        let mut builder = Model::<Wem>::builder("body test");
        let node = builder.add_node(Node::named("root")).unwrap();
        let mut document = RawDocument::new();
        let source = append(
            &mut document,
            &[
                records::PhysicsBody::<records::V3> {
                    id: 81,
                    node: 0,
                    force_channels: records::ForceChannels::from_bits_retain(0x8000_0040),
                    wow: InlineOption::from(Some(records::WowBodyExtension {
                        parent: 92,
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                records::PhysicsBody::<records::V3> {
                    id: 92,
                    node: 0,
                    ..Default::default()
                },
            ],
        )
        .unwrap();
        let mut physics = Physics::default();
        let ids = read(&document, source.cast(), &[node], &mut physics).unwrap();
        assert_eq!(
            physics.body(ids[&81]).unwrap().wow.as_ref().unwrap().parent,
            Some(ids[&92])
        );
        let output_ids = BTreeMap::from([(ids[&81], 1), (ids[&92], 2)]);
        let mut output = RawDocument::new();
        let reference = write(
            &mut output,
            &physics,
            &output_ids,
            &BTreeMap::from([(node, 0)]),
        )
        .unwrap();
        let values = output
            .resolve::<records::PhysicsBody<records::V3>>(reference.cast())
            .unwrap();
        let first = values.get(0).unwrap().unwrap();
        assert_eq!(first.wow.value.unwrap().parent, 2);
        assert_eq!(first.force_channels.bits(), 0x8000_0040);
    }
}
