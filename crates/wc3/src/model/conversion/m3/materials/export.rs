//! Export of active native material bodies and common render state.
use super::{append, append_string, native, nr, read_error, Error};
use crate::model::conversion::{
    m3::{metadata, RootFields},
    Context, IssueKind,
};
use crate::model::wem::{self, semantic as sem};
use native::Envelope;
pub(in crate::model::conversion::m3) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    document: &mut native::Document<'static, E>,
    context: &mut Context,
    root_revision: u32,
) -> Result<RootFields, Error> {
    let mut families = super::Families::default();
    let modes = super::super::animation::modes(model, context)?;
    let mut maps = Vec::new();
    for (index, material) in model.materials.iter().enumerate() {
        let mut native_body = if let Some(native) = &model.native {
            super::body_with_revision(&native.document, native.model, model.profile, index)?
        } else {
            None
        };
        if let Some((body, source_version)) = &native_body {
            let output_version = match body {
                wem::records::M3Body::Standard(_) => 20,
                wem::records::M3Body::Displacement(_) => 4,
                wem::records::M3Body::Composite(_) => 2,
                wem::records::M3Body::Terrain(_) | wem::records::M3Body::Creep(_) => 1,
                wem::records::M3Body::Reflection(_)
                | wem::records::M3Body::LensFlare(_)
                | wem::records::M3Body::DataDriven(_) => 3,
                _ => 0,
            };
            if *source_version != output_version {
                context.issue(format!("materials[{index}].revision"), IssueKind::Normalized,
                    format!("Native material fields are rebuilt in revision {output_version} instead of source revision {source_version}"));
            }
        }
        let mut native_body = native_body.take().map(|value| value.0);
        if let Some(body) = &mut native_body {
            super::super::animation::reconcile_material(body, &modes);
        }
        let mut m = match native_body {
            Some(wem::records::M3Body::Standard(body)) => super::export_standard(
                *body,
                model.native.as_ref().unwrap().document.raw(),
                document,
            )?,
            Some(body) => {
                if root_revision < 29 && matches!(body, wem::records::M3Body::LensFlare(_))
                    || root_revision < 30 && matches!(body, wem::records::M3Body::DataDriven(_))
                {
                    return Err(Error::new(
                        format!("materials[{index}]"),
                        "selected native root revision has no slot for this shader family",
                    ));
                }
                if root_revision < 25
                    && matches!(
                        body,
                        wem::records::M3Body::VolumeNoise(_)
                            | wem::records::M3Body::Stb(_)
                            | wem::records::M3Body::Reflection(_)
                    )
                {
                    return Err(Error::new(
                        format!("materials[{index}]"),
                        "selected native root revision has no slot for this shader family",
                    ));
                }
                let map = families.push(
                    body,
                    model.native.as_ref().unwrap().document.raw(),
                    document,
                )?;
                let name = append_string(document, &material.name)?;
                let i = map.material_index as usize;
                match map.material_type {
                    nr::MaterialKind::Displacement => {
                        families.displacement_materials[i].name = name
                    }
                    nr::MaterialKind::Composite => families.composite_materials[i].name = name,
                    nr::MaterialKind::Terrain => families.terrain_materials[i].name = name,
                    nr::MaterialKind::Volume => families.volume_materials[i].name = name,
                    nr::MaterialKind::VolumeNoise => families.volume_noise_materials[i].name = name,
                    nr::MaterialKind::Creep => families.creep_materials[i].name = name,
                    nr::MaterialKind::SplatTerrainBake => families.stb_materials[i].name = name,
                    nr::MaterialKind::Reflection => families.reflection_materials[i].name = name,
                    nr::MaterialKind::LensFlare => families.lens_flare_materials[i].name = name,
                    nr::MaterialKind::DataDriven => {
                        families.data_driven_materials[i].material_name = name
                    }
                    _ => unreachable!(),
                }
                maps.push(map);
                continue;
            }
            None => nr::StandardMaterial::<nr::V20>::default(),
        };
        m.name = append_string(document, &material.name)?;
        super::render_state::apply(
            &mut m,
            material.blend,
            material.flags,
            context,
            &format!("materials[{index}]"),
        )?;
        if let Some(texture) = material.texture {
            let texture = model.textures.get(texture as usize).ok_or_else(|| {
                Error::new(format!("materials[{index}].texture"), "missing texture")
            })?;
            if texture.id.is_some() && texture.path.is_empty() {
                return Err(Error::new(
                    format!("materials[{index}].texture"),
                    "M3 requires a texture filename",
                ));
            }
            let mut layer = if m.diffuse_layer.entries != 0 {
                document
                    .resolve::<nr::TextureLayer<nr::V26>>(m.diffuse_layer.cast())
                    .map_err(read_error)?
                    .get(0)
                    .ok_or_else(|| Error::new("materials.diffuse", "missing translated layer"))?
                    .map_err(read_error)?
            } else {
                nr::TextureLayer::<nr::V26> {
                    color: nr::AnimationValue {
                        initial_value: [255; 4],
                        null_value: [255; 4],
                        ..Default::default()
                    },
                    rgb_multiply: nr::AnimationValue {
                        initial_value: 1.,
                        null_value: 1.,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            };
            layer.texture_path = append_string(document, &texture.path)?;
            m.diffuse_layer = append(document, &[layer])?.cast();
        }
        let material_index = families.standard.len() as u32;
        families.standard.push(m);
        maps.push(nr::MaterialMap {
            material_type: nr::MaterialKind::Standard,
            material_index,
        });
    }
    metadata::restore_reserved(model, &maps, &mut families.data_driven_materials)?;
    let material_ref = append(document, &families.standard)?.cast();
    let displacement_materials = append(document, &families.displacement_materials)?.cast();
    let composite_materials = append(document, &families.composite_materials)?.cast();
    let terrain_materials = append(document, &families.terrain_materials)?.cast();
    let volume_materials = append(document, &families.volume_materials)?.cast();
    let volume_noise_materials = append(document, &families.volume_noise_materials)?.cast();
    let creep_materials = append(document, &families.creep_materials)?.cast();
    let stb_materials = append(document, &families.stb_materials)?.cast();
    let reflection_materials = append(document, &families.reflection_materials)?.cast();
    let lens_flare_materials = append(document, &families.lens_flare_materials)?.cast();
    let data_driven_materials = append(document, &families.data_driven_materials)?.cast();
    let maps_ref = append(document, &maps)?.cast();
    Ok(RootFields {
        material_maps: maps_ref,
        standard_materials: material_ref,
        displacement_materials,
        composite_materials,
        terrain_materials,
        volume_materials,
        volume_noise_materials,
        creep_materials,
        stb_materials,
        reflection_materials,
        lens_flare_materials,
        data_driven_materials,
        ..Default::default()
    })
}
