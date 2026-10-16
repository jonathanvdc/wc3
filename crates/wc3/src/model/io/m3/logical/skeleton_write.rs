//! Skeleton encoding follows actual graph joints and behavior anchors.
use super::animation::AnimationIds;
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::records as nr;
use crate::model::io::m3::logical::Graph;
use crate::model::{profiles::m3 as surface, Billboard, Inheritance, NodeId, Property};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Skeleton {
    pub nodes: Vec<NodeId>,
    pub indices: BTreeMap<NodeId, u32>,
    pub bones: Vec<nr::Bone<nr::V1>>,
    pub inverse_bind: Vec<nr::InitialReference>,
    pub skin_bone_count: u32,
}
fn value<T: Clone>(
    initial: T,
    controls: Option<&surface::PropertyControl<T>>,
    binding: Property<T>,
    ids: &AnimationIds,
) -> nr::AnimationValue<T> {
    let (mut interpolation, mut flags, mut id, reset, reserved) = match controls {
        Some(c) => (
            c.control.interpolation,
            c.control.flags.bits(),
            c.control.source_id,
            c.reset.clone(),
            c.control.reserved,
        ),
        None => (
            surface::values::Interpolation::Constant,
            0,
            0,
            initial.clone(),
            0,
        ),
    };
    if let Some(mapped) = ids.get(binding.target()) {
        interpolation = mapped.interpolation;
        id = mapped.id;
        if mapped.keyed {
            flags |= 6
        }
    }
    nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(interpolation.raw()),
        flags: nr::AnimationFlags(flags),
        animation_id: id,
        initial_value: initial,
        null_value: reset,
        reserved,
    }
}
fn scalar<T: Clone>(r: &surface::Property<T>, ids: &AnimationIds) -> nr::AnimationValue<T> {
    let mut value = nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(r.control.interpolation.raw()),
        flags: nr::AnimationFlags(r.control.flags.bits()),
        animation_id: r.control.source_id,
        initial_value: r.initial.clone(),
        null_value: r.reset.clone(),
        reserved: r.control.reserved,
    };
    if let Some(mapped) = r.binding.as_ref().and_then(|p| ids.get(p.target())) {
        value.interpolation = nr::Interpolation::from_raw(mapped.interpolation.raw());
        value.animation_id = mapped.id;
        if mapped.keyed {
            value.flags = nr::AnimationFlags(value.flags.bits() | 6)
        }
    }
    value
}
pub(super) fn prepare(
    model: &Graph<'_>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<Skeleton, Error> {
    let mut wanted = BTreeSet::new();
    let mut controls = BTreeMap::new();
    for (id, node) in model.nodes() {
        if let surface::NodeRole::Bone(bone) = &node.controls {
            wanted.insert(id);
            controls.insert(id, bone);
        }
    }
    for (_, feature) in model.features() {
        match &feature.payload {
            surface::Feature::Event(_) | surface::Feature::Extent(_) => {}
            _ => {
                wanted.insert(feature.node);
            }
        }
    }
    for (_, skin) in model.skins() {
        wanted.extend(skin.joints.iter().copied());
    }
    for (_, clip) in model.clips() {
        for event in clip.events() {
            if let Some(node) = event.node {
                wanted.insert(node);
            }
        }
    }
    for event in model.shared_events() {
        if let Some(node) = event.node {
            wanted.insert(node);
        }
    }
    for id in wanted.clone() {
        let mut node = model.node(id).expect("checked feature node");
        while let Some(parent) = node.parent {
            wanted.insert(parent);
            node = model.node(parent).expect("checked parent");
        }
    }
    let nodes: Vec<_> = model
        .nodes()
        .filter_map(|(id, _)| wanted.contains(&id).then_some(id))
        .collect();
    if nodes.len() > u16::MAX as usize {
        return Err(Error::new(
            "m3.skeleton",
            "hierarchy exceeds native bone capacity",
        ));
    }
    let indices: BTreeMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u32))
        .collect();
    let identity = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let mut binds = BTreeMap::new();
    for (_, skin) in model.skins() {
        for (index, id) in skin.joints.iter().enumerate() {
            let matrix = skin
                .inverse_bind()
                .get(index)
                .ok_or_else(|| Error::new("m3.skin", "inverse bind matrix is missing"))?;
            if binds.insert(*id, matrix).is_some_and(|old| old != matrix) {
                return Err(Error::new(
                    "m3.skin",
                    "one native bone cannot have conflicting inverse bind matrices",
                ));
            }
        }
    }
    let skin_bone_count = binds
        .keys()
        .filter_map(|node| indices.get(node))
        .max()
        .map_or(0, |index| index + 1);
    let mut bones = Vec::new();
    let mut inverse_bind = Vec::new();
    for id in &nodes {
        let node = model.node(*id).expect("checked node");
        if node.transform.pivot != [0.; 3] {
            context.unsupported("m3.node.pivot", "M3 bones have no independent pivot")?;
        }
        if !node.visible || node.camera_anchored {
            context.unsupported(
                "m3.node.behavior",
                "M3 bones cannot encode general node visibility or camera anchoring",
            )?;
        }
        let control = controls.get(id).copied();
        if control
            .is_some_and(|c| c.source_id < i64::from(i32::MIN) || c.source_id > i64::from(u32::MAX))
        {
            return Err(Error::new(
                "m3.bone.identifier",
                "identifier/control word exceeds its native 32-bit domain",
            ));
        }
        if node.inheritance != Inheritance::default() {
            context.unsupported(
                "m3.node.inheritance",
                "selective semantic inheritance has no established M3 encoding",
            )?;
        }
        let mut flags = nr::BoneFlags(control.map_or(0, |c| c.extra_flags.bits()) & !0x10);
        match node.billboard {
            Billboard::None => {}
            Billboard::Spherical => flags.set_billboard1(true),
            Billboard::Locked { .. } => context.unsupported(
                "m3.node.billboard",
                "generic locked billboard requires a native billboard behavior",
            )?,
        }
        let rotation = value(
            node.transform.rotation,
            control.map(|c| &c.rotation),
            id.rotation(),
            ids,
        );
        bones.push(nr::Bone {
            id: control.map_or(-1, |c| c.source_id as i32),
            name: Default::default(),
            flags,
            parent_index: node.parent.map(|p| indices[&p] as u16).unwrap_or(u16::MAX),
            padding: control.map_or(0, |c| c.reserved),
            position: value(
                node.transform.translation,
                control.map(|c| &c.position),
                id.translation(),
                ids,
            ),
            rotation: nr::AnimationValue {
                interpolation: rotation.interpolation,
                flags: rotation.flags,
                animation_id: rotation.animation_id,
                initial_value: rotation.initial_value.components(),
                null_value: rotation.null_value.components(),
                reserved: rotation.reserved,
            },
            scale: value(
                node.transform.scale,
                control.map(|c| &c.scale),
                id.scale(),
                ids,
            ),
            batching: control
                .map(|c| scalar(&c.batching, ids))
                .unwrap_or_default(),
        });
        if inverse_bind.len() < skin_bone_count as usize {
            inverse_bind.push(nr::InitialReference {
                matrix: binds.get(id).copied().unwrap_or(identity),
            });
        }
    }
    Ok(Skeleton {
        nodes,
        indices,
        bones,
        inverse_bind,
        skin_bone_count,
    })
}
