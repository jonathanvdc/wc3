//! Native-family surface interpretation into PBR passes and explicit states.
use super::super::textures::{TextureImages, TextureRole};
use super::m2::compile_m2;
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};

use super::wmo::compile_wmo;
use super::{
    layered_uv, Blend, HdAnimation, HdProgramme, ModelMaterial, SurfacePass, UvConvention,
};
use crate::animation_targets::AnimationTargets;
use crate::error::ModelError;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::{array::from_fn, collections::HashMap, fmt::Debug, sync::Arc};

use wc3::model::materials::mdlx;
use wc3::model::materials::program::MaterialStep;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::profiles::Universal;
use wc3::model::{MaterialId, Model, Sampler, Texture};

#[cfg(test)]
pub(crate) fn compile_material_with_images(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: Arc<TextureImages>,
) -> Result<Vec<SurfacePass>, ModelError> {
    compile_material(model, id, &images, &AnimationTargets::new(model))
}

/// Compile a model's selected material set with one shared channel lookup.
pub(crate) fn compile_materials_with_images(
    model: &Model<'_, Universal>,
    ids: impl IntoIterator<Item = MaterialId>,
    images: Arc<TextureImages>,
    targets: &AnimationTargets,
) -> Result<HashMap<MaterialId, Vec<SurfacePass>>, ModelError> {
    ids.into_iter()
        .map(|id| compile_material(model, id, &images, targets).map(|passes| (id, passes)))
        .collect()
}

fn compile_material(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: &Arc<TextureImages>,
    targets: &AnimationTargets,
) -> Result<Vec<SurfacePass>, ModelError> {
    let material = model
        .material(id)
        .ok_or_else(|| ModelError::new(format!("missing semantic material {id:?}")))?;
    let stack = &material.surface;
    let mut passes = Vec::new();
    for (step_index, step) in stack.steps.iter().enumerate() {
        let scoped = Arc::new(images.material(id, step_index as u32));
        let images = &scoped;
        let resolved_images = images.as_ref();
        let mut compiled = match step {
            MaterialStep::Mdlx(layer) => vec![compile_mdlx(
                model,
                layer,
                stack
                    .declaration
                    .as_ref()
                    .unwrap_or(&mdlx::MaterialDeclaration::default()),
                images,
            )?],
            MaterialStep::M2(surface) => {
                vec![compile_m2(model, surface, resolved_images)?]
            }
            MaterialStep::Wmo(surface) => vec![compile_wmo(model, surface, resolved_images)?],
        };

        for pass in &mut compiled {
            pass.animation.images = images.color_images();
        }
        passes.extend(compiled);
    }
    for pass in &mut passes {
        pass.animation.prune_static_bindings(targets);
        for diagnostic in &mut pass.diagnostics {
            diagnostic.message = format!(
                "material {id:?} ({}): {}",
                material.name, diagnostic.message
            );
        }
    }
    Ok(passes)
}

pub(super) fn image(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    images: &TextureImages,
    diagnostics: &mut Vec<RenderIssue>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role(model, id, TextureRole::Color, images, diagnostics)
}

pub(super) fn image_role(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<RenderIssue>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role_with_sampler(model, id, role, None, images, diagnostics)
}

pub(super) fn image_role_with_sampler(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    role: TextureRole,
    sampler: Option<Sampler>,
    images: &TextureImages,
    diagnostics: &mut Vec<RenderIssue>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let Some(id) = id else { return Ok(None) };
    let texture = id;
    let resolved = images
        .get_with_sampler(
            id,
            texture.color_space,
            role,
            sampler.unwrap_or(texture.sampler),
        )
        .cloned();
    if resolved.is_none() {
        diagnostics.push(RenderIssue::new(
            RenderDiagnosticKind::Resource,
            format!(
                "texture {id:?} resource {:?} with role {role:?} is unresolved",
                model.image_resource(texture.resource)
            ),
        ));
    }

    Ok(resolved)
}

