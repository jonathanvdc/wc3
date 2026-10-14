//! Authored projector envelopes and animated frustum/box controls.
use super::{
    animation::AnimationIds,
    bindings::{self, Bindings},
    hierarchy::Skeleton,
    materials::{anim, read_error, revision},
    write::{animated, append},
    Graph, GraphBuilder,
};
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, FeatureId, MaterialId, NodeId, OwnerId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
pub(super) fn read<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::Projector>,
    builder: &mut GraphBuilder<'_>,
    skeleton: &Skeleton<'_>,
    materials: &[MaterialId],
    registry: &mut Bindings,
) -> Result<Vec<FeatureId>, Error> {
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    macro_rules! read {
        ($version:ty) => {
            for row in source
                .resolve::<nr::Projector<$version>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let node = *skeleton.nodes.get(r.bone as usize).ok_or_else(|| {
                    Error::new("m3.projector.node", "projector anchor exceeds skeleton")
                })?;
                let owner = builder.reserve_feature(node);
                let mut payload = surface::Projector {
                    projection_type: surface::values::ProjectionKind::from_raw(
                        r.projection_type.raw(),
                    ),
                    material: *materials
                        .get(r.material_reference_index as usize)
                        .ok_or_else(|| {
                            Error::new("m3.projector.material", "projected material is absent")
                        })?,
                    flags: surface::values::ProjectorFlags(r.flags.bits()),
                    offset: anim(r.offset),
                    pitch: anim(r.pitch),
                    yaw: anim(r.yaw),
                    roll: anim(r.roll),
                    field_of_view: anim(r.field_of_view),
                    aspect_ratio: anim(r.aspect_ratio),
                    near: anim(r.near),
                    far: anim(r.far),
                    box_offset_z_bottom: anim(r.box_offset_z_bottom),
                    box_offset_z_top: anim(r.box_offset_z_top),
                    box_offset_x_left: anim(r.box_offset_x_left),
                    box_offset_x_right: anim(r.box_offset_x_right),
                    box_offset_y_front: anim(r.box_offset_y_front),
                    box_offset_y_back: anim(r.box_offset_y_back),
                    active: anim(r.active),
                    falloff: r.falloff,
                    alpha_init: r.alpha_init,
                    alpha_mid: r.alpha_mid,
                    alpha_end: r.alpha_end,
                    lifetime_attack: r.lifetime_attack,
                    lifetime_attack_to: r.lifetime_attack_to,
                    lifetime_hold: r.lifetime_hold,
                    lifetime_hold_to: r.lifetime_hold_to,
                    lifetime_decay: r.lifetime_decay,
                    lifetime_decay_to: r.lifetime_decay_to,
                    attenuation_distance: r.attenuation_distance,
                    layer: r.layer,
                    lod_reduce: r.lod_reduce,
                    lod_cut: r.lod_cut,
                };
                bindings::field(
                    &mut payload.offset,
                    OwnerId::Feature(owner),
                    "projector.offset",
                    registry,
                );
                bindings::field(
                    &mut payload.pitch,
                    OwnerId::Feature(owner),
                    "projector.pitch",
                    registry,
                );
                bindings::field(
                    &mut payload.yaw,
                    OwnerId::Feature(owner),
                    "projector.yaw",
                    registry,
                );
                bindings::field(
                    &mut payload.roll,
                    OwnerId::Feature(owner),
                    "projector.roll",
                    registry,
                );
                bindings::field(
                    &mut payload.field_of_view,
                    OwnerId::Feature(owner),
                    "projector.field_of_view",
                    registry,
                );
                bindings::field(
                    &mut payload.aspect_ratio,
                    OwnerId::Feature(owner),
                    "projector.aspect_ratio",
                    registry,
                );
                bindings::field(
                    &mut payload.near,
                    OwnerId::Feature(owner),
                    "projector.near",
                    registry,
                );
                bindings::field(
                    &mut payload.far,
                    OwnerId::Feature(owner),
                    "projector.far",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_z_bottom,
                    OwnerId::Feature(owner),
                    "projector.box_offset_z_bottom",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_z_top,
                    OwnerId::Feature(owner),
                    "projector.box_offset_z_top",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_x_left,
                    OwnerId::Feature(owner),
                    "projector.box_offset_x_left",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_x_right,
                    OwnerId::Feature(owner),
                    "projector.box_offset_x_right",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_y_front,
                    OwnerId::Feature(owner),
                    "projector.box_offset_y_front",
                    registry,
                );
                bindings::field(
                    &mut payload.box_offset_y_back,
                    OwnerId::Feature(owner),
                    "projector.box_offset_y_back",
                    registry,
                );
                bindings::field(
                    &mut payload.active,
                    OwnerId::Feature(owner),
                    "projector.active",
                    registry,
                );
                builder.set_feature(owner, surface::Feature::Projector((payload).into()))?;
                result.push(owner);
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        _ => return Err(Error::new("m3.projectors", "unknown projector schema")),
    }
    Ok(result)
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut native::Document<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
    materials: &BTreeMap<MaterialId, u32>,
    ids: &AnimationIds,
) -> Result<Ref<nr::targets::Projector>, Error> {
    let mut rows = Vec::new();
    for (_, attached) in graph.features() {
        let surface::Feature::Projector(r) = &attached.payload else {
            continue;
        };
        rows.push(nr::Projector::<nr::V5> {
            projection_type: nr::ProjectionKind::from_raw(r.projection_type.raw()),
            bone: *bones.get(&attached.node).ok_or_else(|| {
                Error::new("m3.projector.node", "projector anchor has no native bone")
            })?,
            material_reference_index: *materials.get(&r.material).ok_or_else(|| {
                Error::new(
                    "m3.projector.material",
                    "projected material has no native table entry",
                )
            })?,
            flags: nr::ProjectorFlags(r.flags.bits()),
            offset: animated(&r.offset, ids),
            pitch: animated(&r.pitch, ids),
            yaw: animated(&r.yaw, ids),
            roll: animated(&r.roll, ids),
            field_of_view: animated(&r.field_of_view, ids),
            aspect_ratio: animated(&r.aspect_ratio, ids),
            near: animated(&r.near, ids),
            far: animated(&r.far, ids),
            box_offset_z_bottom: animated(&r.box_offset_z_bottom, ids),
            box_offset_z_top: animated(&r.box_offset_z_top, ids),
            box_offset_x_left: animated(&r.box_offset_x_left, ids),
            box_offset_x_right: animated(&r.box_offset_x_right, ids),
            box_offset_y_front: animated(&r.box_offset_y_front, ids),
            box_offset_y_back: animated(&r.box_offset_y_back, ids),
            active: animated(&r.active, ids),
            falloff: r.falloff,
            alpha_init: r.alpha_init,
            alpha_mid: r.alpha_mid,
            alpha_end: r.alpha_end,
            lifetime_attack: r.lifetime_attack,
            lifetime_attack_to: r.lifetime_attack_to,
            lifetime_hold: r.lifetime_hold,
            lifetime_hold_to: r.lifetime_hold_to,
            lifetime_decay: r.lifetime_decay,
            lifetime_decay_to: r.lifetime_decay_to,
            attenuation_distance: r.attenuation_distance,
            layer: r.layer,
            lod_reduce: r.lod_reduce,
            lod_cut: r.lod_cut,
        });
    }
    Ok(append(document, &rows)?.cast())
}
