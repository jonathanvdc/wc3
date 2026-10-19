//! Native particle fields mapped directly into independent authoring state.
use super::*;
use crate::model::common::PackedArray;
use crate::model::io::m2::graph::texture_index;
use crate::model::io::{materialize, owned};
use crate::model::{self as semantic, profiles::m2 as profile, Curve, Time};
use crate::model::{
    M2ParticleDetails, M2ParticleRendering, M2ParticleShape, ParticleEmission, ParticleLifetime,
    ParticleMotion,
};
use glam::{Vec2, Vec3, Vec4};
use profile::values::ParticleHeadTail as GeometrySelection;
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
            Time::seconds(f64::from(track.timestamps.get(i).unwrap()) / 32767.).unwrap()
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
            color: encode_age(value.color.as_ref(), context, |v, _| {
                Ok(v.to_array().map(|c| c * 255.))
            })?,
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
            scale: encode_age(value.scale.as_ref(), context, |v, _| Ok(v.to_array()))?,
            scale_variation: (value.scale_variation).to_array(),
            head_uv: encode_age(value.head_uv.as_ref(), context, |v, _| Ok(v))?,
            tail_uv: encode_age(value.tail_uv.as_ref(), context, |v, _| Ok(v))?,
        })
    }

    fn logical<'a>(&'a self) -> Result<profile::Appearance<'a>, Error> {
        Ok(profile::Appearance {
            legacy_atlas: None,
            color: age(&self.color, |v| Vec3::from_array(v.map(|c| c / 255.)))?,
            alpha: age(&self.alpha, |v| f32::from(v) / 32767.)?,
            scale: age(&self.scale, Vec2::from_array)?,
            scale_variation: Vec2::from_array(self.scale_variation),
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
                .unwrap_or(Vec3::from_array([1.; 3]));
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
                .unwrap_or(Vec2::from_array([1.; 2]));
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
            color: Some(Curve::linear(
                times.clone(),
                owned(colors.map(Vec3::from_array)),
            )?),
            alpha: Some(Curve::linear(times.clone(), owned(alpha))?),
            scale: Some(Curve::linear(times, owned(scale.map(Vec2::from_array)))?),
            ..Default::default()
        })
    }
}
pub(super) fn read<'a, 'data: 'a, V: ModelLayout>(
    value: &'a native::records::ParticleEmitter<'data, V>,
    textures: &[semantic::Texture],
    builder: &mut semantic::ModelBuilder<'a, semantic::profiles::M2>,
) -> Result<profile::ParticleEmitter<'a>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    V::Lifetime<'data>: LogicalAge,
{
    let (blend, kind, color) = value.kind.parts();
    let legacy = value.texture_scale.legacy_parts();
    let head_tail = legacy.map_or_else(
        || match (value.flags.head(), value.flags.tail()) {
            (true, false) => GeometrySelection::Head,
            (false, true) => GeometrySelection::Tail,
            (true, true) => GeometrySelection::Both,
            (false, false) => GeometrySelection::Disabled,
        },
        |parts| GeometrySelection::from_legacy_code(parts[1]),
    );
    let legacy = legacy.unwrap_or([0, 0]);
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
                .map(|mut image| {
                    image.sampler = semantic::Sampler::default();
                    image
                })
                .ok_or_else(|| Error::new("m2.particle.texture", "missing texture resource"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut resource = |path: &'a ZeroVec<'data, u8>| {
        let bytes = path.as_bytes();
        let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
        (!bytes.is_empty())
            .then(|| builder.add_model_resource(semantic::PathResource::Path(Cow::Borrowed(bytes))))
    };
    let spline = &value.spline_points;
    Ok(profile::ParticleEmitter {
        details: M2ParticleDetails {
            id: value.id,
            flags: profile::values::ParticleFlags::from_bits_retain(value.flags.bits()),
            rendering: M2ParticleRendering {
                blend: profile::values::ParticleBlend::from_raw(blend.raw()),
                color_index: color as u16,
                textures,
                texture_program: None,
                model: resource(&value.particle_model_filename),
                child_model: resource(&value.child_model_filename),
                head_tail,
                texture_scale: Vec2::from_array(value.texture_scale.scales()),
                texture_tile_rotation: value.texture_tile_rotation,
                atlas: [value.rows, value.columns],
                tail_length: value.tail_length,
                scroll: value
                    .multi_texture_scroll
                    .values()
                    .map(|v| v.map(|x| f32::from(x) / 512.)),
            },
            shape: M2ParticleShape {
                generator: profile::values::ParticleGenerator::from_raw(kind.raw()),
                legacy_shape: legacy[0],
                vertical_range: builder.add_value(initial(&value.vertical_range).unwrap_or(0.)),
                horizontal_range: builder.add_value(initial(&value.horizontal_range).unwrap_or(0.)),
                area_width: builder.add_value(initial(&value.emission_area_width).unwrap_or(0.)),
                area_length: builder.add_value(initial(&value.emission_area_length).unwrap_or(0.)),
                z_source: builder.add_value(initial(&value.z_source).unwrap_or(0.)),
                spline: PackedArray::from_storage(
                    (materialize(value.spline_points.len(), move |i| spline.get(i).unwrap()))
                        .into_storage(),
                ),
            },
            twinkle_speed: value.twinkle_speed,
            twinkle_percent: value.twinkle_percent,
            twinkle_scale: Vec2::from_array(value.twinkle_scale),
            inherit_velocity: value.inherit_velocity_scale,
            drag: value.drag,
            spin: Vec4::from_array(value.spin.parts()),
            tumble_minimum: Vec3::from_array(value.tumble_minimum),
            tumble_maximum: Vec3::from_array(value.tumble_maximum),
            wind: Vec3::from_array(value.wind),
            wind_time: value.wind_time,
            follow: [
                Vec2::from_array([value.follow_speed_1, value.follow_scale_1]),
                Vec2::from_array([value.follow_speed_2, value.follow_scale_2]),
            ],
        },
        motion: ParticleMotion {
            speed: builder.add_value(initial(&value.emission_speed).unwrap_or(0.)),
            variation: builder.add_value(initial(&value.speed_variation).unwrap_or(0.)),
            gravity: builder.add_value(Vec3::from_array(gravity_vector(
                initial(&value.gravity).unwrap_or(0),
                value.flags.packed_gravity(),
            )?)),
        },
        lifetime: ParticleLifetime {
            duration: builder.add_value(initial(&value.lifespan).unwrap_or(0.)),
            variation: value.lifespan_variation.get(),
        },
        emission: ParticleEmission {
            rate: builder.add_value(initial(&value.emission_rate).unwrap_or(0.)),
            variation: value.emission_rate_variation.get(),
            activation: builder.add_value(initial(&value.enabled).unwrap_or(1) != 0),
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
        let raw = (time * 32767.).round() as u16;
        context.approximate(
            "m2.particle.age",
            (f64::from(raw) / 32767. - time).abs() as f32,
            "age quantized to native fixed15 timestamps",
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
    payload: &profile::ParticleEmitter<'_>,
    textures: &[semantic::Texture],
    context: &mut Context,
    model: &semantic::Model<'_, semantic::profiles::M2>,
) -> Result<native::records::ParticleEmitter<'static, V>, Error>
where
    V::Header: HeaderFields,
    V::Variation: OptionalFloat,
    V::Spin: SpinFields,
    V::TextureScale: UvFields,
    V::Scroll: ScrollFields,
    V::Lifetime<'static>: LogicalAge,
{
    if payload.details.rendering.texture_program.is_some() {
        return Err(Error::new(
            "m2.particle.texture_program",
            "a standalone particle record cannot encode container-level TXAC selectors; export the complete model graph",
        ));
    }
    let mut value = native::records::ParticleEmitter::<V>::new(ModelFlags::default());
    value.id = payload.details.id;
    value.flags = ParticleFlags::from_bits_retain(payload.details.flags.bits());
    let ids = payload
        .details
        .rendering
        .textures
        .iter()
        .map(|id| {
            texture_index(model, textures, *id)
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
    let filename =
        |resource: &Option<semantic::ModelResourceId>| -> Result<ZeroVec<'static, u8>, Error> {
            match resource {
                None => Ok(ZeroVec::new()),
                Some(id) => {
                    let semantic::PathResource::Path(path) = model
                        .model_resource(*id)
                        .ok_or_else(|| Error::new("m2.particle.model", "missing model resource"))?;
                    Ok(ZeroVec::alloc_from_slice(
                        &path.iter().copied().chain([0]).collect::<Vec<_>>(),
                    ))
                }
            }
        };
    value.particle_model_filename = filename(&payload.details.rendering.model)?;
    value.child_model_filename = filename(&payload.details.rendering.child_model)?;
    value.kind = HeaderFields::make(
        ParticleBlend::from_raw(payload.details.rendering.blend.raw()),
        ParticleGenerator::from_raw(payload.details.shape.generator.raw()),
        u32::from(payload.details.rendering.color_index),
    )?;
    value.texture_scale = UvFields::make(
        (payload.details.rendering.texture_scale).to_array(),
        context,
    )?;
    if value.texture_scale.legacy_parts().is_some() {
        let head_tail = payload
            .details
            .rendering
            .head_tail
            .legacy_code()
            .ok_or_else(|| {
                Error::new(
                    "m2.particle.head_tail",
                    "legacy emitter geometry cannot represent disabled heads and tails",
                )
            })?;
        value
            .texture_scale
            .set_legacy_parts([payload.details.shape.legacy_shape, head_tail], context)?;
    } else {
        let (head, tail) = match payload.details.rendering.head_tail {
            GeometrySelection::Head => (true, false),
            GeometrySelection::Tail => (false, true),
            GeometrySelection::Both => (true, true),
            GeometrySelection::Disabled => (false, false),
            GeometrySelection::Unknown(_) => {
                return Err(Error::new(
                    "m2.particle.head_tail",
                    "modern emitter style bits cannot represent an unknown legacy geometry code",
                ));
            }
        };
        value.flags.set_head(head);
        value.flags.set_tail(tail);
        value
            .texture_scale
            .set_legacy_parts([payload.details.shape.legacy_shape, 0], context)?;
    }
    value.texture_tile_rotation = payload.details.rendering.texture_tile_rotation;
    value.rows = payload.details.rendering.atlas[0];
    value.columns = payload.details.rendering.atlas[1];
    value.lifespan_variation = OptionalFloat::make(payload.lifetime.variation, context)?;
    value.emission_rate_variation = OptionalFloat::make(payload.emission.variation, context)?;
    value.lifetime = LogicalAge::make(&payload.appearance, context)?;
    value.tail_length = payload.details.rendering.tail_length;
    value.twinkle_speed = payload.details.twinkle_speed;
    value.twinkle_percent = payload.details.twinkle_percent;
    value.twinkle_scale = (payload.details.twinkle_scale).to_array();
    value.inherit_velocity_scale = payload.details.inherit_velocity;
    value.drag = payload.details.drag;
    value.spin = SpinFields::make((payload.details.spin).to_array(), context)?;
    value.tumble_minimum = (payload.details.tumble_minimum).to_array();
    value.tumble_maximum = (payload.details.tumble_maximum).to_array();
    value.wind = (payload.details.wind).to_array();
    value.wind_time = payload.details.wind_time;
    value.follow_speed_1 = payload.details.follow[0][0];
    value.follow_scale_1 = payload.details.follow[0][1];
    value.follow_speed_2 = payload.details.follow[1][0];
    value.follow_scale_2 = payload.details.follow[1][1];
    value.spline_points = ZeroVec::alloc_from_slice(
        &payload
            .details
            .shape
            .spline
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
    );
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

#[cfg(test)]
#[path = "logical_tests.rs"]
mod tests;
