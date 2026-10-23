//! Owned surface binding evaluation and authored UV transforms.
use super::super::{mesh_animation::MeshAnimation, textures::TextureRole};
use super::wmo::scrolling_transform;
use super::{HdAnimation, HdProgramme, ModelMaterial, SurfaceAnimation, UvConvention};
use crate::animation_targets::AnimationTargets;
use bevy::math::Affine2;
use bevy::math::{Vec3, Vec4};
use bevy::prelude::*;
use std::array::from_fn;
use wc3::model::{Property, PropertyValue, SampledAnimation, ValueSample};
use wc3::model::{SurfaceColor, TextureTransform};

impl HdAnimation {
    fn is_animated(&self, targets: &AnimationTargets) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|slot| targets.contains(slot))
            || self.gain.as_ref().is_some_and(|slot| targets.scalar(slot))
            || self.fresnel.as_ref().is_some_and(|f| {
                targets.contains(&f.color)
                    || targets.scalar(&f.opacity)
                    || targets.scalar(&f.team_color)
            })
    }

    pub(super) fn apply(&self, sample: &impl ValueSample, material: &mut ModelMaterial) -> bool {
        let handles: [Option<&Handle<Image>>; 6] = from_fn(|slot| {
            let binding = self.slots[slot].as_ref()?;
            let id = sample.animated(binding)?;
            let role = match slot {
                1 => TextureRole::Normal,
                2 => TextureRole::Data,
                _ => TextureRole::Color,
            };
            self.images.for_slot(slot as u32, id, role)
        });
        let mut changed = false;
        changed |= replace_handle(&mut material.base.base_color_texture, handles[0]);
        changed |= replace_handle(&mut material.base.normal_map_texture, handles[1]);
        changed |= replace_handle(
            &mut material.base.occlusion_texture,
            if self.programme == HdProgramme::Crystal || self.authored_ambient_occlusion {
                None
            } else {
                handles[2]
            },
        );
        changed |= replace_handle(&mut material.base.metallic_roughness_texture, handles[2]);
        changed |= replace(
            &mut material.base.metallic,
            if handles[2].is_some() { 1. } else { 0. },
        );
        changed |= replace_handle(&mut material.base.emissive_texture, handles[3]);
        let gain = if handles[3].is_some() {
            self.gain
                .as_ref()
                .map_or(1., |value| sample.animated(value))
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
                self.programme as u32,
                handles[2].is_some() as u32,
                handles[4].is_some() as u32,
                handles[5].is_some() as u32,
            ),
        );
        let (color, opacity, team) = self.fresnel.as_ref().map_or(([1.; 3], 0., 0.), |value| {
            (
                (sample.animated(&value.color)).to_array(),
                sample.animated(&value.opacity),
                sample.animated(&value.team_color),
            )
        });
        changed |= replace(
            &mut material.extension.hd.fresnel_color,
            Vec3::from_array(color).extend(1.),
        );
        changed |= replace(
            &mut material.extension.hd.fresnel,
            // z is the checked raw normal-map presence flag for Crystal's
            // shared alpha/normal calculation in depth-only passes.
            Vec4::new(
                opacity,
                team,
                if self.programme == HdProgramme::Crystal && handles[1].is_some() {
                    1.
                } else {
                    0.
                },
                if self.authored_ambient_occlusion {
                    1.
                } else {
                    0.
                },
            ),
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
    fn sampled_transform(
        &self,
        sample: &SampledAnimation<'_, '_>,
        transform: &TextureTransform,
        primary: bool,
    ) -> Affine2 {
        match self.uv_convention {
            UvConvention::M2 => transform.m2_affine(sample),
            UvConvention::Layered if primary => transform.affine(sample, true),
            _ => transform.affine(sample, false),
        }
    }

    /// Records whether the retained slots require instance-owned material updates.
    pub(super) fn prune_static_bindings(&mut self, targets: &AnimationTargets) {
        fn contains<T: PropertyValue>(
            slot: &Option<Property<T>>,
            targets: &AnimationTargets,
        ) -> bool {
            slot.as_ref().is_some_and(|slot| targets.contains(slot))
        }
        fn scalar(slot: &Option<Property<f32>>, targets: &AnimationTargets) -> bool {
            slot.as_ref().is_some_and(|slot| targets.scalar(slot))
        }
        self.animated = contains(&self.color, targets)
            || contains(&self.rgba, targets)
            || contains(&self.texture, targets)
            || contains(&self.emission, targets)
            || scalar(&self.opacity, targets)
            || scalar(&self.weight, targets)
            || scalar(&self.emissive_gain, targets)
            || self.pass_weights.iter().any(|slot| targets.scalar(slot))
            || self
                .input_weights
                .iter()
                .flatten()
                .any(|slot| targets.scalar(slot))
            || [
                &self.transform,
                &self.secondary_transform,
                &self.third_transform,
                &self.fourth_transform,
            ]
            .into_iter()
            .flatten()
            .any(|t| {
                targets.contains(&t.translation)
                    || targets.contains(&t.rotation)
                    || targets.contains(&t.scale)
            })
            || self.hd.as_ref().is_some_and(|hd| hd.is_animated(targets));
    }

    /// Procedural material motion needs clock updates even without any channels.
    pub(crate) fn uses_shared_time(&self) -> bool {
        self.uv_scroll.iter().any(|v| *v != Vec2::ZERO) || self.third_uv_scroll != Vec2::ZERO
    }

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
        self.animated
            || self.uses_shared_time()
            || self.mesh.as_ref().is_some_and(MeshAnimation::is_animated)
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
        let (mut changed, mesh_opacity) = self.apply_base(sample, &mut material.base);
        if let Some(opacity) = material.extension.m2_opacity {
            changed |= opacity.apply(&mut material.base, &mut material.extension);
        } else if let Some(authored) = self.untinted_alpha_mode.filter(|_| self.mesh.is_some()) {
            changed |= replace(
                &mut material.base.alpha_mode,
                mesh_alpha_mode(authored, mesh_opacity),
            );
        }
        if self.secondary_transform.is_some() || self.uv_scroll[1] != Vec2::ZERO {
            let base = self
                .secondary_transform
                .as_ref()
                .map_or(Affine2::IDENTITY, |t| {
                    self.sampled_transform(sample, t, false)
                });
            let next = Mat3::from(scrolling_transform(
                base,
                self.uv_scroll[1],
                sample.shared_time().as_seconds() as f32,
            ));
            changed |= replace(&mut material.extension.combiner.uv_transform, next);
        }
        if self.third_transform.is_some() || self.third_uv_scroll != Vec2::ZERO {
            let base = self
                .third_transform
                .as_ref()
                .map_or(Affine2::IDENTITY, |transform| {
                    self.sampled_transform(sample, transform, false)
                });
            changed |= replace(
                &mut material.extension.combiner.third_transform,
                Mat3::from(scrolling_transform(
                    base,
                    self.third_uv_scroll,
                    sample.shared_time().as_seconds() as f32,
                )),
            );
        }
        if let Some(transform) = &self.fourth_transform {
            changed |= replace(
                &mut material.extension.combiner.fourth_transform,
                Mat3::from(self.sampled_transform(sample, transform, false)),
            );
        }
        for (slot, weight) in self.input_weights.iter().enumerate() {
            if let Some(weight) = weight {
                changed |= replace(
                    &mut material.extension.combiner.weights[slot],
                    sample.animated(weight),
                );
            }
        }

        if let Some(hd) = &self.hd {
            changed |= hd.apply(sample, material);
        }
        changed
    }
    /// Updates only changed values and reports whether the material needs a write.
    fn apply_base(
        &self,
        sample: &SampledAnimation<'_, '_>,
        material: &mut StandardMaterial,
    ) -> (bool, f32) {
        let mut changed = false;
        let mut mesh_opacity = 1.;
        if self.color.is_some()
            || self.rgba.is_some()
            || self.opacity.is_some()
            || self.weight.is_some()
            || !self.pass_weights.is_empty()
            || self.mesh.is_some()
        {
            let current = self
                .untinted
                .unwrap_or_else(|| material.base_color.to_linear());
            let tint = self
                .mesh
                .as_ref()
                .map_or([1.; 4], |mesh| (mesh.sample(sample)).to_array());
            mesh_opacity = tint[3];
            let rgba = SurfaceColor {
                color: self.color.as_ref(),
                rgba: self.rgba.as_ref(),
                opacity: self.opacity.as_ref(),
                weight: self.weight.as_ref(),
                pass_weights: &self.pass_weights,
                fallback: Vec4::from_array(current.to_f32_array()),
                unweighted_opacity: self.unweighted_opacity,
            }
            .sample(sample, Vec4::from_array(tint));
            let next = Color::linear_rgba(rgba[0], rgba[1], rgba[2], rgba[3]);
            if material.base_color != next {
                material.base_color = next;
                changed = true;
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
        if self.transform.is_some() || self.uv_scroll[0] != Vec2::ZERO {
            let next = self
                .transform
                .as_ref()
                .map_or(Affine2::IDENTITY, |transform| {
                    self.sampled_transform(sample, transform, true)
                });
            let next = scrolling_transform(
                next,
                self.uv_scroll[0],
                sample.shared_time().as_seconds() as f32,
            );
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
        (changed, mesh_opacity)
    }
}

pub(super) fn layered_uv(translation: Vec3, rotation: Quat, scale: Vec3) -> Affine2 {
    TextureTransform::centered_affine(translation, rotation, scale)
}
