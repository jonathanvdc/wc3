use crate::materials::{Wc3LayerMaterial, Wc3LayerState};
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use wc3::model::animation::Animatable;
use wc3::model::materials::{Layer, LayerFilterMode, ShaderType};
use wc3::model::V1800;

pub(super) fn layer_texture_id(layer: &Layer<V1800>) -> Animatable<u32> {
    layer
        .texture_slots()
        .iter()
        .find(|slot| slot.texture_type == 0)
        .map(|slot| slot.texture_id.clone())
        .unwrap_or_else(|| layer.texture_id.clone())
}

fn layer_alpha_mode(mode: LayerFilterMode) -> AlphaMode {
    match mode {
        LayerFilterMode::None => AlphaMode::Opaque,
        LayerFilterMode::Transparent => AlphaMode::Mask(0.5),
        LayerFilterMode::Blend => AlphaMode::Blend,
        // Bevy's Add shader path clears alpha. WC3's custom blend state needs
        // the sampled alpha as its source factor, so retain it with Blend.
        LayerFilterMode::Additive | LayerFilterMode::AddAlpha => AlphaMode::Blend,
        LayerFilterMode::Modulate | LayerFilterMode::Modulate2x => AlphaMode::Multiply,
        LayerFilterMode::Unknown(_) => AlphaMode::Blend,
    }
}

pub(super) fn build_layer_material(
    layer: &Layer<V1800>,
    texture_id: &Animatable<u32>,
    textures: &[Option<Handle<Image>>],
) -> Wc3LayerMaterial {
    let alpha = layer.alpha.value().copied().unwrap_or(1.0);
    let texture = texture_id
        .value()
        .and_then(|id| textures.get(*id as usize))
        .cloned()
        .flatten();
    Wc3LayerMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, alpha),
            base_color_texture: texture,
            alpha_mode: if layer.shader_type() == ShaderType::HD_DEFAULT_UNIT
                && layer.filter_mode == LayerFilterMode::Transparent
            {
                AlphaMode::Mask(0.75)
            } else {
                layer_alpha_mode(layer.filter_mode)
            },
            opaque_render_method: OpaqueRendererMethod::Forward,
            unlit: layer.shading_flags.unshaded(),
            fog_enabled: !layer.shading_flags.unfogged(),
            double_sided: layer.shading_flags.two_sided(),
            cull_mode: if layer.shading_flags.two_sided() {
                None
            } else {
                Some(Face::Back)
            },
            ..default()
        },
        extension: Wc3LayerState {
            filter: layer.filter_mode,
            no_depth_test: layer.shading_flags.no_depth_test(),
            no_depth_set: layer.shading_flags.no_depth_set(),
            ..default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_layers_preserve_shader_alpha_for_custom_blending() {
        assert_eq!(
            layer_alpha_mode(LayerFilterMode::Additive),
            AlphaMode::Blend
        );
        assert_eq!(
            layer_alpha_mode(LayerFilterMode::AddAlpha),
            AlphaMode::Blend
        );
    }
}
