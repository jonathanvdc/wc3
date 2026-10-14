use crate::model::semantic::MaterialSlotId;
mod drip;
mod jiggle;
mod simple;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::soft::Settings;
use crate::model::semantic::{MeshId, NodeId};
use crate::model::wem::{records, RawDocument};
use std::collections::BTreeMap;
pub(super) struct ReadContext<'view, 'data, 'ids> {
    pub document: &'view RawDocument<'data>,
    pub nodes: &'ids [NodeId],
    pub meshes: &'ids [MeshId],
    pub slots: &'ids [MaterialSlotId],
}
pub(super) struct WriteContext<'view, 'data> {
    pub document: &'view mut RawDocument<'data>,
    pub nodes: &'view BTreeMap<NodeId, u32>,
    pub meshes: &'view BTreeMap<MeshId, u32>,
    pub slots: &'view BTreeMap<MaterialSlotId, u32>,
}
pub(super) fn read<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftEffect,
) -> Result<Settings<'a>, Error> {
    Ok(Settings {
        inflate: value
            .inflate
            .value
            .as_ref()
            .map(|v| simple::read_inflate(context, v))
            .transpose()?,
        bulge: value
            .bulge
            .value
            .as_ref()
            .map(|v| simple::read_bulge(context, v))
            .transpose()?,
        jiggle: value
            .jiggle
            .value
            .as_ref()
            .map(|v| jiggle::read(context, v))
            .transpose()?,
        wobble: value
            .wobble
            .value
            .as_ref()
            .map(|v| simple::read_wobble(context, v))
            .transpose()?,
        blobify: value
            .blobify
            .value
            .as_ref()
            .map(|v| simple::read_blobify(context, v))
            .transpose()?,
        drip: value
            .drip
            .value
            .as_ref()
            .map(|v| drip::read(context, v))
            .transpose()?,
        squash: value
            .squash
            .value
            .as_ref()
            .map(|v| simple::read_squash(context, v))
            .transpose()?,
        melt: value
            .melt
            .value
            .as_ref()
            .map(|v| simple::read_melt(context, v))
            .transpose()?,
        ripple: value
            .ripple
            .value
            .as_ref()
            .map(|v| simple::read_ripple(context, v))
            .transpose()?,
    })
}
pub(super) fn write(
    context: &mut WriteContext<'_, '_>,
    settings: &Settings<'_>,
    value: &mut records::SoftEffect,
) -> Result<(), Error> {
    value.inflate = settings
        .inflate
        .as_ref()
        .map(|v| simple::write_inflate(context, v))
        .transpose()?
        .into();
    value.bulge = settings
        .bulge
        .as_ref()
        .map(|v| simple::write_bulge(context, v))
        .transpose()?
        .into();
    value.jiggle = settings
        .jiggle
        .as_ref()
        .map(|v| jiggle::write(context, v))
        .transpose()?
        .into();
    value.wobble = settings
        .wobble
        .as_ref()
        .map(|v| simple::write_wobble(context, v))
        .transpose()?
        .into();
    value.blobify = settings
        .blobify
        .as_ref()
        .map(|v| simple::write_blobify(context, v))
        .transpose()?
        .into();
    value.drip = settings
        .drip
        .as_ref()
        .map(|v| drip::write(context, v))
        .transpose()?
        .into();
    value.squash = settings
        .squash
        .as_ref()
        .map(|v| simple::write_squash(context, v))
        .transpose()?
        .into();
    value.melt = settings
        .melt
        .as_ref()
        .map(|v| simple::write_melt(context, v))
        .transpose()?
        .into();
    value.ripple = settings
        .ripple
        .as_ref()
        .map(|v| simple::write_ripple(context, v))
        .transpose()?
        .into();
    Ok(())
}
