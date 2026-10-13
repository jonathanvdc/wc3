//! Particle import conversion components.
use super::*;

pub(super) fn import_particle<V: ModelLayout>(
    value: &native::records::ParticleEmitter<'_, V>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M2ParticleEmitterPayload<wr::V19>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    for<'a> V::Lifetime<'a>: AgeFields,
{
    let (blend, kind, color) = value.kind.parts();
    let [base, base_variation, spin, spin_variation] = value.spin.parts();
    let [texture2_scale, texture3_scale] = value.texture_scale.scales();
    let multi = value.flags.multi_texture();
    let mut payload = wr::M2ParticleEmitterPayload {
        particle_id: value.id as i32,
        flags: wr::M2ParticleFlags::from_bits_retain(value.flags.bits()),
        texture: if multi {
            u32::from(value.texture_id & 31)
        } else {
            u32::from(value.texture_id)
        },
        texture2: if multi {
            u32::from((value.texture_id >> 5) & 31)
        } else {
            u32::MAX
        },
        texture3: if multi {
            u32::from((value.texture_id >> 10) & 31)
        } else {
            u32::MAX
        },
        blend: wr::M2ParticleBlend32::from_raw(u32::from(blend.raw())),
        emitter_type: wr::M2EmitterShape::from_raw(u32::from(kind.raw())),
        color_index: color,
        rows: u32::from(value.rows),
        columns: u32::from(value.columns),
        priority_plane: i32::from(value.texture_tile_rotation),
        speed: initial(&value.emission_speed).unwrap_or(0.),
        speed_variation: initial(&value.speed_variation).unwrap_or(0.),
        vertical_range: initial(&value.vertical_range).unwrap_or(0.),
        horizontal_range: initial(&value.horizontal_range).unwrap_or(0.),
        gravity: gravity_vector(
            initial(&value.gravity).unwrap_or(0),
            value.flags.packed_gravity(),
        )?,
        lifespan: initial(&value.lifespan).unwrap_or(0.),
        lifespan_variation: value.lifespan_variation.get(),
        emission_rate: initial(&value.emission_rate).unwrap_or(0.),
        emission_rate_variation: value.emission_rate_variation.get(),
        width: initial(&value.emission_area_width).unwrap_or(0.),
        length: initial(&value.emission_area_length).unwrap_or(0.),
        z_source: initial(&value.z_source).unwrap_or(0.),
        tail_length: value.tail_length,
        twinkle_speed: value.twinkle_speed,
        twinkle_percent: value.twinkle_percent,
        twinkle_scale: value.twinkle_scale,
        drag: value.drag,
        base_spin: base,
        base_spin_variation: base_variation,
        spin,
        spin_variation,
        wind: value.wind,
        wind_time: value.wind_time,
        texture2_scale,
        texture3_scale,
        inherit_velocity_scale: value.inherit_velocity_scale,
        follow_speed1: value.follow_speed_1,
        follow_scale1: value.follow_scale_1,
        follow_speed2: value.follow_speed_2,
        follow_scale2: value.follow_scale_2,
        tumble_min: value.tumble_minimum,
        tumble_max: value.tumble_maximum,
        color_mult: 1.,
        alpha_mult: 1.,
        ..Default::default()
    };
    payload.spline_points = put(
        doc,
        &value
            .spline_points
            .iter()
            .map(|value| wr::Vector3 { value })
            .collect::<Vec<_>>(),
    )?
    .cast();
    if let Some(scroll) = value.multi_texture_scroll.values() {
        payload.texture2_scroll_mid = [f32::from(scroll[0]) / 512., f32::from(scroll[1]) / 512.];
        payload.texture3_scroll_mid = [f32::from(scroll[2]) / 512., f32::from(scroll[3]) / 512.];
        payload.texture2_scroll_range = [f32::from(scroll[4]) / 512., f32::from(scroll[5]) / 512.];
        payload.texture3_scroll_range = [f32::from(scroll[6]) / 512., f32::from(scroll[7]) / 512.];
    }
    let asset = |doc: &mut wem::RawDocument<'static>,
                 path: &ZeroVec<'_, u8>|
     -> Result<wr::AssetKey, Error> {
        let text = text(path.iter(), "particles.model")?;
        let chars: Vec<_> = text.bytes().map(|value| wr::Character { value }).collect();
        Ok(wr::AssetKey {
            group: 0,
            id: u32::MAX,
            path: put(doc, &chars)?.cast(),
        })
    };
    payload.particle_model = asset(doc, &value.particle_model_filename)?;
    payload.trail_model = asset(doc, &value.child_model_filename)?;
    value.lifetime.import(doc, &mut payload)?;
    Ok(payload)
}
