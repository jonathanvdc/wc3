//! Native light color, cone, attenuation, and independent source controls.
use super::metadata::restore;
use super::{animated, install, native, nr, read_error, revision, wem, wr, Envelope, Error, Ref};
use std::collections::BTreeMap;
pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::Light>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error> {
    if reference.entries == 0 {
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
                let mut values = BTreeMap::new();
                for (key, value) in [
                    ("type", u32::from((r.light_type).raw())),
                    ("flags", (r.flags).bits()),
                    ("lod_cut", r.lod_cut),
                    ("shadow_lod_cut", r.shadow_lod_cut),
                ] {
                    values.insert(format!("m3.light.{key}"), i64::from(value));
                }
                animated(&mut values, "light.color", r.diffuse_color);
                for (key, value) in [
                    ("light.intensity", r.intensity_multiplier),
                    ("light.specular_multiplier", r.specular_multiplier),
                    ("light.decay", r.decay),
                    ("light.attenuation_start", r.attenuation_start),
                    ("light.hot_spot", r.hot_spot),
                    ("light.falloff", r.falloff),
                ] {
                    animated(&mut values, key, value);
                }
                animated(&mut values, "light.specular_color", r.specular_color);
                for (index, value) in r.specular_color.initial_value.into_iter().enumerate() {
                    values.insert(
                        format!("m3.light.specular_color.initial[{index}]"),
                        i64::from(value.to_bits()),
                    );
                }
                values.insert(
                    "m3.light.specular_multiplier.initial".into(),
                    i64::from(r.specular_multiplier.initial_value.to_bits()),
                );
                values.insert(
                    "m3.light.decay.initial".into(),
                    i64::from(r.decay.initial_value.to_bits()),
                );
                let kind = match r.light_type {
                    nr::LightKind::Directional => wr::LightKind::Directional,
                    nr::LightKind::Point => wr::LightKind::Omni,
                    nr::LightKind::Spot => wr::LightKind::Spot,
                    _ => wr::LightKind::from_raw(255),
                };
                let payload = wr::LightPayload::<wr::V19> {
                    kind,
                    shadow_casting: wem::Bool::from(r.flags.shadows()),
                    color: r.diffuse_color.initial_value,
                    intensity: r.intensity_multiplier.initial_value,
                    attenuation_start: r.attenuation_start.initial_value,
                    attenuation_end: r.attenuation_end,
                    hot_spot: r.hot_spot.initial_value,
                    falloff: r.falloff.initial_value,
                    ..Default::default()
                };
                install(
                    doc,
                    nodes,
                    u32::from(r.bone_index),
                    wr::NodePayload::Light(Box::new(payload)),
                    wr::NodeKind::Light,
                    values,
                )?;
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
pub(super) fn export(
    index: usize,
    payload: wr::LightPayload<wr::V19>,
    values: &BTreeMap<String, i64>,
) -> Result<nr::Light<nr::V7>, Error> {
    if payload.shadow_casting_start != 0.
        || payload.shadow_casting_end != 0.
        || payload.quadratic_falloff != 0.
        || payload.linear_falloff != 0.
        || payload.damping != 0.
    {
        return Err(Error::new(
            format!("nodes[{index}].light"),
            "additional WEM attenuation/shadow controls have no M3 mapping",
        ));
    }
    let get = |key: &str| values.get(&format!("m3.light.{key}")).copied().unwrap_or(0);
    let color =
        [0, 1, 2].map(|i| f32::from_bits(get(&format!("specular_color.initial[{i}]")) as u32));
    let light_type = match payload.kind {
        wr::LightKind::Directional => 0,
        wr::LightKind::Omni => 1,
        wr::LightKind::Spot => 2,
        _ => u16::try_from(get("type"))
            .map_err(|_| Error::new("m3.light.kind", "source light type exceeds native width"))?,
    };
    Ok(nr::Light {
        light_type: nr::LightKind::from_raw(light_type),
        bone_index: u16::try_from(index)
            .map_err(|_| Error::new("m3.light.bone", "bone index exceeds native width"))?,
        flags: nr::LightFlags((get("flags") as u32 & !1) | u32::from(payload.shadow_casting.get())),
        lod_cut: get("lod_cut") as u32,
        shadow_lod_cut: get("shadow_lod_cut") as u32,
        diffuse_color: restore(values, "light.color", payload.color),
        intensity_multiplier: restore(values, "light.intensity", payload.intensity),
        specular_color: restore(values, "light.specular_color", color),
        specular_multiplier: restore(
            values,
            "light.specular_multiplier",
            f32::from_bits(get("specular_multiplier.initial") as u32),
        ),
        decay: restore(
            values,
            "light.decay",
            f32::from_bits(get("decay.initial") as u32),
        ),
        attenuation_end: payload.attenuation_end,
        attenuation_start: restore(values, "light.attenuation_start", payload.attenuation_start),
        hot_spot: restore(values, "light.hot_spot", payload.hot_spot),
        falloff: restore(values, "light.falloff", payload.falloff),
    })
}
