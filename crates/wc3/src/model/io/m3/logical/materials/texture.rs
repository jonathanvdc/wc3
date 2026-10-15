//! Direct logical texture decoding from native M3 fields.
use super::{
    anim, color, native, nr, read_error, revision, surface, Envelope, Error, Reader, Slot,
};
use crate::model::materials::bindings::{Coordinates, TextureBinding};
pub(super) fn layer<'a, E: Envelope, D: nr::TextureLayerDialect>(
    r: nr::TextureLayer<D>,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::TextureLayer, Error>
where
    D::NoiseAmplitude: Slot<f32>,
    D::NoiseFrequency: Slot<f32>,
    D::TriplanarOffset: Slot<nr::AnimationValue<[f32; 3]>>,
    D::TriplanarScale: Slot<nr::AnimationValue<[f32; 3]>>,
    D::FresnelTranslation: Slot<[f32; 3]>,
    D::FresnelMask: Slot<[f32; 3]>,
    D::FresnelRotation: Slot<[f32; 2]>,
    D::UvDensity: Slot<u32>,
{
    Ok(surface::TextureLayer {
        id: r.id,
        binding: TextureBinding {
            image: reader.texture(
                source
                    .string(r.texture_path)
                    .map_err(read_error)?
                    .unwrap_or_default(),
            )?,
            coordinates: Coordinates {
                source: surface::values::UvMappingMode::from_raw((r.uv_mapping).raw()),
                transform: surface::CoordinateTransform {
                    uv_offset: anim(r.uv_offset),
                    uv_angle: anim(r.uv_angle),
                    uv_tiling: anim(r.uv_tiling),
                    w_offset: anim(r.w_offset),
                    w_tiling: anim(r.w_tiling),
                    triplanar_offset: anim(r.triplanar_offset.value()),
                    triplanar_scale: anim(r.triplanar_scale.value()),
                    uv_source_related: r.uv_source_related,
                    uv_density: r.uv_density.value(),
                },
            },
            sampling: surface::SamplingProgram {
                flags: surface::values::LayerFlags((r.flags).bits()),
                poc_texture: r.poc_texture,
                noise_amplitude: r.noise_amplitude.value(),
                noise_frequency: r.noise_frequency.value(),
                texture_source: r.texture_source,
                avi_frame_rate: r.avi_frame_rate,
                avi_start: r.avi_start,
                avi_stop: r.avi_stop,
                avi_loop: r.avi_loop,
                avi_sync: r.avi_sync,
                avi_play: anim(r.avi_play),
                avi_restart: anim(r.avi_restart),
                flipbook_rows: r.flipbook_rows,
                flipbook_columns: r.flipbook_columns,
                current_frame: anim(r.current_frame),
            },
            contribution: surface::ColorContribution {
                color: color(r.color),
                color_type: surface::values::ColorChannel::from_raw((r.color_type).raw()),
                rgb_multiply: anim(r.rgb_multiply),
                rgb_add: anim(r.rgb_add),
                map_alpha: anim(r.map_alpha),
            },
        },
        fresnel_mode: surface::values::FresnelMode::from_raw((r.fresnel_mode).raw()),
        fresnel_exponent: r.fresnel_exponent,
        fresnel_min: r.fresnel_min,
        fresnel_max: r.fresnel_max,
        fresnel_translation: r.fresnel_translation.value(),
        fresnel_mask: r.fresnel_mask.value(),
        fresnel_rotation: r.fresnel_rotation.value(),
    })
}

pub(super) fn get_layer<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::TextureLayer>,
) -> Result<Option<surface::TextureLayer>, Error> {
    if reference.entries == 0 {
        return Ok(None);
    }
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::TextureLayer<$v>>(reference.cast())
                .map_err(read_error)?
                .get(0)
                .ok_or_else(|| Error::new("m3.material.layer", "empty layer"))?
                .map_err(read_error)?;
            layer(r, source, reader)
        }};
    }
    let value = match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        8 => read!(nr::V8),
        9 => read!(nr::V9),
        10 => read!(nr::V10),
        11 => read!(nr::V11),
        12 => read!(nr::V12),
        13 => read!(nr::V13),
        14 => read!(nr::V14),
        15 => read!(nr::V15),
        16 => read!(nr::V16),
        17 => read!(nr::V17),
        18 => read!(nr::V18),
        19 => read!(nr::V19),
        20 => read!(nr::V20),
        21 => read!(nr::V21),
        22 => read!(nr::V22),
        23 => read!(nr::V23),
        24 => read!(nr::V24),
        25 => read!(nr::V25),
        26 => read!(nr::V26),
        _ => {
            return Err(Error::new(
                "m3.material.layer",
                "unsupported layer revision",
            ));
        }
    }?;
    Ok(Some(value))
}
