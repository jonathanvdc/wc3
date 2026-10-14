//! Particle fields conversion components.
use super::*;

pub(super) trait OptionalFloat {
    fn get(&self) -> f32;
    fn make(value: f32, context: &mut Context) -> Result<Self, Error>
    where
        Self: Sized;
}

impl OptionalFloat for f32 {
    fn get(&self) -> f32 {
        *self
    }
    fn make(value: f32, _: &mut Context) -> Result<Self, Error> {
        Ok(value)
    }
}

impl OptionalFloat for () {
    fn get(&self) -> f32 {
        0.
    }
    fn make(value: f32, context: &mut Context) -> Result<Self, Error> {
        if value != 0. {
            context.unsupported(
                "particles.variation",
                "target layout has no random variation field",
            )?;
        }
        Ok(())
    }
}

pub(super) trait HeaderFields {
    fn parts(&self) -> (ParticleBlend, ParticleGenerator, u32);
    fn make(blend: ParticleBlend, kind: ParticleGenerator, color: u32) -> Result<Self, Error>
    where
        Self: Sized;
}

impl HeaderFields for native::records::WideParticleHeader {
    fn parts(&self) -> (ParticleBlend, ParticleGenerator, u32) {
        (self.blending, self.emitter, 0)
    }
    fn make(blend: ParticleBlend, kind: ParticleGenerator, color: u32) -> Result<Self, Error> {
        if color != 0 {
            return Err(Error::new(
                "particles.color_index",
                "legacy emitter has no color-table selector",
            ));
        }
        Ok(Self {
            blending: blend,
            emitter: kind,
        })
    }
}

impl HeaderFields for native::records::PackedParticleHeader {
    fn parts(&self) -> (ParticleBlend, ParticleGenerator, u32) {
        (
            ParticleBlend::from_raw(u16::from(self.blending.raw())),
            ParticleGenerator::from_raw(u16::from(self.emitter.raw())),
            u32::from(self.color_index),
        )
    }
    fn make(blend: ParticleBlend, kind: ParticleGenerator, color: u32) -> Result<Self, Error> {
        Ok(Self {
            blending: PackedParticleBlend::from_raw(
                u8::try_from(blend.raw())
                    .map_err(|_| Error::new("particles.blend", "blend exceeds u8"))?,
            ),
            emitter: PackedParticleGenerator::from_raw(
                u8::try_from(kind.raw())
                    .map_err(|_| Error::new("particles.kind", "kind exceeds u8"))?,
            ),
            color_index: u16::try_from(color)
                .map_err(|_| Error::new("particles.color_index", "color selector exceeds u16"))?,
        })
    }
}

pub(super) trait SpinFields {
    fn parts(&self) -> [f32; 4];
    fn make(parts: [f32; 4], context: &mut Context) -> Result<Self, Error>
    where
        Self: Sized;
}

impl SpinFields for f32 {
    fn parts(&self) -> [f32; 4] {
        [0., 0., *self, 0.]
    }
    fn make(parts: [f32; 4], context: &mut Context) -> Result<Self, Error> {
        if parts[0] != 0. || parts[1] != 0. || parts[3] != 0. {
            context.unsupported(
                "particles.spin",
                "legacy emitter has no initial or random spin fields",
            )?;
        }
        Ok(parts[2])
    }
}

impl SpinFields for native::records::ModernSpin {
    fn parts(&self) -> [f32; 4] {
        [
            self.base,
            self.base_variation,
            self.speed,
            self.speed_variation,
        ]
    }
    fn make(parts: [f32; 4], _: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            base: parts[0],
            base_variation: parts[1],
            speed: parts[2],
            speed_variation: parts[3],
        })
    }
}

pub(super) trait UvFields {
    fn legacy_parts(&self) -> Option<[u8; 2]> {
        None
    }
    fn set_legacy_parts(&mut self, parts: [u8; 2], context: &mut Context) -> Result<(), Error> {
        if parts != [0; 2] {
            context.unsupported(
                "particles.legacy_shape",
                "modern particle layouts omit legacy shape and head/tail fields",
            )?;
        }
        Ok(())
    }
    fn scales(&self) -> [f32; 2];
    fn make(scales: [f32; 2], context: &mut Context) -> Result<Self, Error>
    where
        Self: Sized;
}

impl UvFields for native::records::LegacyTextureScale {
    fn legacy_parts(&self) -> Option<[u8; 2]> {
        Some([self.particle_type, self.head_or_tail.raw()])
    }
    fn set_legacy_parts(&mut self, parts: [u8; 2], _: &mut Context) -> Result<(), Error> {
        self.particle_type = parts[0];
        self.head_or_tail = ParticleHeadTail::from_raw(parts[1]);
        Ok(())
    }
    fn scales(&self) -> [f32; 2] {
        [1.; 2]
    }
    fn make(scales: [f32; 2], context: &mut Context) -> Result<Self, Error> {
        if scales != [1.; 2] {
            context.unsupported(
                "particles.uv_scale",
                "legacy particle layouts omit layer UV scales",
            )?;
        }
        Ok(Self::default())
    }
}

impl UvFields for native::records::ModernTextureScale {
    fn scales(&self) -> [f32; 2] {
        self.scales.map(|v| f32::from(v) / 32.)
    }
    fn make(scales: [f32; 2], context: &mut Context) -> Result<Self, Error> {
        let mut values = [0; 2];
        for (i, value) in scales.into_iter().enumerate() {
            if !value.is_finite() || !(0.0..=255.0 / 32.0).contains(&value) {
                return Err(Error::new(
                    "particles.uv_scale",
                    "UV scale exceeds fixed8_5 range",
                ));
            }
            values[i] = (value * 32.).round() as u8;
            context.approximate(
                "particles.uv_scale",
                (value - f32::from(values[i]) / 32.).abs(),
                "UV scale quantization",
            )?;
        }
        Ok(Self { scales: values })
    }
}

