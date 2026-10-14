//! Particle output requires the chosen owner vocabulary to preserve authored field presence.
use super::*;
pub(in crate::model::conversion::wem) fn write<
    'view,
    'data: 'view,
    D: records::M2ParticleEmitterPayloadDialect,
>(
    document: &mut RawDocument<'view>,
    source: &Particle<'data>,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<records::M2ParticleEmitterPayload<D>, Error>
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
    Ok(records::M2ParticleEmitterPayload {
        particle_id: source.particle_id,
        flags: records::M2ParticleFlags::from_bits_retain(source.flags.bits()),
        texture: texture_index(source.texture, textures)?,
        texture2: texture_index(source.texture2, textures)?,
        texture3: texture_index(source.texture3, textures)?,
        blend: match source.blend {
            ParticleBlend::Opaque => records::M2ParticleBlend32::Opaque,
            ParticleBlend::Alpha => records::M2ParticleBlend32::Alpha,
            ParticleBlend::Additive => records::M2ParticleBlend32::Add,
            ParticleBlend::AlphaKey => records::M2ParticleBlend32::AlphaKey,
            ParticleBlend::AdditiveAlphaKey => records::M2ParticleBlend32::AddAlphaKey,
            ParticleBlend::Unknown(code) => records::M2ParticleBlend32::from_raw(code),
        },
        emitter_type: match source.emitter_type {
            EmitterShape::Plane => records::M2EmitterShape::Plane,
            EmitterShape::Sphere => records::M2EmitterShape::Sphere,
            EmitterShape::Spline => records::M2EmitterShape::Spline,
            EmitterShape::Bone => records::M2EmitterShape::Bone,
            EmitterShape::Unknown(code) => records::M2EmitterShape::from_raw(code),
        },
        color_index: source.color_index,
        rows: source.rows,
        columns: source.columns,
        priority_plane: source.priority_plane,
        speed: source.speed.default,
        speed_variation: source.speed_variation.default,
        vertical_range: source.vertical_range.default,
        horizontal_range: source.horizontal_range.default,
        gravity: source.gravity.default,
        lifespan: source.lifespan.default,
        lifespan_variation: source.lifespan_variation,
        emission_rate: source.emission_rate.default,
        emission_rate_variation: source.emission_rate_variation,
        width: source.width.default,
        length: source.length.default,
        z_source: source.z_source.default,
        color_times: put::<records::Real, _>(document, &source.color_times)?,
        colors: put::<records::Vector3, _>(document, &source.colors)?,
        alpha_times: put::<records::Real, _>(document, &source.alpha_times)?,
        alphas: put::<records::Real, _>(document, &source.alphas)?,
        scale_times: put::<records::Real, _>(document, &source.scale_times)?,
        scales: put::<records::Vector2, _>(document, &source.scales)?,
        scale_variation: source.scale_variation,
        head_cell_times: put::<records::Real, _>(document, &source.head_cell_times)?,
        head_cells: put::<records::U32, _>(document, &source.head_cells)?,
        tail_cell_times: put::<records::Real, _>(document, &source.tail_cell_times)?,
        tail_cells: put::<records::U32, _>(document, &source.tail_cells)?,
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
        spline_points: put::<records::Vector3, _>(document, &source.spline_points)?,
        texture2_scale: Field::write(source.texture2_scale)?,
        texture3_scale: Field::write(source.texture3_scale)?,
        texture2_scroll_mid: Field::write(source.texture2_scroll_mid)?,
        texture3_scroll_mid: Field::write(source.texture3_scroll_mid)?,
        texture2_scroll_range: Field::write(source.texture2_scroll_range)?,
        texture3_scroll_range: Field::write(source.texture3_scroll_range)?,
        inherit_velocity_scale: Field::write(source.inherit_velocity_scale)?,
        follow_speed1: Field::write(source.follow_speed1)?,
        follow_scale1: Field::write(source.follow_scale1)?,
        follow_speed2: Field::write(source.follow_speed2)?,
        follow_scale2: Field::write(source.follow_scale2)?,
        tumble_min: Field::write(source.tumble_min)?,
        tumble_max: Field::write(source.tumble_max)?,
        color_mult: Field::write(source.color_mult)?,
        alpha_mult: Field::write(source.alpha_mult)?,
        alpha_cutoff_times: Field::write(
            source
                .alpha_cutoff_times
                .as_ref()
                .map(|values| put::<records::Real, _>(document, values))
                .transpose()?,
        )?,
        alpha_cutoffs: Field::write(
            source
                .alpha_cutoffs
                .as_ref()
                .map(|values| put::<records::Real, _>(document, values))
                .transpose()?,
        )?,
        particle_model: Field::write(
            source
                .particle_model
                .as_ref()
                .map(|key| write_asset(document, key))
                .transpose()?,
        )?,
        trail_model: Field::write(
            source
                .trail_model
                .as_ref()
                .map(|key| write_asset(document, key))
                .transpose()?,
        )?,
    })
}
