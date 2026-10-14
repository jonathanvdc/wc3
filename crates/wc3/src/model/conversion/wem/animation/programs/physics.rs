//! Recorded simulation recipes resolve identities independently from native bake storage.
use super::super::{super::text, routing};
use super::{
    codec::{error, read_words},
    inventory::NativePhysics,
};
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::semantic::{
    profiles::wem::animation::{self, AnyChannelId, RecordedPhysics, WorldForce},
    profiles::wem::physics::ForceChannels,
    NodeId, Values,
};
use crate::model::wem::{records, Bool, RawDocument, Ref};
use std::collections::HashMap;

pub(super) struct Pending<'a> {
    pub value: RecordedPhysics<'a>,
    pub source: Vec<((u32, u32), Values<'a, u32>)>,
    pub soft_source: Option<Vec<(u32, u32)>>,
}
pub(super) fn reals<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Real>,
) -> Result<Values<'a, f32>, Error> {
    let rows = document
        .resolve::<records::Real>(reference.cast())
        .map_err(error)?;
    for row in rows.iter() {
        if !row.map_err(error)?.value.is_finite() {
            return Err(error("nonfinite recorded physics measurement"));
        }
    }
    Ok(Values::view(rows.len(), move |i| {
        rows.get(i)
            .expect("validated row")
            .expect("validated value")
            .value
    }))
}
pub(super) fn channel_ids(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::U32>,
    ids: &HashMap<u32, AnyChannelId>,
) -> Result<Vec<AnyChannelId>, Error> {
    read_words(document, reference)?
        .iter()
        .map(|id| {
            ids.get(&id)
                .copied()
                .ok_or_else(|| error("bake names an undeclared channel UID"))
        })
        .collect()
}
fn curve<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Vector2>,
) -> Result<Values<'a, [f32; 2]>, Error> {
    let rows = document
        .resolve::<records::Vector2>(reference.cast())
        .map_err(error)?;
    for row in rows.iter() {
        let row = row.map_err(error)?;
        if !row.value.into_iter().all(f32::is_finite) {
            return Err(error("nonfinite simulation transition curve"));
        }
    }
    Ok(Values::view(rows.len(), move |i| {
        let row = rows
            .get(i)
            .expect("validated row")
            .expect("validated value");
        row.value
    }))
}
macro_rules! convention{
    ($value:expr,$name:ident,$($variant:ident),+)=>{match $value{$(records::$name::$variant=>animation::$name::$variant,)+records::$name::Unknown(value)=>animation::$name::Unknown(value)}};
}
pub(super) fn settings<'a, D: records::ClipPhysicsDialect>(
    document: &'a RawDocument<'_>,
    source: &records::ClipPhysics<D>,
    nodes: &[NodeId],
) -> Result<RecordedPhysics<'a>, Error> {
    let mut world = Vec::new();
    for row in document
        .resolve::<records::BakeWorldForce>(source.world.cast())
        .map_err(error)?
        .iter()
    {
        let row = row.map_err(error)?;
        let centre_node = if row.centre_node == u32::MAX {
            None
        } else {
            Some(
                *nodes
                    .get(row.centre_node as usize)
                    .ok_or_else(|| error("world force names a missing node"))?,
            )
        };
        world.push(WorldForce {
            kind: convention!(row.kind, BakeWorldForceKind, Wind, Blast),
            start: row.start,
            end: row.end,
            ramp_in: row.ramp_in,
            ramp_out: row.ramp_out,
            strength: row.strength,
            heading: row.heading,
            rise: row.rise,
            gusts: row.gusts,
            gust_speed: row.gust_speed,
            radius: row.radius,
            height: row.height,
            centre_node,
            channels: ForceChannels::from_bits_retain(row.channels.bits()),
        });
    }
    Ok(RecordedPhysics {
        plays: convention!(source.plays, BakePlays, FromClip, Loops, Once),
        comes_from: convention!(source.comes_from, BakeFrom, Auto, Itself, FirstFrame, Clip),
        comes_from_clip: text::read(document, source.comes_from_clip)?,
        preheat: source.preheat,
        goes_to: convention!(source.goes_to, BakeTo, Auto, Nothing, Clip),
        goes_to_clip: text::read(document, source.goes_to_clip)?,
        matching: convention!(source.match_, LoopMatch, Crossfade, Offset, Off),
        matching_where: convention!(source.matching_where, MatchWhere, End, Start, Both),
        window: source.window,
        ease: convention!(
            source.ease,
            MatchEase,
            Smooth,
            Linear,
            EaseIn,
            EaseOut,
            Custom
        ),
        curve: curve(document, source.curve)?,
        tolerance: source.tolerance,
        mesh_measure: source.mesh_measure.0 != 0,
        key_rate: source.key_rate,
        step_rate: source.step_rate,
        floor: convention!(source.floor, BakeFloor, Grid, Off, FollowsRoot),
        travel: source.travel.0 != 0,
        world,
        baked: None,
    })
}

