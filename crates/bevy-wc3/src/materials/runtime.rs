//! Change-driven material updates and sharing of constant authored surfaces.
use super::Wc3LayerMaterial;
use bevy::asset::AssetId;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy)]
pub(crate) struct MaterialTemplateId(u64);
impl MaterialTemplateId {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Only surfaces whose authored properties are constant may share materials.
#[derive(Component)]
pub(crate) struct ConstantMaterial(pub(crate) MaterialTemplateId);

/// Keeps a mesh pass out of constant-material sharing. Clone its material asset
/// before customizing application-owned PBR settings independently of other passes.
#[derive(Component, Default)]
pub struct Wc3IndependentMaterial;

/// Properties owned by authored layer and surface animation. Application PBR
/// settings are left untouched when sampling an existing material.
#[derive(Clone, PartialEq)]
pub(crate) struct MaterialValues {
    color: Color,
    alpha: AlphaMode,
    textures: [Option<Handle<Image>>; 8],
    uv: Affine2,
    depth_bias: f32,
    emissive: LinearRgba,
    metallic: f32,
    roughness: f32,
    maps: UVec4,
    fresnel_color: Vec4,
    fresnel: Vec4,
}
impl MaterialValues {
    pub(crate) fn read(material: &Wc3LayerMaterial) -> Self {
        let base = &material.base;
        let extension = &material.extension;
        Self {
            color: base.base_color,
            alpha: base.alpha_mode,
            textures: [
                base.base_color_texture.clone(),
                base.normal_map_texture.clone(),
                base.occlusion_texture.clone(),
                base.metallic_roughness_texture.clone(),
                base.emissive_texture.clone(),
                extension.orm.clone(),
                extension.team_color.clone(),
                extension.environment.clone(),
            ],
            uv: base.uv_transform,
            depth_bias: base.depth_bias,
            emissive: base.emissive,
            metallic: base.metallic,
            roughness: base.perceptual_roughness,
            maps: extension.hd.maps,
            fresnel_color: extension.hd.fresnel_color,
            fresnel: extension.hd.fresnel,
        }
    }
}

pub(crate) fn apply_material(
    materials: &mut Assets<Wc3LayerMaterial>,
    handle: &mut MeshMaterial3d<Wc3LayerMaterial>,
    candidate: Wc3LayerMaterial,
    constant: bool,
) {
    let Some(current) = materials.get(&handle.0) else {
        return;
    };
    if MaterialValues::read(current) == MaterialValues::read(&candidate) {
        return;
    }
    if constant {
        // Texture overrides on one root must never edit another root's shared asset.
        handle.0 = materials.add(candidate);
    } else if let Some(mut current) = materials.get_mut(&handle.0) {
        *current = candidate;
    }
}

pub(crate) struct Entry {
    values: MaterialValues,
    handle: Handle<Wc3LayerMaterial>,
}

pub(crate) fn share_constant_materials(
    mut parts: Query<
        (&ConstantMaterial, &mut MeshMaterial3d<Wc3LayerMaterial>),
        Without<Wc3IndependentMaterial>,
    >,
    materials: Res<Assets<Wc3LayerMaterial>>,
    mut cache: Local<HashMap<u64, Vec<Entry>>>,
) {
    let mut used = HashSet::<AssetId<Wc3LayerMaterial>>::new();
    for (constant, mut part) in &mut parts {
        let Some(material) = materials.get(&part.0) else {
            continue;
        };
        let entries = cache.entry(constant.0 .0).or_default();
        if entries.iter().any(|entry| entry.handle == part.0) {
            used.insert(part.0.id());
            continue;
        }
        let values = MaterialValues::read(material);
        let handle = if let Some(entry) = entries
            .iter()
            .find(|entry| entry.values == values && materials.contains(&entry.handle))
        {
            entry.handle.clone()
        } else {
            entries.push(Entry {
                values,
                handle: part.0.clone(),
            });
            part.0.clone()
        };
        used.insert(handle.id());
        if part.0 != handle {
            part.0 = handle;
        }
    }
    cache.retain(|_, entries| {
        entries.retain(|entry| used.contains(&entry.handle.id()));
        !entries.is_empty()
    });
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
