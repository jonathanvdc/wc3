//! Owned surface binding evaluation and authored UV transforms.
use super::super::{mesh_animation::MeshAnimation, textures::TextureRole};
use super::{HdAnimation, ModelMaterial, SurfaceAnimation};
use bevy::math::Affine2;
use bevy::prelude::*;
use std::array::from_fn;
use wc3::model::SampledAnimation;
use wc3::model::TextureTransform;

impl HdAnimation {
    fn is_animated(&self) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|value| value.property.is_some())
            || self
                .gain
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self.fresnel.as_ref().is_some_and(|value| {
                value.color.property.is_some()
                    || value.opacity.property.is_some()
                    || value.team_color.property.is_some()
            })
    }

    pub(super) fn apply(
        &self,
        sample: Option<&SampledAnimation<'_, '_>>,
        material: &mut ModelMaterial,
    ) -> bool {
        let handles: [Option<&Handle<Image>>; 6] = from_fn(|slot| {
            let binding = self.slots[slot].as_ref()?;
            let id = sample.map_or(binding.default, |sample| sample.animated(binding))?;
            let role = match slot {
                1 => TextureRole::Normal,
                2 => TextureRole::Data,
                _ => TextureRole::Color,
            };
            self.images.for_role(id, role)
        });
        let mut changed = false;
        changed |= replace_handle(&mut material.base.base_color_texture, handles[0]);
        changed |= replace_handle(&mut material.base.normal_map_texture, handles[1]);
        changed |= replace_handle(&mut material.base.occlusion_texture, handles[2]);
        changed |= replace_handle(&mut material.base.metallic_roughness_texture, handles[2]);
        changed |= replace(
            &mut material.base.metallic,
            if handles[2].is_some() { 1. } else { 0. },
        );
        changed |= replace_handle(&mut material.base.emissive_texture, handles[3]);
        let gain = if handles[3].is_some() {
            self.gain.as_ref().map_or(1., |value| {
                sample.map_or(value.default, |sample| sample.animated(value))
            })
        } else {
            0.
        };
        let tint = material.base.base_color.to_linear();
        changed |= replace(
            &mut material.base.emissive,
            LinearRgba::rgb(gain * tint.red, gain * tint.green, gain * tint.blue),
        );
        changed |= replace_handle(&mut material.extension.orm, handles[2]);
        changed |= replace_handle(&mut material.extension.team_color, handles[4]);
        changed |= replace_handle(&mut material.extension.environment, handles[5]);
        changed |= replace(
            &mut material.extension.hd.maps,
            UVec4::new(
                1,
                handles[2].is_some() as u32,
                handles[4].is_some() as u32,
                handles[5].is_some() as u32,
            ),
        );
        let (color, opacity, team) = self.fresnel.as_ref().map_or(([1.; 3], 0., 0.), |value| {
            (
                sample.map_or(value.color.default, |sample| sample.animated(&value.color)),
                sample.map_or(value.opacity.default, |sample| {
                    sample.animated(&value.opacity)
                }),
                sample.map_or(value.team_color.default, |sample| {
                    sample.animated(&value.team_color)
                }),
            )
        });
        changed |= replace(
            &mut material.extension.hd.fresnel_color,
            Vec3::from_array(color).extend(1.),
        );
        changed |= replace(
            &mut material.extension.hd.fresnel,
            Vec4::new(opacity, team, 0., 0.),
        );
        changed
    }
}

fn replace_handle(target: &mut Option<Handle<Image>>, next: Option<&Handle<Image>>) -> bool {
    if target.as_ref() == next {
        false
    } else {
        *target = next.cloned();
        true
    }
}

fn replace<T: PartialEq>(target: &mut T, next: T) -> bool {
    if *target == next {
        false
    } else {
        *target = next;
        true
    }
}

pub(crate) fn mesh_alpha_mode(authored: AlphaMode, opacity: f32) -> AlphaMode {
    if opacity > 0. && opacity < 1. && matches!(authored, AlphaMode::Opaque | AlphaMode::Mask(_)) {
        AlphaMode::AlphaToCoverage
    } else {
        authored
    }
}

impl SurfaceAnimation {
    pub(crate) fn with_mesh(
        mut self,
        untinted: Color,
        alpha_mode: AlphaMode,
        mesh: Option<&MeshAnimation>,
    ) -> Self {
        self.untinted = Some(untinted.to_linear());
        self.untinted_alpha_mode = Some(alpha_mode);
        self.mesh = mesh.cloned();
        self
    }

