//! Every authored bounds definition is emitted once in its actual detail group.
use super::{animation::AnimationIds, geometry_write::Division, Graph};
use crate::model::conversion::Error;
use crate::model::formats::m3::records;
use crate::model::{profiles::m3::Feature, Bounds, FeatureId, NodeId};
use std::collections::BTreeMap;
fn extent(value: Bounds) -> records::Extent {
    records::Extent {
        minimum: value.minimum,
        maximum: value.maximum,
        radius: value.sphere_radius,
    }
}
pub(super) fn write(
    graph: &Graph<'_>,
    divisions: &mut BTreeMap<u32, Division>,
    bones: &BTreeMap<NodeId, u32>,
    ids: &AnimationIds,
) -> Result<BTreeMap<FeatureId, (u32, u16)>, Error> {
    let mut indices = BTreeMap::new();
    for (id, feature) in graph.features() {
        let Feature::Extent(bounds) = &feature.payload else {
            continue;
        };
        let detail = bounds.detail_level.ok_or_else(|| {
            Error::new(
                "m3.bounds.detail",
                "an active bounds definition requires its authored detail group",
            )
        })?;
        let division = divisions.entry(detail).or_default();
        let index = u16::try_from(division.bounds.len()).map_err(|_| {
            Error::new(
                "m3.bounds.count",
                "detail-group bounds exceed 16-bit indexing capacity",
            )
        })?;
        let mapped = ids.get(bounds.minimum.target());
        division.bounds.push(records::MeshSection::<records::V1> {
            node_index: *bones.get(&feature.node).ok_or_else(|| {
                Error::new(
                    "m3.bounds.node",
                    "bounds definition anchor has no native bone",
                )
            })?,
            bounds: records::AnimationValue {
                interpolation: records::Interpolation::from_raw(
                    mapped
                        .map_or(bounds.control.interpolation, |mapped| mapped.interpolation)
                        .raw(),
                ),
                flags: records::AnimationFlags(
                    bounds.control.flags.bits()
                        | if mapped.is_some_and(|mapped| mapped.keyed) {
                            6
                        } else {
                            0
                        },
                ),
                animation_id: mapped.map_or(bounds.control.source_id, |mapped| mapped.id),
                initial_value: extent(bounds.initial),
                null_value: extent(bounds.reset),
                reserved: bounds.control.reserved,
            },
        });
        indices.insert(id, (detail, index));
    }
    Ok(indices)
}