pub(super) fn blend<E: Debug, C, A, U: Debug>(
    blend: &BlendOperations<E, C, A, U>,
    pass: &mut SurfacePass,
) {
    pass.state.blend = Some(match blend {
        BlendOperations::Opaque | BlendOperations::AlphaKey(_) => Blend::Opaque,
        BlendOperations::Alpha => Blend::Alpha,
        BlendOperations::Additive => Blend::Additive,
        BlendOperations::AdditiveAlpha(_) => Blend::AdditiveAlpha,
        BlendOperations::Modulate => Blend::Modulate,
        BlendOperations::Modulate2x => Blend::Modulate2x,
        _ => Blend::Alpha,
    });
    pass.material.alpha_mode = match blend {
        BlendOperations::Opaque => AlphaMode::Opaque,
        BlendOperations::Alpha => AlphaMode::Blend,
        BlendOperations::AlphaKey(_) => AlphaMode::Mask(128.0 / 255.0),
        // Keep straight-alpha shader output. Bevy's Add/Multiply modes modify
        // fragment RGB/alpha before our independent blend equation runs.
        BlendOperations::AdditiveAlpha(_)
        | BlendOperations::Additive
        | BlendOperations::Modulate
        | BlendOperations::Modulate2x => AlphaMode::Blend,
        BlendOperations::Extension(value) => {
            pass.diagnostics.push(RenderIssue::new(
                RenderDiagnosticKind::Approximation,
                format!(
                    "blend operation {value:?} requires a custom pipeline; using alpha blending"
                ),
            ));
            AlphaMode::Blend
        }
        BlendOperations::Unknown(value) => {
            pass.diagnostics.push(RenderIssue::new(
                RenderDiagnosticKind::Approximation,
                format!("unknown blend selector {value:?}; using alpha blending"),
            ));
            AlphaMode::Blend
        }
    };
}

pub(super) fn rasterization(pass: &mut SurfacePass, two_sided: bool, lighting: Lighting) {
    pass.material.double_sided = two_sided;
    pass.material.cull_mode = if two_sided { None } else { Some(Face::Back) };
    pass.material.unlit = lighting.unlit;
    pass.material.fog_enabled = !lighting.unfogged;
}

pub(super) fn depth(pass: &mut SurfacePass, test: Operation, write: Operation) {
    pass.state.depth_test = !test.is_bypassed();
    pass.state.depth_write = Some(!write.is_bypassed());
}

