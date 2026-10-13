//! Particle appearance conversion components.
use super::*;

pub(super) trait AgeFields: Sized {
    fn import(
        &self,
        doc: &mut wem::RawDocument<'static>,
        value: &mut wr::M2ParticleEmitterPayload<wr::V19>,
    ) -> Result<(), Error>;
    fn export(
        doc: &wem::RawDocument<'_>,
        value: &wr::M2ParticleEmitterPayload<wr::V19>,
        context: &mut Context,
    ) -> Result<Self, Error>;
}

pub(super) fn real_rows(
    doc: &mut wem::RawDocument<'static>,
    values: impl IntoIterator<Item = f32>,
) -> Result<wem::Ref<wr::targets::Real>, Error> {
    put(
        doc,
        &values
            .into_iter()
            .map(|value| wr::Real { value })
            .collect::<Vec<_>>(),
    )
    .map(|r| r.cast())
}

pub(super) fn age_times<T: Value>(
    doc: &mut wem::RawDocument<'static>,
    track: &native::records::ParticleTrack<'_, T>,
) -> Result<wem::Ref<wr::targets::Real>, Error> {
    real_rows(doc, track.timestamps.iter().map(|v| f32::from(v) / 65535.))
}

pub(super) fn age_keys<T: Value>(
    doc: &wem::RawDocument<'_>,
    times: wem::Ref<wr::targets::Real>,
    values: Vec<T>,
    context: &mut Context,
) -> Result<native::records::ParticleTrack<'static, T>, Error> {
    let mut native_times = Vec::new();
    for time in doc.resolve::<wr::Real>(times.cast())?.iter() {
        let time = time?.value;
        if !time.is_finite() || !(0.0..=1.0).contains(&time) {
            return Err(Error::new(
                "particles.age",
                "particle age must be normalized",
            ));
        }
        let quantized = (time * 65535.).round() as u16;
        context.approximate(
            "particles.age",
            (time - f32::from(quantized) / 65535.).abs(),
            "particle-age clock quantization",
        )?;
        native_times.push(quantized);
    }
    if native_times.len() != values.len() {
        return Err(Error::new("particles.age", "age/key counts differ"));
    }
    Ok(native::records::ParticleTrack {
        timestamps: ZeroVec::alloc_from_slice(&native_times),
        values: ZeroVec::alloc_from_slice(&values),
    })
}

