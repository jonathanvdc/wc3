//! Data driven mappings for native M3 materials.
use super::{
    append, append_string, native, nr, read_error, revision, string, text, wem, wr, Envelope,
    Error, Slot,
};
pub(in crate::model::conversion::m3) fn datadriven<E: Envelope, D: nr::DataDrivenMaterialDialect>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    r: nr::DataDrivenMaterial<D>,
) -> Result<wr::M3DataDriven, Error>
where
    D::ExtraHashes: Slot<native::Ref<nr::targets::U32>>,
    D::EffectNameHash2: Slot<u32>,
    D::EffectNameHash3: Slot<u32>,
{
    Ok(wr::M3DataDriven {
        material_name: text(doc, &string(source, r.material_name).map_err(read_error)?)?,
        fragment_hashes: wem::InlineList(
            source
                .resolve::<nr::U32>(r.fragment_hashes.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
        extra_hashes: wem::InlineList(
            source
                .resolve::<nr::U32>(r.extra_hashes.value().cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
        property_blob: wem::InlineList(
            source
                .resolve::<nr::Character>(r.property_blob.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
        texture_paths: wem::InlineList(
            source
                .resolve::<nr::StringReference>(r.texture_paths.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| {
                    let r = r.map_err(read_error)?;
                    text(doc, &string(source, r.value).map_err(read_error)?)
                })
                .collect::<Result<Vec<_>, Error>>()?,
        ),
        unknown108: r.unknown108,
        unknown112: r.unknown112,
        unknown116: r.unknown116,
        effect_name_hash: r.effect_name_hash,
        unknown124: r.unknown124,
        padding128: r.padding128,
        unknown132: r.unknown132,
        unknown136: r.unknown136,
        unknown140: r.unknown140,
        unknown144: r.unknown144,
        unknown148: r.unknown148,
        alpha_fresnel_flags: wr::M3AlphaFresnelFlags((r.alpha_fresnel_flags).bits()),
        shader_type: wr::M3MaterialShaderType::from_raw((r.shader_type).raw()),
        unknown151: r.unknown151,
        effect_name_hash2: r.effect_name_hash2.value(),
        effect_name_hash3: r.effect_name_hash3.value(),
    })
}

pub(in crate::model::conversion::m3) fn get_datadriven<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::DataDrivenMaterial>,
    index: usize,
) -> Result<wr::M3DataDriven, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::DataDrivenMaterial<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "missing data-driven material"))?
                .map_err(read_error)?;
            datadriven(source, doc, r)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        _ => Err(Error::new(
            "m3.material",
            "unsupported data-driven revision",
        )),
    }
}

pub(in crate::model::conversion::m3) fn export_datadriven<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    r: wr::M3DataDriven,
) -> Result<nr::DataDrivenMaterial<nr::V3>, Error> {
    let paths = r
        .texture_paths
        .0
        .into_iter()
        .map(|r| {
            Ok(nr::StringReference {
                value: append_string(doc, source.string(r)?.unwrap_or_default())?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(nr::DataDrivenMaterial {
        material_name: append_string(doc, source.string(r.material_name)?.unwrap_or_default())?,
        fragment_hashes: append(
            doc,
            &r.fragment_hashes
                .0
                .into_iter()
                .map(|value| nr::U32 { value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        extra_hashes: append(
            doc,
            &r.extra_hashes
                .0
                .into_iter()
                .map(|value| nr::U32 { value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        property_blob: append(
            doc,
            &r.property_blob
                .0
                .into_iter()
                .map(|value| nr::Character { value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        texture_paths: append(doc, &paths)?.cast(),
        unknown108: r.unknown108,
        unknown112: r.unknown112,
        unknown116: r.unknown116,
        effect_name_hash: r.effect_name_hash,
        unknown124: r.unknown124,
        padding128: r.padding128,
        unknown132: r.unknown132,
        unknown136: r.unknown136,
        unknown140: r.unknown140,
        unknown144: r.unknown144,
        unknown148: r.unknown148,
        alpha_fresnel_flags: nr::ShaderFresnelFlags((r.alpha_fresnel_flags).bits()),
        shader_type: nr::ShaderType::from_raw(r.shader_type.raw()),
        unknown151: r.unknown151,
        effect_name_hash2: r.effect_name_hash2,
        effect_name_hash3: r.effect_name_hash3,
        ..Default::default()
    })
}
