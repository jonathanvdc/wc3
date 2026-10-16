//! Native model texture composition within Bevy lighting.
use super::compile::{blend, depth, image, rasterization, transform};
use super::{Blend, SurfacePass};
use crate::textures::TextureImages;
use crate::ModelError;
use bevy::math::Affine2;
use bevy::prelude::*;
use wc3::model::materials::{model_shader, state::BlendOperations};
use wc3::model::profiles::Universal;
use wc3::model::values::model::ModelBlend;
use wc3::model::Model;

// IDs agree with the shared composition shader; zero leaves StandardMaterial alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Program {
    mode: UVec4,
    environment: [bool; 2],
    inputs: usize,
}
fn program(shader: u16, inputs: usize) -> Option<Program> {
    if shader & 0x8000 == 0 {
        let mode = if inputs == 2 && matches!(shader & 7, 3 | 7) {
            UVec4::new(
                if shader & 7 == 3 { 16 } else { 17 },
                u32::from(shader & 0x70 != 0),
                0,
                0,
            )
        } else {
            legacy_combiner(shader & !0x88, inputs)?
        };
        return Some(Program {
            mode,
            environment: [shader & 0x80 != 0, shader & 8 != 0],
            inputs,
        });
    }
    let offset = shader & 0x7fff;
    let (mode, count, env) = match offset {
        0 => (2, 2, true),
        1 | 5 => (3, 2, offset == 1),
        2 => (4, 2, true),
        3 => (5, 3, true),
        4 | 6 | 11 => (6, 2, offset != 6),
        7 => (7, 2, true),
        8 => (8, 2, true),
        9 => (9, 3, true),
        10 | 15 => (10, 2, false),
        12 => (11, 2, true),
        13 | 14 => (12, 2, offset == 13),
        16 => (13, 2, true),
        17 => (14, 3, true),
        23 => (15, 2, false),
        _ => return None,
    };
    (inputs == count).then_some(Program {
        mode: UVec4::new(mode, 0, 0, 0),
        environment: [false, env],
        inputs,
    })
}

/// Decodes established legacy selector bits, keeping external shader tables opaque.
pub(super) fn legacy_combiner(shader: u16, inputs: usize) -> Option<UVec4> {
    let lower = shader & 7;
    if shader & 0x8088 != 0 || !(inputs == 1 || (inputs == 2 && !matches!(lower, 3 | 7))) {
        return None;
    }
    Some(UVec4::new(
        1,
        u32::from(shader & 0x70 != 0),
        u32::from(inputs == 2 && lower != 0 && lower != 6),
        u32::from(inputs == 2 && matches!(lower, 4 | 6)),
    ))
}

pub(super) fn compile_model_shader(
    model: &Model<'_, Universal>,
    surface: &model_shader::ShaderPass<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    if matches!(
        surface.state.blend,
        BlendOperations::Extension(ModelBlend::BlendAdd)
    ) {
        pass.state.blend = Some(Blend::Premultiplied);
        pass.material.alpha_mode = AlphaMode::Premultiplied;
    } else {
        blend(&surface.state.blend, &mut pass);
    }
    let raster = &surface.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    depth(&mut pass, raster.depth.test, raster.depth.write);
    let color = surface.color.default;
    pass.animation.color = Some(surface.color.clone());
    pass.animation.opacity = Some(surface.opacity.clone());
    pass.material.base_color =
        Color::linear_rgba(color[0], color[1], color[2], surface.opacity.default);
    pass.material.depth_bias = surface.state.ordering as f32;
    let supported = program(surface.shader, surface.textures.len()).filter(|_| {
        surface
            .textures
            .iter()
            .all(|input| input.coordinates.source >= -1)
            && (surface.textures.len() < 3
                || (surface.textures[0].coordinates.source >= 0
                    && surface.textures[2].coordinates.source
                        == surface.textures[0].coordinates.source))
    });
    if let Some(first) = surface.textures.first() {
        pass.material.base_color_texture =
            image(model, first.image.texture, images, &mut pass.diagnostics)?;
        pass.coordinate = u32::try_from(first.coordinates.source).unwrap_or(0);
        transform(&mut pass, &first.coordinates.transform);
        pass.animation.transform = Some(first.coordinates.transform.clone());
        if surface.batch_flags.bits() & 0x40 == 0 {
            pass.animation.weight = Some(first.contribution.clone());
            pass.material
                .base_color
                .set_alpha(surface.opacity.default * first.contribution.default);
        }
    }
    if let Some(program) = supported {
        pass.state.combiner.mode = program.mode;
        pass.state.combiner.coordinates.z = program.inputs as u32;
        for (slot, input) in surface.textures.iter().enumerate() {
            pass.state.combiner.weights[slot] = input.contribution.default;
            pass.animation.input_weights[slot] = Some(input.contribution.clone());
            let environment =
                slot < 2 && (program.environment[slot] || input.coordinates.source == -1);
            if slot < 2 {
                pass.state.combiner.coordinates[slot] = u32::from(environment);
            }
            if environment && slot == 0 {
                pass.material.uv_transform = Affine2::IDENTITY;
                pass.animation.transform = None;
            }
            if slot == 1 {
                pass.secondary_coordinate =
                    Some(u32::try_from(input.coordinates.source).unwrap_or(0));
                pass.state.secondary =
                    image(model, input.image.texture, images, &mut pass.diagnostics)?;
                if !environment {
                    let mut scratch = SurfacePass::default();
                    transform(&mut scratch, &input.coordinates.transform);
                    pass.state.combiner.uv_transform = Mat3::from(scratch.material.uv_transform);
                    pass.diagnostics.extend(scratch.diagnostics);
                    pass.animation.secondary_transform = Some(input.coordinates.transform.clone());
                }
            } else if slot == 2 {
                pass.state.tertiary =
                    image(model, input.image.texture, images, &mut pass.diagnostics)?;
                let mut scratch = SurfacePass::default();
                transform(&mut scratch, &input.coordinates.transform);
                pass.state.combiner.third_transform = Mat3::from(scratch.material.uv_transform);
                pass.diagnostics.extend(scratch.diagnostics);
                pass.animation.third_transform = Some(input.coordinates.transform.clone());
            }
        }
    } else if surface.shader != 0
        || surface.textures.len() > 1
        || surface.textures.iter().any(|i| i.coordinates.source < 0)
    {
        pass.diagnostics.push(format!("M2 shader {} with {} texture inputs or unsupported coordinates is approximated by its first input; authored combiners are not evaluated",surface.shader,surface.textures.len()));
    }
    Ok(pass)
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
