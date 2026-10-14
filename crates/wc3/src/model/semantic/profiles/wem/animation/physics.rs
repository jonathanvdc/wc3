//! Recorded simulation recipes and editable restoration programmes.
use super::super::physics::{ClothId, ForceChannels, SoftBodyId};
use super::{
    AnyChannelId, BakeFloor, BakeFrom, BakePlays, BakeTo, BakeWorldForceKind, LoopMatch, MatchEase,
    MatchWhere,
};
use crate::model::semantic::{ControllerId, Link, NodeId, SystemId, ValidateLinks, Values};
use std::borrow::Cow;

/// Simulation settings recorded for one playback clip.
#[derive(Clone, Debug, Default)]
pub struct RecordedPhysics<'a> {
    /// Playback convention independently of the current clip's looping flag.
    pub plays: BakePlays,
    /// Pose or animation chosen before simulation.
    pub comes_from: BakeFrom,
    /// Explicit preceding animation name.
    pub comes_from_clip: Cow<'a, str>,
    /// Whole preheat cycles.
    pub preheat: u8,
    /// Pose or animation chosen after simulation.
    pub goes_to: BakeTo,
    /// Explicit following animation name.
    pub goes_to_clip: Cow<'a, str>,
    /// Loop boundary alignment convention.
    pub matching: LoopMatch,
    /// Boundaries participating in matching.
    pub matching_where: MatchWhere,
    /// Transition width relative to duration.
    pub window: f32,
    /// Transition easing convention.
    pub ease: MatchEase,
    /// Authored normalized custom transition curve.
    pub curve: Values<'a, [f32; 2]>,
    /// Recorded positional approximation tolerance.
    pub tolerance: f32,
    /// Measures deformation on mesh vertices.
    pub mesh_measure: bool,
    /// Generated animation keys per second.
    pub key_rate: u8,
    /// Simulation steps per second.
    pub step_rate: u8,
    /// Floor reference convention.
    pub floor: BakeFloor,
    /// Carries the model according to movement speed.
    pub travel: bool,
    /// Ordered authored world forces.
    pub world: Vec<WorldForce>,
    /// Saved editable programmes and recorded error measurements.
    pub baked: Option<BakeReport<'a>>,
}
/// World force applied while producing recorded physics.
#[derive(Clone, Debug, Default)]
pub struct WorldForce {
    /// Wind or blast convention.
    pub kind: BakeWorldForceKind,
    /// Start time.
    pub start: f32,
    /// End time.
    pub end: f32,
    /// Ramp-in duration.
    pub ramp_in: f32,
    /// Ramp-out duration.
    pub ramp_out: f32,
    /// Force strength.
    pub strength: f32,
    /// Heading angle.
    pub heading: f32,
    /// Vertical rise.
    pub rise: f32,
    /// Gust amplitude.
    pub gusts: f32,
    /// Gust speed.
    pub gust_speed: f32,
    /// Spatial radius.
    pub radius: f32,
    /// Spatial height.
    pub height: f32,
    /// Optional actual scene node anchoring the centre.
    pub centre_node: Option<NodeId>,
    /// Local and world force-channel masks selecting affected simulation objects.
    pub channels: ForceChannels,
}
/// Original ordered programme and each saved curve's historical insertion position.
#[derive(Clone, Debug)]
pub struct Restoration<'a> {
    /// Actual independently editable saved programme.
    pub program: ControllerId,
    /// Original curve ordinals in that programme, never file offsets or section indices.
    pub positions: Values<'a, u32>,
}
/// Error measurements for one simulated cloth object.
#[derive(Clone, Debug)]
pub struct ClothMeasurement {
    /// Actual simulation object.
    pub cloth: ClothId,
    /// Fit error.
    pub fit: f32,
    /// Recorded characteristic size.
    pub size: f32,
}
/// Error measurements for one simulated soft-body object.
#[derive(Clone, Debug)]
pub struct SoftMeasurement {
    /// Actual simulation object.
    pub body: SoftBodyId,
    /// Number of keyed lumps.
    pub lumps: u32,
    /// Number of generated keys.
    pub keys: u32,
    /// Positional approximation error.
    pub within: f32,
    /// Normalized following error.
    pub follows: f32,
    /// Jiggle push relative to its limit.
    pub push: f32,
    /// Residual motion at the end of a once-played clip.
    pub at_end: f32,
}
/// Recorded bake results, independent of stale cache fingerprints.
#[derive(Clone, Debug, Default)]
pub struct BakeReport<'a> {
    /// Actual simulation world owning cloth and soft-body measurements, when present.
    pub system: Option<SystemId>,
    /// Editable pre-bake programmes with original curve positions.
    pub source: Vec<Restoration<'a>>,
    /// Declarations overwritten during baking.
    pub channels: Vec<AnyChannelId>,
    /// Declarations introduced during baking.
    pub appended: Vec<AnyChannelId>,
    /// Number of affected nodes.
    pub nodes: u32,
    /// Number of generated keys.
    pub keys: u32,
    /// Number of TCB tracks converted to Hermite.
    pub restated: u32,
    /// Largest positional approximation error.
    pub max_error: f32,
    /// Angular mismatch before seam correction in degrees.
    pub seam_before: f32,
    /// Angular mismatch after seam correction in degrees.
    pub seam_after: f32,
    /// Per-loop angular settling measurements in degrees.
    pub settling: Values<'a, f32>,
    /// Optional cloth measurements; absence is independently authored.
    pub cloth: Option<Vec<ClothMeasurement>>,
    /// Optional soft-body measurements; absence is independently authored.
    pub soft: Option<Vec<SoftMeasurement>>,
    /// Declarations overwritten during soft-body baking.
    pub soft_channels: Option<Vec<AnyChannelId>>,
    /// Editable programmes before soft-body baking.
    pub soft_source: Option<Vec<ControllerId>>,
    /// Whether baking created the settings; absence predates that declaration.
    pub made_settings: Option<bool>,
}
impl ValidateLinks for RecordedPhysics<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for force in &self.world {
            force.centre_node.visit_links(visitor);
        }
        if let Some(report) = &self.baked {
            report.system.visit_links(visitor);
            for restoration in &report.source {
                restoration.program.visit_links(visitor);
            }
            if let Some(programs) = &report.soft_source {
                programs.visit_links(visitor);
            }
        }
        let finite = [self.window, self.tolerance]
            .into_iter()
            .all(f32::is_finite)
            && self.world.iter().all(|force| {
                [
                    force.start,
                    force.end,
                    force.ramp_in,
                    force.ramp_out,
                    force.strength,
                    force.heading,
                    force.rise,
                    force.gusts,
                    force.gust_speed,
                    force.radius,
                    force.height,
                ]
                .into_iter()
                .all(f32::is_finite)
            })
            && self.baked.as_ref().is_none_or(|report| {
                [report.max_error, report.seam_before, report.seam_after]
                    .into_iter()
                    .all(f32::is_finite)
                    && report.settling.iter().all(f32::is_finite)
                    && report
                        .cloth
                        .iter()
                        .flatten()
                        .all(|value| value.fit.is_finite() && value.size.is_finite())
                    && report.soft.iter().flatten().all(|value| {
                        [value.within, value.follows, value.push, value.at_end]
                            .into_iter()
                            .all(f32::is_finite)
                    })
            })
            && self
                .curve
                .iter()
                .all(|value| value.into_iter().all(f32::is_finite));
        if !finite {
            visitor(Link::InvalidValue);
        }
    }
}

