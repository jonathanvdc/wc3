use super::textures::Wc3TextureBindings;
use super::Wc3LayerMaterial;
use crate::animation::{clocks::SamplingTime, Wc3Animation};
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
        *visibility = if layer.evaluate(
            &mut material,
            bindings,
            SamplingTime::live(animation.time()),
        ) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

impl AnimatedLayer {
    pub(crate) fn evaluate(
        &self,
        material: &mut Wc3LayerMaterial,
        bindings: Option<&Wc3TextureBindings>,
        time: SamplingTime<'_>,
    ) -> bool {
        let alpha = time.sample(&self.alpha).unwrap_or(1.0);
        let geoset_alpha = self
            .geoset_alpha
            .as_ref()
            .map(|alpha| time.sample(alpha).unwrap_or(1.0))
            .unwrap_or(1.0);
        let visible = alpha * geoset_alpha > 0.0;
        if self.geoset_alpha.is_some() {
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
        let texture_id = time.sample(&self.texture_id).unwrap_or(0);
        let [red, green, blue] = self
            .geoset_color
            .as_ref()
            .map(|color| time.sample(color).unwrap_or([1.0; 3]))
            .unwrap_or([1.0; 3]);
        // Tint is a multiplier in the shader, not an sRGB display color.
        material.base.base_color = Color::linear_rgba(red, green, blue, alpha * geoset_alpha);
        if let Some(bindings) = bindings {
            material.base.base_color_texture = bindings.bitmap(texture_id as usize);
        }
        visible
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
