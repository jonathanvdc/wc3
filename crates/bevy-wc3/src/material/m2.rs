//! Native model texture composition within Bevy lighting.
use super::compile::{blend, depth, image, rasterization};
use super::opacity::M2Opacity;
use super::{Blend, SurfacePass, UvConvention};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
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
pub(super) struct ShaderProgram {
    pub(super) mode: UVec4,
    environment: [bool; 2],
    inputs: usize,
    edge_fade: bool,
}
pub(super) fn program(step: &m2::MaterialStep<'_>) -> Option<ShaderProgram> {
    let (mode, inputs, edge_fade) = match step {
        m2::MaterialStep::Diffuse(v) => (
            UVec4::new(1, u32::from(v.use_texture_alpha), 0, 0),
            1,
            false,
        ),
        m2::MaterialStep::Modulate(v) => (
            UVec4::new(
                1,
                u32::from(v.use_diffuse_alpha),
                u32::from(v.use_modulation_alpha),
                u32::from(v.doubled),
            ),
            2,
            false,
        ),
        m2::MaterialStep::Additive(v) => {
            let mode = match v.opacity {
                m2::AdditiveOpacity::Opaque => UVec4::new(16, 0, 0, 0),
                m2::AdditiveOpacity::Diffuse => UVec4::new(17, 1, 0, 0),
                m2::AdditiveOpacity::DiffuseAndEmission => UVec4::new(16, 1, 0, 0),
            };
            (mode, 2, false)
        }
        m2::MaterialStep::MaskedDoubledModulation(_) => (UVec4::new(2, 0, 0, 0), 2, false),
        m2::MaterialStep::OpaqueAdditiveAlpha(_) => (UVec4::new(3, 0, 0, 0), 2, false),
        m2::MaterialStep::Illumination(_) => (UVec4::new(27, 0, 0, 0), 2, false),
        m2::MaterialStep::InverseMaskedOpaqueAdditiveAlpha(_) => (UVec4::new(4, 0, 0, 0), 2, false),
        m2::MaterialStep::MaskedDoubledModulationGlow(_) => (UVec4::new(5, 0, 0, 0), 3, false),
        m2::MaterialStep::ModulatedAdditiveAlpha(_) => (UVec4::new(6, 0, 0, 0), 2, false),
        m2::MaterialStep::InverseMaskedModulatedAdditiveAlpha(_) => {
            (UVec4::new(7, 0, 0, 0), 2, false)
        }
        m2::MaterialStep::DoubleAlphaInterpolation(_) => (UVec4::new(8, 0, 0, 0), 2, false),
        m2::MaterialStep::DoubledModulationInterpolation(_) => (UVec4::new(9, 0, 0, 0), 3, false),
        m2::MaterialStep::WeightedOpaqueAdditiveAlpha(_) => (UVec4::new(10, 0, 0, 0), 2, false),
        m2::MaterialStep::ModulatedAdditiveInverseMask(_) => (UVec4::new(20, 0, 0, 0), 2, false),
        m2::MaterialStep::MaskedModulation(_) => (UVec4::new(11, 0, 0, 0), 2, false),
        m2::MaterialStep::WeightedModulatedAdditiveAlpha(_) => (UVec4::new(12, 0, 0, 0), 2, false),
        m2::MaterialStep::WeightedModulationAdditive(_) => (UVec4::new(13, 0, 0, 0), 2, false),
        m2::MaterialStep::MaskedDoubledModulationAlpha(_) => (UVec4::new(14, 0, 0, 0), 3, false),
        m2::MaterialStep::DualCrossfade(_) => (UVec4::new(18, 0, 0, 0), 3, false),
        m2::MaterialStep::FacingOpacity(_) => (UVec4::new(22, 0, 0, 0), 1, true),
        m2::MaterialStep::DoubledModulationDoubleInterpolation(_) => {
            (UVec4::new(19, 0, 0, 0), 3, false)
        }
        m2::MaterialStep::ModulatedFacingOpacity(_) => (UVec4::new(1, 1, 1, 0), 2, true),
        m2::MaterialStep::MaskedDualCrossfade(_) => (UVec4::new(23, 0, 0, 0), 4, false),
        m2::MaterialStep::AlphaInterpolation(_) => (UVec4::new(15, 0, 0, 0), 2, false),
        m2::MaterialStep::Guild(_) => (UVec4::new(24, 0, 0, 0), 3, false),
        m2::MaterialStep::GuildEmblem(_) => (UVec4::new(25, 0, 0, 0), 2, false),
        m2::MaterialStep::OpaqueGuild(_) => (UVec4::new(26, 0, 0, 0), 3, false),
        m2::MaterialStep::DiffuseWithoutTextureAlpha(_) => (UVec4::new(21, 0, 0, 0), 1, false),
        m2::MaterialStep::DoubledModulatedFacingOpacity(_) => (UVec4::new(1, 1, 1, 1), 2, true),
        m2::MaterialStep::DepthModulatedFacingOpacity(_) => (UVec4::new(1, 1, 1, 0), 2, true),
        m2::MaterialStep::Unresolved(_) => return None,
    };
    let bindings = step.inputs();
    Some(ShaderProgram {
        mode,
        inputs,
        edge_fade,
        environment: [0, 1].map(|slot| {
            bindings
                .get(slot)
                .is_some_and(|input| input.coordinates.source == -1)
        }),
    })
}

