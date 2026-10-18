//! Native-family surface interpretation into PBR passes and explicit states.
use super::super::textures::{TextureImages, TextureRole};
use super::m2::compile_m2;
use super::m3::compile_m3_with_targets;
use super::wmo::compile_wmo;
use super::{layered_uv, Blend, HdAnimation, ModelMaterial, SurfacePass};
use crate::animation_targets::AnimationTargets;
use crate::error::ModelError;
use bevy::material::OpaqueRendererMethod;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::{array::from_fn, collections::HashMap, fmt::Debug, sync::Arc};
use wc3::model::appearance::UniversalAppearance;
use wc3::model::materials::m3;
use wc3::model::materials::program::MaterialStep;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{diablo3, mdlx};
use wc3::model::profiles::Universal;
use wc3::model::PropertyValues;
use wc3::model::{MaterialId, Model, Sampler, Texture, TextureTransform};

#[cfg(test)]
pub(crate) fn compile_material_with_images(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: Arc<TextureImages>,
) -> Result<Vec<SurfacePass>, ModelError> {
    compile_material(
        model,
        id,
        &images,
        &AnimationTargets::new(model),
        &mut Vec::new(),
    )
}

/// Compile a model's selected material set with one shared channel lookup.
pub(crate) fn compile_materials_with_images(
    model: &Model<'_, Universal>,
    ids: impl IntoIterator<Item = MaterialId>,
    images: Arc<TextureImages>,
    targets: &AnimationTargets,
) -> Result<HashMap<MaterialId, Vec<SurfacePass>>, ModelError> {
    ids.into_iter()
        .map(|id| {
            compile_material(model, id, &images, targets, &mut Vec::new())
                .map(|passes| (id, passes))
        })
        .collect()
}

fn compile_material(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: &Arc<TextureImages>,
    targets: &AnimationTargets,
    path: &mut Vec<MaterialId>,
) -> Result<Vec<SurfacePass>, ModelError> {
    if path.contains(&id) {
        return Err(ModelError(format!(
            "cyclic composite material reference: {path:?} -> {id:?}"
        )));
    }
    path.push(id);
    let material = model
        .material(id)
        .ok_or_else(|| ModelError(format!("missing semantic material {id:?}")))?;
    let stack = &material.surface;
    let mut passes = Vec::new();
    for (step_index, step) in stack.steps.iter().enumerate() {
        let scoped = Arc::new(images.material(id, step_index as u32));
        let images = &scoped;
        let resolved_images = images.as_ref();
        let mut compiled = match step {
            MaterialStep::Mdlx(layer) => vec![compile_mdlx(model, layer, images)?],
            MaterialStep::M2(surface) => {
                vec![compile_m2(model, surface, resolved_images)?]
            }
            MaterialStep::Wmo(surface) => vec![compile_wmo(model, surface, resolved_images)?],
            MaterialStep::M3(m3::MaterialStep::Composite(composite)) => {
                let mut passes = Vec::new();
                for section in &composite.sections {
                    let mut section_passes =
                        compile_material(model, section.material, images, targets, path)?;
                    for pass in &mut section_passes {
                        let alpha = pass.material.base_color.alpha();
                        pass.animation.unweighted_opacity.get_or_insert(alpha);
                        pass.animation
                            .pass_weights
                            .push(section.map_multiplier.value);
                        pass.material
                            .base_color
                            .set_alpha(alpha * model.values()[section.map_multiplier.value]);
                        pass.material.depth_bias = composite.priority as f32;
                        if model.values()[section.map_multiplier.value] != 1.
                            || targets.scalar_changes(
                                model.values()[section.map_multiplier.value],
                                &section.map_multiplier.value,
                            )
                        {
                            if let AlphaMode::Mask(cutoff) = pass.material.alpha_mode {
                                pass.state.combiner.clip.x = cutoff;
                            }
                            if matches!(
                                pass.material.alpha_mode,
                                AlphaMode::Opaque | AlphaMode::Mask(_)
                            ) {
                                pass.material.alpha_mode = AlphaMode::Blend;
                                pass.state.blend = Some(Blend::Alpha);
                            }
                        }
                    }
                    passes.extend(section_passes);
                }
                if passes.len() > 1 {
                    for pass in &mut passes {
                        pass.diagnostics.push("M3 composite section order is retained in pass records, but Bevy render phases may reorder material passes; native within-composite ordering is not guaranteed".into());
                    }
                }
                passes
            }
            MaterialStep::Diablo3(surface) => vec![compile_diablo3(surface)],
            MaterialStep::Diablo4(shader) => {
                let mut pass = SurfacePass::default();
                pass.diagnostics.push(format!(
                    "Diablo IV shader map {} needs an application shader interpreter; texture semantic codes are unresolved", shader.shader_map
                ));
                vec![pass]
            }
            MaterialStep::M3(surface) => vec![compile_m3_with_targets(
                model,
                surface,
                resolved_images,
                targets,
            )?],
        };
        if !matches!(step, MaterialStep::M3(m3::MaterialStep::Composite(_))) {
            for pass in &mut compiled {
                pass.animation.images = images.color_images();
            }
        }
        passes.extend(compiled);
    }
    // Preserve the diagnostic placeholder for an unresolved external selection;
    // this is renderer fallback output, not an invented semantic shader pass.
    if stack.steps.is_empty()
        && matches!(model.appearance(), UniversalAppearance::D4(appearance) if appearance.slots().any(|(_, slot)| slot.variants().any(|(_, material)| material == id)))
    {
        let mut pass = SurfacePass::default();
        pass.diagnostics
            .push("Diablo IV external material shader is unresolved".into());
        passes.push(pass);
    }
    for pass in &mut passes {
        pass.animation.prune_static_bindings(targets);
        for diagnostic in &mut pass.diagnostics {
            *diagnostic = format!("material {id:?} ({}): {diagnostic}", material.name);
        }
    }
    path.pop();
    Ok(passes)
}

