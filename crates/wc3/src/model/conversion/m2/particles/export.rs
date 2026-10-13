//! Particle export conversion components.
use super::*;

pub(super) fn export_particle<V: ModelLayout>(
    payload: &wr::M2ParticleEmitterPayload<wr::V19>,
    doc: &wem::RawDocument<'_>,
    context: &mut Context,
) -> Result<native::records::ParticleEmitter<'static, V>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    for<'a> V::Lifetime<'a>: AgeFields,
{
    let multi = ParticleFlags::from_bits_retain(payload.flags.bits()).multi_texture();
    let texture_id = if multi {
        if [payload.texture, payload.texture2, payload.texture3]
            .iter()
            .any(|v| *v > 31)
        {
            return Err(Error::new(
                "particles.texture",
                "multi-texture indices exceed packed five-bit slots",
            ));
        }
        (payload.texture | (payload.texture2 << 5) | (payload.texture3 << 10)) as u16
    } else {
        u16::try_from(payload.texture)
            .map_err(|_| Error::new("particles.texture", "texture index exceeds u16"))?
    };
    let filename = |asset: &wr::AssetKey| -> Result<ZeroVec<'static, u8>, Error> {
        if asset.id != u32::MAX && asset.id != 0 {
            return Err(Error::new(
                "particles.model",
                "numeric particle-model identities need recursive-model ID chunks",
            ));
        }
        let text = doc.string(asset.path)?.unwrap_or_default();
        Ok(if text.is_empty() {
            ZeroVec::new()
        } else {
            ZeroVec::alloc_from_slice(&text.bytes().chain([0]).collect::<Vec<_>>())
        })
    };
    let mut value = native::records::ParticleEmitter::<V>::new(ModelFlags::default());
    value.id = payload.particle_id as u32;
    value.flags = ParticleFlags::from_bits_retain(payload.flags.bits());
    value.texture_id = texture_id;
    value.particle_model_filename = filename(&payload.particle_model)?;
    value.child_model_filename = filename(&payload.trail_model)?;
    value.kind = HeaderFields::make(payload.blend, payload.emitter_type, payload.color_index)?;
    value.texture_scale =
        UvFields::make([payload.texture2_scale, payload.texture3_scale], context)?;
    value.texture_tile_rotation = i16::try_from(payload.priority_plane)
        .map_err(|_| Error::new("particles.priority", "priority exceeds i16"))?;
    value.rows = u16::try_from(payload.rows)
        .map_err(|_| Error::new("particles.rows", "atlas rows exceed u16"))?;
    value.columns = u16::try_from(payload.columns)
        .map_err(|_| Error::new("particles.columns", "atlas columns exceed u16"))?;
    value.lifespan_variation = OptionalFloat::make(payload.lifespan_variation, context)?;
    value.emission_rate_variation = OptionalFloat::make(payload.emission_rate_variation, context)?;
    value.tail_length = payload.tail_length;
    value.twinkle_speed = payload.twinkle_speed;
    value.twinkle_percent = payload.twinkle_percent;
    value.twinkle_scale = payload.twinkle_scale;
    value.inherit_velocity_scale = payload.inherit_velocity_scale;
    value.drag = payload.drag;
    value.spin = SpinFields::make(
        [
            payload.base_spin,
            payload.base_spin_variation,
            payload.spin,
            payload.spin_variation,
        ],
        context,
    )?;
    value.tumble_minimum = payload.tumble_min;
    value.tumble_maximum = payload.tumble_max;
    value.wind = payload.wind;
    value.wind_time = payload.wind_time;
    value.follow_speed_1 = payload.follow_speed1;
    value.follow_scale_1 = payload.follow_scale1;
    value.follow_speed_2 = payload.follow_speed2;
    value.follow_scale_2 = payload.follow_scale2;
    value.spline_points = ZeroVec::alloc_from_slice(
        &doc.resolve::<wr::Vector3>(payload.spline_points.cast())?
            .iter()
            .map(|v| v.map(|v| v.value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    value.lifetime = AgeFields::export(doc, payload, context)?;
    let scroll = [
        payload.texture2_scroll_mid,
        payload.texture3_scroll_mid,
        payload.texture2_scroll_range,
        payload.texture3_scroll_range,
    ]
    .into_iter()
    .flatten()
    .map(|v| {
        if !v.is_finite() || v * 512. < f32::from(i16::MIN) || v * 512. > f32::from(i16::MAX) {
            return Err(Error::new(
                "particles.scroll",
                "UV scrolling exceeds fixed16_9 range",
            ));
        }
        let encoded = (v * 512.).round() as i16;
        context.approximate(
            "particles.scroll",
            (v - f32::from(encoded) / 512.).abs(),
            "UV scroll quantization",
        )?;
        Ok(encoded)
    })
    .collect::<Result<Vec<_>, Error>>()?;
    value.multi_texture_scroll = ScrollFields::make(
        (V::VERSION >= 264).then(|| scroll.try_into().unwrap()),
        context,
    )?;
    Ok(value)
}
