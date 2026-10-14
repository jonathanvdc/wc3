//! Logical material identities and complete source-family traversal.
use super::super::bindings::{self, Bindings};
use super::{native, nr, read_error, surface, Envelope, Error, Reader, Slot};
use crate::model::conversion::Context;

use crate::model::conversion::m3::logical::GraphBuilder;
use crate::model::semantic::{self as sem, MaterialId};
use sem::OwnerId;
use std::borrow::Cow;
use std::collections::BTreeMap;
pub(super) fn complete_maps<E: Envelope>(
    source: &native::Model<'_, E>,
) -> Result<Vec<nr::MaterialMap>, Error> {
    let native_root = source.root_record().map_err(read_error)?;
    let (map_ref, _standard_ref) =
        native::visit_m3_root!(native_root, |r| (r.material_maps, r.standard_materials));
    let mut maps = source
        .document()
        .resolve::<nr::MaterialMap>(map_ref.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let counts = native::visit_m3_root!(native_root, |r| [
        (1, r.standard_materials.entries),
        (2, r.displacement_materials.entries),
        (3, r.composite_materials.entries),
        (4, r.terrain_materials.entries),
        (5, r.volume_materials.entries),
        (7, r.creep_materials.entries),
        (8, r.hair_materials.entries),
        (
            6,
            Slot::<native::Ref<nr::targets::VolumeNoiseMaterial>>::value(&r.volume_noise_materials)
                .entries
        ),
        (
            9,
            Slot::<native::Ref<nr::targets::STBMaterial>>::value(&r.stb_materials).entries
        ),
        (
            10,
            Slot::<native::Ref<nr::targets::ReflectionMaterial>>::value(&r.reflection_materials)
                .entries
        ),
        (
            11,
            Slot::<native::Ref<nr::targets::LensFlare>>::value(&r.lens_flare_materials).entries
        ),
        (
            12,
            Slot::<native::Ref<nr::targets::DataDrivenMaterial>>::value(&r.data_driven_materials)
                .entries
        ),
    ]);
    for (kind, count) in counts {
        for material_index in 0..count {
            if !maps.iter().any(|map| {
                map.material_type == nr::MaterialKind::from_raw(kind)
                    && map.material_index == material_index
            }) {
                maps.push(nr::MaterialMap {
                    material_type: nr::MaterialKind::from_raw(kind),
                    material_index,
                });
            }
        }
    }
    Ok(maps)
}

fn body<'a, E: Envelope>(
    source: &'a native::Model<'_, E>,
    reader: &mut Reader<'a, '_>,
    map: nr::MaterialMap,
) -> Result<surface::Material<'a>, Error> {
    let root = source.root_record().map_err(read_error)?;
    let document = source.document();
    match map.material_type {
        nr::MaterialKind::Standard => {
            let reference: native::Ref<nr::targets::StandardMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.standard_materials));
            Ok(surface::Material::Standard(
                (super::standard::get_standard(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Displacement => {
            let reference: native::Ref<nr::targets::DisplacementMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.displacement_materials));
            Ok(surface::Material::Displacement(
                (super::displacementmaterial::get_displacementmaterial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Composite => {
            let reference: native::Ref<nr::targets::CompositeMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.composite_materials));
            Ok(surface::Material::Composite(
                (super::compositematerial::get_compositematerial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Terrain => {
            let reference: native::Ref<nr::targets::TerrainMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.terrain_materials));
            Ok(surface::Material::Terrain(
                (super::terrainmaterial::get_terrainmaterial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Volume => {
            let reference: native::Ref<nr::targets::VolumeMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.volume_materials));
            Ok(surface::Material::Volume(
                (super::volumematerial::get_volumematerial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::VolumeNoise => {
            let reference: native::Ref<nr::targets::VolumeNoiseMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.volume_noise_materials));
            Ok(surface::Material::VolumeNoise(
                (super::volumenoisematerial::get_volumenoisematerial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Creep => {
            let reference: native::Ref<nr::targets::CreepMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.creep_materials));
            Ok(surface::Material::Creep(
                (super::creepmaterial::get_creepmaterial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Hair => {
            let reference: native::Ref<nr::targets::HairMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.hair_materials));
            Ok(surface::Material::Hair(
                (super::hair::get_hair(document, reader, reference, map.material_index as usize)?)
                    .into(),
            ))
        }
        nr::MaterialKind::SplatTerrainBake => {
            let reference: native::Ref<nr::targets::STBMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.stb_materials));
            Ok(surface::Material::SplatTerrainBake(
                (super::stbmaterial::get_stbmaterial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Reflection => {
            let reference: native::Ref<nr::targets::ReflectionMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.reflection_materials));
            Ok(surface::Material::Reflection(
                (super::reflectionmaterial::get_reflectionmaterial(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::LensFlare => {
            let reference: native::Ref<nr::targets::LensFlare> =
                native::visit_m3_root!(root, |r| Slot::value(&r.lens_flare_materials));
            Ok(surface::Material::LensFlare(
                (super::flare::get_lensflare(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::DataDriven => {
            let reference: native::Ref<nr::targets::DataDrivenMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.data_driven_materials));
            Ok(surface::Material::DataDriven(
                (super::data_driven::get_datadriven(
                    document,
                    reader,
                    reference,
                    map.material_index as usize,
                )?)
                .into(),
            ))
        }
        nr::MaterialKind::Unknown(code) => Err(Error::new(
            "m3.material",
            format!("unknown material kind {code}"),
        )),
    }
}
pub(in crate::model::conversion::m3::logical) fn read<'a, E: Envelope>(
    source: &'a native::Model<'_, E>,
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
    bindings: &mut Bindings,
) -> Result<Vec<MaterialId>, Error> {
    let maps = complete_maps(source)?;
    let mut materials = Vec::with_capacity(maps.len());
    for _ in &maps {
        materials.push(builder.reserve_material());
    }
    let mut reader = Reader {
        builder,
        textures: BTreeMap::new(),
        materials: materials.clone(),
        name: None,
    };
    for (index, map) in maps.into_iter().enumerate() {
        if let nr::MaterialKind::Unknown(code) = map.material_type {
            context.unsupported(
                format!("materials[{index}]"),
                format!("unknown source material family {code}"),
            )?;
            reader.builder.set_material(
                materials[index],
                sem::Material {
                    name: Cow::Borrowed(""),
                    surface: surface::Material::Standard((surface::Standard::default()).into()),
                },
            )?;
            continue;
        }
        reader.name = None;
        let mut body = body(source, &mut reader, map)?;
        bindings::material(&mut body, OwnerId::Material(materials[index]), bindings);
        let name = reader.name.take().unwrap_or(Cow::Borrowed(""));
        reader.builder.set_material(
            materials[index],
            sem::Material {
                name,
                surface: body,
            },
        )?;
    }
    Ok(materials)
}
