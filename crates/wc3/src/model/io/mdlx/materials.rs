//! Native material decoding and shader interpretation into typed semantic steps.
use super::animation::Decoder;
use super::storage::Storage;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::materials::{Layer, LayerFresnel};
use crate::model::formats::mdlx::{self as native, Animatable, FormatVersion};
use crate::model::materials::bindings::{Coordinates, TextureBinding};
use crate::model::materials::mdlx::{AlphaAdditiveEncoding, LayerOptions};
use crate::model::materials::state::{DepthState, Lighting, Operation, PassState};
use crate::model::profiles::mdlx::{
    Blend, Fresnel, Material, MaterialDeclaration, MaterialInputs, MaterialStep, ReforgedInputs,
    Shader, Shading, TextureSlot, UnresolvedInputs,
};
use crate::model::{Construct, PropertyValues};
use crate::model::{Quat, Texture, TextureTransform};
use glam::Vec3;

pub(super) fn read<'source, 'a, V: FormatVersion, S: Storage<'source, 'a>>(
    source: &'source native::Material<'_, V>,
    textures: &[Texture],
    transforms: &[&native::TextureAnimation<'_>],
    values: &mut PropertyValues,
) -> Result<Material<'a>, Error> {
    let texture = |index: u32| {
        if index == u32::MAX {
            return Ok(None);
        }
        textures
            .get(index as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| {
                Error::new(
                    "material.texture",
                    "texture selector is outside the model texture table",
                )
            })
    };
    let flags = source.render_mode;
    let mut unresolved = flags;
    unresolved.set_constant_color(false);
    unresolved.set_two_sided(false);
    unresolved.set_sort_primitives_far_z(false);
    unresolved.set_sort_primitives_near_z(false);
    unresolved.set_full_resolution(false);
    let declaration = MaterialDeclaration {
        priority_plane: source.priority_plane,
        constant_color: flags.constant_color(),
        two_sided: flags.two_sided(),
        sort_near_z: flags.sort_primitives_near_z(),
        sort_far_z: flags.sort_primitives_far_z(),
        full_resolution: flags.full_resolution(),
        unclassified: unresolved.bits().into(),
    };
    let mut steps = Vec::with_capacity(source.layers.len());
    for layer in &source.layers {
        let flags = layer.shading_flags;
        let mut unresolved = flags;
        unresolved.set_unshaded(false);
        unresolved.set_sphere_env_map(false);
        unresolved.set_two_sided(false);
        unresolved.set_unfogged(false);
        unresolved.set_no_depth_test(false);
        unresolved.set_no_depth_set(false);
        let fresnel = layer.try_fresnel().ok().map(|value| Fresnel {
            color: values.add(Vec3::from_array(
                value.color.value().copied().unwrap_or_default(),
            )),
            opacity: values.add(value.opacity.value().copied().unwrap_or_default()),
            team_color: values.add(value.team_color.value().copied().unwrap_or_default()),
        });
        let mut slots = Vec::new();
        for slot in layer.try_texture_slots().unwrap_or_default() {
            slots.push(TextureSlot {
                slot: slot.texture_type,
                binding: TextureBinding {
                    image: values.add(texture(
                        slot.texture_id.value().copied().unwrap_or_default(),
                    )?),
                    ..Construct::construct(values)
                },
            });
        }
        let mut inputs = MaterialInputs {
            binding: TextureBinding {
                image: values.add(texture(
                    layer.texture_id.value().copied().unwrap_or_default(),
                )?),
                coordinates: Coordinates {
                    source: layer.coordinate_id,
                    transform: if layer.texture_animation_id == u32::MAX {
                        Construct::construct(values)
                    } else {
                        transforms
                            .get(layer.texture_animation_id as usize)
                            .ok_or_else(|| {
                                Error::new("material.texture_transform", "transform does not exist")
                            })?;
                        Construct::construct(values)
                    },
                },
                ..Construct::construct(values)
            },
            opacity: values.add(layer.alpha.value().copied().unwrap_or_default()),
            emissive_gain: layer
                .try_emissive_gain()
                .ok()
                .map(|value| values.add(emissive_default(value))),
        };
        let state = PassState {
            blend: match layer.filter_mode {
                native::LayerFilterMode::None => Blend::Opaque,
                native::LayerFilterMode::Transparent => Blend::AlphaKey(()),
                native::LayerFilterMode::Blend => Blend::Alpha,
                native::LayerFilterMode::Additive => {
                    Blend::AdditiveAlpha(AlphaAdditiveEncoding::Additive)
                }
                native::LayerFilterMode::AddAlpha => {
                    Blend::AdditiveAlpha(AlphaAdditiveEncoding::AddAlpha)
                }
                native::LayerFilterMode::Modulate => Blend::Modulate,
                native::LayerFilterMode::Modulate2x => Blend::Modulate2x,
                native::LayerFilterMode::Unknown(value) => Blend::Unknown(value),
            },
            rasterization: Shading {
                depth: DepthState {
                    test: Operation::bypassed(flags.no_depth_test()),
                    write: Operation::bypassed(flags.no_depth_set()),
                    bias: (),
                },
                faces: flags.two_sided(),
                lighting: Lighting {
                    unlit: flags.unshaded(),
                    unfogged: flags.unfogged(),
                },
                controls: LayerOptions {
                    sphere_environment: flags.sphere_env_map(),
                    unclassified: unresolved.bits().into(),
                },
            },
            ordering: (),
        };
        let shader_name = S::text(source.try_shader().unwrap_or_default());
        let selector = layer.try_shader_type().ok().map(|value| value.id());
        let family = shader_family(selector, &shader_name);
        let shader = match family {
            ShaderFamily::Classic if classic_texture(layer).is_some() => {
                let primary = classic_texture(layer).expect("checked classic representation");
                inputs.binding.image =
                    values.add(texture(primary.value().copied().unwrap_or_default())?);
                Shader::Classic(inputs)
            }
            ShaderFamily::ReforgedDefault => Shader::ReforgedDefault(ReforgedInputs {
                inputs,
                textures: slots,
                fresnel,
            }),
            _ => Shader::Unresolved(UnresolvedInputs {
                shader_name,
                selector,
                inputs,
                textures: slots,
                fresnel,
            }),
        };
        steps.push(MaterialStep {
            state,
            declaration: declaration.clone(),
            shader,
        });
    }
    Ok(Material { steps })
}

