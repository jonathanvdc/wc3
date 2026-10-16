//! Native-family surface interpretation into PBR passes and explicit states.
use super::super::textures::{is_default_unit, TextureImages, TextureRole};
use super::model::compile_model_shader;
use super::programmed::compile_programmed_with_targets;
use super::world::compile_world;
use super::{layered_uv, Blend, HdAnimation, ModelMaterial, SurfacePass};
use crate::animation_targets::AnimationTargets;
use crate::error::ModelError;
use bevy::material::OpaqueRendererMethod;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::{array::from_fn, collections::HashMap, fmt::Debug, sync::Arc};
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{diablo3, layered, programmed, SurfaceSet};
use wc3::model::profiles::Universal;
use wc3::model::{Animated, ColorSpace, MaterialId, Model, Sampler, TextureId, TextureTransform};

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
    let resolved_images = images.as_ref();
    let material = model
        .material(id)
        .ok_or_else(|| ModelError(format!("missing semantic material {id:?}")))?;
    let mut passes = match &material.surface {
        SurfaceSet::Layered(surface) => surface
            .layers
            .iter()
            .map(|layer| compile_layer(model, surface, layer, images))
            .collect::<Result<Vec<_>, _>>()?,
        SurfaceSet::ModelShader(surface) => {
            vec![compile_model_shader(model, surface, resolved_images)?]
        }
        SurfaceSet::World(surface) => vec![compile_world(model, surface, resolved_images)?],
        SurfaceSet::Programmed(surface) => {
            if let programmed::Material::Composite(composite) = &**surface {
                let mut passes = Vec::new();
                for section in &composite.sections {
                    let mut section_passes =
                        compile_material(model, section.material, images, targets, path)?;
                    for pass in &mut section_passes {
                        let alpha = pass.material.base_color.alpha();
                        pass.animation.unweighted_opacity.get_or_insert(alpha);
                        pass.animation.pass_weights.push(Animated {
                            default: section.map_multiplier.initial,
                            binding: section
                                .map_multiplier
                                .binding
                                .clone()
                                .filter(|_| {
                                    targets.scalar_changes(
                                        section.map_multiplier.initial,
                                        &section.map_multiplier.binding,
                                    )
                                })
                                .map(Into::into),
                        });
                        pass.material
                            .base_color
                            .set_alpha(alpha * section.map_multiplier.initial);
                        pass.material.depth_bias = composite.priority as f32;
                        if section.map_multiplier.initial != 1.
                            || targets.scalar_changes(
                                section.map_multiplier.initial,
                                &section.map_multiplier.binding,
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
            } else {
                vec![compile_programmed_with_targets(
                    model,
                    surface,
                    resolved_images,
                    targets,
                )?]
            }
        }
        SurfaceSet::D3(surface) => vec![compile_diablo3(surface)],
        SurfaceSet::D4(surface) => {
            let mut pass = SurfacePass::default();
            pass.diagnostics.push(match &surface.shader {
                Some(shader) => format!(
                    "Diablo IV shader map {} needs an application shader interpreter; texture semantic codes are unresolved",
                    shader.shader_map
                ),
                None => "Diablo IV external material shader is unresolved".into(),
            });
            vec![pass]
        }
    };
    for pass in &mut passes {
        pass.animation.prune_static_bindings(targets);
        pass.animation.images = images.color_images().clone();
        for diagnostic in &mut pass.diagnostics {
            *diagnostic = format!("material {id:?} ({}): {diagnostic}", material.name);
        }
    }
    path.pop();
    Ok(passes)
}

pub(super) fn image(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role(model, id, TextureRole::Color, images, diagnostics)
}

pub(super) fn image_role(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role_with_sampler(model, id, role, None, images, diagnostics)
}

pub(super) fn image_role_with_sampler(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    role: TextureRole,
    sampler: Option<Sampler>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let Some(id) = id else { return Ok(None) };
    let texture = model
        .texture(id)
        .ok_or_else(|| ModelError(format!("missing semantic texture {id:?}")))?;
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
    if let ColorSpace::Unknown(code) = texture.color_space {
        diagnostics.push(format!(
            "texture {id:?} transfer-function selector {code} is unresolved; using role {role:?}"
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

pub(super) fn transform(pass: &mut SurfacePass, transform: &TextureTransform) {
    let translation = transform.translation.default;
    let scale = transform.scale.default;
    let rotation = Quat::from_array(transform.rotation.default.0);
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

fn compile_layer(
    model: &Model<'_, Universal>,
    surface: &layered::LayeredSurface<'_>,
    layer: &layered::Layer,
    images: &Arc<TextureImages>,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    blend(&layer.state.blend, &mut pass);
    let raster = &layer.state.rasterization;
    rasterization(
        &mut pass,
        surface.two_sided || raster.faces,
        raster.lighting,
    );
    depth(&mut pass, raster.depth.test, raster.depth.write);
    pass.coordinate = layer.binding.coordinates.source;
    pass.material.base_color = Color::linear_rgba(1.0, 1.0, 1.0, layer.opacity.default);
    pass.material.base_color_texture = image(
        model,
        layer.binding.image.default,
        images,
        &mut pass.diagnostics,
    )?;
    pass.animation.opacity = Some(layer.opacity.clone());
    pass.animation.texture = Some(layer.binding.image.clone());
    pass.animation.emissive_gain = layer.emissive_gain.clone();
    pass.material.depth_bias = surface.priority_plane as f32;
    if let Some(id) = layer.binding.coordinates.transform {
        let uv = model
            .texture_transform(id)
            .ok_or_else(|| ModelError(format!("missing texture transform {id:?}")))?;
        pass.material.uv_transform = layered_uv(
            uv.translation.default,
            uv.rotation.default.0,
            uv.scale.default,
        );
        pass.animation.centered_transform = true;
        pass.animation.transform = Some(uv.clone());
    }
    if raster.controls.sphere_environment {
        pass.diagnostics
            .push("sphere-environment coordinates require a generated-coordinate shader".into());
    }
    if is_default_unit(surface, layer) {
        let mut slots = from_fn(|_| None);
        slots[0] = Some(layer.binding.image.clone());
        for slot in &layer.textures {
            if let Some(binding) = slots.get_mut(slot.slot as usize) {
                *binding = Some(slot.binding.image.clone());
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
                image_role(model, binding.default, role, images, &mut pass.diagnostics)?;
            }
        }
        let hd = HdAnimation {
            slots,
            fresnel: layer.fresnel.clone(),
            gain: layer.emissive_gain.clone(),
            images: images.clone(),
        };
        let mut compiled = ModelMaterial {
            base: pass.material,
            extension: pass.state,
        };
        hd.apply(None, &mut compiled);
        compiled.base.reflectance = StandardMaterial::default().reflectance;
        pass.material = compiled.base;
        pass.state = compiled.extension;
        if matches!(layer.state.blend, BlendOperations::AlphaKey(_)) {
            pass.material.alpha_mode = AlphaMode::Mask(0.75);
        }
        pass.material.opaque_render_method = OpaqueRendererMethod::Forward;
        // HD slot zero owns diffuse selection; the classic binding must not
        // overwrite it transiently and mark an unchanged asset as modified.
        pass.animation.texture = None;
        pass.animation.hd = Some(hd);
        pass.diagnostics.push("DefaultUnit Fresnel and authored environment reflection use the renderer's PBR approximation; game fidelity remains unverified".into());
    } else if (!surface.shader.is_empty() && surface.shader != "Shader_SD_FixedFunction")
        || layer
            .shader
            .is_some_and(|shader| shader != 0 && shader != 2)
        || !layer.textures.is_empty()
        || layer.fresnel.is_some()
    {
        pass.diagnostics.push(format!(
            "extended layered shader {:?} needs its authored texture-slot and Fresnel interpreter",
            surface.shader
        ));
    }
    if surface.constant_color || surface.sort_near_z || surface.sort_far_z {
        pass.diagnostics.push(
            "constant-color or primitive-level sorting controls are not evaluated by this PBR pass"
                .into(),
        );
    }
    if let Some(gain) = layer
        .emissive_gain
        .as_ref()
        .filter(|_| pass.animation.hd.is_none())
    {
        pass.material.emissive = LinearRgba::rgb(gain.default, gain.default, gain.default);
        pass.material.emissive_texture = pass.material.base_color_texture.clone();
    }
    Ok(pass)
}

fn compile_diablo3(surface: &diablo3::Material<'_>) -> SurfacePass {
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
