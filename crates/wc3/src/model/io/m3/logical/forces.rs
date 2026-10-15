//! Force and vertex-warp behavior retain their complete authored controls.
use super::{
    bindings::{self, Bindings},
    hierarchy::Skeleton,
    materials::{anim, read_error, revision},
};
use super::{Graph, GraphBuilder};
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::profiles::m3::properties::{Field, FieldPath};
use crate::model::NodeId;
use crate::model::{profiles::m3 as surface, OwnerId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
pub(super) fn force<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: Ref<nr::targets::Force>,
    builder: &mut GraphBuilder<'a>,
    skeleton: &Skeleton<'a>,
    registry: &mut Bindings,
) -> Result<(), Error> {
    if reference.is_empty() {
        return Ok(());
    }
    macro_rules! read {
        ($v:ty) => {
            for row in source
                .resolve::<nr::Force<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let node = *skeleton
                    .nodes
                    .get(r.bone_index as usize)
                    .ok_or_else(|| Error::new("m3.force.node", "anchor exceeds skeleton"))?;
                let feature = builder.reserve_feature(node);
                let mut payload = surface::Force {
                    force_type: surface::values::ForceKind::from_raw(r.force_type.raw()),
                    force_shape: surface::values::ForceShape::from_raw(r.force_shape.raw()),
                    unknown: r.unknown,
                    flags: surface::values::ForceFlags(r.flags.bits()),
                    local_channels: surface::values::ForceChannels(r.local_channels.bits()),
                    strength: anim(r.strength),
                    width: anim(r.width),
                    height: anim(r.height),
                    length: anim(r.length),
                };
                bindings::field(
                    &mut payload.strength,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Force).child(Field::Strength),
                    registry,
                );
                bindings::field(
                    &mut payload.width,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Force).child(Field::Width),
                    registry,
                );
                bindings::field(
                    &mut payload.height,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Force).child(Field::Height),
                    registry,
                );
                bindings::field(
                    &mut payload.length,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Force).child(Field::Length),
                    registry,
                );
                builder.set_feature(feature, surface::Feature::Force((payload).into()))?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        _ => return Err(Error::new("m3.forces", "unsupported force revision")),
    }
    Ok(())
}
pub(super) fn warp<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: Ref<nr::targets::Warp>,
    builder: &mut GraphBuilder<'a>,
    skeleton: &Skeleton<'a>,
    registry: &mut Bindings,
) -> Result<(), Error> {
    if reference.is_empty() {
        return Ok(());
    }
    for row in source
        .resolve::<nr::Warp>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let node = *skeleton
            .nodes
            .get(r.bone_index as usize)
            .ok_or_else(|| Error::new("m3.warp.node", "anchor exceeds skeleton"))?;
        let feature = builder.reserve_feature(node);
        let mut payload = surface::Warp {
            warp_type: r.warp_type,
            unknown: r.unknown,
            radius: anim(r.radius),
            height: anim(r.height),
            strength: anim(r.strength),
            angular: anim(r.angular),
            axial: anim(r.axial),
            radial: anim(r.radial),
        };
        bindings::field(
            &mut payload.radius,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Radius),
            registry,
        );
        bindings::field(
            &mut payload.height,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Height),
            registry,
        );
        bindings::field(
            &mut payload.strength,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Strength),
            registry,
        );
        bindings::field(
            &mut payload.angular,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Angular),
            registry,
        );
        bindings::field(
            &mut payload.axial,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Axial),
            registry,
        );
        bindings::field(
            &mut payload.radial,
            OwnerId::Feature(feature),
            &FieldPath::root(Field::Warp).child(Field::Radial),
            registry,
        );
        builder.set_feature(feature, surface::Feature::Warp((payload).into()))?;
    }
    Ok(())
}

pub(super) fn write<E: Envelope>(
    model: &Graph<'_>,
    document: &mut native::Container<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
    ids: &super::animation::AnimationIds,
) -> Result<(Ref<nr::targets::Force>, Ref<nr::targets::Warp>), Error> {
    use super::write::{animated, append};
    let mut forces = Vec::new();
    let mut warps = Vec::new();
    for (_, attached) in model.features() {
        match &attached.payload {
            surface::Feature::Force(force) => {
                let bone_index = *bones.get(&attached.node).ok_or_else(|| {
                    Error::new("m3.force.node", "force anchor has no native bone")
                })?;
                forces.push(nr::Force::<nr::V2> {
                    bone_index,
                    force_type: nr::ForceKind::from_raw(force.force_type.raw()),
                    force_shape: nr::ForceShape::from_raw(force.force_shape.raw()),
                    unknown: force.unknown,
                    flags: nr::ForceFlags(force.flags.bits()),
                    local_channels: nr::ForceChannels(force.local_channels.bits()),
                    strength: animated(&force.strength, ids),
                    width: animated(&force.width, ids),
                    height: animated(&force.height, ids),
                    length: animated(&force.length, ids),
                });
            }
            surface::Feature::Warp(warp) => {
                let bone_index = *bones
                    .get(&attached.node)
                    .ok_or_else(|| Error::new("m3.warp.node", "warp anchor has no native bone"))?;
                warps.push(nr::Warp {
                    bone_index,
                    warp_type: warp.warp_type,
                    unknown: warp.unknown,
                    radius: animated(&warp.radius, ids),
                    height: animated(&warp.height, ids),
                    strength: animated(&warp.strength, ids),
                    angular: animated(&warp.angular, ids),
                    axial: animated(&warp.axial, ids),
                    radial: animated(&warp.radial, ids),
                });
            }
            _ => {}
        }
    }
    Ok((
        append(document, &forces)?.cast(),
        append(document, &warps)?.cast(),
    ))
}
