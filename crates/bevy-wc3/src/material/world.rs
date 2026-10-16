//! World-model surface interpretation and continuously scrolling UV coordinates.
use super::super::textures::{world_sampler, TextureImages, TextureRole};
use super::compile::{blend, image_role_with_sampler, rasterization};
use super::SurfacePass;
use crate::ModelError;
use bevy::math::Affine2;
use bevy::prelude::*;
use wc3::model::materials::world as world_surface;
use wc3::model::profiles::Universal;
use wc3::model::Model;

pub(super) fn compile_world(
    model: &Model<'_, Universal>,
    surface: &world_surface::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    blend(&surface.state.blend, &mut pass);
    let raster = &surface.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    pass.state.world.flags = UVec4::new(1, (surface.shader == Some(4)) as u32, 0, 0);
    pass.material.base_color_texture = image_role_with_sampler(
        model,
        surface.textures.first().and_then(|slot| slot.image),
        TextureRole::Color,
        Some(world_sampler(
            surface,
            surface
                .textures
                .first()
                .and_then(|slot| slot.image)
                .and_then(|id| model.texture(id))
                .map(|texture| texture.sampler)
                .unwrap_or_default(),
        )),
        images,
        &mut pass.diagnostics,
    )?;
    if raster.controls.self_illuminated {
        let color = surface.self_illumination;
        pass.material.emissive = LinearRgba::rgb(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
    if !matches!(surface.shader, Some(0 | 4 | 16)) {
        pass.diagnostics.push(format!("WMO shader {:?} needs its authored texture/specular combiner; showing its first diffuse input with Bevy lighting", surface.shader));
    }
    pass.diagnostics.push("WMO vertex RGB supplies additive baked diffuse illumination through Bevy PBR; vertex alpha is retained as interior/exterior transition data but its scene-light interpolation is not evaluated, and application ambient lighting remains authoritative".into());
    pass.animation.uv_scroll = surface.uv_scroll;
    Ok(pass)
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