pub(super) struct BakeFields {
    pub source: Ref<records::targets::SubTrackContainer>,
    pub positions: Ref<records::targets::U32>,
    pub channels: Ref<records::targets::U32>,
    pub appended: Ref<records::targets::U32>,
    pub nodes: u32,
    pub keys: u32,
    pub restated: u32,
    pub max_error: f32,
    pub seam_before: f32,
    pub seam_after: f32,
    pub settling: Ref<records::targets::Real>,
    pub cloth_ids: Option<Ref<records::targets::U32>>,
    pub cloth_fits: Option<Ref<records::targets::Real>>,
    pub cloth_sizes: Option<Ref<records::targets::Real>>,
    pub soft_ids: Option<Ref<records::targets::U32>>,
    pub soft_lumps: Option<Ref<records::targets::U32>>,
    pub soft_keys: Option<Ref<records::targets::U32>>,
    pub soft_within: Option<Ref<records::targets::Real>>,
    pub soft_follows: Option<Ref<records::targets::Real>>,
    pub soft_push: Option<Ref<records::targets::Real>>,
    pub soft_at_end: Option<Ref<records::targets::Real>>,
    pub soft_channels: Option<Ref<records::targets::U32>>,
    pub soft_source: Option<Ref<records::targets::SubTrackContainer>>,
    pub made_settings: Option<bool>,
    pub fingerprints: [u64; 2],
}
pub(super) fn recipe<'a>(
    document: &'a RawDocument<'_>,
    source: &NativePhysics,
    nodes: &[NodeId],
    context: &mut Context,
) -> Result<(RecordedPhysics<'a>, Option<BakeFields>), Error> {
    use super::inventory::Field;
    macro_rules! decode {
        ($source:expr) => {{
            let source = $source;
            if source.mesh_measure.0 > 1 || source.travel.0 > 1 || source.baked.presence > 1 {
                context.issue(
                    "wem.physics.booleans",
                    IssueKind::Normalized,
                    "nonzero simulation booleans and report presence are represented as true",
                );
            }
            let value = settings(document, source, nodes)?;
            let bake = source.baked.value.as_ref().map(|bake| BakeFields {
                source: bake.source,
                positions: bake.positions,
                channels: bake.channels,
                appended: bake.appended,
                nodes: bake.nodes,
                keys: bake.keys,
                restated: bake.restated,
                max_error: bake.max_error,
                seam_before: bake.seam_before,
                seam_after: bake.seam_after,
                settling: bake.settling,
                cloth_ids: bake.cloth_ids.get(),
                cloth_fits: bake.cloth_fits.get(),
                cloth_sizes: bake.cloth_sizes.get(),
                soft_ids: bake.soft_ids.get(),
                soft_lumps: bake.soft_lumps.get(),
                soft_keys: bake.soft_keys.get(),
                soft_within: bake.soft_within.get(),
                soft_follows: bake.soft_follows.get(),
                soft_push: bake.soft_push.get(),
                soft_at_end: bake.soft_at_end.get(),
                soft_channels: bake.soft_channels.get(),
                soft_source: bake.soft_source.get(),
                made_settings: Field::get(&bake.made_settings).map(|value: Bool| value.0 != 0),
                fingerprints: [bake.inputs, bake.written],
            });
            Ok((value, bake))
        }};
    }
    match source {
        NativePhysics::Legacy(value) => decode!(value),
        NativePhysics::Cloth(value) => decode!(value),
        NativePhysics::Soft(value) => decode!(value),
    }
}

