//! Typed payload schema lowering and selected-programme read validation.
use super::effect_compile::{error, EffectCompileError};
use super::LayerEvent;
use crate::model::scene::popcorn::graph::EventRoute;
use crate::model::scene::popcorn::graph::{BoundProgram, Callback};
use crate::model::scene::popcorn::payload::{Field as PayloadField, Layout as PayloadLayout};
use crate::model::scene::popcorn::{Address, EffectLayer, EffectLimitation, Instruction};
pub(super) fn payload_layout(event: &LayerEvent<'_>) -> Result<PayloadLayout, EffectCompileError> {
    PayloadLayout::new(
        event
            .payload
            .iter()
            .map(|field| PayloadField {
                name: field.name.into(),
                native_type: field.native_type.0,
                kind: field.native_type.value_type(),
                footprint: field.footprint_bytes,
                role: field.kind,
            })
            .collect(),
    )
    .map_err(error)
}
pub(super) fn payload_reads(
    bound: &BoundProgram,
    layout: &PayloadLayout,
) -> Result<Vec<usize>, EffectCompileError> {
    let mut reads = Vec::new();
    for operation in &bound.program.operations {
        if let Instruction::Call {
            function,
            arguments,
            ..
        } = &operation.instruction
        {
            if let Callback::Extract(kind) = bound.callbacks[*function as usize] {
                let Address::Constant(words) = arguments[0].address else {
                    return Err(error(
                        "payload extraction requires a checked constant root declaration slot",
                    ));
                };
                let slot = words[0] as usize;
                if layout
                    .fields
                    .get(slot)
                    .is_none_or(|field| field.kind != Some(kind))
                {
                    return Err(error(
                        "payload extraction root declaration slot or typed layout is invalid",
                    ));
                }
                if !reads.contains(&slot) {
                    reads.push(slot);
                }
            }
        }
    }
    Ok(reads)
}

pub(super) fn payload_policies(
    layers: &[EffectLayer],
    events: &[EventRoute],
) -> Vec<EffectLimitation> {
    let (mut interpolation, mut index, mut control, mut built) = (false, false, false, false);
    for layer in layers {
        for bound in Some(&layer.init).into_iter().chain(layer.evolve.as_ref()) {
            for operation in &bound.program.operations {
                if let Instruction::Call {
                    function,
                    arguments,
                    ..
                } = &operation.instruction
                {
                    match bound.callbacks[*function as usize] {
                        Callback::BuildElement => {
                            built = true;
                            if let Address::Constant(words) = arguments[3].address {
                                interpolation |= words[0] & 255 != 0;
                                index |= words[0] == 768;
                            } else {
                                interpolation = true;
                                index = true;
                            }
                        }
                        Callback::InitPayload => {
                            control |= !matches!(arguments[0].address, Address::Constant(words) if words[0] == 0);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let mut policies = Vec::new();
    if interpolation {
        policies.push(EffectLimitation::PayloadInterpolation);
    }
    if index {
        policies.push(EffectLimitation::PayloadIndex);
    }
    if control {
        policies.push(EffectLimitation::PayloadDraftControl);
    }
    if built
        && events
            .iter()
            .any(|event| event.payload.position.is_some() || event.payload.orientation.is_some())
    {
        policies.push(EffectLimitation::PayloadFrame);
    }
    policies
}