pub(super) fn bind<'source, 'a, V: FormatVersion, S: Storage<'source, 'a>>(
    source: &'source native::Material<'_, V>,
    textures: &[Texture],
    transforms: &[&native::TextureAnimation<'_>],
    decoder: &mut Decoder<'a, '_>,
) -> Result<Material<'a>, Error> {
    let mut material = read::<_, S>(source, textures, transforms, decoder.builder.values_mut())?;

    for (source, layer) in source.layers.iter().zip(&mut material.steps) {
        let classic = matches!(layer.shader, Shader::Classic(_));
        let inputs = layer.shader.inputs_mut();
        if source.texture_animation_id != u32::MAX {
            let transform = &transforms[source.texture_animation_id as usize];
            inputs.binding.coordinates.transform = TextureTransform {
                translation: decoder.track(
                    transform.translation.as_ref(),
                    Vec3::ZERO,
                    "layer.translation",
                    |value| value.into(),
                )?,
                rotation: decoder.track(
                    transform.rotation.as_ref(),
                    Quat::IDENTITY,
                    "layer.rotation",
                    Quat::from_array,
                )?,
                scale: decoder.track(
                    transform.scaling.as_ref(),
                    Vec3::ONE,
                    "layer.scale",
                    |value| value.into(),
                )?,
            };
        }
        let primary = if classic {
            classic_texture(source).expect("normalized classic representation")
        } else {
            &source.texture_id
        };
        let map = texture_map(textures, primary)?;
        inputs.binding.image = decoder.bind(primary, "layer.texture", map)?;
        inputs.opacity = decoder.bind(&source.alpha, "layer.opacity", |value| value)?;
        if let Ok(source) = source.try_emissive_gain() {
            inputs.emissive_gain = Some(decoder.track(
                source.track(),
                emissive_default(source),
                "layer.emissive_gain",
                |value| value,
            )?);
        }
        if let (Ok(source), Some(target)) = (source.try_fresnel(), layer.shader.fresnel_mut()) {
            *target = Fresnel {
                color: decoder.bind(&source.color, "layer.fresnel_color", |value| value.into())?,
                opacity: decoder.bind(&source.opacity, "layer.fresnel_opacity", |value| value)?,
                team_color: decoder.bind(
                    &source.team_color,
                    "layer.fresnel_team_color",
                    |value| value,
                )?,
            };
        }
        for (source, slot) in source
            .try_texture_slots()
            .unwrap_or_default()
            .iter()
            .zip(layer.shader.textures_mut())
        {
            let map = texture_map(textures, &source.texture_id)?;
            slot.binding.image = decoder.bind(&source.texture_id, "layer.texture_slot", map)?;
        }
    }
    Ok(material)
}

