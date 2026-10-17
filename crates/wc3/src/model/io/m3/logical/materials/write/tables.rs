//! Material tables are built from handles after all semantic edits.
use super::{append, native, nr, surface, Envelope, Error, Writer};

#[derive(Default)]
pub(in crate::model::io::m3::logical) struct Tables {
    pub map: native::Ref<nr::targets::MaterialMap>,
    pub standard: native::Ref<nr::targets::StandardMaterial>,
    pub displacement: native::Ref<nr::targets::DisplacementMaterial>,
    pub composite: native::Ref<nr::targets::CompositeMaterial>,
    pub terrain: native::Ref<nr::targets::TerrainMaterial>,
    pub volume: native::Ref<nr::targets::VolumeMaterial>,
    pub volume_noise: native::Ref<nr::targets::VolumeNoiseMaterial>,
    pub creep: native::Ref<nr::targets::CreepMaterial>,
    pub hair: native::Ref<nr::targets::HairMaterial>,
    pub splat_terrain_bake: native::Ref<nr::targets::STBMaterial>,
    pub reflection: native::Ref<nr::targets::ReflectionMaterial>,
    pub lens_flare: native::Ref<nr::targets::LensFlare>,
    pub data_driven: native::Ref<nr::targets::DataDrivenMaterial>,
}
pub(in crate::model::io::m3::logical) fn encode<E: Envelope>(
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<Tables, Error> {
    let mut maps = Vec::new();
    let mut standard = Vec::new();
    let mut displacement = Vec::new();
    let mut composite = Vec::new();
    let mut terrain = Vec::new();
    let mut volume = Vec::new();
    let mut volume_noise = Vec::new();
    let mut creep = Vec::new();
    let mut hair = Vec::new();
    let mut splat_terrain_bake = Vec::new();
    let mut reflection = Vec::new();
    let mut lens_flare = Vec::new();
    let mut data_driven = Vec::new();
    let entries: Vec<_> = writer.model.materials().iter().map(|(id, _)| id).collect();
    writer.materials = entries
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u32))
        .collect();
    for id in entries {
        let material = writer
            .model
            .material(id)
            .expect("material from checked arena");
        let body = &material.surface;
        let (kind, index) = match body {
            surface::MaterialStep::Standard(value) => {
                let index = standard.len() as u32;
                standard.push(super::standard::encode(&material.name, value, writer)?);
                (nr::MaterialKind::Standard, index)
            }
            surface::MaterialStep::Displacement(value) => {
                let index = displacement.len() as u32;
                displacement.push(super::displacementmaterial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::Displacement, index)
            }
            surface::MaterialStep::Composite(value) => {
                let index = composite.len() as u32;
                composite.push(super::compositematerial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::Composite, index)
            }
            surface::MaterialStep::Terrain(value) => {
                let index = terrain.len() as u32;
                terrain.push(super::terrainmaterial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::Terrain, index)
            }
            surface::MaterialStep::Volume(value) => {
                let index = volume.len() as u32;
                volume.push(super::volumematerial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::Volume, index)
            }
            surface::MaterialStep::VolumeNoise(value) => {
                let index = volume_noise.len() as u32;
                volume_noise.push(super::volumenoisematerial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::VolumeNoise, index)
            }
            surface::MaterialStep::Creep(value) => {
                let index = creep.len() as u32;
                creep.push(super::creepmaterial::encode(&material.name, value, writer)?);
                (nr::MaterialKind::Creep, index)
            }
            surface::MaterialStep::Hair(value) => {
                let index = hair.len() as u32;
                hair.push(super::hair::encode(&material.name, value, writer)?);
                (nr::MaterialKind::Hair, index)
            }
            surface::MaterialStep::SplatTerrainBake(value) => {
                let index = splat_terrain_bake.len() as u32;
                splat_terrain_bake.push(super::stbmaterial::encode(&material.name, value, writer)?);
                (nr::MaterialKind::SplatTerrainBake, index)
            }
            surface::MaterialStep::Reflection(value) => {
                let index = reflection.len() as u32;
                reflection.push(super::reflectionmaterial::encode(
                    &material.name,
                    value,
                    writer,
                )?);
                (nr::MaterialKind::Reflection, index)
            }
            surface::MaterialStep::LensFlare(value) => {
                let index = lens_flare.len() as u32;
                lens_flare.push(super::flare::encode(&material.name, value, writer)?);
                (nr::MaterialKind::LensFlare, index)
            }
            surface::MaterialStep::DataDriven(value) => {
                let index = data_driven.len() as u32;
                data_driven.push(super::data_driven::encode(&material.name, value, writer)?);
                (nr::MaterialKind::DataDriven, index)
            }
        };
        maps.push(nr::MaterialMap {
            material_type: kind,
            material_index: index,
        });
    }
    Ok(Tables {
        map: append(writer.document, &maps)?.cast(),
        standard: append(writer.document, &standard)?.cast(),
        displacement: append(writer.document, &displacement)?.cast(),
        composite: append(writer.document, &composite)?.cast(),
        terrain: append(writer.document, &terrain)?.cast(),
        volume: append(writer.document, &volume)?.cast(),
        volume_noise: append(writer.document, &volume_noise)?.cast(),
        creep: append(writer.document, &creep)?.cast(),
        hair: append(writer.document, &hair)?.cast(),
        splat_terrain_bake: append(writer.document, &splat_terrain_bake)?.cast(),
        reflection: append(writer.document, &reflection)?.cast(),
        lens_flare: append(writer.document, &lens_flare)?.cast(),
        data_driven: append(writer.document, &data_driven)?.cast(),
    })
}
