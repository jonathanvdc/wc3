//! Native M2 activation/lock declarations and the current facing approximation.
use crate::model::conversion::{Context, Error};
use crate::model::properties::BillboardFlags;
use crate::model::{Billboard, BillboardFrame, BillboardScale, EulerLocks};
pub(super) fn decode(flags: BillboardFlags) -> Billboard {
    if !flags.enabled {
        return Billboard::None;
    }
    let locks = match (flags.x, flags.y, flags.z) {
        (false, false, false) => {
            return Billboard::Spherical {
                frame: BillboardFrame::Yz,
                scale: BillboardScale::BasisLengths,
            }
        }
        (true, false, false) => EulerLocks::X,
        (false, true, false) => EulerLocks::Y,
        (false, false, true) => EulerLocks::Z,
        (true, true, false) => EulerLocks::Xy,
        (true, false, true) => EulerLocks::Xz,
        (false, true, true) => EulerLocks::Yz,
        (true, true, true) => return Billboard::None,
    };
    Billboard::Euler(locks)
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
        Billboard::Spherical { frame: BillboardFrame::Yz, scale: BillboardScale::BasisLengths } => flags.enabled = true,
        Billboard::Euler(locks) => {
            flags.enabled = true;
            [flags.x, flags.y, flags.z] = locks.axes();
        },
        _ => context.unsupported("node.billboard", "M2 requires YZ spherical facing with composed basis lengths or Euler lock approximations")?,
    }
    Ok(flags)
}
