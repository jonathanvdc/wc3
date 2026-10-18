//! Native event commands retain authored affine transforms and game parameters.
use super::super::GraphBuilder;
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{
    profiles::m3 as surface, ControllerId, Event, EventId, FeatureId, NodeId, Time, TimeSource,
};
use native::Envelope;
use std::collections::BTreeMap;
use zerovec::ZeroVec;
pub(super) type Cache = BTreeMap<(u32, u32), FeatureId>;
trait OptionalWord {
    fn optional(self) -> Option<u32>;
}
impl OptionalWord for () {
    fn optional(self) -> Option<u32> {
        None
    }
}
impl OptionalWord for u32 {
    fn optional(self) -> Option<u32> {
        Some(self)
    }
}
fn error(error: native::ReadError) -> Error {
    Error::new("m3.animation.event", error.to_string())
}
pub(super) fn read<'a, E: Envelope>(
    document: &'a native::Container<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    layer: ControllerId,
    nodes: &[NodeId],
    builder: &mut GraphBuilder<'a>,
    cache: &mut Cache,
) -> Result<Vec<EventId>, Error> {
    let mut features = Vec::new();
    let block = document
        .resolve::<nr::AnimationBlockEvent>(controller.sdev.cast())
        .map_err(error)?
        .get(selector as usize)
        .ok_or_else(|| Error::new("m3.animation.event", "missing block"))?
        .map_err(error)?;
    let times = document
        .resolve::<nr::I32>(block.timestamps.cast())
        .map_err(error)?;
    macro_rules! events {
        ($v:ty) => {{
            let rows = document
                .resolve::<nr::Event<$v>>(block.keys.cast())
                .map_err(error)?;
            if rows.len() != times.len() {
                return Err(Error::new(
                    "m3.animation.event",
                    "timestamp/event count differs",
                ));
            }
            for (index, (time, row)) in times.iter().zip(rows.iter()).enumerate() {
                let time = time.map_err(error)?.value;
                let row = row.map_err(error)?;
                let name = document
                    .string(row.name)
                    .map_err(error)?
                    .unwrap_or_default();
                let anchor =
                    if row.bone_index == u16::MAX {
                        None
                    } else {
                        Some(*nodes.get(usize::from(row.bone_index)).ok_or_else(|| {
                            Error::new("m3.animation.event", "missing bone anchor")
                        })?)
                    };
                let key = (block.keys.index, index as u32);
                let feature = if let Some(feature) = cache.get(&key).copied() {
                    feature
                } else {
                    let owner = if let Some(anchor) = anchor {
                        anchor
                    } else {
                        builder.add_named_node("")?
                    };
                    let payload = surface::EventCommand {
                        event_type: row.event_type,
                        transform: row.transform,
                        options: document
                            .string(row.option_string)
                            .map_err(error)?
                            .unwrap_or_default()
                            .into(),
                        render_target_channel: row.rtt_channel_index.optional(),
                        parameter: row.extra_parameter.optional(),
                        control: row.unknown,
                    };
                    let feature =
                        builder.add_feature(owner, surface::Feature::Event((payload).into()))?;
                    cache.insert(key, feature);
                    feature
                };
                let occurrence = builder.add_controller_event(
                    layer,
                    Event {
                        time: Time::seconds(f64::from(time) / 1000.)?,
                        clock: TimeSource::Timeline,
                        name: name.into(),
                        node: anchor,
                        feature: Some(feature),
                        payload: ZeroVec::new(),
                    },
                )?;
                features.push(occurrence);
            }
        }};
    }
    if block.keys.entries != 0 {
        let revision = document
            .referenced_section(block.keys)
            .map_err(error)?
            .ok_or_else(|| Error::new("m3.animation.event", "missing event section"))?
            .revision();
        match revision {
            0 => events!(nr::V0),
            1 => events!(nr::V1),
            2 => events!(nr::V2),
            _ => {
                return Err(Error::new(
                    "m3.animation.event",
                    "unsupported event revision",
                ));
            }
        }
    }
    Ok(features)
}
