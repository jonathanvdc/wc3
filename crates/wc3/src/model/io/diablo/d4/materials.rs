//! External appearance selections and supplied shader resources have separate authoring authority.
use crate::model::conversion::{Context, Error};
use crate::model::formats::diablo::d4::records as native;
use crate::model::materials::diablo4::{TextureContribution, TexturePlayback};
use crate::model::materials::parameters::ShaderValue;
use crate::model::profiles::diablo::d4::{
    AppearanceMaterialControls, GraphAbsence, Material, MaterialValues, ParameterControls,
    ScalarValue, Shader, TextureInput, TextureSelection, VectorValue, D4,
};
use crate::model::profiles::NoControls;
use crate::model::{Model, ModelBuilder, Resource, SnoNamespace, Texture};
use zerovec::ZeroVec;

pub(super) fn read_appearance(
    value: &native::SubObjectAppearance<'_>,
    context: &mut Context,
) -> Result<Material, Error> {
    if !value.anim_cloth_overrides.is_empty() {
        context.unsupported(
            "d4.material.animation_cloth_overrides",
            "animation-dependent cloth selections have no established logical mapping",
        )?;
    }
    Ok(Material {
        appearance: AppearanceMaterialControls {
            flags: value.flags,
            base_material: value.material,
            override_material: value.override_material,
            cloth: value.cloth,
            high_quality_cloth_override: value.high_quality_cloth_override,
            mounted_cloth_overrides: value.mounted_cloth_overrides,
            effect_group: value.effect_group,
            variant_materials: value.variant_materials.iter().collect(),
        },
        shader: None,
    })
}
pub(super) fn write_appearance(
    value: &Material,
    context: &mut Context,
) -> Result<native::SubObjectAppearance<'static>, Error> {
    if value.shader.is_some() {
        context.unsupported("d4.material.shader", "appearance output contains external shader references; resolved shader edits require separate material-document output")?;
    }
    let value = &value.appearance;
    Ok(native::SubObjectAppearance {
        flags: value.flags,
        material: value.base_material,
        override_material: value.override_material,
        cloth: value.cloth,
        high_quality_cloth_override: value.high_quality_cloth_override,
        mounted_cloth_overrides: value.mounted_cloth_overrides,
        effect_group: value.effect_group,
        variant_materials: ZeroVec::alloc_from_slice(&value.variant_materials),
        ..Default::default()
    }
    .into_owned())
}

