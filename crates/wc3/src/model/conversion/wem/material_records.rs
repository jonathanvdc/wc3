//! Material envelopes select independent common and specialized shader authorities.
use super::{materials, shader_materials, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::starcraft::StarcraftMaterial;
use crate::model::semantic::profiles::wem::{
    Material as Surface, RepresentationAuthority, ShaderFamily, SpecializedSurface,
};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Material, TextureId};
use crate::model::wem::records::{
    D3Material, M2Material, M3Material, M3MaterialDialect, MaterialRecord, MdxMaterial,
    NativeMaterial, NativeSync, V0, V1, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19, V2, V3,
    V4, V5, V6, V7, V8, V9,
};
use crate::model::wem::{self, records, NativeAbi, RawDocument, Record, RecordLayout, Ref, Target};
use std::borrow::Cow;
use std::collections::BTreeMap;

fn one<T, U: Target>(document: &RawDocument<'_>, reference: Ref<U>) -> Result<T, Error>
where
    T: Record + RecordLayout + for<'record> wem::Read<'record>,
{
    if reference.entries != 1 {
        return Err(Error::new(
            "wem.material",
            "specialized payload must contain one record",
        ));
    }
    document
        .resolve(reference.cast::<T>())
        .map_err(|error| Error::new("wem.material", error.to_string()))?
        .get(0)
        .ok_or_else(|| Error::new("wem.material", "missing specialized payload"))?
        .map_err(|error| Error::new("wem.material", error.to_string()))
}
fn put<T: Record + wem::Write, U: Target>(
    document: &mut RawDocument<'_>,
    value: T,
) -> Result<Ref<U>, Error> {
    let section = document
        .append(&[value])
        .map_err(|error| Error::new("wem.material", error.to_string()))?;
    Ok(Ref::from_section(1, section.index()))
}

fn decode_starcraft<'view, D: M3MaterialDialect<SourceVersion = u32>, A: records::NativeLayout>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::M3Material>,
    texture_resolver: &mut dyn FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<(StarcraftMaterial<'view>, u32), Error> {
    let source: M3Material<D, A> = one(document, reference)?;
    let revision = source.source_version;
    Ok((
        shader_materials::read_starcraft(document, &source, &mut |path| texture_resolver(path))?,
        revision,
    ))
}
fn starcraft_dialect<'view, A: records::NativeLayout>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::M3Material>,
    revision: u32,
    texture_resolver: &mut dyn FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<(StarcraftMaterial<'view>, u32), Error> {
    match revision {
        0 => decode_starcraft::<V0, A>(document, reference, texture_resolver),
        1 => decode_starcraft::<V1, A>(document, reference, texture_resolver),
        2 => decode_starcraft::<V2, A>(document, reference, texture_resolver),
        3 => decode_starcraft::<V3, A>(document, reference, texture_resolver),
        4 => decode_starcraft::<V4, A>(document, reference, texture_resolver),
        5 => decode_starcraft::<V5, A>(document, reference, texture_resolver),
        6 => decode_starcraft::<V6, A>(document, reference, texture_resolver),
        7 => decode_starcraft::<V7, A>(document, reference, texture_resolver),
        8 => decode_starcraft::<V8, A>(document, reference, texture_resolver),
        9 => decode_starcraft::<V9, A>(document, reference, texture_resolver),
        10 => decode_starcraft::<V10, A>(document, reference, texture_resolver),
        11 => decode_starcraft::<V11, A>(document, reference, texture_resolver),
        12 => decode_starcraft::<V12, A>(document, reference, texture_resolver),
        13 => decode_starcraft::<V13, A>(document, reference, texture_resolver),
        14 => decode_starcraft::<V14, A>(document, reference, texture_resolver),
        15 => decode_starcraft::<V15, A>(document, reference, texture_resolver),
        16 => decode_starcraft::<V16, A>(document, reference, texture_resolver),
        17 => decode_starcraft::<V17, A>(document, reference, texture_resolver),
        18 => decode_starcraft::<V18, A>(document, reference, texture_resolver),
        19 => decode_starcraft::<V19, A>(document, reference, texture_resolver),
        _ => Err(Error::new(
            "wem.material",
            "unsupported StarCraft shader revision",
        )),
    }
}
fn read_starcraft<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::M3Material>,
    texture_resolver: &mut dyn FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<(StarcraftMaterial<'view>, u32), Error> {
    let revision = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.material", error.to_string()))?
        .ok_or_else(|| Error::new("wem.material", "missing specialized shader section"))?
        .revision();
    match document.options().native_abi {
        Some(NativeAbi::Abi32) => {
            starcraft_dialect::<records::Abi32>(document, reference, revision, texture_resolver)
        }
        Some(NativeAbi::Abi64) => {
            starcraft_dialect::<records::Abi64>(document, reference, revision, texture_resolver)
        }
        None => Err(Error::new(
            "wem.material",
            "StarCraft shader requires an explicit producer ABI",
        )),
    }
}

type DecodedMaterial<'a> = (Material<'a, Wem>, Option<(ShaderFamily, u32)>);

pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    value: &MaterialRecord,
    textures: &[TextureId],
    texture_resolver: &mut dyn FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<DecodedMaterial<'view>, Error> {
    let mut revision = None;
    let specialized = match &value.native {
        NativeMaterial::None => None,
        NativeMaterial::Mdx(reference) if reference.entries == 0 => {
            Some(SpecializedSurface::Declared(ShaderFamily::Warcraft))
        }
        NativeMaterial::M2(reference) if reference.entries == 0 => {
            Some(SpecializedSurface::Declared(ShaderFamily::WarcraftOnline))
        }
        NativeMaterial::M3(reference) if reference.entries == 0 => {
            Some(SpecializedSurface::Declared(ShaderFamily::StarCraft))
        }
        NativeMaterial::D3(reference) if reference.entries == 0 => {
            Some(SpecializedSurface::Declared(ShaderFamily::Diablo))
        }
        NativeMaterial::Mdx(reference) => {
            let source: MdxMaterial = one(document, *reference)?;
            revision = Some((ShaderFamily::Warcraft, source.source_version));
            Some(SpecializedSurface::Warcraft(
                shader_materials::read_warcraft(document, &source, textures)?,
            ))
        }
        NativeMaterial::M2(reference) => {
            let source: M2Material = one(document, *reference)?;
            revision = Some((ShaderFamily::WarcraftOnline, source.source_version));
            Some(SpecializedSurface::Wow(shader_materials::read_wow(&source)))
        }
        NativeMaterial::M3(reference) => {
            let (surface, source_version) = read_starcraft(document, *reference, texture_resolver)?;
            revision = Some((ShaderFamily::StarCraft, source_version));
            Some(SpecializedSurface::Starcraft(Box::new(surface)))
        }
        NativeMaterial::D3(reference) => {
            let version = document
                .referenced_section(*reference)
                .map_err(|error| Error::new("wem.material", error.to_string()))?
                .ok_or_else(|| Error::new("wem.material", "missing Diablo shader section"))?
                .revision();
            macro_rules! decode {
                ($dialect:ty) => {{
                    let source: D3Material<$dialect> = one(document, *reference)?;
                    revision = Some((ShaderFamily::Diablo, source.source_version));
                    Some(SpecializedSurface::Diablo(Box::new(
                        shader_materials::read_diablo(document, &source)?,
                    )))
                }};
            }
            match version {
                1 => decode!(V1),
                2 => decode!(V2),
                _ => {
                    return Err(Error::new(
                        "wem.material",
                        "unsupported Diablo shader revision",
                    ));
                }
            }
        }
    };
    Ok((
        Material {
            name: text::read(document, value.name)?,
            surface: Surface {
                common: materials::read(document, &value.common, textures)?,
                specialized,
                authority: match value.sync {
                    NativeSync::Absent => RepresentationAuthority::Unspecified,
                    NativeSync::InSync => RepresentationAuthority::Synchronized,
                    NativeSync::NativeAuthoritative => RepresentationAuthority::Specialized,
                    NativeSync::CommonEdited => RepresentationAuthority::Common,
                    NativeSync::Unknown(code) => RepresentationAuthority::Unknown(code),
                },
            },
        },
        revision,
    ))
}

