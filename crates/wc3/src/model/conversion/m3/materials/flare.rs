//! Flare mappings for native M3 materials.
use super::{
    anim, append, append_string, color, get_layer, native, native_anim, native_color,
    native_layer_ref, nr, read_error, revision, string, text, wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn subflares<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: native::Ref<nr::targets::SubFlare>,
) -> Result<wem::InlineList<wr::M3SubFlare>, Error> {
    if reference.entries == 0 {
        return Ok(wem::InlineList::default());
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
                rows.push(wr::M3SubFlare {
                    index: r.index,
                    position: r.position,
                    size_xy: r.size_x_y,
                    scale_xy: r.scale_x_y,
                    fade_in: r.fade_in,
                    fade_out: r.fade_out,
                    color_alpha: wr::M3ColorBGRA {
                        b: r.color_alpha[0],
                        g: r.color_alpha[1],
                        r: r.color_alpha[2],
                        a: r.color_alpha[3],
                    },
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
    Ok(wem::InlineList(rows))
}

pub(in crate::model::conversion::m3) fn fixed_color(
    r: nr::AnimationValue<nr::FixedBytes<4>>,
) -> wr::M3AnimRef<wr::M3ColorBGRA> {
    color(nr::AnimationValue {
        interpolation: r.interpolation,
        flags: r.flags,
        animation_id: r.animation_id,
        initial_value: r.initial_value.0,
        null_value: r.null_value.0,
        reserved: r.reserved,
    })
}

pub(in crate::model::conversion::m3) fn lensflare<E: Envelope, D: nr::LensFlareDialect>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    r: nr::LensFlare<D>,
) -> Result<wr::M3LensFlare, Error>
where
    D::LibName: Slot<native::Ref<nr::targets::Character>>,
    D::Color: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::Hdr: Slot<nr::AnimationValue<f32>>,
    D::Size: Slot<nr::AnimationValue<f32>>,
{
    Ok(wr::M3LensFlare {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        flare_map: get_layer(source, doc, r.flare_map)?,
        mask_map: get_layer(source, doc, r.mask_map)?,
        sub_flares: subflares(source, r.sub_flares)?,
        columns: r.columns,
        rows: r.rows,
        distance_fade: r.distance_fade,
        lib_name: text(
            doc,
            &string(source, r.lib_name.value()).map_err(read_error)?,
        )?,
        intensity: anim(r.intensity),
        color: fixed_color(r.color.value()),
        hdr: anim(r.hdr.value()),
        size: anim(r.size.value()),
    })
}

pub(in crate::model::conversion::m3) fn get_lensflare<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::LensFlare>,
    index: usize,
) -> Result<wr::M3LensFlare, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::LensFlare<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "missing flare"))?
                .map_err(read_error)?;
            lensflare(source, doc, r)
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

pub(in crate::model::conversion::m3) fn export_lensflare<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    r: wr::M3LensFlare,
) -> Result<nr::LensFlare<nr::V3>, Error> {
    let sub = r
        .sub_flares
        .0
        .into_iter()
        .map(|r| nr::SubFlare::<nr::V2> {
            index: r.index,
            position: r.position,
            size_x_y: r.size_xy,
            scale_x_y: r.scale_xy,
            fade_in: r.fade_in,
            fade_out: r.fade_out,
            color_alpha: [
                r.color_alpha.b,
                r.color_alpha.g,
                r.color_alpha.r,
                r.color_alpha.a,
            ],
            face_center: r.face_center,
            offset: r.offset,
        })
        .collect::<Vec<_>>();
    let color = native_color(r.color);
    Ok(nr::LensFlare {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        flare_map: native_layer_ref(r.flare_map, source, doc)?,
        mask_map: native_layer_ref(r.mask_map, source, doc)?,
        sub_flares: append(doc, &sub)?.cast(),
        columns: r.columns,
        rows: r.rows,
        distance_fade: r.distance_fade,
        lib_name: append_string(doc, source.string(r.lib_name)?.unwrap_or_default())?,
        intensity: native_anim(r.intensity),
        color: nr::AnimationValue {
            interpolation: color.interpolation,
            flags: color.flags,
            animation_id: color.animation_id,
            initial_value: nr::FixedBytes(color.initial_value),
            null_value: nr::FixedBytes(color.null_value),
            reserved: color.reserved,
        },
        hdr: native_anim(r.hdr),
        size: native_anim(r.size),
    })
}
