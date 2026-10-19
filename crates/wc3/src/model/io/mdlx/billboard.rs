//! Native declarations and their effective camera-facing operations.
use crate::model::conversion::{Context, Error};
use crate::model::properties::BillboardFlags;
use crate::model::{Billboard, BillboardAxis, BillboardFrame, BillboardScale};
pub(super) fn decode(flags: BillboardFlags) -> Billboard {
    if flags.enabled {
        Billboard::Spherical {
            frame: BillboardFrame::Yz,
            scale: BillboardScale::SignedFactors,
        }
    } else if flags.x {
        Billboard::Axial(BillboardAxis::X)
    } else if flags.y {
        Billboard::Axial(BillboardAxis::Y)
    } else if flags.z {
        Billboard::Axial(BillboardAxis::Z)
    } else {
        Billboard::None
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
        Billboard::Spherical { frame: BillboardFrame::Yz, scale: BillboardScale::SignedFactors } => flags.enabled = true,
        Billboard::Axial(BillboardAxis::X) => flags.x = true,
        Billboard::Axial(BillboardAxis::Y) => flags.y = true,
        Billboard::Axial(BillboardAxis::Z) => flags.z = true,
        _ => context.unsupported("node.billboard", "Warcraft III requires YZ spherical facing with signed scale factors or a single axial constraint")?,
    }
    Ok(flags)
}
