//! Native Diablo material declarations map to the Diablo semantic vocabulary.
use crate::model::conversion::{Context, Error};
use crate::model::formats::diablo::d3::records as native;
use crate::model::materials::bindings::Coordinates;
use crate::model::materials::diablo3::{TextureContribution, TextureTransform};
use crate::model::materials::parameters::{OpaqueWord, ShaderTag};
use crate::model::profiles::diablo::d3::{
    AssetRef, Group, Material, MaterialColors, MaterialFlags, ParameterType, TagValue, TexAnim,
    TextureEntry, TextureType, UberMaterial, UvFlags, UvTransformMode,
};
use zerovec::ZeroVec;
fn asset(id: i32, group: Group) -> AssetRef {
    AssetRef { id, group }
}
fn anim(v: native::MaterialTextureAnim) -> TexAnim {
    TexAnim {
        amount: v.amount,
        rate0: v.rate0,
        rate1: v.rate1,
    }
}
fn native_anim(v: &TexAnim) -> native::MaterialTextureAnim {
    native::MaterialTextureAnim {
        amount: v.amount,
        rate0: v.rate0,
        rate1: v.rate1,
    }
}
pub(super) fn read(v: &native::SubObjectAppearance<'_>) -> Material<'static> {
    let c = &v.material.colors;
    Material {
        appearance_control: v.unknown00,
        uber: UberMaterial {
            shader_map: asset(v.material.shader_map, Group::ShaderMap),
            colors: MaterialColors {
                diffuse: c.diffuse,
                specular: c.specular,
                emissive: c.emissive,
                ambient: c.ambient,
                shininess: c.shininess,
                material_flags: MaterialFlags::from_bits_retain(c.material_flags),
            },
            textures: v
                .material
                .textures
                .iter()
                .map(|t| TextureEntry {
                    image: asset(t.texture, Group::Textures),
                    coordinates: Coordinates {
                        source: UvTransformMode(t.uv_transform_mode),
                        transform: TextureTransform {
                            matrix: [t.uv_row0, t.uv_row1, t.uv_row2, t.uv_row3],
                            animations: [
                                anim(t.anim_u),
                                anim(t.anim_v),
                                anim(t.anim_rotate),
                                anim(t.anim3),
                                anim(t.anim4),
                                anim(t.anim5),
                            ],
                            flags: UvFlags::from_bits_retain(t.uv_flags),
                        },
                    },
                    sampling: (),
                    contribution: TextureContribution {
                        role: TextureType(t.texture_type),
                        unclassified_controls: [t.unknown04, t.unknown9_c],
                    },
                })
                .collect(),
        },
        base_material: asset(v.material_sno, Group::Material),
        cloth: asset(v.cloth, Group::Cloth),
        shader_params: v
            .shader_params
            .iter()
            .map(|t| TagValue {
                controls: ParameterType(t.value_type),
                key: ShaderTag(t.tag_id),
                value: OpaqueWord(t.value),
            })
            .collect(),
        opaque_passes: Vec::new(),
        translucent_passes: Vec::new(),
    }
}
fn id(value: &AssetRef, expected: Group) -> Result<i32, Error> {
    if value.group != expected {
        return Err(Error::new(
            "d3.material.resource",
            "SNO group does not match native field",
        ));
    }
    Ok(value.id)
}
pub(super) fn write(
    v: &Material<'_>,
    context: &mut Context,
) -> Result<native::SubObjectAppearance<'static>, Error> {
    if !v.opaque_passes.is_empty() || !v.translucent_passes.is_empty() {
        context.unsupported("d3.material.passes","appearance materials reference shader resources; embedded render passes require separate shader-document export")?;
    }
    Ok(native::SubObjectAppearance {
        unknown00: v.appearance_control,
        cloth: id(&v.cloth, Group::Cloth)?,
        shader_params: write_tags(&v.shader_params),
        material: write_uber(&v.uber)?,
        material_sno: id(&v.base_material, Group::Material)?,
        ..Default::default()
    })
}
pub(super) fn write_tags(values: &[TagValue]) -> ZeroVec<'static, native::TagMapEntry> {
    let params = values
        .iter()
        .map(|t| native::TagMapEntry {
            value_type: t.controls.0,
            tag_id: t.key.0,
            value: t.value.0,
        })
        .collect::<Vec<_>>();
    ZeroVec::alloc_from_slice(&params)
}
pub(super) fn write_uber(v: &UberMaterial) -> Result<native::UberMaterial<'static>, Error> {
    let c = &v.colors;
    let textures = v
        .textures
        .iter()
        .map(|t| {
            Ok(native::MaterialTextureEntry {
                texture_type: t.contribution.role.0,
                unknown04: t.contribution.unclassified_controls[0],
                texture: id(&t.image, Group::Textures)?,
                uv_transform_mode: t.coordinates.source.0,
                uv_row0: t.coordinates.transform.matrix[0],
                uv_row1: t.coordinates.transform.matrix[1],
                uv_row2: t.coordinates.transform.matrix[2],
                uv_row3: t.coordinates.transform.matrix[3],
                anim_u: native_anim(&t.coordinates.transform.animations[0]),
                anim_v: native_anim(&t.coordinates.transform.animations[1]),
                anim_rotate: native_anim(&t.coordinates.transform.animations[2]),
                anim3: native_anim(&t.coordinates.transform.animations[3]),
                anim4: native_anim(&t.coordinates.transform.animations[4]),
                anim5: native_anim(&t.coordinates.transform.animations[5]),
                uv_flags: t.coordinates.transform.flags.bits(),
                unknown9_c: t.contribution.unclassified_controls[1],
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(native::UberMaterial {
        shader_map: id(&v.shader_map, Group::ShaderMap)?,
        colors: native::MaterialColors {
            diffuse: c.diffuse,
            specular: c.specular,
            emissive: c.emissive,
            ambient: c.ambient,
            shininess: c.shininess,
            material_flags: c.material_flags.bits(),
        },
        textures: ZeroVec::alloc_from_slice(&textures),
        ..Default::default()
    })
}