impl AgeFields for native::records::ModernParticleLifetime<'_> {
    fn import(
        &self,
        doc: &mut wem::RawDocument<'static>,
        value: &mut wr::M2ParticleEmitterPayload<wr::V19>,
    ) -> Result<(), Error> {
        value.color_times = age_times(doc, &self.color)?;
        value.colors = put(
            doc,
            &self
                .color
                .values
                .iter()
                .map(|value| wr::Vector3 { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        value.alpha_times = age_times(doc, &self.alpha)?;
        value.alphas = real_rows(doc, self.alpha.values.iter().map(|v| f32::from(v) / 32767.))?;
        value.scale_times = age_times(doc, &self.scale)?;
        value.scales = put(
            doc,
            &self
                .scale
                .values
                .iter()
                .map(|value| wr::Vector2 { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        value.scale_variation = self.scale_variation;
        value.head_cell_times = age_times(doc, &self.head_uv)?;
        value.head_cells = put(
            doc,
            &self
                .head_uv
                .values
                .iter()
                .map(|value| wr::U32 {
                    value: u32::from(value),
                })
                .collect::<Vec<_>>(),
        )?
        .cast();
        value.tail_cell_times = age_times(doc, &self.tail_uv)?;
        value.tail_cells = put(
            doc,
            &self
                .tail_uv
                .values
                .iter()
                .map(|value| wr::U32 {
                    value: u32::from(value),
                })
                .collect::<Vec<_>>(),
        )?
        .cast();
        Ok(())
    }
    fn export(
        doc: &wem::RawDocument<'_>,
        value: &wr::M2ParticleEmitterPayload<wr::V19>,
        context: &mut Context,
    ) -> Result<Self, Error> {
        let colors = doc
            .resolve::<wr::Vector3>(value.colors.cast())?
            .iter()
            .map(|v| v.map(|v| v.value))
            .collect::<Result<Vec<_>, _>>()?;
        let scales = doc
            .resolve::<wr::Vector2>(value.scales.cast())?
            .iter()
            .map(|v| v.map(|v| v.value))
            .collect::<Result<Vec<_>, _>>()?;
        let mut alpha = Vec::new();
        for row in doc.resolve::<wr::Real>(value.alphas.cast())?.iter() {
            let value = row?.value;
            if !value.is_finite() || !(0.0..=65535.0 / 32767.0).contains(&value) {
                return Err(Error::new(
                    "particles.alpha",
                    "alpha exceeds the native fixed16 range",
                ));
            }
            let quantized = (value * 32767.).round() as u16;
            context.approximate(
                "particles.alpha",
                (value - f32::from(quantized) / 32767.).abs(),
                "particle alpha quantization",
            )?;
            alpha.push(quantized);
        }
        let head = doc
            .resolve::<wr::U32>(value.head_cells.cast())?
            .iter()
            .map(|v| {
                u16::try_from(v?.value)
                    .map_err(|_| Error::new("particles.head_cell", "cell exceeds u16"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let tail = doc
            .resolve::<wr::U32>(value.tail_cells.cast())?
            .iter()
            .map(|v| {
                u16::try_from(v?.value)
                    .map_err(|_| Error::new("particles.tail_cell", "cell exceeds u16"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            color: age_keys(doc, value.color_times, colors, context)?,
            alpha: age_keys(doc, value.alpha_times, alpha, context)?,
            scale: age_keys(doc, value.scale_times, scales, context)?,
            scale_variation: value.scale_variation,
            head_uv: age_keys(doc, value.head_cell_times, head, context)?,
            tail_uv: age_keys(doc, value.tail_cell_times, tail, context)?,
        })
    }
}

impl AgeFields for native::records::LegacyParticleLifetime {
    fn import(
        &self,
        doc: &mut wem::RawDocument<'static>,
        value: &mut wr::M2ParticleEmitterPayload<wr::V19>,
    ) -> Result<(), Error> {
        let times = [0., self.midpoint, 1.];
        value.color_times = real_rows(doc, times)?;
        value.alpha_times = value.color_times;
        value.scale_times = value.color_times;
        value.colors = put(
            doc,
            &self.colors_bgra.map(|v| wr::Vector3 {
                value: [
                    f32::from(v[2]) / 255.,
                    f32::from(v[1]) / 255.,
                    f32::from(v[0]) / 255.,
                ],
            }),
        )?
        .cast();
        value.alphas = real_rows(doc, self.colors_bgra.map(|v| f32::from(v[3]) / 255.))?;
        value.scales = put(doc, &self.scales.map(|v| wr::Vector2 { value: [v; 2] }))?.cast();
        value.head_cell_times = real_rows(doc, [0., 1.])?;
        value.tail_cell_times = value.head_cell_times;
        value.head_cells = put(
            doc,
            &[
                wr::U32 {
                    value: u32::from(self.head_lifespan_uv[0])
                        | u32::from(self.head_lifespan_uv[1]) << 16,
                },
                wr::U32 {
                    value: u32::from(self.head_decay_uv[0])
                        | u32::from(self.head_decay_uv[1]) << 16,
                },
            ],
        )?
        .cast();
        value.tail_cells = put(
            doc,
            &[
                wr::U32 {
                    value: u32::from(self.tail_lifespan_uv[0] as u16)
                        | u32::from(self.tail_lifespan_uv[1] as u16) << 16,
                },
                wr::U32 {
                    value: u32::from(self.tail_decay_uv[0] as u16)
                        | u32::from(self.tail_decay_uv[1] as u16) << 16,
                },
            ],
        )?
        .cast();
        Ok(())
    }
    fn export(
        doc: &wem::RawDocument<'_>,
        value: &wr::M2ParticleEmitterPayload<wr::V19>,
        context: &mut Context,
    ) -> Result<Self, Error> {
        let times = doc.resolve::<wr::Real>(value.color_times.cast())?;
        let colors = doc.resolve::<wr::Vector3>(value.colors.cast())?;
        let alpha = doc.resolve::<wr::Real>(value.alphas.cast())?;
        let scales = doc.resolve::<wr::Vector2>(value.scales.cast())?;
        if times.len() != 3 || colors.len() != 3 || alpha.len() != 3 || scales.len() != 3 {
            return Err(Error::new(
                "particles.age",
                "legacy lifetime needs three synchronized appearance keys",
            ));
        }
        let mut colors_bgra = [[0; 4]; 3];
        let mut native_scales = [0.; 3];
        for i in 0..3 {
            let color = colors.get(i).unwrap()?.value;
            let alpha = alpha.get(i).unwrap()?.value;
            for (component, value) in [color[2], color[1], color[0], alpha]
                .into_iter()
                .enumerate()
            {
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(Error::new(
                        "particles.color",
                        "legacy color component must be normalized",
                    ));
                }
                let byte = (value * 255.).round() as u8;
                context.approximate(
                    "particles.color",
                    (value - f32::from(byte) / 255.).abs(),
                    "legacy particle color quantization",
                )?;
                colors_bgra[i][component] = byte;
            }
            let scale = scales.get(i).unwrap()?.value;
            if scale[0] != scale[1] {
                return Err(Error::new(
                    "particles.scale",
                    "legacy particle scale must be uniform",
                ));
            }
            native_scales[i] = scale[0];
        }
        let head = doc.resolve::<wr::U32>(value.head_cells.cast())?;
        let tail = doc.resolve::<wr::U32>(value.tail_cells.cast())?;
        let head0 = head.get(0).transpose()?.map_or(0, |v| v.value);
        let head1 = head.get(1).transpose()?.map_or(0, |v| v.value);
        let tail0 = tail.get(0).transpose()?.map_or(0, |v| v.value);
        let tail1 = tail.get(1).transpose()?.map_or(0, |v| v.value);
        Ok(Self {
            midpoint: times.get(1).unwrap()?.value,
            colors_bgra,
            scales: native_scales,
            head_lifespan_uv: [head0 as u16, (head0 >> 16) as u16, 0],
            head_decay_uv: [head1 as u16, (head1 >> 16) as u16, 0],
            tail_lifespan_uv: [tail0 as i16, (tail0 >> 16) as i16],
            tail_decay_uv: [tail1 as i16, (tail1 >> 16) as i16],
        })
    }
}
