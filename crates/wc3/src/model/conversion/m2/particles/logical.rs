//! Native particle fields mapped directly into independent authoring state.
use super::*;
use crate::model::conversion::{materialize, owned};
use crate::model::semantic::{
    self as semantic, profiles::m2 as profile, Animated, Curve, Resource, Time,
};
use crate::model::semantic::{
    AgeDrivenParticle, AgeDrivenParticleRendering, AgeDrivenParticleShape, ParticleEmission,
    ParticleLifetime, ParticleMotion,
};
use zerovec::ZeroVec;

pub(super) trait LogicalAge: Sized {
    fn make(value: &profile::Appearance<'_>, context: &mut Context) -> Result<Self, Error>;

    fn logical<'a>(&'a self) -> Result<profile::Appearance<'a>, Error>;
}
fn age<'a, T: Value, U: semantic::LinearValue + 'a>(
    track: &'a native::records::ParticleTrack<'_, T>,
    map: impl Fn(T) -> U + Send + Sync + 'a,
) -> Result<Option<Curve<'a, U>>, Error>
where
    T::ULE: Send + Sync,
{
    if track.timestamps.is_empty() {
        return Ok(None);
    }
    Ok(Some(Curve::linear(
        materialize(track.timestamps.len(), move |i| {
            Time::seconds(f64::from(track.timestamps.get(i).unwrap()) / 65535.).unwrap()
        }),
        materialize(track.values.len(), move |i| {
            map(track.values.get(i).unwrap())
        }),
    )?))
}
impl LogicalAge for native::records::ModernParticleLifetime<'_> {
    fn make(value: &profile::Appearance<'_>, context: &mut Context) -> Result<Self, Error> {
        if value.legacy_atlas.is_some() {
            context.unsupported(
                "m2.particle.atlas",
                "modern age tracks do not encode classic atlas segments",
            )?
        }
        Ok(Self {
            color: encode_age(
                value.color.as_ref(),
                context,
                |v, _| Ok(v.map(|c| c * 255.)),
            )?,
            alpha: encode_age(value.alpha.as_ref(), context, |v, c| {
                if !v.is_finite() || !(0. ..=65535. / 32767.).contains(&v) {
                    return Err(Error::new(
                        "m2.particle.alpha",
                        "opacity exceeds unsigned fixed16 range",
                    ));
                }
                let raw = (v * 32767.).round() as u16;
                c.approximate(
                    "m2.particle.alpha",
                    (f32::from(raw) / 32767. - v).abs(),
                    "age opacity quantized",
                )?;
                Ok(raw)
            })?,
            scale: encode_age(value.scale.as_ref(), context, |v, _| Ok(v))?,
            scale_variation: value.scale_variation,
            head_uv: encode_age(value.head_uv.as_ref(), context, |v, _| Ok(v))?,
            tail_uv: encode_age(value.tail_uv.as_ref(), context, |v, _| Ok(v))?,
        })
    }

    fn logical<'a>(&'a self) -> Result<profile::Appearance<'a>, Error> {
        Ok(profile::Appearance {
            legacy_atlas: None,
            color: age(&self.color, |v| v.map(|c| c / 255.))?,
            alpha: age(&self.alpha, |v| f32::from(v) / 32767.)?,
            scale: age(&self.scale, |v| v)?,
            scale_variation: self.scale_variation,
            head_uv: age(&self.head_uv, |v| v)?,
            tail_uv: age(&self.tail_uv, |v| v)?,
        })
    }
}
impl LogicalAge for native::records::LegacyParticleLifetime {
    fn make(value: &profile::Appearance<'_>, context: &mut Context) -> Result<Self, Error> {
        let mut row = Self::default();
        let curves = [
            value.color.as_ref().map(|v| v.times()),
            value.alpha.as_ref().map(|v| v.times()),
            value.scale.as_ref().map(|v| v.times()),
        ];
        let times = curves.into_iter().flatten().next();
        if let Some(times) = times {
            if times.len() != 3
                || times.get(0).unwrap().as_seconds() != 0.
                || times.get(2).unwrap().as_seconds() != 1.
            {
                return Err(Error::new(
                    "m2.particle.age",
                    "classic appearance requires beginning/midpoint/end keys",
                ));
            }
            row.midpoint = times.get(1).unwrap().as_seconds() as f32;
            for other in curves.into_iter().flatten() {
                if other.iter().ne(times.iter()) {
                    return Err(Error::new(
                        "m2.particle.age",
                        "classic age curves must share the midpoint",
                    ));
                }
            }
        }
        for i in 0..3 {
            let rgb = value
                .color
                .as_ref()
                .and_then(|c| c.values().get(i))
                .unwrap_or([1.; 3]);
            let alpha = value
                .alpha
                .as_ref()
                .and_then(|c| c.values().get(i))
                .unwrap_or(1.);
            let rgba = [rgb[2], rgb[1], rgb[0], alpha];
            for (j, v) in rgba.into_iter().enumerate() {
                if !(0. ..=1.).contains(&v) {
                    return Err(Error::new(
                        "m2.particle.color",
                        "classic appearance uses normalized byte colors",
                    ));
                }
                let raw = (v * 255.).round() as u8;
                context.approximate(
                    "m2.particle.color",
                    (f32::from(raw) / 255. - v).abs(),
                    "appearance color quantized to a byte",
                )?;
                row.colors_bgra[i][j] = raw;
            }
            let scale = value
                .scale
                .as_ref()
                .and_then(|c| c.values().get(i))
                .unwrap_or([1.; 2]);
            if scale[0] != scale[1] {
                context.unsupported("m2.particle.scale", "classic appearance has uniform scale")?
            }
            row.scales[i] = scale[0];
        }
        if let Some(atlas) = &value.legacy_atlas {
            row.head_lifespan_uv[..2].copy_from_slice(&atlas.head_lifespan);
            row.head_decay_uv[..2].copy_from_slice(&atlas.head_decay);
            row.tail_lifespan_uv = atlas.tail_lifespan;
            row.tail_decay_uv = atlas.tail_decay;
        }
        if value.head_uv.is_some() || value.tail_uv.is_some() {
            context.unsupported(
                "m2.particle.atlas",
                "classic appearance has atlas segments rather than keyed cell selection",
            )?
        }
        Ok(row)
    }

