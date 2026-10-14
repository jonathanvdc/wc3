//! Native Diablo material declarations share their authored vocabulary with WEM.
use crate::model::conversion::{Context, Error};
use crate::model::diablo::d3::records as native;
use crate::model::semantic::profiles::diablo::d3::{
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
                    texture_type: TextureType::Unknown(t.texture_type),
                    unclassified_control: t.unknown04,
                    texture: asset(t.texture, Group::Textures),
                    uv_transform_mode: UvTransformMode::Unknown(t.uv_transform_mode),
                    uv_row0: t.uv_row0,
                    uv_row1: t.uv_row1,
                    uv_row2: t.uv_row2,
                    uv_row3: t.uv_row3,
                    anim_u: anim(t.anim_u),
                    anim_v: anim(t.anim_v),
                    anim_rotate: anim(t.anim_rotate),
                    anim3: anim(t.anim3),
                    anim4: anim(t.anim4),
                    anim5: anim(t.anim5),
                    uv_flags: UvFlags::from_bits_retain(t.uv_flags),
                    unclassified_control2: t.unknown9_c,
                })
                .collect(),
        },
        base_material: asset(v.material_sno, Group::Material),
        cloth: asset(v.cloth, Group::Cloth),
        shader_params: v
            .shader_params
            .iter()
            .map(|t| TagValue {
                value_type: ParameterType::Unknown(t.value_type),
                tag_id: t.tag_id,
                value: t.value,
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
            value_type: match t.value_type {
                ParameterType::Unknown(v) => v,
            },
            tag_id: t.tag_id,
            value: t.value,
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
                texture_type: match t.texture_type {
                    TextureType::Unknown(v) => v,
                },
                unknown04: t.unclassified_control,
                texture: id(&t.texture, Group::Textures)?,
                uv_transform_mode: match t.uv_transform_mode {
                    UvTransformMode::Unknown(v) => v,
                },
                uv_row0: t.uv_row0,
                uv_row1: t.uv_row1,
                uv_row2: t.uv_row2,
                uv_row3: t.uv_row3,
                anim_u: native_anim(&t.anim_u),
                anim_v: native_anim(&t.anim_v),
                anim_rotate: native_anim(&t.anim_rotate),
                anim3: native_anim(&t.anim3),
                anim4: native_anim(&t.anim4),
                anim5: native_anim(&t.anim5),
                uv_flags: t.uv_flags.bits(),
                unknown9_c: t.unclassified_control2,
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
