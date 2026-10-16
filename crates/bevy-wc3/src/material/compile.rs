//! Native-family surface interpretation into PBR passes and explicit states.
use super::super::textures::{is_default_unit, world_sampler, TextureImages, TextureRole};
use super::{layered_uv, Blend, HdAnimation, ModelMaterial, SurfacePass};
use crate::error::ModelError;
use bevy::material::OpaqueRendererMethod;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::{array::from_fn, fmt::Debug, sync::Arc};
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{diablo3, layered, model_shader, programmed, world, SurfaceSet};
use wc3::model::profiles::Universal;
use wc3::model::values::model::ModelBlend;
use wc3::model::{Animated, ColorSpace, MaterialId, Model, Sampler, TextureId, TextureTransform};

pub(crate) fn compile_material_with_images(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: Arc<TextureImages>,
) -> Result<Vec<SurfacePass>, ModelError> {
    let resolved_images = images.as_ref();
    let material = model
        .material(id)
        .ok_or_else(|| ModelError(format!("missing semantic material {id:?}")))?;
    let mut passes = match &material.surface {
        SurfaceSet::Layered(surface) => surface
            .layers
            .iter()
            .map(|layer| compile_layer(model, surface, layer, &images))
            .collect::<Result<Vec<_>, _>>()?,
        SurfaceSet::ModelShader(surface) => {
            vec![compile_model_shader(model, surface, resolved_images)?]
        }
        SurfaceSet::World(surface) => vec![compile_world(model, surface, resolved_images)?],
        SurfaceSet::Programmed(surface) => {
            vec![compile_programmed(model, surface, resolved_images)?]
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
        pass.animation.images = images.color_images().clone();
        for diagnostic in &mut pass.diagnostics {
            *diagnostic = format!("material {id:?} ({}): {diagnostic}", material.name);
        }
    }
    Ok(passes)
}

fn image(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role(model, id, TextureRole::Color, images, diagnostics)
}

fn image_role(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role_with_sampler(model, id, role, None, images, diagnostics)
}

fn image_role_with_sampler(
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
            texture.resource
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

fn transform(pass: &mut SurfacePass, transform: &TextureTransform) {
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

fn compile_model_shader(
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
    if let Some(input) = surface.textures.first() {
        pass.material.base_color_texture =
            image(model, input.image.texture, images, &mut pass.diagnostics)?;
        if input.coordinates.source < 0 {
            pass.diagnostics.push(format!(
                "M2 generated texture coordinates {} need a shader interpreter",
                input.coordinates.source
            ));
        } else {
            pass.coordinate = input.coordinates.source as u32;
        }
        transform(&mut pass, &input.coordinates.transform);
        pass.animation.transform = Some(input.coordinates.transform.clone());
        pass.animation.weight = Some(input.contribution.clone());
        pass.material
            .base_color
            .set_alpha(surface.opacity.default * input.contribution.default);
    }
    if let Some(mode) = legacy_combiner(surface.shader, surface.textures.len()).filter(|_| {
        surface
            .textures
            .iter()
            .all(|input| input.coordinates.source >= 0)
    }) {
        pass.state.combiner.mode = mode;
        if let Some(second) = surface.textures.get(1) {
            pass.state.secondary =
                image(model, second.image.texture, images, &mut pass.diagnostics)?;
            pass.secondary_coordinate = u32::try_from(second.coordinates.source).ok();
            let mut scratch = SurfacePass::default();
            transform(&mut scratch, &second.coordinates.transform);
            pass.state.combiner.uv_transform = Mat3::from(scratch.material.uv_transform);
            pass.diagnostics.extend(scratch.diagnostics);
            pass.animation.secondary_transform = Some(second.coordinates.transform.clone());
            if second.contribution.property.is_some() || second.contribution.default != 1.0 {
                pass.diagnostics.push(
                    "independent M2 secondary texture contribution weight is not evaluated".into(),
                );
            }
        }
    } else if surface.shader != 0 || surface.textures.len() > 1 {
        pass.diagnostics.push(format!("M2 shader {} with {} texture inputs is approximated by its first input; authored combiners are not evaluated", surface.shader, surface.textures.len()));
    }
    Ok(pass)
}

/// Decodes established legacy selector bits, keeping external shader tables opaque.
/// Bit semantics were researched independently from the noclip M2 skin renderer;
/// the implementation here operates directly on our semantic texture inputs.
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

pub(super) fn compile_world(
    model: &Model<'_, Universal>,
    surface: &world::Material<'_>,
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
    if surface.uv_scroll != [[0.0; 2]; 2] {
        pass.diagnostics
            .push("WMO UV scroll requires runtime surface animation".into());
    }
    Ok(pass)
}

fn compile_programmed(
    model: &Model<'_, Universal>,
    surface: &programmed::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    if let programmed::Material::Standard(surface) = surface {
        blend(&surface.state.blend, &mut pass);
        if let Some(diffuse) = &surface.diffuse.input {
            pass.material.base_color_texture =
                image(model, diffuse.binding.image, images, &mut pass.diagnostics)?;
            let color = diffuse.binding.contribution.color.initial;
            pass.material.base_color = Color::linear_rgba(
                color[0],
                color[1],
                color[2],
                color[3] * diffuse.binding.contribution.map_alpha.initial,
            );
            pass.animation.rgba = Some(Animated {
                default: color,
                property: diffuse.binding.contribution.color.binding.clone(),
            });
            pass.animation.opacity = Some(Animated {
                default: diffuse.binding.contribution.map_alpha.initial,
                property: diffuse.binding.contribution.map_alpha.binding.clone(),
            });
        }
        if let Some(normal) = &surface.normals.base {
            pass.material.normal_map_texture = image_role(
                model,
                normal.binding.image,
                TextureRole::Normal,
                images,
                &mut pass.diagnostics,
            )?;
        }
        if let Some(emissive) = &surface.emissive.input.primary.input {
            pass.material.emissive_texture =
                image(model, emissive.binding.image, images, &mut pass.diagnostics)?;
            let color = emissive.binding.contribution.color.initial;
            let gain = surface.emissive.parameters;
            pass.material.emissive =
                LinearRgba::rgb(color[0] * gain, color[1] * gain, color[2] * gain);
            pass.animation.emission = Some(Animated {
                default: color,
                property: emissive.binding.contribution.color.binding.clone(),
            });
            pass.animation.emission_scale = gain;
        }
        pass.diagnostics.push("M3 standard surface uses primary diffuse, normal and emission roles; shader controls, layer composition, opacity programs and specular response need an interpreter".into());
    } else {
        pass.diagnostics
            .push("M3 material family requires a dedicated shader interpreter".into());
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