    fn logical<'a>(&'a self) -> Result<profile::Appearance<'a>, Error> {
        let times = owned(
            [0., f64::from(self.midpoint), 1.]
                .into_iter()
                .map(Time::seconds)
                .collect::<Result<Vec<_>, _>>()?,
        );
        let colors = self.colors_bgra.map(|c| {
            [
                f32::from(c[2]) / 255.,
                f32::from(c[1]) / 255.,
                f32::from(c[0]) / 255.,
            ]
        });
        let alpha = self.colors_bgra.map(|c| f32::from(c[3]) / 255.);
        let scale = self.scales.map(|v| [v, v]);
        Ok(profile::Appearance {
            legacy_atlas: Some(profile::LegacyAtlas {
                head_lifespan: self.head_lifespan_uv[..2].try_into().unwrap(),
                head_decay: self.head_decay_uv[..2].try_into().unwrap(),
                tail_lifespan: self.tail_lifespan_uv,
                tail_decay: self.tail_decay_uv,
            }),
            color: Some(Curve::linear(times.clone(), owned(colors.to_vec()))?),
            alpha: Some(Curve::linear(times.clone(), owned(alpha.to_vec()))?),
            scale: Some(Curve::linear(times, owned(scale.to_vec()))?),
            ..Default::default()
        })
    }
}
pub(super) fn read<'a, 'data: 'a, V: ModelLayout>(
    value: &'a native::records::ParticleEmitter<'data, V>,
    textures: &[semantic::TextureId],
) -> Result<profile::Particle<'a>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    V::Lifetime<'data>: LogicalAge,
{
    let (blend, kind, color) = value.kind.parts();
    let legacy = value.texture_scale.legacy_parts().unwrap_or([0, 0]);
    let multi = value.flags.multi_texture();
    let ids = if multi {
        vec![
            value.texture_id & 31,
            (value.texture_id >> 5) & 31,
            (value.texture_id >> 10) & 31,
        ]
    } else {
        vec![value.texture_id]
    };
    let textures = ids
        .into_iter()
        .map(|i| {
            textures
                .get(i as usize)
                .copied()
                .ok_or_else(|| Error::new("m2.particle.texture", "missing texture resource"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let resource = |path: &'a ZeroVec<'data, u8>| {
        let bytes = path.as_bytes();
        let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
        (!bytes.is_empty()).then_some(Resource::Path(Cow::Borrowed(bytes)))
    };
    let spline = &value.spline_points;
    Ok(profile::Particle {
        details: AgeDrivenParticle {
            id: value.id,
            flags: profile::values::ParticleFlags::from_bits_retain(value.flags.bits()),
            rendering: AgeDrivenParticleRendering {
                blend: profile::values::ParticleBlend::from_raw(blend.raw()),
                color_index: color as u16,
                textures,
                model: resource(&value.particle_model_filename),
                child_model: resource(&value.child_model_filename),
                head_tail: profile::values::ParticleHeadTail::from_raw(legacy[1]),
                texture_scale: value.texture_scale.scales(),
                texture_tile_rotation: value.texture_tile_rotation,
                atlas: [value.rows, value.columns],
                tail_length: value.tail_length,
                scroll: value
                    .multi_texture_scroll
                    .values()
                    .map(|v| v.map(|x| f32::from(x) / 512.)),
            },
            shape: AgeDrivenParticleShape {
                generator: profile::values::ParticleGenerator::from_raw(kind.raw()),
                legacy_shape: legacy[0],
                vertical_range: Animated::constant(initial(&value.vertical_range).unwrap_or(0.)),
                horizontal_range: Animated::constant(
                    initial(&value.horizontal_range).unwrap_or(0.),
                ),
                area_width: Animated::constant(initial(&value.emission_area_width).unwrap_or(0.)),
                area_length: Animated::constant(initial(&value.emission_area_length).unwrap_or(0.)),
                z_source: Animated::constant(initial(&value.z_source).unwrap_or(0.)),
                spline: materialize(value.spline_points.len(), move |i| spline.get(i).unwrap()),
            },
            twinkle_speed: value.twinkle_speed,
            twinkle_percent: value.twinkle_percent,
            twinkle_scale: value.twinkle_scale,
            inherit_velocity: value.inherit_velocity_scale,
            drag: value.drag,
            spin: value.spin.parts(),
            tumble_minimum: value.tumble_minimum,
            tumble_maximum: value.tumble_maximum,
            wind: value.wind,
            wind_time: value.wind_time,
            follow: [
                [value.follow_speed_1, value.follow_scale_1],
                [value.follow_speed_2, value.follow_scale_2],
            ],
        },
        motion: ParticleMotion {
            speed: Animated::constant(initial(&value.emission_speed).unwrap_or(0.)),
            variation: Animated::constant(initial(&value.speed_variation).unwrap_or(0.)),
            gravity: Animated::constant(gravity_vector(
                initial(&value.gravity).unwrap_or(0),
                value.flags.packed_gravity(),
            )?),
        },
        lifetime: ParticleLifetime {
            duration: Animated::constant(initial(&value.lifespan).unwrap_or(0.)),
            variation: value.lifespan_variation.get(),
        },
        emission: ParticleEmission {
            rate: Animated::constant(initial(&value.emission_rate).unwrap_or(0.)),
            variation: value.emission_rate_variation.get(),
            activation: Animated::constant(initial(&value.enabled).unwrap_or(1) != 0),
            trigger: (),
        },
        appearance: value.lifetime.logical()?,
    })
}

fn encode_age<T: semantic::CurveValue, U: Value>(
    curve: Option<&Curve<'_, T>>,
    context: &mut Context,
    map: impl Fn(T, &mut Context) -> Result<U, Error>,
) -> Result<native::records::ParticleTrack<'static, U>, Error> {
    let Some(curve) = curve else {
        return Ok(Default::default());
    };
    if curve.interpolation() != semantic::Interpolation::Linear {
        context.unsupported(
            "m2.particle.age",
            "native particle-age ramps use linear interpolation",
        )?
    }
    let mut times = Vec::new();
    let mut values = Vec::new();
    for (i, time) in curve.times().iter().enumerate() {
        let time = time.as_seconds();
        if !(0. ..=1.).contains(&time) {
            return Err(Error::new(
                "m2.particle.age",
                "age must be within zero and one",
            ));
        }
        let raw = (time * 65535.).round() as u16;
        context.approximate(
            "m2.particle.age",
            (f64::from(raw) / 65535. - time).abs() as f32,
            "age quantized to normalized u16",
        )?;
        times.push(raw);
        values.push(map(curve.values().get(i).unwrap(), context)?)
    }
    Ok(native::records::ParticleTrack {
        timestamps: ZeroVec::alloc_from_slice(&times),
        values: ZeroVec::alloc_from_slice(&values),
    })
}
pub(super) fn write<V: ModelLayout>(
    payload: &profile::Particle<'_>,
    textures: &[semantic::TextureId],
    context: &mut Context,
) -> Result<native::records::ParticleEmitter<'static, V>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    V::Lifetime<'static>: LogicalAge,
{
    let mut value = native::records::ParticleEmitter::<V>::new(ModelFlags::default());
    value.id = payload.details.id;
    value.flags = ParticleFlags::from_bits_retain(payload.details.flags.bits());
    let ids = payload
        .details
        .rendering
        .textures
        .iter()
        .map(|id| {
            textures
                .iter()
                .position(|t| t == id)
                .ok_or_else(|| Error::new("m2.particle.texture", "missing texture resource"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    value.texture_id = if value.flags.multi_texture() {
        if ids.len() != 3 || ids.iter().any(|i| *i > 31) {
            return Err(Error::new(
                "m2.particle.texture",
                "multi-texture particles require three five-bit indices",
            ));
        }
        (ids[0] | (ids[1] << 5) | (ids[2] << 10)) as u16
    } else {
        u16::try_from(
            *ids.first()
                .ok_or_else(|| Error::new("m2.particle.texture", "particle has no texture"))?,
        )
        .map_err(|_| Error::new("m2.particle.texture", "texture index exceeds u16"))?
    };
    let filename = |resource: &Option<Resource<'_>>| -> Result<ZeroVec<'static, u8>, Error> {
        match resource {
            None => Ok(ZeroVec::new()),
            Some(Resource::Path(path)) => Ok(ZeroVec::alloc_from_slice(
                &path.iter().copied().chain([0]).collect::<Vec<_>>(),
            )),
            _ => Err(Error::new(
                "m2.particle.model",
                "particle model requires a filename identity",
            )),
        }
    };
    value.particle_model_filename = filename(&payload.details.rendering.model)?;
    value.child_model_filename = filename(&payload.details.rendering.child_model)?;
    value.kind = HeaderFields::make(
        ParticleBlend::from_raw(payload.details.rendering.blend.raw()),
        ParticleGenerator::from_raw(payload.details.shape.generator.raw()),
        u32::from(payload.details.rendering.color_index),
    )?;
    value.texture_scale = UvFields::make(payload.details.rendering.texture_scale, context)?;
    value.texture_scale.set_legacy_parts(
        [
            payload.details.shape.legacy_shape,
            payload.details.rendering.head_tail.raw(),
        ],
        context,
    )?;
    value.texture_tile_rotation = payload.details.rendering.texture_tile_rotation;
    value.rows = payload.details.rendering.atlas[0];
    value.columns = payload.details.rendering.atlas[1];
    value.lifespan_variation = OptionalFloat::make(payload.lifetime.variation, context)?;
    value.emission_rate_variation = OptionalFloat::make(payload.emission.variation, context)?;
    value.lifetime = LogicalAge::make(&payload.appearance, context)?;
    value.tail_length = payload.details.rendering.tail_length;
    value.twinkle_speed = payload.details.twinkle_speed;
    value.twinkle_percent = payload.details.twinkle_percent;
    value.twinkle_scale = payload.details.twinkle_scale;
    value.inherit_velocity_scale = payload.details.inherit_velocity;
    value.drag = payload.details.drag;
    value.spin = SpinFields::make(payload.details.spin, context)?;
    value.tumble_minimum = payload.details.tumble_minimum;
    value.tumble_maximum = payload.details.tumble_maximum;
    value.wind = payload.details.wind;
    value.wind_time = payload.details.wind_time;
    value.follow_speed_1 = payload.details.follow[0][0];
    value.follow_scale_1 = payload.details.follow[0][1];
    value.follow_speed_2 = payload.details.follow[1][0];
    value.follow_scale_2 = payload.details.follow[1][1];
    value.spline_points =
        ZeroVec::alloc_from_slice(&payload.details.shape.spline.iter().collect::<Vec<_>>());
    let scroll = payload
        .details
        .rendering
        .scroll
        .map(|v| v.map(|v| (v * 512.).round() as i16));
    if let Some(original) = payload.details.rendering.scroll {
        for (i, v) in original.into_iter().enumerate() {
            if !v.is_finite() || v * 512. < f32::from(i16::MIN) || v * 512. > f32::from(i16::MAX) {
                return Err(Error::new(
                    "m2.particle.scroll",
                    "UV scrolling exceeds fixed16 range",
                ));
            }
            context.approximate(
                "m2.particle.scroll",
                (f32::from(scroll.unwrap()[i]) / 512. - v).abs(),
                "UV scrolling quantized",
            )?
        }
    }
    value.multi_texture_scroll = ScrollFields::make(scroll, context)?;
    Ok(value)
}