pub(super) trait ScrollFields {
    fn required_flag(&self) -> ModelFlags {
        ModelFlags::default()
    }

    fn values(&self) -> Option<[i16; 8]>;
    fn make(values: Option<[i16; 8]>, context: &mut Context) -> Result<Self, Error>
    where
        Self: Sized;
}

impl ScrollFields for () {
    fn values(&self) -> Option<[i16; 8]> {
        None
    }
    fn make(values: Option<[i16; 8]>, context: &mut Context) -> Result<Self, Error> {
        if values.is_some_and(|v| v != [0; 8]) {
            context.unsupported(
                "particles.uv_scroll",
                "target emitter has no UV scrolling fields",
            )?;
        }
        Ok(())
    }
}

impl ScrollFields for native::records::FlaggedScroll {
    fn required_flag(&self) -> ModelFlags {
        let mut flags = ModelFlags::default();
        flags.set_particle_multi_texture(self.values.is_some());
        flags
    }

    fn values(&self) -> Option<[i16; 8]> {
        self.values
    }
    fn make(values: Option<[i16; 8]>, _: &mut Context) -> Result<Self, Error> {
        Ok(Self { values })
    }
}

impl ScrollFields for native::records::ExtendedScroll {
    fn values(&self) -> Option<[i16; 8]> {
        Some(self.values)
    }
    fn make(values: Option<[i16; 8]>, _: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            values: values.unwrap_or([0; 8]),
        })
    }
}

pub(super) trait RibbonTailFields: Sized {
    fn parts(&self) -> [i16; 3];
    fn make(parts: [i16; 3], context: &mut Context) -> Result<Self, Error>;
}

impl RibbonTailFields for () {
    fn parts(&self) -> [i16; 3] {
        [0; 3]
    }
    fn make(parts: [i16; 3], context: &mut Context) -> Result<Self, Error> {
        if parts != [0; 3] {
            context.unsupported(
                "ribbons.order",
                "target ribbon lacks priority and lookup fields",
            )?;
        }
        Ok(())
    }
}

impl RibbonTailFields for native::records::RibbonModern {
    fn parts(&self) -> [i16; 3] {
        [
            self.priority_plane,
            i16::from(self.color_index),
            i16::from(self.texture_transform_index),
        ]
    }
    fn make(parts: [i16; 3], _: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            priority_plane: parts[0],
            color_index: i8::try_from(parts[1])
                .map_err(|_| Error::new("ribbons.color", "color index exceeds i8"))?,
            texture_transform_index: i8::try_from(parts[2])
                .map_err(|_| Error::new("ribbons.transform", "transform index exceeds i8"))?,
        })
    }
}

pub(in super::super) fn gravity_vector(word: u32, packed: bool) -> Result<[f32; 3], Error> {
    if !packed {
        return Ok([0., 0., -f32::from_bits(word)]);
    }
    let bytes = word.to_le_bytes();
    let x = f32::from(bytes[0] as i8) / 128.;
    let y = f32::from(bytes[1] as i8) / 128.;
    let xy = x * x + y * y;
    if xy > 1. {
        return Err(Error::new(
            "particles.gravity",
            "compressed gravity direction lies outside the unit hemisphere",
        ));
    }
    let encoded = i16::from_le_bytes([bytes[2], bytes[3]]);
    let magnitude = f32::from(encoded) * 0.04238648;
    let z = (1. - xy).sqrt().copysign(magnitude);
    let magnitude = magnitude.abs();
    Ok([x * magnitude, y * magnitude, z * magnitude])
}
pub(in super::super) fn gravity_word(
    vector: [f32; 3],
    packed: bool,
    context: &mut Context,
    path: &str,
) -> Result<u32, Error> {
    if !packed {
        if vector[0] != 0. || vector[1] != 0. {
            return Err(Error::new(
                path,
                "scalar native gravity cannot represent a lateral direction",
            ));
        }
        return Ok((-vector[2]).to_bits());
    }
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if !magnitude.is_finite() {
        return Err(Error::new(path, "gravity vector must be finite"));
    }
    if magnitude == 0. {
        return Ok(0);
    }
    let x = (vector[0] / magnitude * 128.).round();
    let y = (vector[1] / magnitude * 128.).round();
    let z = (magnitude / 0.04238648).round().copysign(vector[2]);
    if x < f32::from(i8::MIN)
        || x > f32::from(i8::MAX)
        || y < f32::from(i8::MIN)
        || y > f32::from(i8::MAX)
        || z < f32::from(i16::MIN)
        || z > f32::from(i16::MAX)
    {
        return Err(Error::new(
            path,
            "gravity vector exceeds compressed component ranges",
        ));
    }
    let [z0, z1] = (z as i16).to_le_bytes();
    let result = u32::from_le_bytes([x as i8 as u8, y as i8 as u8, z0, z1]);
    let decoded = gravity_vector(result, true)?;
    let error = vector
        .into_iter()
        .zip(decoded)
        .map(|(a, b)| (a - b).abs())
        .fold(0., f32::max);
    context.approximate(
        path,
        error,
        "particle gravity direction and magnitude quantization",
    )?;
    Ok(result)
}
