//! Common bodies are rebuilt from editable shader inputs and coefficients.
use crate::model::conversion::wem::materials::values::{
    read_blend_mode, read_combiner_op, read_composite_op, read_cull_mode, read_legacy_slot,
    read_pbr_slot, read_surface_channel, write_blend_mode, write_combiner_op, write_composite_op,
    write_cull_mode, write_legacy_slot, write_pbr_slot, write_surface_channel,
};
use crate::model::conversion::wem::materials::{features, input, one, put, rows};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::{
    Body, CombinerStage, CommonSurface, CompositeLayer, MaterialFlags, SlotInput,
};
use crate::model::semantic::TextureId;
use crate::model::wem::{records, InlineList, RawDocument};
use std::collections::BTreeMap;
pub(crate) fn read(
    document: &RawDocument<'_>,
    value: &records::CommonMaterial,
    textures: &[TextureId],
) -> Result<CommonSurface, Error> {
    let body = match value.body {
        records::MaterialBody::Composite(reference) => {
            let revision = document
                .referenced_section(reference)
                .map_err(|e| Error::new("wem.material", e.to_string()))?
                .ok_or_else(|| Error::new("wem.material", "missing composite body"))?
                .revision();
            let (
                layers,
                diffuse,
                emissive,
                specular,
                specular_exponent,
                environment,
                simulate_roughness,
            ) = match revision {
                1 => {
                    let body: records::CompositeBody<records::V1> = one(document, reference)?;
                    (
                        body.layers,
                        body.diffuse_factor,
                        body.emissive_factor,
                        body.specular_factor,
                        body.specular_exponent,
                        body.environment_factor,
                        false,
                    )
                }
                2 => {
                    let body: records::CompositeBody<records::V2> = one(document, reference)?;
                    (
                        body.layers,
                        body.diffuse_factor,
                        body.emissive_factor,
                        body.specular_factor,
                        body.specular_exponent,
                        body.environment_factor,
                        body.simulate_roughness.get(),
                    )
                }
                _ => {
                    return Err(Error::new(
                        "wem.material.composite",
                        "unsupported shader revision",
                    ));
                }
            };
            let layers = rows::<records::CompositeLayer>(document, layers.cast())?
                .into_iter()
                .map(|layer| {
                    Ok(CompositeLayer::new(
                        input::read(&layer.input, textures)?,
                        read_surface_channel(layer.target),
                        read_composite_op(layer.op),
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Body::Composite {
                layers,
                diffuse,
                emissive,
                specular,
                specular_exponent,
                environment,
                simulate_roughness,
            }
        }
        records::MaterialBody::Combiners(reference) => {
            let body: records::CombinersBody = one(document, reference)?;
            let stages = rows::<records::CombinerStage>(document, body.stages.cast())?
                .into_iter()
                .map(|stage| {
                    Ok(CombinerStage::new(
                        input::read(&stage.input, textures)?,
                        read_combiner_op(stage.rgb),
                        read_combiner_op(stage.alpha),
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Body::Combiners {
                stages,
                diffuse: body.diffuse_factor,
                emissive: body.emissive_factor,
            }
        }
        records::MaterialBody::LegacyDeferred(reference) => {
            let body: records::LegacyDeferredBody = one(document, reference)?;
            let slots = body
                .slots
                .0
                .into_iter()
                .map(|slot| {
                    Ok(SlotInput::new(
                        read_legacy_slot(slot.slot),
                        input::read(&slot.input, textures)?,
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Body::LegacyDeferred {
                slots,
                diffuse: body.diffuse_factor,
                emissive: body.emissive_factor,
                specular: body.specular_factor,
                specular_exponent: body.specular_exponent,
                environment: body.environment_factor,
            }
        }
        records::MaterialBody::PbrDeferred(reference) => {
            let body: records::PbrDeferredBody = one(document, reference)?;
            let slots = body
                .slots
                .0
                .into_iter()
                .map(|slot| {
                    Ok(SlotInput::new(
                        read_pbr_slot(slot.slot),
                        input::read(&slot.input, textures)?,
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Body::PbrDeferred {
                slots,
                base_color: body.base_color_factor,
                metallic: body.metallic_factor,
                roughness: body.roughness_factor,
                emissive: body.emissive_factor,
            }
        }
        records::MaterialBody::Count => {
            return Err(Error::new(
                "wem.material",
                "shader count sentinel has no authored body",
            ));
        }
    };
    let layer_ids = body.layer_ids().collect::<Vec<_>>();
    let features = features::read(document, value.features, &layer_ids)?;
    Ok(CommonSurface {
        blend: read_blend_mode(value.blend),
        cull: read_cull_mode(value.cull),
        depth_test: value.depth.test.get(),
        depth_write: value.depth.write.get(),
        depth_bias: value.depth.bias,
        alpha_test_threshold: value.alpha_test_threshold,
        priority: value.priority_plane,
        flags: MaterialFlags::from_bits_retain(value.flags.bits()),
        body,
        features,
    })
}
pub(crate) fn write(
    document: &mut RawDocument<'_>,
    value: &CommonSurface,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<records::CommonMaterial, Error> {
    let body = match &value.body {
        Body::Composite {
            layers,
            diffuse,
            emissive,
            specular,
            specular_exponent,
            environment,
            simulate_roughness,
        } => {
            let layers = layers
                .iter()
                .map(|layer| {
                    Ok(records::CompositeLayer {
                        input: input::write(&layer.input, textures)?,
                        target: write_surface_channel(layer.target),
                        op: write_composite_op(layer.operation),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let layers = put(document, &layers)?;
            records::MaterialBody::Composite(put(
                document,
                &[records::CompositeBody::<records::V2> {
                    layers,
                    diffuse_factor: *diffuse,
                    emissive_factor: *emissive,
                    specular_factor: *specular,
                    specular_exponent: *specular_exponent,
                    environment_factor: *environment,
                    simulate_roughness: (*simulate_roughness).into(),
                }],
            )?)
        }
        Body::Combiners {
            stages,
            diffuse,
            emissive,
        } => {
            let stages = stages
                .iter()
                .map(|stage| {
                    Ok(records::CombinerStage {
                        input: input::write(&stage.input, textures)?,
                        rgb: write_combiner_op(stage.rgb),
                        alpha: write_combiner_op(stage.alpha),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let stages = put(document, &stages)?;
            records::MaterialBody::Combiners(put(
                document,
                &[records::CombinersBody {
                    stages,
                    diffuse_factor: *diffuse,
                    emissive_factor: *emissive,
                }],
            )?)
        }
        Body::LegacyDeferred {
            slots,
            diffuse,
            emissive,
            specular,
            specular_exponent,
            environment,
        } => {
            let slots = slots
                .iter()
                .map(|slot| {
                    Ok(records::LegacySlotBinding {
                        slot: write_legacy_slot(slot.slot),
                        input: input::write(&slot.input, textures)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            records::MaterialBody::LegacyDeferred(put(
                document,
                &[records::LegacyDeferredBody {
                    slots: InlineList(slots),
                    diffuse_factor: *diffuse,
                    emissive_factor: *emissive,
                    specular_factor: *specular,
                    specular_exponent: *specular_exponent,
                    environment_factor: *environment,
                }],
            )?)
        }
        Body::PbrDeferred {
            slots,
            base_color,
            metallic,
            roughness,
            emissive,
        } => {
            let slots = slots
                .iter()
                .map(|slot| {
                    Ok(records::PbrSlotBinding {
                        slot: write_pbr_slot(slot.slot),
                        input: input::write(&slot.input, textures)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            records::MaterialBody::PbrDeferred(put(
                document,
                &[records::PbrDeferredBody {
                    slots: InlineList(slots),
                    base_color_factor: *base_color,
                    metallic_factor: *metallic,
                    roughness_factor: *roughness,
                    emissive_factor: *emissive,
                }],
            )?)
        }
    };
    let layer_indices = value
        .body
        .layer_ids()
        .enumerate()
        .map(|(index, id)| {
            Ok((
                id,
                u32::try_from(index)
                    .map_err(|_| Error::new("wem.material.layers", "layer count exceeds u32"))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    let features = features::write(document, &value.features, &layer_indices)?;
    Ok(records::CommonMaterial {
        blend: write_blend_mode(value.blend),
        cull: write_cull_mode(value.cull),
        depth: records::DepthState {
            test: value.depth_test.into(),
            write: value.depth_write.into(),
            bias: value.depth_bias,
        },
        alpha_test_threshold: value.alpha_test_threshold,
        priority_plane: value.priority,
        flags: records::MaterialFlags::from_bits_retain(value.flags.bits()),
        body,
        features,
    })
}
