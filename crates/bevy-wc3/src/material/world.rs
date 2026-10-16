//! World-model surface interpretation and continuously scrolling UV coordinates.
use super::super::textures::{world_sampler, TextureImages, TextureRole};
use super::compile::{blend, image_role_with_sampler, rasterization};
use super::{Blend, SurfacePass};
use crate::ModelError;
use bevy::math::Affine2;
use bevy::prelude::*;
use wc3::model::materials::world as world_surface;
use wc3::model::materials::{state::BlendOperations, world::WorldBlend};
use wc3::model::profiles::Universal;
use wc3::model::Model;

pub(super) fn compile_world(
    model: &Model<'_, Universal>,
    surface: &world_surface::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    match surface.state.blend {
        BlendOperations::Extension(operation) => {
            pass.state.blend = Some(match operation {
                WorldBlend::ModulateAdd => Blend::ModulateAdd,
                WorldBlend::InverseSourceAlphaAdd => Blend::InverseSourceAlphaAdd,
            });
            pass.material.alpha_mode = AlphaMode::Blend;
        }
        _ => blend(&surface.state.blend, &mut pass),
    }
    let raster = &surface.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    pass.state.world.flags = UVec4::new(1, (surface.shader == Some(4)) as u32, 0, 0);
    pass.material.base_color_texture =
        world_image(model, surface, 0, images, &mut pass.diagnostics)?;
    if let Some((shader, count, secondary, third)) = programme(surface.shader) {
        if count > 1 {
            pass.state.world.composition =
                UVec4::new(shader + 1, count, u32::from(count > 2) * 3, 0);
        }
        if count > 1 {
            pass.secondary_coordinate = Some(secondary.unwrap_or(0));
            pass.state.secondary = world_image(model, surface, 1, images, &mut pass.diagnostics)?;
            if secondary.is_none() {
                pass.state.combiner.coordinates.y = if matches!(shader, 3 | 5) { 2 } else { 1 };
            }
        }
        if let Some(third) = third {
            pass.third_coordinate = Some(third);
            pass.state.tertiary = world_image(model, surface, 2, images, &mut pass.diagnostics)?;
        }
        pass.animation.uv_scroll = surface.uv_scroll;
        if count == 1 {
            pass.animation.uv_scroll[1] = [0.; 2];
        }
        if secondary.is_none() && count > 1 {
            pass.animation.uv_scroll[1] = [0.; 2];
            if third.is_some() {
                pass.animation.third_uv_scroll = surface.uv_scroll[1];
            }
        }
    } else {
        pass.animation.uv_scroll[0] = surface.uv_scroll[0];
        pass.diagnostics.push(format!("WMO shader {:?} needs its authored texture/specular combiner; showing its first diffuse input with Bevy lighting", surface.shader));
    }
    if let Some(shader @ (16 | 21)) = surface.shader {
        pass.diagnostics.push(format!("WMO shader {shader} is partially rendered through its first diffuse image; the specialized terrain/LOD client path is not evaluated"));
    }
    if raster.controls.self_illuminated {
        let color = surface.self_illumination;
        pass.material.emissive = LinearRgba::rgb(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
    Ok(pass)
}

// Native programme routes: None generates reflection coordinates; raw indices
// refer to the group's authored UV sets, independently of texture slot order.
fn programme(shader: Option<u32>) -> Option<(u32, u32, Option<u32>, Option<u32>)> {
    let shader = shader?;
    let (count, secondary, third) = match shader {
        0 | 4 | 16 | 21 => (1, Some(0), None),
        3 | 5 => (2, None, None),
        6 | 9 | 13 | 15 | 19 => (2, Some(1), None),
        18 | 20 => (3, Some(1), Some(2)),
        11 | 12 | 17 => (3, None, Some(1)),
        _ => return None,
    };
    Some((shader, count, secondary, third))
}

fn world_image(
    model: &Model<'_, Universal>,
    surface: &world_surface::Material<'_>,
    slot: usize,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let image = surface.textures.get(slot).and_then(|slot| slot.image);
    let sampler = image
        .and_then(|id| model.texture(id))
        .map(|texture| texture.sampler)
        .unwrap_or_default();
    image_role_with_sampler(
        model,
        image,
        TextureRole::Color,
        Some(world_sampler(surface, sampler)),
        images,
        diagnostics,
    )
}

/// Adds authored model-time translation after the input texture transform.
/// Clamp addressing deliberately keeps offsets outside the unit square.
pub(super) fn scrolling_transform(base: Affine2, velocity: [f32; 2], seconds: f32) -> Affine2 {
    Affine2 {
        translation: base.translation + Vec2::new(velocity[0] * seconds, velocity[1] * seconds),
        ..base
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "world_capture_tests.rs"]
mod world_capture_tests;