    pub(crate) fn is_animated(&self) -> bool {
        self.hd.as_ref().is_some_and(HdAnimation::is_animated)
            || self.mesh.as_ref().is_some_and(MeshAnimation::is_animated)
            || self
                .rgba
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .emission
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .color
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .opacity
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .weight
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .texture
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .emissive_gain
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self.transform.as_ref().is_some_and(|value| {
                value.translation.property.is_some()
                    || value.rotation.property.is_some()
                    || value.scale.property.is_some()
            })
            || self.secondary_transform.as_ref().is_some_and(|value| {
                value.translation.property.is_some()
                    || value.rotation.property.is_some()
                    || value.scale.property.is_some()
            })
    }
    pub(crate) fn apply_to_assets(
        &self,
        sample: &SampledAnimation<'_, '_>,
        handle: &Handle<ModelMaterial>,
        materials: &mut Assets<ModelMaterial>,
    ) -> bool {
        let Some(mut material) = materials.get_mut(handle) else {
            return false;
        };
        let changed = self.apply_to_material(sample, material.bypass_change_detection());
        if changed {
            let _ = material.into_inner();
        }
        changed
    }
    pub(crate) fn apply_to_material(
        &self,
        sample: &SampledAnimation<'_, '_>,
        material: &mut ModelMaterial,
    ) -> bool {
        let mut changed = self.apply(sample, &mut material.base);
        if let Some(transform) = &self.secondary_transform {
            let next = Mat3::from(sampled_transform(sample, transform));
            changed |= replace(&mut material.extension.combiner.uv_transform, next);
        }
        if let Some(hd) = &self.hd {
            changed |= hd.apply(Some(sample), material);
        }
        changed
    }
    /// Updates only changed values and reports whether the material needs a write.
    pub(crate) fn apply(
        &self,
        sample: &SampledAnimation<'_, '_>,
        material: &mut StandardMaterial,
    ) -> bool {
        let mut changed = false;
        if self.color.is_some()
            || self.rgba.is_some()
            || self.opacity.is_some()
            || self.weight.is_some()
            || self.mesh.is_some()
        {
            let rgb = self.color.as_ref().map(|v| sample.animated(v));
            let rgba = self.rgba.as_ref().map(|v| sample.animated(v));
            let current = self
                .untinted
                .unwrap_or_else(|| material.base_color.to_linear());
            let rgb = rgb.or(rgba.map(|v| [v[0], v[1], v[2]])).unwrap_or([
                current.red,
                current.green,
                current.blue,
            ]);
            let alpha = self
                .opacity
                .as_ref()
                .map(|v| sample.animated(v))
                .unwrap_or_else(|| {
                    if self.rgba.is_none() && self.weight.is_none() {
                        current.alpha
                    } else {
                        1.
                    }
                })
                * rgba.map_or(1.0, |v| v[3])
                * self
                    .weight
                    .as_ref()
                    .map(|v| sample.animated(v))
                    .unwrap_or(1.0);
            let tint = self
                .mesh
                .as_ref()
                .map_or([1.; 4], |mesh| mesh.sample(sample));
            let next = Color::linear_rgba(
                rgb[0] * tint[0],
                rgb[1] * tint[1],
                rgb[2] * tint[2],
                alpha * tint[3],
            );
            if material.base_color != next {
                material.base_color = next;
                changed = true;
            }
            if self.mesh.is_some() {
                if let Some(authored) = self.untinted_alpha_mode {
                    let mode = mesh_alpha_mode(authored, tint[3]);
                    if material.alpha_mode != mode {
                        material.alpha_mode = mode;
                        changed = true;
                    }
                }
            }
        }
        if let Some(texture) = &self.texture {
            let selected = sample.animated(texture);
            let next = selected.and_then(|id| self.images.get(&id));
            if material.base_color_texture.as_ref() != next {
                material.base_color_texture = next.cloned();
                changed = true;
            }
        }
        if let Some(transform) = &self.transform {
            let next = if self.centered_transform {
                layered_uv(
                    sample.animated(&transform.translation),
                    sample.animated(&transform.rotation).0,
                    sample.animated(&transform.scale),
                )
            } else {
                sampled_transform(sample, transform)
            };
            if material.uv_transform != next {
                material.uv_transform = next;
                changed = true;
            }
        }
        if let Some(gain) = self.emissive_gain.as_ref().filter(|_| self.hd.is_none()) {
            let gain = sample.animated(gain);
            let next = LinearRgba::rgb(gain, gain, gain);
            if material.emissive != next {
                material.emissive = next;
                changed = true;
            }
            if material.emissive_texture != material.base_color_texture {
                material.emissive_texture = material.base_color_texture.clone();
                changed = true;
            }
        }
        if let Some(emission) = &self.emission {
            let value = sample.animated(emission);
            let next = LinearRgba::rgb(
                value[0] * self.emission_scale,
                value[1] * self.emission_scale,
                value[2] * self.emission_scale,
            );
            if material.emissive != next {
                material.emissive = next;
                changed = true;
            }
        }
        changed
    }
}

fn sampled_transform(sample: &SampledAnimation<'_, '_>, transform: &TextureTransform) -> Affine2 {
    let translation = sample.animated(&transform.translation);
    let scale = sample.animated(&transform.scale);
    let rotation = Quat::from_array(sample.animated(&transform.rotation).0);
    let (_, _, z) = rotation.to_euler(EulerRot::XYZ);
    Affine2::from_scale_angle_translation(
        Vec2::new(scale[0], scale[1]),
        z,
        Vec2::new(translation[0], translation[1]),
    )
}

pub(super) fn layered_uv(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Affine2 {
    let center = Vec3::new(0.5, 0.5, 0.);
    let matrix = Mat4::from_translation(Vec3::from_array(translation) + center)
        * Mat4::from_scale_rotation_translation(
            Vec3::from_array(scale),
            Quat::from_array(rotation).normalize(),
            Vec3::ZERO,
        )
        * Mat4::from_translation(-center);
    Affine2::from_cols(
        matrix.x_axis.truncate().truncate(),
        matrix.y_axis.truncate().truncate(),
        matrix.w_axis.truncate().truncate(),
    )
}
