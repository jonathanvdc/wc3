//! Data-driven shader parameters remain an explicit language value.
use super::{append, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::DataDrivenMaterialStep<'_>,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::DataDrivenMaterial<nr::V3>, Error> {
    let mut paths = Vec::with_capacity(r.textures.len());
    for id in &r.textures {
        let path = writer.texture(*id)?;
        paths.push(nr::StringReference {
            value: writer.string(path)?,
        });
    }
    let parameter_bytes: Vec<nr::Character> = match &r.parameters {
        surface::DataDrivenParameters::Unresolved(values) => {
            values.iter().map(|value| nr::Character { value }).collect()
        }
        surface::DataDrivenParameters::Named(values) if values.groups.is_empty() => Vec::new(),
        surface::DataDrivenParameters::Named(_) => {
            return Err(Error::new(
                "m3.material.parameters",
                "named shader dictionary encoding is not established for native M3",
            ));
        }
    };
    Ok(nr::DataDrivenMaterial {
        material_name: writer.string(name)?,
        fragment_hashes: append(
            writer.document,
            &r.fragment_hashes
                .iter()
                .map(|value| nr::U32 { value: *value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        extra_hashes: append(
            writer.document,
            &r.extra_hashes
                .iter()
                .map(|value| nr::U32 { value: *value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        property_blob: append(writer.document, &parameter_bytes)?.cast(),
        texture_paths: append(writer.document, &paths)?.cast(),
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
