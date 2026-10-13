//! Families mappings for native M3 materials.
use super::{
    anim, append, append_string, export_datadriven, export_lensflare, export_standard,
    get_datadriven, get_layer, get_lensflare, get_standard, native, native_anim, native_layer_ref,
    nr, read_error, revision, string, text, wem, wr, Envelope, Error, Slot,
};

pub(in crate::model::conversion::m3) fn body<E: Envelope>(
    model: &native::Model<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    map: nr::MaterialMap,
) -> Result<(wr::M3Body, u32), Error> {
    let source = model.document();
    let root = model.root_record().map_err(read_error)?;
    match map.material_type {
        1 => {
            let reference = native::visit_m3_root!(root, |r| r.standard_materials);
            Ok((
                wr::M3Body::Standard(Box::new(get_standard(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        2 => {
            let reference: native::Ref<nr::targets::DisplacementMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.displacement_materials));
            Ok((
                wr::M3Body::Displacement(Box::new(get_displacementmaterial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        3 => {
            let reference: native::Ref<nr::targets::CompositeMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.composite_materials));
            Ok((
                wr::M3Body::Composite(Box::new(get_compositematerial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        4 => {
            let reference: native::Ref<nr::targets::TerrainMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.terrain_materials));
            Ok((
                wr::M3Body::Terrain(Box::new(get_terrainmaterial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        5 => {
            let reference: native::Ref<nr::targets::VolumeMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.volume_materials));
            Ok((
                wr::M3Body::Volume(Box::new(get_volumematerial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        6 => {
            let reference: native::Ref<nr::targets::VolumeNoiseMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.volume_noise_materials));
            Ok((
                wr::M3Body::VolumeNoise(Box::new(get_volumenoisematerial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        7 => {
            let reference: native::Ref<nr::targets::CreepMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.creep_materials));
            Ok((
                wr::M3Body::Creep(Box::new(get_creepmaterial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        9 => {
            let reference: native::Ref<nr::targets::STBMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.stb_materials));
            Ok((
                wr::M3Body::Stb(Box::new(get_stbmaterial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        10 => {
            let reference: native::Ref<nr::targets::ReflectionMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.reflection_materials));
            Ok((
                wr::M3Body::Reflection(Box::new(get_reflectionmaterial(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        11 => {
            let reference: native::Ref<nr::targets::LensFlare> =
                native::visit_m3_root!(root, |r| Slot::value(&r.lens_flare_materials));
            Ok((
                wr::M3Body::LensFlare(Box::new(get_lensflare(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        12 => {
            let reference: native::Ref<nr::targets::DataDrivenMaterial> =
                native::visit_m3_root!(root, |r| Slot::value(&r.data_driven_materials));
            Ok((
                wr::M3Body::DataDriven(Box::new(get_datadriven(
                    source,
                    doc,
                    reference,
                    map.material_index as usize,
                )?)),
                revision(source, reference).map_err(read_error)?,
            ))
        }
        _ => Err(Error::new(
            "m3.material",
            "material family has no native WEM mapping",
        )),
    }
}

#[derive(Default)]
pub(in crate::model::conversion::m3) struct Families {
    pub(in crate::model::conversion::m3) lens_flare_materials: Vec<nr::LensFlare<nr::V3>>,
    pub(in crate::model::conversion::m3) data_driven_materials: Vec<nr::DataDrivenMaterial<nr::V3>>,
    pub(in crate::model::conversion::m3) standard: Vec<nr::StandardMaterial<nr::V20>>,
    pub(in crate::model::conversion::m3) displacement_materials:
        Vec<nr::DisplacementMaterial<nr::V4>>,
    pub(in crate::model::conversion::m3) composite_materials: Vec<nr::CompositeMaterial<nr::V2>>,
    pub(in crate::model::conversion::m3) terrain_materials: Vec<nr::TerrainMaterial<nr::V1>>,
    pub(in crate::model::conversion::m3) volume_materials: Vec<nr::VolumeMaterial>,
    pub(in crate::model::conversion::m3) volume_noise_materials: Vec<nr::VolumeNoiseMaterial>,
    pub(in crate::model::conversion::m3) creep_materials: Vec<nr::CreepMaterial<nr::V1>>,
    pub(in crate::model::conversion::m3) stb_materials: Vec<nr::STBMaterial>,
    pub(in crate::model::conversion::m3) reflection_materials: Vec<nr::ReflectionMaterial<nr::V3>>,
}

impl Families {
    pub(in crate::model::conversion::m3) fn push<E: Envelope>(
        &mut self,
        body: wr::M3Body,
        source: &wem::RawDocument<'_>,
        doc: &mut native::Document<'static, E>,
    ) -> Result<nr::MaterialMap, Error> {
        let (kind, index) = match body {
            wr::M3Body::Standard(b) => {
                let i = self.standard.len();
                self.standard.push(export_standard(*b, source, doc)?);
                (1, i)
            }
            wr::M3Body::Displacement(b) => {
                let i = self.displacement_materials.len();
                self.displacement_materials
                    .push(export_displacementmaterial(*b, source, doc)?);
                (2, i)
            }
            wr::M3Body::Composite(b) => {
                let i = self.composite_materials.len();
                self.composite_materials
                    .push(export_compositematerial(*b, source, doc)?);
                (3, i)
            }
            wr::M3Body::Terrain(b) => {
                let i = self.terrain_materials.len();
                self.terrain_materials
                    .push(export_terrainmaterial(*b, source, doc)?);
                (4, i)
            }
            wr::M3Body::Volume(b) => {
                let i = self.volume_materials.len();
                self.volume_materials
                    .push(export_volumematerial(*b, source, doc)?);
                (5, i)
            }
            wr::M3Body::VolumeNoise(b) => {
                let i = self.volume_noise_materials.len();
                self.volume_noise_materials
                    .push(export_volumenoisematerial(*b, source, doc)?);
                (6, i)
            }
            wr::M3Body::Creep(b) => {
                let i = self.creep_materials.len();
                self.creep_materials
                    .push(export_creepmaterial(*b, source, doc)?);
                (7, i)
            }
            wr::M3Body::Stb(b) => {
                let i = self.stb_materials.len();
                self.stb_materials
                    .push(export_stbmaterial(*b, source, doc)?);
                (9, i)
            }
            wr::M3Body::Reflection(b) => {
                let i = self.reflection_materials.len();
                self.reflection_materials
                    .push(export_reflectionmaterial(*b, source, doc)?);
                (10, i)
            }
            wr::M3Body::LensFlare(b) => {
                let i = self.lens_flare_materials.len();
                self.lens_flare_materials
                    .push(export_lensflare(source, doc, *b)?);
                (11, i)
            }
            wr::M3Body::DataDriven(b) => {
                let i = self.data_driven_materials.len();
                self.data_driven_materials
                    .push(export_datadriven(source, doc, *b)?);
                (12, i)
            }
        };
        Ok(nr::MaterialMap {
            material_type: kind,
            material_index: index as u32,
        })
    }
}

mod displacementmaterial;
use displacementmaterial::export_displacementmaterial;
use displacementmaterial::get_displacementmaterial;

mod compositematerial;
use compositematerial::export_compositematerial;
use compositematerial::get_compositematerial;

mod terrainmaterial;
use terrainmaterial::export_terrainmaterial;
use terrainmaterial::get_terrainmaterial;

mod volumematerial;
use volumematerial::export_volumematerial;
use volumematerial::get_volumematerial;

mod volumenoisematerial;
use volumenoisematerial::export_volumenoisematerial;
use volumenoisematerial::get_volumenoisematerial;

mod creepmaterial;
use creepmaterial::export_creepmaterial;
use creepmaterial::get_creepmaterial;

mod stbmaterial;
use stbmaterial::export_stbmaterial;
use stbmaterial::get_stbmaterial;

mod reflectionmaterial;
use reflectionmaterial::export_reflectionmaterial;
use reflectionmaterial::get_reflectionmaterial;