fn compile_mdlx(
    model: &Model<'_, Universal>,
    surface: &mdlx::MaterialStep,
    declaration: &mdlx::MaterialDeclaration<'_>,
    images: &Arc<TextureImages>,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    let state = &surface.state;
    let inputs = surface.shader.inputs();
    let binding = &inputs.binding;
    let opacity = &inputs.opacity;
    let emissive_gain = &inputs.emissive_gain;
    let hd_inputs = match &surface.shader {
        mdlx::Shader::ReforgedDefault(inputs) | mdlx::Shader::ReforgedCrystal(inputs) => {
            Some(inputs)
        }
        _ => None,
    };
    pass.sort_near = declaration.sort_near_z;
    pass.sort_far = declaration.sort_far_z;
    blend(&state.blend, &mut pass);
    let raster = &state.rasterization;
    let options = raster.controls;
    if options.wrap_u
        || options.wrap_v
        || options.unlit
        || options.back_faces_for_shadows
        || (options.ambient_occlusion && hd_inputs.is_none())
    {
        pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Unsupported, "authored layer wrapping, Unlit, back-face shadow, or ambient-occlusion declarations require an additional interpreter"));
    }
    rasterization(
        &mut pass,
        declaration.two_sided || raster.faces,
        raster.lighting,
    );
    depth(&mut pass, raster.depth.test, raster.depth.write);
    pass.coordinate = binding.coordinates.source;
    pass.material.base_color = Color::linear_rgba(1.0, 1.0, 1.0, model.values()[*opacity]);
    if hd_inputs.is_none() {
        pass.material.base_color_texture = image(
            model,
            model.values()[binding.image],
            images,
            &mut pass.diagnostics,
        )?;
    }
    pass.animation.opacity = Some(*opacity);
    pass.animation.texture = Some(binding.image);
    pass.animation.emissive_gain = *emissive_gain;
    pass.material.depth_bias = declaration.priority_plane as f32;
    let uv = &binding.coordinates.transform;
    pass.material.uv_transform = layered_uv(
        model.values()[uv.translation],
        model.values()[uv.rotation],
        model.values()[uv.scale],
    );
    pass.animation.uv_convention = UvConvention::Layered;
    pass.animation.transform = Some(uv.clone());
    if raster.controls.sphere_environment {
        pass.diagnostics.push(RenderIssue::new(
            RenderDiagnosticKind::Unsupported,
            "sphere-environment coordinates require a generated-coordinate shader",
        ));
    }
    if let Some(hd_inputs) = hd_inputs {
        let mut slots = from_fn(|_| None);
        slots[0] = Some(binding.image);
        for slot in &hd_inputs.textures {
            if matches!(surface.shader, mdlx::Shader::ReforgedCrystal(_)) && slot.slot == 5 {
                continue; // Retained semantic storage, not a consumed Crystal role.
            }
            if let Some(binding) = slots.get_mut(slot.slot as usize) {
                *binding = Some(slot.binding.image);
            } else {
                pass.diagnostics.push(RenderIssue::new(
                    RenderDiagnosticKind::Unsupported,
                    format!("DefaultUnit texture slot {} is unsupported", slot.slot),
                ));
            }
        }
        for (slot, binding) in slots.iter().enumerate() {
            if let Some(binding) = binding {
                let role = match slot {
                    1 => TextureRole::Normal,
                    2 => TextureRole::Data,
                    _ => TextureRole::Color,
                };
                image_role(
                    model,
                    model.values()[*binding],
                    role,
                    &images.slot(slot as u32),
                    &mut pass.diagnostics,
                )?;
            }
        }
        let programme = if matches!(surface.shader, mdlx::Shader::ReforgedCrystal(_)) {
            HdProgramme::Crystal
        } else {
            HdProgramme::DefaultUnit
        };
        let authored_ambient_occlusion =
            programme == HdProgramme::DefaultUnit && options.ambient_occlusion;
        if authored_ambient_occlusion {
            if slots[2].is_some() {
                pass.secondary_coordinate = Some(1);
            }
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Approximation,
                "DefaultUnit authored occlusion samples ORM red at raw UV1 when that stream and image are present; Bevy applies it to ambient diffuse and combines SSAO, rather than attenuating all native illumination"));
        }
        let hd = HdAnimation {
            programme,
            authored_ambient_occlusion,
            slots,
            fresnel: hd_inputs.fresnel.clone(),
            gain: *emissive_gain,
            images: images.clone(),
        };
        let mut compiled = ModelMaterial {
            base: pass.material,
            extension: pass.state,
        };
        hd.apply(model.values(), &mut compiled);
        compiled.base.reflectance = StandardMaterial::default().reflectance;
        pass.material = compiled.base;
        pass.state = compiled.extension;
        if matches!(state.blend, BlendOperations::AlphaKey(_)) {
            pass.material.alpha_mode = AlphaMode::Mask(0.75);
        }
        pass.material.opaque_render_method = OpaqueRendererMethod::Forward;
        // HD slot zero owns diffuse selection; the classic binding must not
        // overwrite it transiently and mark an unchanged asset as modified.
        pass.animation.texture = None;
        pass.animation.hd = Some(hd);
        if programme == HdProgramme::Crystal {
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Approximation,
                "Crystal uses Bevy PBR/shadow/normal transforms; grazing offsets clamp abs(view normal Z) to 0.0001. Intended material equations are source-derived, not native-client verified"));
        } else {
            pass.diagnostics.push(RenderIssue::new(RenderDiagnosticKind::Approximation, "DefaultUnit Fresnel and authored environment reflection use the renderer's PBR approximation; game fidelity remains unverified"));
        }
    } else if let mdlx::Shader::Unresolved(_) = &surface.shader {
        pass.diagnostics.push(RenderIssue::new(
            RenderDiagnosticKind::Unsupported,
            format!(
            "extended layered shader {:?} needs its authored texture-slot and Fresnel interpreter",
            declaration.shader_name
        ),
        ));
    }
    if declaration.constant_color || declaration.sort_near_z || declaration.sort_far_z {
        pass.diagnostics.push(RenderIssue::new(
            RenderDiagnosticKind::Unsupported,
            "constant-color or primitive-level sorting controls are not evaluated by this PBR pass",
        ));
    }
    if let Some(gain) = emissive_gain
        .as_ref()
        .filter(|_| pass.animation.hd.is_none())
    {
        pass.material.emissive = LinearRgba::rgb(
            model.values()[*gain],
            model.values()[*gain],
            model.values()[*gain],
        );
        pass.material.emissive_texture = pass.material.base_color_texture.clone();
    }
    Ok(pass)
}