pub(super) fn write(
    document: &mut RawDocument<'_>,
    value: &Material<'_, Wem>,
    textures: &BTreeMap<TextureId, u32>,
    source_version: u32,
    filename_resolver: &dyn Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<MaterialRecord, Error> {
    let native = match &value.surface.specialized {
        None => NativeMaterial::None,
        Some(SpecializedSurface::Declared(family)) => match family {
            ShaderFamily::Warcraft => NativeMaterial::Mdx(Ref::default()),
            ShaderFamily::WarcraftOnline => NativeMaterial::M2(Ref::default()),
            ShaderFamily::StarCraft => NativeMaterial::M3(Ref::default()),
            ShaderFamily::Diablo => NativeMaterial::D3(Ref::default()),
        },
        Some(SpecializedSurface::Warcraft(surface)) => {
            let body =
                shader_materials::write_warcraft(document, surface, source_version, textures)?;
            NativeMaterial::Mdx(put(document, body)?)
        }
        Some(SpecializedSurface::Wow(surface)) => NativeMaterial::M2(put(
            document,
            shader_materials::write_wow(surface, source_version),
        )?),
        Some(SpecializedSurface::Starcraft(surface)) => {
            if document.options().native_abi != Some(NativeAbi::Abi64) {
                return Err(Error::new(
                    "wem.material",
                    "StarCraft shader writer requires the declared 64-bit producer layout",
                ));
            }
            let body =
                shader_materials::write_starcraft(document, surface, source_version, &|id| {
                    filename_resolver(id)
                })?;
            NativeMaterial::M3(put(document, body)?)
        }
        Some(SpecializedSurface::Diablo(surface)) => NativeMaterial::D3(
            shader_materials::write_diablo(document, surface, source_version)?.append(document)?,
        ),
    };
    Ok(MaterialRecord {
        name: text::write_bytes(document, value.name.as_bytes())?,
        common: materials::write(document, &value.surface.common, textures)?,
        native,
        sync: match value.surface.authority {
            RepresentationAuthority::Unspecified => NativeSync::Absent,
            RepresentationAuthority::Synchronized => NativeSync::InSync,
            RepresentationAuthority::Specialized => NativeSync::NativeAuthoritative,
            RepresentationAuthority::Common => NativeSync::CommonEdited,
            RepresentationAuthority::Unknown(code) => NativeSync::Unknown(code),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::semantic::profiles::wem::{
        BlendMode, Body, CommonSurface, CullMode, MaterialFlags,
    };

    #[test]
    fn declared_empty_shader_families_and_edit_authority_roundtrip() {
        for family in [
            ShaderFamily::Warcraft,
            ShaderFamily::WarcraftOnline,
            ShaderFamily::StarCraft,
            ShaderFamily::Diablo,
        ] {
            let value = Material {
                name: "unresolved shader".into(),
                surface: Surface {
                    common: CommonSurface {
                        blend: BlendMode::Opaque,
                        cull: CullMode::None,
                        depth_test: true,
                        depth_write: true,
                        depth_bias: 0.,
                        alpha_test_threshold: 0.,
                        priority: 0,
                        flags: MaterialFlags::default(),
                        body: Body::Combiners {
                            stages: Vec::new(),
                            diffuse: [1.; 4],
                            emissive: [0.; 4],
                        },
                        features: Vec::new(),
                    },
                    specialized: Some(SpecializedSurface::Declared(family)),
                    authority: RepresentationAuthority::Specialized,
                },
            };
            let mut document = RawDocument::new();
            let encoded = write(&mut document, &value, &BTreeMap::new(), 0, &|_| {
                Err(Error::new("test", "unexpected texture"))
            })
            .unwrap();
            let (decoded, revision) = read(&document, &encoded, &[], &mut |_| {
                Err(Error::new("test", "unexpected texture"))
            })
            .unwrap();
            assert_eq!(decoded.name, "unresolved shader");
            assert!(
                matches!(decoded.surface.specialized, Some(SpecializedSurface::Declared(actual)) if actual == family)
            );
            assert_eq!(
                decoded.surface.authority,
                RepresentationAuthority::Specialized
            );
            assert_eq!(revision, None);
        }
    }
}
