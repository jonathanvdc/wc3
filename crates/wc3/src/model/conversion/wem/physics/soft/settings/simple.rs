use super::super::super::codec::{mapped, read_vectors, vectors, words};
use super::super::leaf::{node, node_ids, node_index, string, write_string};
use super::{ReadContext, WriteContext};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::soft::*;
use crate::model::wem::{records, Bool};
pub(super) fn read_inflate<'a>(
    _context: &ReadContext<'a, '_, '_>,
    value: &records::SoftInflate,
) -> Result<Inflate, Error> {
    Ok(Inflate {
        along: Along::from_raw(value.along.raw()),
        toward_place: value.toward_place.get(),
        round: value.round,
        keep_volume: value.keep_volume.get(),
        pulse: Pulse::from_raw(value.pulse.raw()),
        rate: value.rate,
        depth: value.depth,
        smooth: value.smooth,
    })
}
pub(super) fn write_inflate(
    _context: &mut WriteContext<'_, '_>,
    value: &Inflate,
) -> Result<records::SoftInflate, Error> {
    Ok(records::SoftInflate {
        along: records::SoftAlong::from_raw(value.along.raw()),
        toward_place: Bool::from(value.toward_place),
        round: value.round,
        keep_volume: Bool::from(value.keep_volume),
        pulse: records::SoftPulse::from_raw(value.pulse.raw()),
        rate: value.rate,
        depth: value.depth,
        smooth: value.smooth,
    })
}
pub(super) fn read_wobble<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftWobble,
) -> Result<Wobble, Error> {
    Ok(Wobble {
        pattern: WobblePattern::from_raw(value.pattern.raw()),
        speed: value.speed,
        size: value.size,
        overlap: value.overlap,
        axis: Axis::from_raw(value.axis.raw()),
        axis_bone: node(value.axis_bone, context.nodes)?,
        keep_volume: value.keep_volume.get(),
    })
}
pub(super) fn write_wobble(
    context: &mut WriteContext<'_, '_>,
    value: &Wobble,
) -> Result<records::SoftWobble, Error> {
    Ok(records::SoftWobble {
        pattern: records::SoftWobblePattern::from_raw(value.pattern.raw()),
        speed: value.speed,
        size: value.size,
        overlap: value.overlap,
        axis: records::SoftAxis::from_raw(value.axis.raw()),
        axis_bone: node_index(value.axis_bone, context.nodes)?,
        keep_volume: Bool::from(value.keep_volume),
    })
}
pub(super) fn read_blobify<'a>(
    _context: &ReadContext<'a, '_, '_>,
    value: &records::SoftBlobify,
) -> Result<Blobify, Error> {
    Ok(Blobify {
        roundness: value.roundness,
        swell: value.swell,
        keep_volume: value.keep_volume.get(),
    })
}
pub(super) fn write_blobify(
    _context: &mut WriteContext<'_, '_>,
    value: &Blobify,
) -> Result<records::SoftBlobify, Error> {
    Ok(records::SoftBlobify {
        roundness: value.roundness,
        swell: value.swell,
        keep_volume: Bool::from(value.keep_volume),
    })
}
pub(super) fn read_squash<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftSquash,
) -> Result<Squash, Error> {
    Ok(Squash {
        mode: SquashMode::from_raw(value.mode.raw()),
        axis: Axis::from_raw(value.axis.raw()),
        axis_bone: node(value.axis_bone, context.nodes)?,
        stretch: value.stretch,
        squash: value.squash,
        turning: value.turning,
        drag: value.drag,
        wind_up: value.wind_up,
        anchor: Anchor::from_raw(value.anchor.raw()),
        smoothing: value.smoothing,
    })
}
pub(super) fn write_squash(
    context: &mut WriteContext<'_, '_>,
    value: &Squash,
) -> Result<records::SoftSquash, Error> {
    Ok(records::SoftSquash {
        mode: records::SoftSquashMode::from_raw(value.mode.raw()),
        axis: records::SoftAxis::from_raw(value.axis.raw()),
        axis_bone: node_index(value.axis_bone, context.nodes)?,
        stretch: value.stretch,
        squash: value.squash,
        turning: value.turning,
        drag: value.drag,
        wind_up: value.wind_up,
        anchor: records::SoftAnchor::from_raw(value.anchor.raw()),
        smoothing: value.smoothing,
    })
}
pub(super) fn read_melt<'a>(
    _context: &ReadContext<'a, '_, '_>,
    value: &records::SoftMelt,
) -> Result<Melt, Error> {
    Ok(Melt {
        order: MeltOrder::from_raw(value.order.raw()),
        solidity: value.solidity,
        on_grid: value.on_grid.get(),
        floor: value.floor,
        puddle: value.puddle,
        uneven: value.uneven,
        backward: value.backward.get(),
        stays_in_decay: value.stays_in_decay.get(),
    })
}
pub(super) fn write_melt(
    _context: &mut WriteContext<'_, '_>,
    value: &Melt,
) -> Result<records::SoftMelt, Error> {
    Ok(records::SoftMelt {
        order: records::SoftMeltOrder::from_raw(value.order.raw()),
        solidity: value.solidity,
        on_grid: Bool::from(value.on_grid),
        floor: value.floor,
        puddle: value.puddle,
        uneven: value.uneven,
        backward: Bool::from(value.backward),
        stays_in_decay: Bool::from(value.stays_in_decay),
    })
}
pub(super) fn read_ripple<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftRipple,
) -> Result<Ripple<'a>, Error> {
    Ok(Ripple {
        along: Along::from_raw(value.along.raw()),
        plays: RipplePlays::from_raw(value.plays.raw()),
        from: ComesFrom::from_raw(value.from.raw()),
        event: string(context.document, value.event)?.into(),
        wavelength: value.wavelength,
        speed: value.speed,
        through: Through::from_raw(value.through.raw()),
        water: value.water.get(),
        envelope: value.envelope,
        fade_distance: value.fade_distance,
        fade_time: value.fade_time,
    })
}
pub(super) fn write_ripple(
    context: &mut WriteContext<'_, '_>,
    value: &Ripple<'_>,
) -> Result<records::SoftRipple, Error> {
    Ok(records::SoftRipple {
        along: records::SoftAlong::from_raw(value.along.raw()),
        plays: records::SoftRipplePlays::from_raw(value.plays.raw()),
        from: records::SoftComesFrom::from_raw(value.from.raw()),
        event: write_string(context.document, &value.event)?,
        wavelength: value.wavelength,
        speed: value.speed,
        through: records::SoftThrough::from_raw(value.through.raw()),
        water: Bool::from(value.water),
        envelope: value.envelope,
        fade_distance: value.fade_distance,
        fade_time: value.fade_time,
    })
}
pub(super) fn read_bulge<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftBulge,
) -> Result<Bulge<'a>, Error> {
    Ok(Bulge {
        along: Along::from_raw(value.along.raw()),
        shape: BulgeShape::from_raw(value.shape.raw()),
        keep_volume: value.keep_volume.get(),
        from: ComesFrom::from_raw(value.from.raw()),
        event: string(context.document, value.event)?.into(),
        rise: value.rise,
        fall: value.fall,
        travels: value.travels.get(),
        chain: node_ids(context.document, value.chain, context.nodes)?,
        line: read_vectors(context.document, value.line)?,
    })
}
pub(super) fn write_bulge(
    context: &mut WriteContext<'_, '_>,
    value: &Bulge<'_>,
) -> Result<records::SoftBulge, Error> {
    Ok(records::SoftBulge {
        along: records::SoftAlong::from_raw(value.along.raw()),
        shape: records::SoftBulgeShape::from_raw(value.shape.raw()),
        keep_volume: Bool::from(value.keep_volume),
        from: records::SoftComesFrom::from_raw(value.from.raw()),
        event: write_string(context.document, &value.event)?,
        rise: value.rise,
        fall: value.fall,
        travels: Bool::from(value.travels),
        chain: words(
            context.document,
            value
                .chain
                .iter()
                .map(|id| mapped(context.nodes, id, "soft chain"))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        line: vectors(context.document, &value.line)?,
    })
}
