//! Native empty-extent encoding separated from finite semantic bounds.
use crate::model::conversion::Error;
use crate::model::formats::mdlx::GeosetExtent;
use crate::model::profiles::mdlx::Extent;
use crate::model::Bounds;

pub(super) fn read(source: GeosetExtent) -> Result<Extent, Error> {
    if source.bounds_radius == 0. && source.minimum == [1.; 3] && source.maximum == [-1.; 3] {
        return Ok(Extent::Empty);
    }
    let bounds = Bounds {
        minimum: source.minimum,
        maximum: source.maximum,
        sphere_radius: source.bounds_radius,
    };
    bounds.validate()?;
    Ok(Extent::Bounds(bounds))
}
pub(super) fn write(source: Extent) -> GeosetExtent {
    match source {
        Extent::Empty => GeosetExtent {
            minimum: [1.; 3],
            maximum: [-1.; 3],
            bounds_radius: 0.,
        },
        Extent::Bounds(bounds) => GeosetExtent {
            minimum: bounds.minimum,
            maximum: bounds.maximum,
            bounds_radius: bounds.sphere_radius,
        },
    }
}
