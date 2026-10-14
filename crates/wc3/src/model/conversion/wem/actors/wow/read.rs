//! Particle declarations retain borrowed age-key buffers and historical field presence.
use super::*;
pub(in crate::model::conversion::wem) fn read<'view, D: records::M2ParticleEmitterPayloadDialect>(
    document: &'view RawDocument<'_>,
    source: &records::M2ParticleEmitterPayload<D>,
    textures: &[TextureId],
) -> Result<Particle<'view>, Error>
where
    D::Texture2Scale: Field<f32>,
    D::Texture3Scale: Field<f32>,
    D::Texture2ScrollMid: Field<[f32; 2]>,
    D::Texture3ScrollMid: Field<[f32; 2]>,
    D::Texture2ScrollRange: Field<[f32; 2]>,
    D::Texture3ScrollRange: Field<[f32; 2]>,
    D::InheritVelocityScale: Field<f32>,
    D::FollowSpeed1: Field<f32>,
    D::FollowScale1: Field<f32>,
    D::FollowSpeed2: Field<f32>,
    D::FollowScale2: Field<f32>,
    D::TumbleMin: Field<[f32; 3]>,
    D::TumbleMax: Field<[f32; 3]>,
    D::ColorMult: Field<f32>,
    D::AlphaMult: Field<f32>,
    D::AlphaCutoffTimes: Field<Ref<records::targets::Real>>,
    D::AlphaCutoffs: Field<Ref<records::targets::Real>>,
    D::ParticleModel: Field<records::AssetKey>,
    D::TrailModel: Field<records::AssetKey>,
{
    Ok(Particle {
        particle_id: source.particle_id,
        flags: ParticleFlags::from_bits_retain(source.flags.bits()),
        texture: texture(source.texture, textures)?,
        texture2: texture(source.texture2, textures)?,
        texture3: texture(source.texture3, textures)?,
        blend: match source.blend {
            records::M2ParticleBlend32::Opaque => ParticleBlend::Opaque,
            records::M2ParticleBlend32::Alpha => ParticleBlend::Alpha,
            records::M2ParticleBlend32::Add => ParticleBlend::Additive,
            records::M2ParticleBlend32::AlphaKey => ParticleBlend::AlphaKey,
            records::M2ParticleBlend32::AddAlphaKey => ParticleBlend::AdditiveAlphaKey,
            records::M2ParticleBlend32::Unknown(code) => ParticleBlend::Unknown(code),
        },
        emitter_type: match source.emitter_type {
            records::M2EmitterShape::Plane => EmitterShape::Plane,
            records::M2EmitterShape::Sphere => EmitterShape::Sphere,
            records::M2EmitterShape::Spline => EmitterShape::Spline,
            records::M2EmitterShape::Bone => EmitterShape::Bone,
            records::M2EmitterShape::Unknown(code) => EmitterShape::Unknown(code),
        },
        color_index: source.color_index,
        rows: source.rows,
        columns: source.columns,
        priority_plane: source.priority_plane,
        speed: Animated::constant(source.speed),
        speed_variation: Animated::constant(source.speed_variation),
        vertical_range: Animated::constant(source.vertical_range),
        horizontal_range: Animated::constant(source.horizontal_range),
        gravity: Animated::constant(source.gravity),
        lifespan: Animated::constant(source.lifespan),
        lifespan_variation: source.lifespan_variation,
        emission_rate: Animated::constant(source.emission_rate),
        emission_rate_variation: source.emission_rate_variation,
        width: Animated::constant(source.width),
        length: Animated::constant(source.length),
        z_source: Animated::constant(source.z_source),
        color_times: values::<records::Real, _>(document, source.color_times)?,
        colors: values::<records::Vector3, _>(document, source.colors)?,
        alpha_times: values::<records::Real, _>(document, source.alpha_times)?,
        alphas: values::<records::Real, _>(document, source.alphas)?,
        scale_times: values::<records::Real, _>(document, source.scale_times)?,
        scales: values::<records::Vector2, _>(document, source.scales)?,
        scale_variation: source.scale_variation,
        head_cell_times: values::<records::Real, _>(document, source.head_cell_times)?,
        head_cells: values::<records::U32, _>(document, source.head_cells)?,
        tail_cell_times: values::<records::Real, _>(document, source.tail_cell_times)?,
        tail_cells: values::<records::U32, _>(document, source.tail_cells)?,
        tail_length: source.tail_length,
        twinkle_speed: source.twinkle_speed,
        twinkle_percent: source.twinkle_percent,
        twinkle_scale: source.twinkle_scale,
        drag: source.drag,
        base_spin: source.base_spin,
        base_spin_variation: source.base_spin_variation,
        spin: source.spin,
        spin_variation: source.spin_variation,
        wind: source.wind,
        wind_time: source.wind_time,
        spline_points: values::<records::Vector3, _>(document, source.spline_points)?,
        texture2_scale: source.texture2_scale.read(),
        texture3_scale: source.texture3_scale.read(),
        texture2_scroll_mid: source.texture2_scroll_mid.read(),
        texture3_scroll_mid: source.texture3_scroll_mid.read(),
        texture2_scroll_range: source.texture2_scroll_range.read(),
        texture3_scroll_range: source.texture3_scroll_range.read(),
        inherit_velocity_scale: source.inherit_velocity_scale.read(),
        follow_speed1: source.follow_speed1.read(),
        follow_scale1: source.follow_scale1.read(),
        follow_speed2: source.follow_speed2.read(),
        follow_scale2: source.follow_scale2.read(),
        tumble_min: source.tumble_min.read(),
        tumble_max: source.tumble_max.read(),
        color_mult: source.color_mult.read(),
        alpha_mult: source.alpha_mult.read(),
        alpha_cutoff_times: source
            .alpha_cutoff_times
            .read()
            .map(|reference| values::<records::Real, _>(document, reference))
            .transpose()?,
        alpha_cutoffs: source
            .alpha_cutoffs
            .read()
            .map(|reference| values::<records::Real, _>(document, reference))
            .transpose()?,
        particle_model: source
            .particle_model
            .read()
            .map(|key| read_asset(document, &key))
            .transpose()?,
        trail_model: source
            .trail_model
            .read()
            .map(|key| read_asset(document, &key))
            .transpose()?,
    })
}
