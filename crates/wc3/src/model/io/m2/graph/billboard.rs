//! Native M2 camera-facing declarations.
use crate::model::conversion::{Context, Error};
use crate::model::error::Location;
use crate::model::properties::BillboardFlags;
use crate::model::{Billboard, BillboardAxis, BillboardFrame, BillboardScale};
pub(super) fn decode(flags: BillboardFlags) -> Billboard {
    match (flags.enabled, flags.x, flags.y, flags.z) {
        (true, false, false, false) => Billboard::Spherical {
            frame: BillboardFrame::Yz,
            scale: BillboardScale::LocalBasisLengths,
        },
        (false, true, false, false) => Billboard::ScreenAxis(BillboardAxis::X),
        (false, false, true, false) => Billboard::ScreenAxis(BillboardAxis::Y),
        (false, false, false, true) => Billboard::ScreenAxis(BillboardAxis::Z),
        _ => Billboard::None,
    }
}
pub(super) fn encode(
    value: Billboard,
    original: BillboardFlags,
    context: &mut Context,
) -> Result<BillboardFlags, Error> {
    if value == decode(original) {
        return Ok(original);
    }
    let mut flags = BillboardFlags::default();
    match value {
        Billboard::None => {},
        Billboard::Spherical { frame: BillboardFrame::Yz, scale: BillboardScale::LocalBasisLengths } => flags.enabled = true,
        Billboard::ScreenAxis(BillboardAxis::X) => flags.x = true,
        Billboard::ScreenAxis(BillboardAxis::Y) => flags.y = true,
        Billboard::ScreenAxis(BillboardAxis::Z) => flags.z = true,
        _ => context.unsupported(Location::new().field("node").field("billboard"), "M2 requires YZ spherical facing with local signed basis lengths or a camera screen axis")?,
    }
    Ok(flags)
}
