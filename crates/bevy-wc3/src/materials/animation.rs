//! Animated surface inputs shared by SD and Reforged mesh materials.
use super::runtime::{apply_material, ConstantMaterial};
use super::textures::LinearImages;
use crate::preparation::shared::Shared;
use bevy::asset::AssetEvent;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::array::from_fn;
use wc3::model::animation::{Animatable, Interpolate, TextureAnimation, TrackValue};
use wc3::model::materials::{Layer, LayerFresnel, ShaderType};
use wc3::model::V1800;

use crate::animation::{clocks::SamplingTime, Wc3Animation};
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;

#[derive(Component, Clone)]
pub(crate) struct AnimatedSurface {
    pub(crate) root: Entity,
    pub(crate) slots: [Option<Shared<Animatable<'static, u32>>>; 6],
    pub(crate) emissive_gain: Shared<Animatable<'static, f32>>,
    pub(crate) fresnel: Shared<LayerFresnel<'static>>,
    pub(crate) texture_animation: Option<Shared<TextureAnimation<'static>>>,
    pub(crate) hd: bool,
    pub(crate) priority_plane: i32,
}

impl AnimatedSurface {
    pub(crate) fn new(
        layer: &Shared<Layer<'static, V1800>>,
        texture_animation: Option<Shared<TextureAnimation<'static>>>,
        priority_plane: i32,
    ) -> Self {
        let mut slots = [const { None }; 6];
        for (index, slot) in layer.texture_slots().iter().enumerate() {
            if let Some(target) = slots.get_mut(slot.texture_type as usize) {
                *target = Some(layer.select(move |layer| &layer.texture_slots()[index].texture_id));
            }
        }
        Self {
            root: Entity::PLACEHOLDER,
            slots,
            emissive_gain: layer.map(|layer| layer.emissive_gain()),
            fresnel: layer.map(|layer| layer.fresnel()),
            texture_animation,
            hd: layer.shader_type() == ShaderType::HD_DEFAULT_UNIT,
            priority_plane,
        }
    }
}

fn value<'data, T: TrackValue + Interpolate + Copy>(
    input: &Animatable<'data, T>,
    time: SamplingTime<'_>,
    fallback: T,
) -> T {
    time.sample(input).unwrap_or(fallback)
}

fn texture_transform_at_time<'data>(
    definition: Option<&TextureAnimation<'data>>,
    time: SamplingTime<'_>,
) -> Affine2 {
    let Some(definition) = definition else {
        return Affine2::IDENTITY;
    };
    let translation = definition
        .translation
        .as_ref()
        .and_then(|track| time.track(track))
        .map(Vec3::from_array)
        .unwrap_or(Vec3::ZERO);
    let rotation = definition
        .rotation
        .as_ref()
        .and_then(|track| time.track(track))
        .map(Quat::from_array)
        .unwrap_or(Quat::IDENTITY)
        .normalize();
    let scale = definition
        .scaling
        .as_ref()
        .and_then(|track| time.track(track))
        .map(Vec3::from_array)
        .unwrap_or(Vec3::ONE);
    let matrix = Mat4::from_translation(translation + Vec3::new(0.5, 0.5, 0.0))
        * Mat4::from_scale_rotation_translation(scale, rotation, Vec3::ZERO)
        * Mat4::from_translation(Vec3::new(-0.5, -0.5, 0.0));
    Affine2::from_cols(
        matrix.x_axis.truncate().truncate(),
        matrix.y_axis.truncate().truncate(),
        matrix.w_axis.truncate().truncate(),
    )
}

pub(crate) fn animate_surface(
    roots: Query<(&Wc3Animation, Ref<Wc3TextureBindings>)>,
    mut surfaces: Query<(
        &AnimatedSurface,
        &mut MeshMaterial3d<Wc3LayerMaterial>,
        Option<&ConstantMaterial>,
    )>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<LinearImages>,
    mut events: MessageReader<AssetEvent<Image>>,
) {
    let mut images_changed = false;
    for event in events.read() {
        images_changed = true;
        match event {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                cache.remove(*id);
            }
            _ => {}
        }
    }
    cache.retain_loaded(&images);
    for (surface, mut handle, constant) in &mut surfaces {
        let Ok((animation, bindings)) = roots.get(surface.root) else {
            continue;
        };
        if constant.is_some() && !bindings.is_changed() && !handle.is_changed() && !images_changed {
            continue;
        }
        let Some(mut material) = materials.get(&handle.0).cloned() else {
            continue;
        };
        surface.evaluate(
            &mut material,
            &bindings,
            SamplingTime::live(animation.time()),
            &mut images,
            &mut cache,
        );
        let previous = handle.0.id();
        apply_material(
            &mut materials,
            handle.bypass_change_detection(),
            material,
            constant.is_some(),
        );
        if handle.0.id() != previous {
            handle.set_changed();
        }
    }
}

impl AnimatedSurface {
    pub(crate) fn is_constant(&self) -> bool {
        self.texture_animation.as_ref().is_none_or(|animation| {
            animation.translation.is_none()
                && animation.rotation.is_none()
                && animation.scaling.is_none()
        }) && (!self.hd
            || (self
                .slots
                .iter()
                .flatten()
                .all(|slot| slot.track().is_none())
                && self.emissive_gain.track().is_none()
                && self.fresnel.color.track().is_none()
                && self.fresnel.opacity.track().is_none()
                && self.fresnel.team_color.track().is_none()))
    }

    pub(crate) fn evaluate(
        &self,
        material: &mut Wc3LayerMaterial,
        bindings: &Wc3TextureBindings,
        time: SamplingTime<'_>,
        images: &mut Assets<Image>,
        cache: &mut LinearImages,
    ) {
        material.base.uv_transform =
            texture_transform_at_time(self.texture_animation.as_deref(), time);
        material.base.depth_bias = self.priority_plane as f32;
        if !self.hd {
            return;
        }
        let textures: [Option<Handle<Image>>; 6] = from_fn(|role| {
            self.slots[role]
                .as_ref()
                .and_then(|track| bindings.bitmap(value(track, time, 0) as usize))
        });
        material.base.normal_map_texture = cache.resolve(textures[1].clone(), images);
        let orm = cache.resolve(textures[2].clone(), images);
        material.base.occlusion_texture = orm.clone();
        material.base.metallic_roughness_texture = orm.clone();
        material.base.metallic = if orm.is_some() { 1.0 } else { 0.0 };
        material.base.perceptual_roughness = 1.0;
        material.base.emissive_texture = textures[3].clone();
        let gain = if textures[3].is_some() {
            value(&self.emissive_gain, time, 1.0)
        } else {
            0.0
        };
        let tint = material.base.base_color.to_linear();
        material.base.emissive =
            LinearRgba::rgb(gain * tint.red, gain * tint.green, gain * tint.blue);
        material.extension.orm = orm;
        material.extension.team_color = textures[4].clone();
        material.extension.environment = textures[5].clone();
        material.extension.hd.maps = UVec4::new(
            1,
            material.extension.orm.is_some() as u32,
            textures[4].is_some() as u32,
            textures[5].is_some() as u32,
        );
        material.extension.hd.fresnel_color =
            Vec3::from_array(value(&self.fresnel.color, time, [1.0; 3])).extend(1.0);
        material.extension.hd.fresnel = Vec4::new(
            value(&self.fresnel.opacity, time, 0.0),
            value(&self.fresnel.team_color, time, 0.0),
            0.0,
            0.0,
        );
    }
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
