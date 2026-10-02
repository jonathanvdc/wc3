//! Animated surface inputs shared by SD and Reforged mesh materials.
use super::textures::LinearImages;
use bevy::asset::AssetEvent;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::array::from_fn;
use wc3::model::animation::{Animatable, Interpolate, TextureAnimation, TrackValue};
use wc3::model::materials::{Layer, LayerFresnel, ShaderType};
use wc3::model::V1800;

use crate::animation::{sample, Wc3Animation};
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;

#[derive(Component, Clone)]
pub(crate) struct AnimatedSurface {
    pub(crate) root: Entity,
    pub(crate) slots: [Option<Animatable<u32>>; 6],
    pub(crate) emissive_gain: Animatable<f32>,
    pub(crate) fresnel: LayerFresnel,
    pub(crate) texture_animation: Option<TextureAnimation>,
    pub(crate) hd: bool,
    pub(crate) priority_plane: i32,
}

impl AnimatedSurface {
    pub(crate) fn new(
        layer: &Layer<V1800>,
        texture_animation: Option<TextureAnimation>,
        priority_plane: i32,
    ) -> Self {
        let mut slots = [const { None }; 6];
        for slot in layer.texture_slots() {
            if let Some(target) = slots.get_mut(slot.texture_type as usize) {
                *target = Some(slot.texture_id.clone());
            }
        }
        Self {
            root: Entity::PLACEHOLDER,
            slots,
            emissive_gain: layer.emissive_gain(),
            fresnel: layer.fresnel(),
            texture_animation,
            hd: layer.shader_type() == ShaderType::HD_DEFAULT_UNIT,
            priority_plane,
        }
    }
}

fn value<T: TrackValue + Interpolate + Copy>(
    input: &Animatable<T>,
    animation: &Wc3Animation,
    fallback: T,
) -> T {
    input
        .track()
        .and_then(|track| sample(track, animation))
        .or_else(|| input.value().copied())
        .unwrap_or(fallback)
}

pub(crate) fn texture_transform(
    definition: Option<&TextureAnimation>,
    animation: &Wc3Animation,
) -> Affine2 {
    let Some(definition) = definition else {
        return Affine2::IDENTITY;
    };
    let translation = definition
        .translation
        .as_ref()
        .and_then(|track| sample(track, animation))
        .map(Vec3::from_array)
        .unwrap_or(Vec3::ZERO);
    let rotation = definition
        .rotation
        .as_ref()
        .and_then(|track| sample(track, animation))
        .map(Quat::from_array)
        .unwrap_or(Quat::IDENTITY)
        .normalize();
    let scale = definition
        .scaling
        .as_ref()
        .and_then(|track| sample(track, animation))
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
    roots: Query<(&Wc3Animation, &Wc3TextureBindings)>,
    surfaces: Query<(&AnimatedSurface, &MeshMaterial3d<Wc3LayerMaterial>)>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<LinearImages>,
    mut events: MessageReader<AssetEvent<Image>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                cache.remove(*id);
            }
            _ => {}
        }
    }
    cache.retain_loaded(&images);
    for (surface, handle) in &surfaces {
        let Ok((animation, bindings)) = roots.get(surface.root) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        material.base.uv_transform =
            texture_transform(surface.texture_animation.as_ref(), animation);
        material.base.depth_bias = surface.priority_plane as f32;
        if !surface.hd {
            continue;
        }
        let textures: [Option<Handle<Image>>; 6] = from_fn(|role| {
            surface.slots[role]
                .as_ref()
                .and_then(|track| bindings.bitmap(value(track, animation, 0) as usize))
        });
        material.base.normal_map_texture = cache.resolve(textures[1].clone(), &mut images);
        let orm = cache.resolve(textures[2].clone(), &mut images);
        material.base.occlusion_texture = orm.clone();
        material.base.metallic_roughness_texture = orm.clone();
        material.base.metallic = if orm.is_some() { 1.0 } else { 0.0 };
        material.base.perceptual_roughness = 1.0;
        material.base.emissive_texture = textures[3].clone();
        let gain = if textures[3].is_some() {
            value(&surface.emissive_gain, animation, 1.0)
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
            Vec3::from_array(value(&surface.fresnel.color, animation, [1.0; 3])).extend(1.0);
        material.extension.hd.fresnel = Vec4::new(
            value(&surface.fresnel.opacity, animation, 0.0),
            value(&surface.fresnel.team_color, animation, 0.0),
            0.0,
            0.0,
        );
    }
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
