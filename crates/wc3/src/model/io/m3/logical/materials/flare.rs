//! Direct logical flare decoding from native M3 fields.
use super::texture::get_layer;
use super::{
    anim, fixed_color, identity, native, nr, read_error, revision, string, surface, Envelope,
    Error, Reader, Slot,
};
pub(super) fn subflares<E: Envelope>(
    source: &native::Container<'_, E>,
    reference: native::Ref<nr::targets::SubFlare>,
) -> Result<Vec<surface::SubFlare>, Error> {
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    let mut rows = Vec::new();
    macro_rules! read {
        ($v:ty) => {
            for r in source
                .resolve::<nr::SubFlare<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = r.map_err(read_error)?;
                rows.push(surface::SubFlare {
                    index: r.index,
                    position: r.position,
                    size_xy: r.size_x_y,
                    scale_xy: r.scale_x_y,
                    fade_in: r.fade_in,
                    fade_out: r.fade_out,
                    color_alpha: [
                        r.color_alpha[2],
                        r.color_alpha[1],
                        r.color_alpha[0],
                        r.color_alpha[3],
                    ]
                    .map(|v| f32::from(v) / 255.),
                    face_center: r.face_center,
                    offset: r.offset,
                });
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        _ => return Err(Error::new("m3.material.sub_flares", "unsupported revision")),
    }
    Ok(identity(rows))
}

pub(super) fn lensflare<'a, E: Envelope, D: nr::LensFlareDialect>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    r: nr::LensFlare<D>,
) -> Result<surface::LensFlareMaterialStep<'a>, Error>
where
    D::LibName: Slot<native::Ref<nr::targets::Character>>,
    D::Color: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::Hdr: Slot<nr::AnimationValue<f32>>,
    D::Size: Slot<nr::AnimationValue<f32>>,
{
    reader.name = Some(string(source, r.name)?);
    Ok(surface::LensFlareMaterialStep {
        flare_map: get_layer(source, reader, r.flare_map)?,
        mask_map: get_layer(source, reader, r.mask_map)?,
        sub_flares: subflares(source, r.sub_flares)?,
        columns: r.columns,
        rows: r.rows,
        distance_fade: r.distance_fade,
        lib_name: string(source, r.lib_name.value())?,
        intensity: anim(r.intensity, reader.builder.values_mut()),
        color: fixed_color(r.color.value(), reader.builder.values_mut()),
        hdr: anim(r.hdr.value(), reader.builder.values_mut()),
        size: anim(r.size.value(), reader.builder.values_mut()),
    })
}

pub(super) fn get_lensflare<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::LensFlare>,
    index: usize,
) -> Result<surface::LensFlareMaterialStep<'a>, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::LensFlare<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "missing flare"))?
                .map_err(read_error)?;
            lensflare(source, reader, r)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        _ => Err(Error::new("m3.material", "unsupported flare revision")),
    }
}