pub(super) fn report<'a>(
    document: &'a RawDocument<'_>,
    source: &BakeFields,
    ids: &HashMap<u32, AnyChannelId>,
    routing: &routing::ReadContext<'_, '_>,
) -> Result<animation::BakeReport<'a>, Error> {
    let system = routing.simulation.physics.map(|(id, _)| id);
    let cloth = if let Some(reference) = source.cloth_ids {
        let keys = read_words(document, reference)?;
        let fits = reals(
            document,
            source
                .cloth_fits
                .ok_or_else(|| error("missing cloth fit declaration"))?,
        )?;
        let sizes = reals(
            document,
            source
                .cloth_sizes
                .ok_or_else(|| error("missing cloth size declaration"))?,
        )?;
        if keys.len() != fits.len() || keys.len() != sizes.len() {
            return Err(error("cloth bake measurement arrays differ in length"));
        }
        let mut values = Vec::new();
        for i in 0..keys.len() {
            let uid = keys.get(i).expect("checked length");
            let cloth = routing
                .simulation
                .physics
                .and_then(|(_, maps)| maps.cloths.get(&uid))
                .copied()
                .ok_or_else(|| error("cloth bake names a missing simulation object"))?;
            values.push(animation::ClothMeasurement {
                cloth,
                fit: fits.get(i).expect("checked length"),
                size: sizes.get(i).expect("checked length"),
            });
        }
        Some(values)
    } else {
        None
    };
    let soft = if let Some(reference) = source.soft_ids {
        let keys = read_words(document, reference)?;
        let lumps = read_words(
            document,
            source
                .soft_lumps
                .ok_or_else(|| error("missing soft lump counts"))?,
        )?;
        let counts = read_words(
            document,
            source
                .soft_keys
                .ok_or_else(|| error("missing soft key counts"))?,
        )?;
        let within = reals(
            document,
            source
                .soft_within
                .ok_or_else(|| error("missing soft positional errors"))?,
        )?;
        let follows = reals(
            document,
            source
                .soft_follows
                .ok_or_else(|| error("missing soft following errors"))?,
        )?;
        let push = reals(
            document,
            source
                .soft_push
                .ok_or_else(|| error("missing soft push measurements"))?,
        )?;
        let at_end = reals(
            document,
            source
                .soft_at_end
                .ok_or_else(|| error("missing soft residual measurements"))?,
        )?;
        if [
            lumps.len(),
            counts.len(),
            within.len(),
            follows.len(),
            push.len(),
            at_end.len(),
        ]
        .into_iter()
        .any(|length| length != keys.len())
        {
            return Err(error("soft bake measurement arrays differ in length"));
        }
        let mut values = Vec::new();
        for i in 0..keys.len() {
            let uid = keys.get(i).expect("checked length");
            let body = routing
                .simulation
                .physics
                .and_then(|(_, maps)| maps.soft.bodies.get(&uid))
                .copied()
                .ok_or_else(|| error("soft bake names a missing simulation object"))?;
            values.push(animation::SoftMeasurement {
                body,
                lumps: lumps.get(i).expect("checked length"),
                keys: counts.get(i).expect("checked length"),
                within: within.get(i).expect("checked length"),
                follows: follows.get(i).expect("checked length"),
                push: push.get(i).expect("checked length"),
                at_end: at_end.get(i).expect("checked length"),
            });
        }
        Some(values)
    } else {
        None
    };
    Ok(animation::BakeReport {
        system,
        source: Vec::new(),
        channels: channel_ids(document, source.channels, ids)?,
        appended: channel_ids(document, source.appended, ids)?,
        nodes: source.nodes,
        keys: source.keys,
        restated: source.restated,
        max_error: source.max_error,
        seam_before: source.seam_before,
        seam_after: source.seam_after,
        settling: reals(document, source.settling)?,
        cloth,
        soft,
        soft_channels: source
            .soft_channels
            .map(|reference| channel_ids(document, reference, ids))
            .transpose()?,
        soft_source: source.soft_source.map(|_| Vec::new()),
        made_settings: source.made_settings,
    })
}
