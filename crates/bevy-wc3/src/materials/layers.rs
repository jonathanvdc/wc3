use super::runtime::{apply_material, ConstantMaterial};
use super::textures::Wc3TextureBindings;
use super::Wc3LayerMaterial;
use crate::animation::{clocks::SamplingTime, Wc3Animation};
use crate::preparation::shared::Shared;
use bevy::prelude::*;
use wc3::model::animation::Animatable;
use wc3::model::materials::LayerFilterMode;

#[derive(Component)]
pub(crate) struct AnimatedLayer {
    pub(crate) root: Entity,
    pub(crate) alpha: Shared<Animatable<'static, f32>>,
    pub(crate) geoset_alpha: Option<Shared<Animatable<'static, f32>>>,
    pub(crate) geoset_color: Option<Shared<Animatable<'static, [f32; 3]>>>,
    pub(crate) texture_id: Shared<Animatable<'static, u32>>,
}

pub(crate) fn animate_layers(
    instances: Query<(&Wc3Animation, Option<Ref<Wc3TextureBindings>>)>,
    mut layers: Query<(
        &AnimatedLayer,
        &mut MeshMaterial3d<Wc3LayerMaterial>,
        Option<&ConstantMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
) {
    for (layer, mut material_handle, constant, mut visibility) in &mut layers {
        let Ok((animation, bindings)) = instances.get(layer.root) else {
            continue;
        };
        if constant.is_some()
            && bindings
                .as_ref()
                .is_none_or(|bindings| !bindings.is_changed())
            && !material_handle.is_changed()
        {
            continue;
        }
        let Some(mut material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        let next_visibility = if layer.evaluate(
            &mut material,
            bindings.as_deref(),
            SamplingTime::live(animation.time()),
        ) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(next_visibility);
        let previous = material_handle.0.id();
        apply_material(
            &mut materials,
            material_handle.bypass_change_detection(),
            material,
            constant.is_some(),
        );
        if material_handle.0.id() != previous {
            material_handle.set_changed();
        }
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