pub(super) fn image(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role(model, id, TextureRole::Color, images, diagnostics)
}

pub(super) fn image_role(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role_with_sampler(model, id, role, None, images, diagnostics)
}

pub(super) fn image_role_with_sampler(
    model: &Model<'_, Universal>,
    id: Option<Texture>,
    role: TextureRole,
    sampler: Option<Sampler>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
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
        diagnostics.push(format!(
            "texture {id:?} resource {:?} with role {role:?} is unresolved",
            model.image_resource(texture.resource)
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
            pass.diagnostics.push(format!(
                "blend operation {value:?} requires a custom pipeline; using alpha blending"
            ));
            AlphaMode::Blend
        }
        BlendOperations::Unknown(value) => {
            pass.diagnostics.push(format!(
                "unknown blend selector {value:?}; using alpha blending"
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

pub(super) fn transform(
    pass: &mut SurfacePass,
    transform: &TextureTransform,
    values: &PropertyValues,
) {
    let translation = values[transform.translation];
    let scale = values[transform.scale];
    let rotation = Quat::from_array(values[transform.rotation].0);
    let (x, y, z) = rotation.to_euler(EulerRot::XYZ);
    pass.material.uv_transform = Affine2::from_scale_angle_translation(
        Vec2::new(scale[0], scale[1]),
        z,
        Vec2::new(translation[0], translation[1]),
    );
    if x.abs() > 1e-5 || y.abs() > 1e-5 {
        pass.diagnostics
            .push("texture rotation outside the UV plane requires generated coordinates".into());
    }
}

fn compile_mdlx(
    model: &Model<'_, Universal>,
    surface: &mdlx::MaterialStep<'_>,
    images: &Arc<TextureImages>,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    let (state, binding, opacity, emissive_gain, hd_inputs) = match surface {
        mdlx::MaterialStep::Classic(step) => (
            &step.inputs.state,
            &step.inputs.binding,
            &step.inputs.opacity,
            &step.inputs.emissive_gain,
            None,
        ),
        mdlx::MaterialStep::ReforgedDefault(step) => (
            &step.inputs.state,
            &step.inputs.binding,
            &step.inputs.opacity,
            &step.inputs.emissive_gain,
            Some(step),
        ),
        mdlx::MaterialStep::Unresolved(step) => (
            &step.layer.state,
            &step.layer.binding,
            &step.layer.opacity,
            &step.layer.emissive_gain,
            None,
        ),
    };
    let declaration = surface.declaration();
    pass.sort_near = declaration.sort_near_z;
    pass.sort_far = declaration.sort_far_z;
    blend(&state.blend, &mut pass);
    let raster = &state.rasterization;
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
        model.values()[uv.rotation].0,
        model.values()[uv.scale],
    );
    pass.animation.centered_transform = true;
    pass.animation.transform = Some(uv.clone());
    if raster.controls.sphere_environment {
        pass.diagnostics
            .push("sphere-environment coordinates require a generated-coordinate shader".into());
    }
    if let Some(hd_inputs) = hd_inputs {
        let mut slots = from_fn(|_| None);
        slots[0] = Some(binding.image);
        for slot in &hd_inputs.textures {
            if let Some(binding) = slots.get_mut(slot.slot as usize) {
                *binding = Some(slot.binding.image);
            } else {
                pass.diagnostics.push(format!(
                    "DefaultUnit texture slot {} is unsupported",
                    slot.slot
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
        let hd = HdAnimation {
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
        pass.diagnostics.push("DefaultUnit Fresnel and authored environment reflection use the renderer's PBR approximation; game fidelity remains unverified".into());
    } else if let mdlx::MaterialStep::Unresolved(step) = surface {
        pass.diagnostics.push(format!(
            "extended layered shader {:?} needs its authored texture-slot and Fresnel interpreter",
            step.shader_name
        ));
    }
    if declaration.constant_color || declaration.sort_near_z || declaration.sort_far_z {
        pass.diagnostics.push(
            "constant-color or primitive-level sorting controls are not evaluated by this PBR pass"
                .into(),
        );
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

fn compile_diablo3(surface: &diablo3::MaterialStep<'_>) -> SurfacePass {
    let mut pass = SurfacePass::default();
    let color = &surface.uber.colors;
    pass.material.base_color = Color::linear_rgba(
        color.diffuse[0],
        color.diffuse[1],
        color.diffuse[2],
        color.diffuse[3],
    );
    pass.material.emissive =
        LinearRgba::rgb(color.emissive[0], color.emissive[1], color.emissive[2]);
    pass.diagnostics.push(format!("Diablo III shader map {} requires an application shader interpreter; authored constant diffuse/emission are displayed, texture-role codes and render passes remain unresolved", surface.uber.shader_map.id));
    pass
}
