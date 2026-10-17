//! Complete light definitions attached to their real skeleton anchors.
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
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: Ref<nr::targets::Light>,
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
                .resolve::<nr::Light<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let node = *skeleton
                    .nodes
                    .get(r.bone_index as usize)
                    .ok_or_else(|| Error::new("m3.light.node", "anchor exceeds skeleton"))?;
                let feature = builder.reserve_feature(node);
                let mut light = surface::Light {
                    light_type: surface::values::LightKind::from_raw(r.light_type.raw()),
                    flags: surface::values::LightFlags(r.flags.bits()),
                    lod_cut: r.lod_cut,
                    shadow_lod_cut: r.shadow_lod_cut,
                    diffuse_color: anim(r.diffuse_color, builder.values_mut()),
                    intensity_multiplier: anim(r.intensity_multiplier, builder.values_mut()),
                    specular_color: anim(r.specular_color, builder.values_mut()),
                    specular_multiplier: anim(r.specular_multiplier, builder.values_mut()),
                    decay: anim(r.decay, builder.values_mut()),
                    attenuation_end: r.attenuation_end,
                    attenuation_start: anim(r.attenuation_start, builder.values_mut()),
                    hot_spot: anim(r.hot_spot, builder.values_mut()),
                    falloff: anim(r.falloff, builder.values_mut()),
                };
                bindings::field(
                    &mut light.diffuse_color,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::DiffuseColor),
                    registry,
                );
                bindings::field(
                    &mut light.intensity_multiplier,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::IntensityMultiplier),
                    registry,
                );
                bindings::field(
                    &mut light.specular_color,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::SpecularColor),
                    registry,
                );
                bindings::field(
                    &mut light.specular_multiplier,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::SpecularMultiplier),
                    registry,
                );
                bindings::field(
                    &mut light.decay,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::Decay),
                    registry,
                );
                bindings::field(
                    &mut light.attenuation_start,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::AttenuationStart),
                    registry,
                );
                bindings::field(
                    &mut light.hot_spot,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::HotSpot),
                    registry,
                );
                bindings::field(
                    &mut light.falloff,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Light).child(Field::Falloff),
                    registry,
                );
                builder.set_feature(feature, surface::Feature::Light((light).into()))?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        _ => return Err(Error::new("m3.lights", "unsupported light revision")),
    }
    Ok(())
}

pub(super) fn write<E: Envelope>(
    model: &Graph<'_>,
    document: &mut native::Container<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
    ids: &super::animation::AnimationIds,
) -> Result<Ref<nr::targets::Light>, Error> {
    use super::write::{animated, append};
    let mut rows = Vec::new();
    for (_, attached) in model.features() {
        let surface::Feature::Light(light) = &attached.payload else {
            continue;
        };
        let bone_index = *bones
            .get(&attached.node)
            .ok_or_else(|| Error::new("m3.light.node", "light anchor has no native bone"))?;
        rows.push(nr::Light::<nr::V7> {
            bone_index: u16::try_from(bone_index).map_err(|_| {
                Error::new("m3.light.node", "light anchor exceeds native u16 capacity")
            })?,
            light_type: nr::LightKind::from_raw(light.light_type.raw()),
            flags: nr::LightFlags(light.flags.bits()),
            lod_cut: light.lod_cut,
            shadow_lod_cut: light.shadow_lod_cut,
            diffuse_color: animated(&light.diffuse_color, ids),
            intensity_multiplier: animated(&light.intensity_multiplier, ids),
            specular_color: animated(&light.specular_color, ids),
            specular_multiplier: animated(&light.specular_multiplier, ids),
            decay: animated(&light.decay, ids),
            attenuation_end: light.attenuation_end,
            attenuation_start: animated(&light.attenuation_start, ids),
            hot_spot: animated(&light.hot_spot, ids),
            falloff: animated(&light.falloff, ids),
        });
    }
    Ok(append(document, &rows)?.cast())
}