/// Normalize only inert version storage and the canonical diffuse-slot layout.
/// Native version conversion uses legacy Static(0) as its placeholder after
/// moving the complete texture selection and animation into slot zero.
fn classic_texture<'layer, 'data, V: FormatVersion>(
    layer: &'layer Layer<'data, V>,
) -> Option<&'layer Animatable<'data, u32>> {
    if layer
        .try_fresnel()
        .is_ok_and(|fresnel| *fresnel != LayerFresnel::default())
    {
        return None;
    }
    match layer.try_texture_slots().unwrap_or_default() {
        [] => Some(&layer.texture_id),
        [slot]
            if slot.texture_type == 0
                && (layer.texture_id == Animatable::Static(0)
                    || layer.texture_id == slot.texture_id) =>
        {
            Some(&slot.texture_id)
        }
        _ => None,
    }
}

fn texture_map<'a>(
    textures: &[Texture],
    source: &native::Animatable<'_, u32>,
) -> Result<impl Fn(u32) -> Option<Texture> + Clone + Send + Sync + 'a, Error> {
    let mut selectors = vec![source.value().copied().unwrap_or_default()];
    if let Some(track) = source.track() {
        if let Some(keys) = track.step_keys().or_else(|| track.linear_keys()) {
            selectors.extend(keys.iter().map(|key| key.value));
        } else if let Some(keys) = track.hermite_keys().or_else(|| track.bezier_keys()) {
            selectors.extend(
                keys.iter()
                    .flat_map(|key| [key.value, key.in_tangent, key.out_tangent]),
            );
        }
    }
    if selectors
        .iter()
        .any(|selector| *selector != u32::MAX && *selector as usize >= textures.len())
    {
        return Err(Error::new(
            "material.texture",
            "animated selector references an absent texture",
        ));
    }
    let textures = textures.to_vec();
    Ok(move |selector| textures.get(selector as usize).copied())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShaderFamily {
    Classic,
    ReforgedDefault,
    Unresolved,
}
fn shader_family(selector: Option<u32>, name: &str) -> ShaderFamily {
    match selector {
        Some(1) => ShaderFamily::ReforgedDefault,
        Some(0 | 2) => ShaderFamily::Classic,
        Some(_) => ShaderFamily::Unresolved,
        None => match name {
            "Shader_HD_DefaultUnit" => ShaderFamily::ReforgedDefault,
            "" | "Shader_SD_FixedFunction" => ShaderFamily::Classic,
            _ => ShaderFamily::Unresolved,
        },
    }
}

/// Native emissive headers use NaN for an unspecified unity multiplier. This
/// applies only to the stored header; authored animation keys stay unchanged.
fn emissive_default(source: &Animatable<'_, f32>) -> f32 {
    match source.value().copied() {
        Some(value) if !value.is_nan() => value,
        _ => 1.,
    }
}

#[cfg(test)]
mod tests {
    use super::{shader_family, ShaderFamily};

    #[test]
    fn numeric_selection_takes_precedence_over_names() {
        for name in [
            "",
            "Shader_SD_FixedFunction",
            "Shader_HD_DefaultUnit",
            "Shader_HD_Crystal",
        ] {
            for (selector, family) in [
                (0, ShaderFamily::Classic),
                (2, ShaderFamily::Classic),
                (1, ShaderFamily::ReforgedDefault),
                (99, ShaderFamily::Unresolved),
            ] {
                assert_eq!(shader_family(Some(selector), name), family);
            }
        }
        for (name, family) in [
            ("", ShaderFamily::Classic),
            ("Shader_SD_FixedFunction", ShaderFamily::Classic),
            ("Shader_HD_DefaultUnit", ShaderFamily::ReforgedDefault),
            ("Shader_HD_Crystal", ShaderFamily::Unresolved),
        ] {
            assert_eq!(shader_family(None, name), family);
        }
    }
}
