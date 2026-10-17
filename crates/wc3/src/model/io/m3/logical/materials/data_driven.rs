//! Direct shader parameters and resource links without an encoded intermediate.
use super::{
    identity, native, nr, read_error, revision, string, surface, Envelope, Error, Reader, Slot,
};
use zerovec::ZeroVec;

fn parameters<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: native::Ref<nr::targets::Character>,
) -> Result<surface::DataDrivenParameters<'a>, Error> {
    let Some(section) = source.referenced_section(reference).map_err(read_error)? else {
        return Ok(surface::DataDrivenParameters::Unresolved(ZeroVec::new()));
    };
    if section.tag() != *b"CHAR" || section.revision() != 0 {
        return Err(Error::new(
            "m3.material.parameters",
            "parameter language is not CHAR storage",
        ));
    }
    let bytes = section
        .bytes()
        .get(..reference.entries as usize)
        .ok_or_else(|| {
            Error::new(
                "m3.material.parameters",
                "parameter span exceeds source values",
            )
        })?;
    Ok(surface::DataDrivenParameters::Unresolved(
        ZeroVec::new_borrowed(bytes),
    ))
}
pub(super) fn datadriven<'a, E: Envelope, D: nr::DataDrivenMaterialDialect>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    r: nr::DataDrivenMaterial<D>,
) -> Result<surface::DataDrivenMaterialStep<'a>, Error>
where
    D::ExtraHashes: Slot<native::Ref<nr::targets::U32>>,
    D::EffectNameHash2: Slot<u32>,
    D::EffectNameHash3: Slot<u32>,
{
    reader.name = Some(string(source, r.material_name)?);
    Ok(surface::DataDrivenMaterialStep {
        fragment_hashes: identity(
            source
                .resolve::<nr::U32>(r.fragment_hashes.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
        extra_hashes: identity(
            source
                .resolve::<nr::U32>(r.extra_hashes.value().cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
        parameters: parameters(source, r.property_blob)?,
        textures: identity(
            source
                .resolve::<nr::StringReference>(r.texture_paths.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| {
                    let r = r.map_err(read_error)?;
                    reader
                        .texture(
                            source
                                .string(r.value)
                                .map_err(read_error)?
                                .unwrap_or_default(),
                        )?
                        .ok_or_else(|| {
                            Error::new("m3.material.texture", "empty linked texture resource")
                        })
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
        alpha_fresnel_flags: surface::values::ShaderFresnelFlags((r.alpha_fresnel_flags).bits()),
        shader_type: surface::values::ShaderType::from_raw((r.shader_type).raw()),
        unknown151: r.unknown151,
        effect_name_hash2: r.effect_name_hash2.value(),
        effect_name_hash3: r.effect_name_hash3.value(),
    })
}

pub(super) fn get_datadriven<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::DataDrivenMaterial>,
    index: usize,
) -> Result<surface::DataDrivenMaterialStep<'a>, Error> {
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::DataDrivenMaterial<$v>>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "missing data-driven material"))?
                .map_err(read_error)?;
            datadriven(source, reader, r)
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
