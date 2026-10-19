//! World-model surface interpretation and continuously scrolling UV coordinates.
use super::super::textures::{TextureImages, TextureRole};
use super::compile::{blend, image_role_with_sampler, rasterization};
use super::{Blend, SurfacePass};
use crate::ModelError;
use bevy::math::Affine2;
use bevy::math::Vec2;
use bevy::prelude::*;
use wc3::model::materials::wmo as wmo_shader;
use wc3::model::materials::{state::BlendOperations, wmo::WorldBlend};
use wc3::model::profiles::Universal;
use wc3::model::{Model, Texture};

pub(super) fn compile_wmo(
    model: &Model<'_, Universal>,
    surface: &wmo_shader::MaterialStep<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    let declaration = surface.declaration();
    match declaration.state.blend {
        BlendOperations::Extension(operation) => {
            pass.state.blend = Some(match operation {
                WorldBlend::ModulateAdd => Blend::ModulateAdd,
                WorldBlend::InverseSourceAlphaAdd => Blend::InverseSourceAlphaAdd,
            });
            pass.material.alpha_mode = AlphaMode::Blend;
        }
        _ => blend(&declaration.state.blend, &mut pass),
    }
    let raster = &declaration.state.rasterization;
    pass.exterior_lit = raster.controls.exterior_lit;
    rasterization(&mut pass, raster.faces, raster.lighting);
    pass.state.world.flags = UVec4::new(
        1,
        matches!(
            surface,
            wmo_shader::MaterialStep::DiffuseWithoutTextureAlpha(_)
        ) as u32,
        0,
        0,
    );
    pass.material.base_color_texture =
        world_image(model, surface, 0, images, &mut pass.diagnostics)?;
    if let Some((shader, count, secondary, third)) = programme(surface) {
        if count > 1 {
            pass.state.world.composition = UVec4::new(shader, count, u32::from(count > 2) * 3, 0);
        }
        if count > 1 {
            pass.secondary_coordinate = Some(secondary.unwrap_or(0));
            pass.state.secondary = world_image(model, surface, 1, images, &mut pass.diagnostics)?;
            if secondary.is_none() {
                pass.state.combiner.coordinates.y = if matches!(
                    surface,
                    wmo_shader::MaterialStep::ReflectionEmission(_)
                        | wmo_shader::MaterialStep::DiffuseWeightedReflectionEmission(_)
                ) {
                    2
                } else {
                    1
                };
            }
        }
        if let Some(third) = third {
            pass.third_coordinate = Some(third);
            pass.state.tertiary = world_image(model, surface, 2, images, &mut pass.diagnostics)?;
        }
        pass.animation.uv_scroll = declaration.uv_scroll;
        if count == 1 {
            pass.animation.uv_scroll[1] = Vec2::ZERO;
        }
        if secondary.is_none() && count > 1 {
            pass.animation.uv_scroll[1] = Vec2::ZERO;
            if third.is_some() {
                pass.animation.third_uv_scroll = declaration.uv_scroll[1];
            }
        }
    } else {
        pass.animation.uv_scroll[0] = declaration.uv_scroll[0];
        if let wmo_shader::MaterialStep::Unresolved(step) = surface {
            if !surface.is_partial() {
                pass.diagnostics.push(format!("WMO shader {:?} needs its authored texture/specular combiner; showing its first diffuse input with Bevy lighting", step.shader));
            }
        }
    }
    if surface.is_partial() {
        pass.diagnostics.push("WMO shader is partially rendered through its first diffuse image; the specialized terrain/LOD client path is not evaluated".into());
    }
    if raster.controls.self_illuminated {
        let color = declaration.self_illumination;
        pass.material.emissive = LinearRgba::rgb(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
    Ok(pass)
}

// Typed equations select GPU modes and coordinate routes without native IDs.
fn programme(step: &wmo_shader::MaterialStep<'_>) -> Option<(u32, u32, Option<u32>, Option<u32>)> {
    use wmo_shader::MaterialStep;
    let (mode, count, secondary, third) = match step {
        MaterialStep::Diffuse(_) => (1, 1, Some(0), None),
        MaterialStep::DiffuseWithoutTextureAlpha(_) => (5, 1, Some(0), None),
        MaterialStep::ReflectionEmission(_) => (4, 2, None, None),
        MaterialStep::DiffuseWeightedReflectionEmission(_) => (6, 2, None, None),
        MaterialStep::LayeredDiffuse(_) => (7, 2, Some(1), None),
        MaterialStep::MaskedEmission(_) => (10, 2, Some(1), None),
        MaterialStep::MaskedDiffuseModulationReflection(_) => (12, 3, None, Some(1)),
        MaterialStep::DiffuseMultipleEmission(_) => (13, 3, None, Some(1)),
        MaterialStep::OpaqueLayeredDiffuse(_) => (14, 2, Some(1), None),
        MaterialStep::LayeredDiffuseEmission(_) => (16, 2, Some(1), None),
        MaterialStep::MaskedDiffuseModulation(_) => (18, 3, None, Some(1)),
        MaterialStep::LayeredDiffuseModulation(_) => (19, 3, Some(1), Some(2)),
        MaterialStep::DiffuseModulation(_) => (20, 2, Some(1), None),
        MaterialStep::TextureMaskedLayeredModulation(_) => (21, 3, Some(1), Some(2)),
        MaterialStep::Unresolved(_) => return None,
    };
    Some((mode, count, secondary, third))
}

fn world_image(
    model: &Model<'_, Universal>,
    surface: &wmo_shader::MaterialStep<'_>,
    slot: usize,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let inputs = surface.inputs();
    let input = inputs.get(slot);
    image_role_with_sampler(
        model,
        input.and_then(|input| input.image).map(Texture::new),
        TextureRole::Color,
        input.map(|input| input.sampling),
        &images.slot(slot as u32),
        diagnostics,
    )
}

/// Adds authored model-time translation after the input texture transform.
/// Clamp addressing deliberately keeps offsets outside the unit square.
pub(super) fn scrolling_transform(base: Affine2, velocity: Vec2, seconds: f32) -> Affine2 {
    Affine2 {
        translation: base.translation + Vec2::new(velocity[0] * seconds, velocity[1] * seconds),
        ..base
    }
}

#[cfg(test)]
#[path = "wmo_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "wmo_capture_tests.rs"]
mod world_capture_tests;
