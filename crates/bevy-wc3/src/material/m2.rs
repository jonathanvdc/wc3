//! Native model texture composition within Bevy lighting.
use super::compile::{blend, depth, image, rasterization, transform};
use super::{Blend, SurfacePass};
use crate::textures::TextureImages;
use crate::ModelError;
use bevy::math::Affine2;
use bevy::prelude::*;
use wc3::model::materials::{m2, state::BlendOperations};
use wc3::model::profiles::Universal;
use wc3::model::values::model::ModelBlend;
use wc3::model::Model;

// IDs agree with the shared composition shader; zero leaves StandardMaterial alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShaderProgram {
    mode: UVec4,
    environment: [bool; 2],
    inputs: usize,
    edge_fade: bool,
}
fn program(shader: u16, inputs: usize) -> Option<ShaderProgram> {
    if shader & 0x8000 == 0 {
        if shader & !0x40ff != 0 {
            return None;
        }
        let inputs = inputs.min(2);
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
        return Some(ShaderProgram {
            mode,
            environment: [shader & 0x80 != 0, shader & 8 != 0],
            inputs,
            edge_fade: false,
        });
    }
    let offset = shader & 0x7fff;
    let (mode, count, env) = match offset {
        0 => (2, 2, true),
        1 | 5 => (3, 2, offset == 1),
        2 => (4, 2, true),
        3 => (5, 3, true),
        4 | 6 => (6, 2, offset != 6),
        11 => (20, 2, true),
        7 => (7, 2, true),
        8 => (8, 2, true),
        9 => (9, 3, true),
        10 | 15 => (10, 2, false),
        12 => (11, 2, true),
        13 | 14 => (12, 2, offset == 13),
        16 => (13, 2, true),
        17 | 24 => (14, 3, true),
        18 => (18, 3, false),
        19 | 25 | 34 => (22, 1, false),
        20 => (19, 3, true),
        21 | 35 => (1, 2, false),
        32 => (21, 1, false),
        33 => (1, 2, false),
        22 => (23, 4, false),
        23 => (15, 2, false),
        26 | 28 => (if offset == 26 { 24 } else { 26 }, 3, false),
        27 => (25, 2, false),
        _ => return None,
    };
    (inputs == count).then_some(ShaderProgram {
        mode: if matches!(offset, 21 | 35) {
            UVec4::new(mode, 1, 1, 0)
        } else if offset == 33 {
            UVec4::new(mode, 1, 1, 1)
        } else {
            UVec4::new(mode, 0, 0, 0)
        },
        environment: [offset == 25, env && count > 1],
        inputs,
        edge_fade: matches!(offset, 19 | 21 | 25 | 33 | 34 | 35),
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

// Four samples need at most three raw streams. Reuse authored coordinates before
// requesting another stream, independently of each sample's animated transform.
fn fourth_route(surface: &m2::MaterialStep<'_>) -> Option<u32> {
    let inputs = &surface.textures;
    let source = inputs.get(3)?.coordinates.source;
    if source < 0 {
        return None;
    }
    if source == inputs[0].coordinates.source {
        return Some(0);
    }
    if source == inputs[1].coordinates.source {
        return Some(2);
    }
    let third = inputs[2].coordinates.source;
    if source == third
        || (third >= 0
            && (third == inputs[0].coordinates.source || third == inputs[1].coordinates.source))
    {
        return Some(3);
    }
    None
}

pub(super) fn compile_m2(
    model: &Model<'_, Universal>,
    surface: &m2::MaterialStep<'_>,
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
    let supported = program(surface.shader, surface.textures.len()).filter(|program| {
        surface
            .textures
            .iter()
            .take(program.inputs)
            .all(|input| input.coordinates.source >= -1)
            && (program.inputs < 4 || fourth_route(surface).is_some())
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
        let palette = surface.guild_palette;
        pass.state.combiner.guild_background = Vec3::from_array(palette.background).extend(1.);
        pass.state.combiner.guild_emblem = Vec3::from_array(palette.emblem).extend(1.);
        pass.state.combiner.guild_border = Vec3::from_array(palette.border).extend(1.);
        pass.state.combiner.clip.y = if program.edge_fade { 1. } else { 0. };
        pass.state.combiner.coordinates.z = program.inputs as u32;
        if program.inputs < surface.textures.len() {
            pass.diagnostics.push("M2 legacy texture equation uses its first two inputs; surplus native bindings are retained but not sampled".into());
        }
        for (slot, input) in surface.textures.iter().take(program.inputs).enumerate() {
            pass.state.combiner.weights[slot] = input.contribution.default;
            pass.animation.input_weights[slot] = Some(input.contribution.clone());
            let environment = program.environment.get(slot).copied().unwrap_or(false)
                || input.coordinates.source == -1;
            if slot < 3 {
                let coordinate_flag = if slot == 2 { 3 } else { slot };
                pass.state.combiner.coordinates[coordinate_flag] = u32::from(environment);
            }
            if environment && slot == 0 {
                pass.material.uv_transform = Affine2::IDENTITY;
                pass.animation.transform = None;
            }
            if slot == 1 {
                let source = u32::try_from(input.coordinates.source).unwrap_or(0);
                if environment
                    || pass.state.combiner.coordinates.x != 0
                    || source != pass.coordinate
                {
                    pass.secondary_coordinate = Some(source);
                }
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
                let source = u32::try_from(input.coordinates.source).unwrap_or(0);
                let route = if environment {
                    1
                } else if pass.state.combiner.coordinates.x == 0 && pass.coordinate == source {
                    0
                } else if pass.state.combiner.coordinates.y == 0
                    && pass.secondary_coordinate == Some(source)
                {
                    2
                } else {
                    3
                };
                pass.state.combiner.coordinates.w = route;
                if matches!(route, 1 | 3) {
                    pass.third_coordinate = Some(source);
                }
                pass.state.tertiary =
                    image(model, input.image.texture, images, &mut pass.diagnostics)?;
                if !environment {
                    let mut scratch = SurfacePass::default();
                    transform(&mut scratch, &input.coordinates.transform);
                    pass.state.combiner.third_transform = Mat3::from(scratch.material.uv_transform);
                    pass.diagnostics.extend(scratch.diagnostics);
                    pass.animation.third_transform = Some(input.coordinates.transform.clone());
                }
            } else if slot == 3 {
                let route = fourth_route(surface).expect("validated fourth coordinate route");
                pass.state.combiner.additional_coordinates.x = route;
                if route == 3 && pass.third_coordinate.is_none() {
                    pass.third_coordinate = Some(input.coordinates.source as u32);
                }
                pass.state.fourth =
                    image(model, input.image.texture, images, &mut pass.diagnostics)?;
                let mut scratch = SurfacePass::default();
                transform(&mut scratch, &input.coordinates.transform);
                pass.state.combiner.fourth_transform = Mat3::from(scratch.material.uv_transform);
                pass.diagnostics.extend(scratch.diagnostics);
                pass.animation.fourth_transform = Some(input.coordinates.transform.clone());
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
#[path = "m2_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "m2_capture_tests.rs"]
mod capture_tests;

#[cfg(test)]
#[path = "m2_routes_tests.rs"]
mod route_tests;