pub(super) fn read_shader(
    value: &native::UberMaterial<'_>,
    model: &mut ModelBuilder<'_, D4>,
    context: &mut Context,
) -> Result<Shader, Error> {
    let textures = value
        .mat_tex_list
        .iter()
        .map(|entry| {
            let native = &entry.mat_tex;
            if !native.tex_anim.is_empty() {
                context.unsupported(
                    "d4.material.texture.animation",
                    "shader-local texture animation programs require a dedicated semantic mapping",
                )?;
            }
            let selection = if native.tex < 0 {
                TextureSelection::Absent(native.tex)
            } else {
                let resource = Resource::Sno {
                    namespace: SnoNamespace::Diablo4,
                    group: 44,
                    id: native.tex as u32,
                };
                let existing = model
                    .model()
                    .textures()
                    .find(|(_, image)| image.resource == resource)
                    .map(|(id, _)| id);
                TextureSelection::Image(match existing {
                    Some(id) => id,
                    None => model.add_texture(Texture {
                        resource,
                        controls: NoControls,
                        color_space: Default::default(),
                        sampler: Default::default(),
                    })?,
                })
            };
            Ok(TextureInput {
                image: selection,
                coordinates: (),
                sampling: TexturePlayback {
                    animation_state_index: native.tex_anim_state_index,
                },
                contribution: TextureContribution {
                    semantic: entry.shader_tex,
                    unclassified_control: native.unk_e1a4010,
                },
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let values = value
        .run_time_material_values
        .iter()
        .map(|set| {
            if !set.material_scalar_graphs.is_empty() || !set.material_vector_graphs.is_empty() {
                context.unsupported(
                    "d4.material.parameter.graphs",
                    "shader parameter graph programs have no established semantic mapping",
                )?;
            }
            if set
                .material_scalar_values
                .iter()
                .any(|v| v.value.path_val_graph_index >= 0)
                || set
                    .material_vector_values
                    .iter()
                    .any(|v| v.value.path_val_graph_index >= 0)
            {
                context.unsupported(
                    "d4.material.parameter.graph_selection",
                    "active parameter graph selections require an established semantic mapping",
                )?;
            }
            Ok(MaterialValues {
                scalars: set
                    .material_scalar_values
                    .iter()
                    .map(|entry| {
                        let value = &entry.value;
                        ScalarValue {
                            key: ShaderValue(value.material_value),
                            value: value.value,
                            controls: ParameterControls {
                                graph: GraphAbsence::new(value.path_val_graph_index)
                                    .unwrap_or_default(),
                                light_group: value.light_group,
                            },
                        }
                    })
                    .collect(),
                vectors: set
                    .material_vector_values
                    .iter()
                    .map(|entry| {
                        let value = &entry.value;
                        VectorValue {
                            key: ShaderValue(value.material_value),
                            value: value.value,
                            controls: ParameterControls {
                                graph: GraphAbsence::new(value.path_val_graph_index)
                                    .unwrap_or_default(),
                                light_group: value.light_group,
                            },
                        }
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Shader {
        asset: None,
        shader_map: value.shader_map,
        value_set_override: value.material_value_set_override,
        sort_priority: value.mat.sort_pri,
        textures,
        values,
        texture_animation_state_count: value.tex_anim_state_count,
        has_graph: value.has_graph,
    })
}
pub(super) fn write_shader(
    value: &Shader,
    model: &Model<'_, D4>,
    _context: &mut Context,
) -> Result<native::UberMaterial<'static>, Error> {
    let textures = value
        .textures
        .iter()
        .map(|input| {
            let tex = match input.image {
                TextureSelection::Absent(code) if code < 0 => code,
                TextureSelection::Image(id) => {
                    match model.texture(id).map(|image| &image.resource) {
                        Some(Resource::Sno {
                            namespace: SnoNamespace::Diablo4,
                            group: 44,
                            id,
                        }) => i32::try_from(*id).map_err(|_| {
                            Error::new(
                                "d4.material.texture",
                                "texture SNO exceeds the signed native identity range",
                            )
                        })?,
                        _ => return Err(Error::new(
                            "d4.material.texture",
                            "shader input requires a Diablo IV texture SNO from its owning model",
                        )),
                    }
                }
                _ => {
                    return Err(Error::new(
                        "d4.material.texture",
                        "absent texture selection must retain a negative native code",
                    ))
                }
            };
            Ok(native::MaterialTextureEntry {
                shader_tex: input.contribution.semantic,
                mat_tex: native::MaterialTexture {
                    tex,
                    tex_anim_state_index: input.sampling.animation_state_index,
                    unk_e1a4010: input.contribution.unclassified_control,
                    ..Default::default()
                },
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let values = value
        .values
        .iter()
        .map(|set| {
            Ok(native::RunTimeMaterialValues {
                material_scalar_values: set
                    .scalars
                    .iter()
                    .map(|v| native::RunTimeMaterialScalarValueEntry {
                        value: native::RunTimeMaterialValueFloat {
                            material_value: v.key.0,
                            value: v.value,
                            path_val_graph_index: v.controls.graph.code(),
                            light_group: v.controls.light_group,
                        },
                    })
                    .collect(),
                material_vector_values: set
                    .vectors
                    .iter()
                    .map(|v| native::RunTimeMaterialVectorValueEntry {
                        value: native::RunTimeMaterialValueBcVec4 {
                            material_value: v.key.0,
                            value: v.value,
                            path_val_graph_index: v.controls.graph.code(),
                            light_group: v.controls.light_group,
                        },
                    })
                    .collect(),
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(native::UberMaterial {
        shader_map: value.shader_map,
        material_value_set_override: value.value_set_override,
        mat: native::Material {
            sort_pri: value.sort_priority,
        },
        mat_tex_list: textures,
        run_time_material_values: values,
        tex_anim_state_count: value.texture_animation_state_count,
        has_graph: value.has_graph,
        ..Default::default()
    })
}
