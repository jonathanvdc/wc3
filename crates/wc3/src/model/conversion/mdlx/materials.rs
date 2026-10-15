//! Direct native layered-material decoding into logical surface fields.
use super::animation::Decoder;
use crate::model::conversion::Error;
use crate::model::mdlx::{self as native, ModelVersion};
use crate::model::semantic::materials::bindings::{Coordinates, TextureBinding};
use crate::model::semantic::materials::layered::LayerOptions;
use crate::model::semantic::materials::state::{DepthState, Lighting, Operation, PassState};
use crate::model::semantic::profiles::mdlx as profile;
use crate::model::semantic::profiles::mdlx::{
    Blend, Fresnel, Layer, Material, Shading, TextureSlot,
};
use crate::model::semantic::{Animated, MaterialId, TextureId, TextureTransformId};

pub(super) fn read<'a, V: ModelVersion>(
    source: &'a native::Material<'_, V>,
    textures: &[TextureId],
    transforms: &[TextureTransformId],
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
    let mut layers = Vec::with_capacity(source.layers.len());
    for layer in &source.layers {
        let flags = layer.shading_flags;
        let mut unresolved = flags;
        unresolved.set_unshaded(false);
        unresolved.set_sphere_env_map(false);
        unresolved.set_wrap_width(false);
        unresolved.set_wrap_height(false);
        unresolved.set_two_sided(false);
        unresolved.set_unfogged(false);
        unresolved.set_no_depth_test(false);
        unresolved.set_no_depth_set(false);
        let fresnel = layer.try_fresnel().ok().map(|value| Fresnel {
            color: Animated::constant(value.color.value().copied().unwrap_or_default()),
            opacity: Animated::constant(value.opacity.value().copied().unwrap_or_default()),
            team_color: Animated::constant(value.team_color.value().copied().unwrap_or_default()),
        });
        let mut slots = Vec::new();
        for slot in layer.try_texture_slots().unwrap_or_default() {
            slots.push(TextureSlot {
                slot: slot.texture_type,
                binding: TextureBinding {
                    image: Animated::constant(texture(
                        slot.texture_id.value().copied().unwrap_or_default(),
                    )?),
                    ..Default::default()
                },
            });
        }
        layers.push(Layer {
            binding: TextureBinding {
                image: Animated::constant(texture(
                    layer.texture_id.value().copied().unwrap_or_default(),
                )?),
                coordinates: Coordinates {
                    source: layer.coordinate_id,
                    transform: if layer.texture_animation_id == u32::MAX {
                        None
                    } else {
                        Some(
                            *transforms
                                .get(layer.texture_animation_id as usize)
                                .ok_or_else(|| {
                                    Error::new(
                                        "material.texture_transform",
                                        "transform does not exist",
                                    )
                                })?,
                        )
                    },
                },
                ..Default::default()
            },
            opacity: Animated::constant(layer.alpha.value().copied().unwrap_or_default()),
            emissive_gain: layer
                .try_emissive_gain()
                .ok()
                .map(|value| Animated::constant(value.value().copied().unwrap_or_default())),
            fresnel: fresnel,
            shader: layer.try_shader_type().ok().map(|value| value.id()),
            textures: slots,
            state: PassState {
                blend: match layer.filter_mode {
                    native::LayerFilterMode::None => Blend::Opaque,
                    native::LayerFilterMode::Transparent => Blend::AlphaKey(()),
                    native::LayerFilterMode::Blend => Blend::Alpha,
                    native::LayerFilterMode::Additive => Blend::Additive,
                    native::LayerFilterMode::AddAlpha => Blend::AdditiveAlpha(()),
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
                        wrap_u: flags.wrap_width(),
                        wrap_v: flags.wrap_height(),
                        unclassified: unresolved.bits().into(),
                    },
                },
                ordering: (),
            },
        });
    }
    let flags = source.render_mode;
    let mut unresolved = flags;
    unresolved.set_constant_color(false);
    unresolved.set_two_sided(false);
    unresolved.set_sort_primitives_far_z(false);
    unresolved.set_sort_primitives_near_z(false);
    unresolved.set_full_resolution(false);
    Ok(Material {
        priority_plane: source.priority_plane,
        shader: source.try_shader().unwrap_or_default(),
        constant_color: flags.constant_color(),
        two_sided: flags.two_sided(),
        sort_near_z: flags.sort_primitives_near_z(),
        sort_far_z: flags.sort_primitives_far_z(),
        full_resolution: flags.full_resolution(),
        unclassified: unresolved.bits().into(),
        layers,
    })
}

pub(super) fn bind<'a, V: ModelVersion>(
    source: &'a native::Material<'_, V>,
    textures: &[TextureId],
    transforms: &[TextureTransformId],
    id: MaterialId,
    decoder: &mut Decoder<'a, '_>,
) -> Result<Material<'a>, Error> {
    let mut material = read(source, textures, transforms)?;
    let owner = id;
    for (index, (source, layer)) in source.layers.iter().zip(&mut material.layers).enumerate() {
        let layer_key = profile::properties::Layer(index);
        let map = texture_map(textures, &source.texture_id)?;
        layer.binding.image = decoder.bind(&source.texture_id, owner, layer_key.texture(), map)?;
        layer.opacity = decoder.bind(&source.alpha, owner, layer_key.opacity(), |value| value)?;
        if let Ok(source) = source.try_emissive_gain() {
            layer.emissive_gain =
                Some(decoder.bind(source, owner, layer_key.emissive_gain(), |value| value)?);
        }
        if let Ok(source) = source.try_fresnel() {
            layer.fresnel = Some(Fresnel {
                color: decoder.bind(&source.color, owner, layer_key.fresnel_color(), |value| {
                    value
                })?,
                opacity: decoder.bind(
                    &source.opacity,
                    owner,
                    layer_key.fresnel_opacity(),
                    |value| value,
                )?,
                team_color: decoder.bind(
                    &source.team_color,
                    owner,
                    layer_key.fresnel_team_color(),
                    |value| value,
                )?,
            });
        }
        for (slot_index, (source, slot)) in source
            .try_texture_slots()
            .unwrap_or_default()
            .iter()
            .zip(&mut layer.textures)
            .enumerate()
        {
            let map = texture_map(textures, &source.texture_id)?;
            slot.binding.image = decoder.bind(
                &source.texture_id,
                owner,
                layer_key.texture_slot(slot_index),
                map,
            )?;
        }
    }
    Ok(material)
}

fn texture_map<'a>(
    textures: &[TextureId],
    source: &native::Animatable<'_, u32>,
) -> Result<impl Fn(u32) -> Option<TextureId> + Clone + Send + Sync + 'a, Error> {
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
