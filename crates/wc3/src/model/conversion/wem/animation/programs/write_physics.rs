//! Recorded physics lowers authored identities and recipes without reusing stale fingerprints.
use super::super::{super::text, routing};
use super::{
    codec::{append, error, words},
    controllers::ProgramLists,
    declarations::Written,
    inventory::NativePhysics,
};
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::semantic::profiles::wem::animation::{self, BakeReport, RecordedPhysics};
use crate::model::wem::{records, Bool, InlineOption, RawDocument, Ref};

fn reals(
    document: &mut RawDocument<'_>,
    values: impl IntoIterator<Item = f32>,
) -> Result<Ref<records::targets::Real>, Error> {
    Ok(append(
        document,
        &values
            .into_iter()
            .map(|value| records::Real { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn channel_words(
    document: &mut RawDocument<'_>,
    values: &[animation::AnyChannelId],
    declarations: &Written,
) -> Result<Ref<records::targets::U32>, Error> {
    words(
        document,
        values
            .iter()
            .map(|id| {
                declarations
                    .ids
                    .get(id)
                    .copied()
                    .ok_or_else(|| error("bake names a foreign declaration"))
            })
            .collect::<Result<Vec<_>, Error>>()?,
    )
}
macro_rules! convention {($value:expr,$name:ident,$($variant:ident),+)=>{match $value{$(animation::$name::$variant=>records::$name::$variant,)+animation::$name::Unknown(value)=>records::$name::Unknown(value)}};}
fn report(
    document: &mut RawDocument<'_>,
    source: &BakeReport<'_>,
    programs: &ProgramLists,
    declarations: &Written,
    routing: &routing::WriteContext<'_, '_>,
    context: &mut Context,
) -> Result<records::PhysicsBake<records::V7>, Error> {
    let physics = routing
        .simulation
        .physics
        .filter(|(id, _)| Some(*id) == source.system)
        .map(|(_, value)| value);
    let mut out = records::PhysicsBake::<records::V7> {
        source: programs.reference(
            &source
                .source
                .iter()
                .map(|value| value.program)
                .collect::<Vec<_>>(),
        )?,
        positions: words(
            document,
            source
                .source
                .iter()
                .flat_map(|value| value.positions.iter()),
        )?,
        channels: channel_words(document, &source.channels, declarations)?,
        appended: channel_words(document, &source.appended, declarations)?,
        nodes: source.nodes,
        keys: source.keys,
        restated: source.restated,
        max_error: source.max_error,
        seam_before: source.seam_before,
        seam_after: source.seam_after,
        settling: reals(document, source.settling.iter())?,
        ..Default::default()
    };
    if let Some(values) = &source.cloth {
        out.cloth_ids = words(
            document,
            values
                .iter()
                .map(|value| {
                    physics
                        .and_then(|maps| maps.cloths.get(&value.cloth))
                        .copied()
                        .ok_or_else(|| error("cloth bake measurement belongs to another world"))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        )?;
        out.cloth_fits = reals(document, values.iter().map(|value| value.fit))?;
        out.cloth_sizes = reals(document, values.iter().map(|value| value.size))?;
    }
    if let Some(values) = &source.soft {
        out.soft_ids = words(
            document,
            values
                .iter()
                .map(|value| {
                    physics
                        .and_then(|maps| maps.soft.bodies.get(&value.body))
                        .copied()
                        .ok_or_else(|| error("soft bake measurement belongs to another world"))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        )?;
        out.soft_lumps = words(document, values.iter().map(|value| value.lumps))?;
        out.soft_keys = words(document, values.iter().map(|value| value.keys))?;
        out.soft_within = reals(document, values.iter().map(|value| value.within))?;
        out.soft_follows = reals(document, values.iter().map(|value| value.follows))?;
        out.soft_push = reals(document, values.iter().map(|value| value.push))?;
        out.soft_at_end = reals(document, values.iter().map(|value| value.at_end))?;
    }
    if let Some(values) = &source.soft_channels {
        out.soft_channels = channel_words(document, values, declarations)?;
    }
    if let Some(values) = &source.soft_source {
        out.soft_source = programs.reference(values)?;
    }
    out.made_settings = Bool(u8::from(source.made_settings.unwrap_or(false)));
    context.issue(
        "wem.physics.bake.fingerprints",
        IssueKind::Normalized,
        "rebuilt programmes have no recomputed producer cache fingerprints",
    );
    Ok(out)
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    source: &RecordedPhysics<'_>,
    programs: &ProgramLists,
    declarations: &Written,
    routing: &routing::WriteContext<'_, '_>,
    context: &mut Context,
) -> Result<NativePhysics, Error> {
    let mut world = Vec::new();
    for value in &source.world {
        let centre_node = value
            .centre_node
            .map(|id| {
                routing
                    .simulation
                    .nodes
                    .get(&id)
                    .copied()
                    .ok_or_else(|| error("world force belongs to another model"))
            })
            .transpose()?
            .unwrap_or(u32::MAX);
        world.push(records::BakeWorldForce {
            kind: convention!(value.kind, BakeWorldForceKind, Wind, Blast),
            start: value.start,
            end: value.end,
            ramp_in: value.ramp_in,
            ramp_out: value.ramp_out,
            strength: value.strength,
            heading: value.heading,
            rise: value.rise,
            gusts: value.gusts,
            gust_speed: value.gust_speed,
            radius: value.radius,
            height: value.height,
            centre_node,
            channels: records::ForceChannels::from_bits_retain(value.channels.bits()),
        });
    }
    let curve = append(
        document,
        &source
            .curve
            .iter()
            .map(|value| records::Vector2 { value })
            .collect::<Vec<_>>(),
    )?
    .cast();
    let world = append(document, &world)?.cast();
    let baked = source
        .baked
        .as_ref()
        .map(|value| report(document, value, programs, declarations, routing, context))
        .transpose()?;
    let out = records::ClipPhysics::<records::V7> {
        plays: convention!(source.plays, BakePlays, FromClip, Loops, Once),
        comes_from: convention!(source.comes_from, BakeFrom, Auto, Itself, FirstFrame, Clip),
        comes_from_clip: text::write_bytes(document, source.comes_from_clip.as_bytes())?,
        preheat: source.preheat,
        goes_to: convention!(source.goes_to, BakeTo, Auto, Nothing, Clip),
        goes_to_clip: text::write_bytes(document, source.goes_to_clip.as_bytes())?,
        match_: convention!(source.matching, LoopMatch, Crossfade, Offset, Off),
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
        curve,
        tolerance: source.tolerance,
        mesh_measure: Bool(u8::from(source.mesh_measure)),
        key_rate: source.key_rate,
        step_rate: source.step_rate,
        floor: convention!(source.floor, BakeFloor, Grid, Off, FollowsRoot),
        travel: Bool(u8::from(source.travel)),
        world,
        baked: InlineOption::from(baked),
    };
    lower(out, source.baked.as_ref())
}

fn lower(
    source: records::ClipPhysics<records::V7>,
    report: Option<&BakeReport<'_>>,
) -> Result<NativePhysics, Error> {
    if report.is_some_and(|value| {
        value.soft.is_some()
            || value.soft_channels.is_some()
            || value.soft_source.is_some()
            || value.made_settings.is_some()
    }) {
        if report.is_some_and(|value| {
            value.cloth.is_none()
                || value.soft.is_none()
                || value.soft_channels.is_none()
                || value.soft_source.is_none()
                || value.made_settings.is_none()
        }) {
            return Err(error(
                "soft bake declarations must be authored together with cloth measurements",
            ));
        }
        return Ok(NativePhysics::Soft(source));
    }
    if report.is_some_and(|value| value.cloth.is_some()) {
        let baked = source
            .baked
            .value
            .map(|bake| records::PhysicsBake::<records::V6> {
                source: bake.source,
                positions: bake.positions,
                channels: bake.channels,
                appended: bake.appended,
                inputs: bake.inputs,
                written: bake.written,
                nodes: bake.nodes,
                keys: bake.keys,
                restated: bake.restated,
                max_error: bake.max_error,
                seam_before: bake.seam_before,
                seam_after: bake.seam_after,
                settling: bake.settling,
                cloth_ids: bake.cloth_ids,
                cloth_fits: bake.cloth_fits,
                cloth_sizes: bake.cloth_sizes,
                soft_ids: (),
                soft_lumps: (),
                soft_keys: (),
                soft_within: (),
                soft_follows: (),
                soft_push: (),
                soft_at_end: (),
                soft_channels: (),
                soft_source: (),
                made_settings: (),
            });
        return Ok(NativePhysics::Cloth(records::ClipPhysics::<records::V6> {
            plays: source.plays,
            comes_from: source.comes_from,
            comes_from_clip: source.comes_from_clip,
            preheat: source.preheat,
            goes_to: source.goes_to,
            goes_to_clip: source.goes_to_clip,
            match_: source.match_,
            matching_where: source.matching_where,
            window: source.window,
            ease: source.ease,
            curve: source.curve,
            tolerance: source.tolerance,
            mesh_measure: source.mesh_measure,
            key_rate: source.key_rate,
            step_rate: source.step_rate,
            floor: source.floor,
            travel: source.travel,
            world: source.world,
            baked: InlineOption::from(baked),
        }));
    }
    let baked = source
        .baked
        .value
        .map(|bake| records::PhysicsBake::<records::V5> {
            source: bake.source,
            positions: bake.positions,
            channels: bake.channels,
            appended: bake.appended,
            inputs: bake.inputs,
            written: bake.written,
            nodes: bake.nodes,
            keys: bake.keys,
            restated: bake.restated,
            max_error: bake.max_error,
            seam_before: bake.seam_before,
            seam_after: bake.seam_after,
            settling: bake.settling,
            cloth_ids: (),
            cloth_fits: (),
            cloth_sizes: (),
            soft_ids: (),
            soft_lumps: (),
            soft_keys: (),
            soft_within: (),
            soft_follows: (),
            soft_push: (),
            soft_at_end: (),
            soft_channels: (),
            soft_source: (),
            made_settings: (),
        });
    Ok(NativePhysics::Legacy(records::ClipPhysics::<records::V5> {
        plays: source.plays,
        comes_from: source.comes_from,
        comes_from_clip: source.comes_from_clip,
        preheat: source.preheat,
        goes_to: source.goes_to,
        goes_to_clip: source.goes_to_clip,
        match_: source.match_,
        matching_where: source.matching_where,
        window: source.window,
        ease: source.ease,
        curve: source.curve,
        tolerance: source.tolerance,
        mesh_measure: source.mesh_measure,
        key_rate: source.key_rate,
        step_rate: source.step_rate,
        floor: source.floor,
        travel: source.travel,
        world: source.world,
        baked: InlineOption::from(baked),
    }))
}