impl RecordedPhysics<'_> {
    pub(crate) fn validate<'a>(
        &self,
        channels: &super::Channels,
        mut programme: impl FnMut(ControllerId) -> Option<usize>,
        mut world: impl FnMut(SystemId) -> Option<&'a super::super::physics::Physics<'a>>,
    ) -> Result<(), crate::model::semantic::Error> {
        use crate::model::semantic::Error;
        let Some(report) = &self.baked else {
            return Ok(());
        };
        for id in report
            .channels
            .iter()
            .chain(&report.appended)
            .chain(report.soft_channels.iter().flatten())
        {
            if channels.declaration(*id).is_none() {
                return Err(Error::new(
                    "wem.physics.bake.channel",
                    "bake declaration belongs to another model",
                ));
            }
        }
        for restored in &report.source {
            if programme(restored.program) != Some(restored.positions.len()) {
                return Err(Error::new(
                    "wem.physics.bake.restore",
                    "saved curve insertion ordinals do not match programme tracks",
                ));
            }
        }
        let soft_present = report.soft.is_some()
            || report.soft_channels.is_some()
            || report.soft_source.is_some()
            || report.made_settings.is_some();
        if soft_present
            && (report.cloth.is_none()
                || report.soft.is_none()
                || report.soft_channels.is_none()
                || report.soft_source.is_none()
                || report.made_settings.is_none())
        {
            return Err(Error::new(
                "wem.physics.bake.soft",
                "soft declarations and cloth measurements must be authored together",
            ));
        }
        let physics = report.system.and_then(&mut world);
        for value in report.cloth.iter().flatten() {
            if physics.is_none_or(|world| world.cloth(value.cloth).is_none()) {
                return Err(Error::new(
                    "wem.physics.bake.cloth",
                    "measurement belongs to another simulation world",
                ));
            }
        }
        for value in report.soft.iter().flatten() {
            if physics.is_none_or(|world| world.soft_body(value.body).is_none()) {
                return Err(Error::new(
                    "wem.physics.bake.soft",
                    "measurement belongs to another simulation world",
                ));
            }
        }
        Ok(())
    }
}
