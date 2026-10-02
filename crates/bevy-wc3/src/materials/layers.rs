use super::textures::Wc3TextureBindings;
use super::Wc3LayerMaterial;
use crate::animation::Wc3Animation;
use bevy::prelude::*;
use wc3::model::animation::Animatable;
use wc3::model::materials::LayerFilterMode;

#[derive(Component)]
pub(crate) struct AnimatedLayer {
    pub(crate) root: Entity,
    pub(crate) alpha: Animatable<f32>,
    pub(crate) geoset_alpha: Option<Animatable<f32>>,
    pub(crate) geoset_color: Option<Animatable<[f32; 3]>>,
    pub(crate) texture_id: Animatable<u32>,
}

pub(crate) fn animate_layers(
    instances: Query<(&Wc3Animation, Option<&Wc3TextureBindings>)>,
    mut layers: Query<(
        &AnimatedLayer,
        &MeshMaterial3d<Wc3LayerMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
) {
    for (layer, material_handle, mut visibility) in &mut layers {
        let Ok((animation, bindings)) = instances.get(layer.root) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let time = animation.time();
        let alpha = layer.alpha.sample(&time).unwrap_or(1.0);
        let geoset_alpha = layer
            .geoset_alpha
            .as_ref()
            .map(|alpha| alpha.sample(&time).unwrap_or(1.0))
            .unwrap_or(1.0);
        *visibility = if alpha * geoset_alpha <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if layer.geoset_alpha.is_some() {
            let partial = (0.0..1.0).contains(&geoset_alpha);
            match material.extension.filter {
                LayerFilterMode::None => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Opaque
                    };
                }
                LayerFilterMode::Transparent => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Mask(if material.extension.hd.maps.x != 0 {
                            0.75
                        } else {
                            0.5
                        })
                    };
                }
                _ => {}
            }
        }
        let texture_id = layer.texture_id.sample(&time).unwrap_or(0);
        let [red, green, blue] = layer
            .geoset_color
            .as_ref()
            .map(|color| color.sample(&time).unwrap_or([1.0; 3]))
            .unwrap_or([1.0; 3]);
        // Tint is a multiplier in the shader, not an sRGB display color.
        material.base.base_color = Color::linear_rgba(red, green, blue, alpha * geoset_alpha);
        if let Some(bindings) = bindings {
            material.base.base_color_texture = bindings.bitmap(texture_id as usize);
        }
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
