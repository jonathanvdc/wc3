//! Structured native extents become one feature with three editable typed channels.
use super::super::GraphBuilder;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{
    profiles::m3 as surface, Channel, Clock, ControllerId, Curve, FeatureId, Interpolation, Time,
    Track, Values,
};
use native::Envelope;

fn error(error: native::ReadError) -> Error {
    Error::new("m3.animation.extent", error.to_string())
}
pub(super) fn read<'a, E: Envelope + Send + Sync>(
    document: &'a native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    identity: u32,
    layer: ControllerId,
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
) -> Result<Vec<FeatureId>, Error> {
    let block = document
        .resolve::<nr::AnimationBlockExtent>(controller.sdmb.cast())
        .map_err(error)?
        .get((selector & 0xffff) as usize)
        .ok_or_else(|| Error::new("m3.animation.extent", "missing block"))?
        .map_err(error)?;
    let times = document
        .resolve::<nr::I32>(block.timestamps.cast())
        .map_err(error)?;
    let extents = document
        .resolve::<nr::ExtentRecord>(block.keys.cast())
        .map_err(error)?;
    if times.len() != extents.len() {
        return Err(Error::new(
            "m3.animation.extent",
            "timestamp/value count differs",
        ));
    }
    for row in times.iter() {
        row.map_err(error)?;
    }
    for row in extents.iter() {
        let extent = row.map_err(error)?.value;
        if !extent
            .minimum
            .iter()
            .chain(extent.maximum.iter())
            .all(|v| v.is_finite())
            || !extent.radius.is_finite()
            || extent.radius < 0.
            || (0..3).any(|i| extent.minimum[i] > extent.maximum[i])
        {
            return Err(Error::new("m3.animation.extent", "invalid animated bounds"));
        }
    }
    let existing: Vec<_> = builder
        .model()
        .features()
        .filter_map(|(id, feature)| match &feature.payload {
            surface::Feature::Extent(value) if value.control.source_id == identity => {
                Some((id, (**value).clone()))
            }
            _ => None,
        })
        .collect();
    if existing.is_empty() {
        return Err(Error::new(
            "m3.animation.extent",
            "animated bounds have no declared extent destination",
        ));
    }
    let time_values = Values::view(times.len(), move |i| {
        Time::seconds(f64::from(times.get(i).unwrap().unwrap().value) / 1000.).unwrap()
    });
    let mut features = Vec::new();
    for (feature, binding) in existing {
        let mode = match binding.control.interpolation {
            surface::values::Interpolation::Constant => Interpolation::Step,
            surface::values::Interpolation::Linear => Interpolation::Linear,
            _ => {
                context.unsupported(
                    "m3.animation.extent.interpolation",
                    "extent has an unknown interpolation control",
                )?;
                continue;
            }
        };
        let minimum = binding.minimum.clone();
        let maximum = binding.maximum.clone();
        let radius = binding.radius.clone();
        let min_rows = extents.clone();
        let max_rows = extents.clone();
        let radius_rows = extents.clone();
        builder.add_controller_channel(
            layer,
            Channel::Vector3(Track {
                target: minimum.clone(),
                curve: Curve::from_samples(
                    time_values.clone(),
                    Values::view(min_rows.len(), move |i| {
                        min_rows.get(i).unwrap().unwrap().value.minimum
                    }),
                    mode,
                )?,
                clock: Clock::Timeline,
            }),
        )?;
        builder.add_controller_channel(
            layer,
            Channel::Vector3(Track {
                target: maximum.clone(),
                curve: Curve::from_samples(
                    time_values.clone(),
                    Values::view(max_rows.len(), move |i| {
                        max_rows.get(i).unwrap().unwrap().value.maximum
                    }),
                    mode,
                )?,
                clock: Clock::Timeline,
            }),
        )?;
        builder.add_controller_channel(
            layer,
            Channel::Scalar(Track {
                target: radius.clone(),
                curve: Curve::from_samples(
                    time_values.clone(),
                    Values::view(radius_rows.len(), move |i| {
                        radius_rows.get(i).unwrap().unwrap().value.radius
                    }),
                    mode,
                )?,
                clock: Clock::Timeline,
            }),
        )?;
        builder.set_feature(feature, surface::Feature::Extent((binding).into()))?;
        features.push(feature);
    }
    Ok(features)
}
