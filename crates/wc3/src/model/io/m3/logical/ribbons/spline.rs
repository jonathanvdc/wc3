//! Authored ribbon spline controls, anchored to actual graph nodes.

use super::super::{materials::read_error, particles::context::Reader};
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::profiles::m3::properties::{Field, FieldPath};
use crate::model::{profiles::m3 as surface, NodeId};
use native::{Envelope, Ref};
use std::borrow::Cow;
pub(super) fn read<'view, E: Envelope>(
    reader: &mut Reader<'view, '_, '_, E>,
    nodes: &[NodeId],
    reference: Ref<nr::targets::SplineRibbon>,
) -> Result<Cow<'view, [surface::SplinePoint]>, Error> {
    let mut rows = Vec::new();
    for (index, row) in reader
        .source
        .resolve::<nr::SplineRibbon>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = FieldPath::root(Field::Ribbon)
            .child(Field::SplinePoints)
            .indexed(index);
        rows.push(surface::SplinePoint {
            node: *nodes.get(r.bone_index as usize).ok_or_else(|| {
                Error::new(
                    "m3.ribbon.spline.node",
                    "spline attachment exceeds skeleton",
                )
            })?,
            emission_offset: r.emission_offset,
            emission_vector: r.emission_vector,
            reserved: r.reserved,
            emission_vector_norm_factor: r.emission_vector_norm_factor,
            velocity_norm_factor: r.velocity_norm_factor,
            velocity: reader.property(&prefix.child(Field::Velocity), r.velocity),
            velocity_base_factor: reader.property(
                &prefix.child(Field::VelocityBaseFactor),
                r.velocity_base_factor,
            ),
            velocity_end_factor: reader.property(
                &prefix.child(Field::VelocityEndFactor),
                r.velocity_end_factor,
            ),
            yaw: reader.property(&prefix.child(Field::Yaw), r.yaw),
            pitch: reader.property(&prefix.child(Field::Pitch), r.pitch),
            yaw_variation: surface::VariationControl {
                kind: surface::values::Variation::from_raw((r.yaw_type).raw()),
                amplitude: reader.property(&prefix.child(Field::YawAmplitude), r.yaw_amplitude),
                frequency: reader.property(&prefix.child(Field::YawFrequency), r.yaw_frequency),
            },
            pitch_variation: surface::VariationControl {
                kind: surface::values::Variation::from_raw((r.pitch_type).raw()),
                amplitude: reader.property(&prefix.child(Field::PitchAmplitude), r.pitch_amplitude),
                frequency: reader.property(&prefix.child(Field::PitchFrequency), r.pitch_frequency),
            },
            velocity_variation: surface::VariationControl {
                kind: surface::values::Variation::from_raw(r.velocity_type.raw()),
                amplitude: reader.property(
                    &prefix.child(Field::VelocityAmplitude),
                    r.velocity_amplitude,
                ),
                frequency: reader.property(
                    &prefix.child(Field::VelocityFrequency),
                    r.velocity_frequency,
                ),
            },
        });
    }
    Ok(Cow::Owned(rows))
}

pub(super) fn write<E: Envelope>(
    points: &[surface::SplinePoint],
    writer: &mut super::super::particles::writer::Writer<'_, '_, E>,
    bones: &std::collections::BTreeMap<NodeId, u32>,
) -> Result<Ref<nr::targets::SplineRibbon>, Error> {
    use super::super::write::{animated, append};
    let mut rows = Vec::new();
    for r in points.iter() {
        rows.push(nr::SplineRibbon {
            bone_index: *bones
                .get(&r.node)
                .ok_or_else(|| Error::new("ribbon.spline.node", "missing spline anchor node"))?,
            emission_offset: r.emission_offset,
            emission_vector: r.emission_vector,
            reserved: r.reserved,
            emission_vector_norm_factor: r.emission_vector_norm_factor,
            velocity_norm_factor: r.velocity_norm_factor,
            velocity: animated(&r.velocity, writer.ids),
            velocity_base_factor: animated(&r.velocity_base_factor, writer.ids),
            velocity_end_factor: animated(&r.velocity_end_factor, writer.ids),
            yaw: animated(&r.yaw, writer.ids),
            pitch: animated(&r.pitch, writer.ids),
            yaw_type: nr::Variation::from_raw(r.yaw_variation.kind.raw()),
            yaw_amplitude: animated(&r.yaw_variation.amplitude, writer.ids),
            yaw_frequency: animated(&r.yaw_variation.frequency, writer.ids),
            pitch_type: nr::Variation::from_raw(r.pitch_variation.kind.raw()),
            pitch_amplitude: animated(&r.pitch_variation.amplitude, writer.ids),
            pitch_frequency: animated(&r.pitch_variation.frequency, writer.ids),
            velocity_type: nr::Variation::from_raw(r.velocity_variation.kind.raw()),
            velocity_amplitude: animated(&r.velocity_variation.amplitude, writer.ids),
            velocity_frequency: animated(&r.velocity_variation.frequency, writer.ids),
        });
    }
    Ok(append(writer.document, &rows)?.cast())
}
