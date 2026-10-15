//! Static and animated culling bounds are real behaviors anchored to skeleton joints.
use super::{
    hierarchy::Skeleton,
    materials::{read_error, revision},
    GraphBuilder,
};
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, Bounds, FeatureId};
use native::{Envelope, Ref};
fn bounds(value: nr::Extent) -> Bounds {
    Bounds {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.radius,
    }
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reference: Ref<nr::targets::MeshSection>,
    skeleton: &Skeleton<'a>,
    detail_level: u32,
    builder: &mut GraphBuilder<'a>,
) -> Result<Vec<FeatureId>, Error> {
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let mut features = Vec::new();
    macro_rules! read {
        ($v:ty) => {
            for row in source
                .resolve::<nr::MeshSection<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let node = *skeleton.nodes.get(r.node_index as usize).ok_or_else(|| {
                    Error::new("m3.mesh.bounds.node", "extent anchor exceeds skeleton")
                })?;
                let feature = builder.reserve_feature(node);
                let control = surface::AnimationControl {
                    interpolation: surface::values::Interpolation::from_raw(
                        r.bounds.interpolation.raw(),
                    ),
                    flags: surface::values::AnimationFlags(r.bounds.flags.bits()),
                    source_id: r.bounds.animation_id,
                    reserved: r.bounds.reserved,
                };
                builder.set_feature(
                    feature,
                    surface::Feature::Extent(
                        (surface::ExtentBinding {
                            detail_level: Some(detail_level),
                            initial: bounds(r.bounds.initial_value),
                            reset: bounds(r.bounds.null_value),
                            control,
                            minimum: surface::properties::EXTENT_MINIMUM.bind(feature).widen(),
                            maximum: surface::properties::EXTENT_MAXIMUM.bind(feature).widen(),
                            radius: surface::properties::EXTENT_RADIUS.bind(feature).widen(),
                        })
                        .into(),
                    ),
                )?;
                features.push(feature);
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        _ => {
            return Err(Error::new(
                "m3.mesh.bounds",
                "unsupported extent record revision",
            ));
        }
    }
    Ok(features)
}