// Four samples need at most three raw streams. Reuse authored coordinates before
// requesting another stream, independently of each sample's animated transform.
fn fourth_route(inputs: &[&m2::TextureUnit]) -> Option<u32> {
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
    if let m2::MaterialStep::Unresolved(value) = surface {
        if let Some(equation) = value.compatibility_equation() {
            let mut pass = compile_m2(model, &equation, images)?;
            // The alias uses raw UV0 for its third sample. Preserve the authored
            // transform in the unresolved model, but never animate it here.
            pass.state.combiner.third_transform = Mat3::IDENTITY;
            pass.animation.third_transform = None;
            pass.diagnostics.push(RenderIssue::new(
                RenderDiagnosticKind::Unverified,
                format!(
                    "M2 selector {:#06x} uses a three-image compatibility approximation with raw third-input UV0; native equations, constants and vertex behavior remain unknown",
                    value.shader
                ),
            ));
            return Ok(pass);
        }
        if let Some(equation) = value.partial_equation() {
            let mut pass = compile_m2(model, &equation, images)?;
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Unverified, "M2 single-input UV1 transform ownership remains unresolved; rendering the established texture equation"));
            return Ok(pass);
        }
    }
    let mut pass = SurfacePass::default();
    pass.animation.uv_convention = UvConvention::M2;
    let declaration = surface.declaration();
    let inputs = surface.inputs();
    if matches!(
        declaration.state.blend,
        BlendOperations::Extension(ModelBlend::BlendAdd)
    ) {
        pass.state.blend = Some(Blend::Premultiplied);
        pass.material.alpha_mode = AlphaMode::Premultiplied;
    } else {
        blend(&declaration.state.blend, &mut pass);
    }
    let raster = &declaration.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    depth(&mut pass, raster.depth.test, raster.depth.write);
    let color = model.values()[declaration.color];
    pass.animation.color = Some(declaration.color);
    pass.animation.opacity = Some(declaration.opacity);
    pass.material.base_color = Color::linear_rgba(
        color[0],
        color[1],
        color[2],
        model.values()[declaration.opacity],
    );
    pass.material.depth_bias = declaration.state.ordering as f32;
    let supported = program(surface).filter(|program| {
        inputs
            .iter()
            .take(program.inputs)
            .all(|input| input.coordinates.source >= -1)
            && (program.inputs < 4 || fourth_route(&inputs).is_some())
    });
    if let Some(first) = inputs.first() {
        pass.material.base_color_texture =
            image(model, first.image.texture, images, &mut pass.diagnostics)?;
        pass.coordinate = u32::try_from(first.coordinates.source).unwrap_or(0);
        pass.material.uv_transform = first.coordinates.transform.m2_affine(model.values());
        pass.animation.transform = Some(first.coordinates.transform.clone());
        if declaration.batch_flags.bits() & 0x40 == 0 {
            pass.animation.weight = Some(first.contribution);
            pass.material.base_color.set_alpha(
                model.values()[declaration.opacity] * model.values()[first.contribution],
            );
        }
    }
    if let Some(program) = supported {
        pass.state.combiner.mode = program.mode;
        if matches!(surface, m2::MaterialStep::Illumination(_)) {
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Unverified,
                "M2 illumination uses a primary-modulated emissive interpretation; native game fidelity has no representative asset validation"));
        }
        let palette = surface.guild_palette().unwrap_or_default();
        pass.state.combiner.guild_background = palette.background.extend(1.);
        pass.state.combiner.guild_emblem = palette.emblem.extend(1.);
        pass.state.combiner.guild_border = palette.border.extend(1.);
        pass.state.combiner.clip.y = if program.edge_fade { 1. } else { 0. };
        pass.state.combiner.coordinates.z = program.inputs as u32;
        if program.inputs < inputs.len() {
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Approximation, "M2 legacy texture equation uses its first two inputs; surplus native bindings are retained but not sampled"));
        }
        for (slot, input) in inputs.iter().take(program.inputs).enumerate() {
            pass.state.combiner.weights[slot] = model.values()[input.contribution];
            pass.animation.input_weights[slot] = Some(input.contribution);
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
                pass.state.secondary = image(
                    model,
                    input.image.texture,
                    &images.slot(slot as u32),
                    &mut pass.diagnostics,
                )?;
                if !environment {
                    pass.state.combiner.uv_transform =
                        Mat3::from(input.coordinates.transform.m2_affine(model.values()));
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
                pass.state.tertiary = image(
                    model,
                    input.image.texture,
                    &images.slot(slot as u32),
                    &mut pass.diagnostics,
                )?;
                if !environment {
                    pass.state.combiner.third_transform =
                        Mat3::from(input.coordinates.transform.m2_affine(model.values()));
                    pass.animation.third_transform = Some(input.coordinates.transform.clone());
                }
            } else if slot == 3 {
                let route = fourth_route(&inputs).expect("validated fourth coordinate route");
                pass.state.combiner.additional_coordinates.x = route;
                if route == 3 && pass.third_coordinate.is_none() {
                    pass.third_coordinate = Some(input.coordinates.source as u32);
                }
                pass.state.fourth = image(
                    model,
                    input.image.texture,
                    &images.slot(slot as u32),
                    &mut pass.diagnostics,
                )?;
                pass.state.combiner.fourth_transform =
                    Mat3::from(input.coordinates.transform.m2_affine(model.values()));
                pass.animation.fourth_transform = Some(input.coordinates.transform.clone());
            }
        }
    } else if !matches!(surface, m2::MaterialStep::Unresolved(v) if v.shader == 0)
        || inputs.len() > 1
        || inputs.iter().any(|i| i.coordinates.source < 0)
    {
        pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Approximation, format!("M2 shader with {} texture inputs or unsupported coordinates is approximated by its first input; authored combiners are not evaluated",inputs.len())));
    }
    let opacity = M2Opacity::new(pass.material.alpha_mode, pass.state.blend);
    pass.state.m2_opacity = Some(opacity);
    opacity.apply(&mut pass.material, &mut pass.state);
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
