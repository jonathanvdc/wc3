//! Authored deformation controls and actual surface/hierarchy relationships.
use super::{
    Along, Anchor, Axis, BulgeShape, ComesFrom, DripPreset, JigglePreset, MeltOrder, NeckLaw,
    Pulse, RipplePlays, SquashMode, SurfacePoint, Through, WobblePattern,
};
use crate::model::semantic::MaterialSlotId;
use crate::model::semantic::{NodeId, Values};
use std::borrow::Cow;
/// Inflation direction, smoothing, and pulsation behavior.
#[derive(Clone, Debug, Default)]
pub struct Inflate {
    /// Direction used to move the surface.
    pub along: Along,
    /// Fades influence with distance from the attachment.
    pub toward_place: bool,
    /// Blend toward a rounded surface.
    pub round: f32,
    /// Compensates changes in enclosed volume.
    pub keep_volume: bool,
    /// Repetition pattern.
    pub pulse: Pulse,
    /// Pulses per second.
    pub rate: f32,
    /// Fraction of inflation lost between pulses.
    pub depth: f32,
    /// Requested surface-normal smoothing passes.
    pub smooth: u8,
}
/// A bulge and its optional event-driven travel path.
#[derive(Clone, Debug, Default)]
pub struct Bulge<'a> {
    /// Displacement direction.
    pub along: Along,
    /// Surface bulge interpretation.
    pub shape: BulgeShape,
    /// Compensates enclosed-volume changes.
    pub keep_volume: bool,
    /// Source of effect activation.
    pub from: ComesFrom,
    /// Symbolic event name used for activation.
    pub event: Cow<'a, str>,
    /// Impact rise time in seconds.
    pub rise: f32,
    /// Impact decay time in seconds.
    pub fall: f32,
    /// Moves the deformation along the path.
    pub travels: bool,
    /// Hierarchy nodes defining the travel chain.
    pub chain: Values<'a, NodeId>,
    /// Bind-time model-space travel positions.
    pub line: Values<'a, [f32; 3]>,
}
/// An independently identified jiggle impulse and its actual surface attachment.
#[derive(Clone, Debug)]
pub struct Impulse {
    /// Authored impulse identity used by activation controls.
    pub id: u32,
    /// Actual surface point receiving the impulse.
    pub place: SurfacePoint,
    /// Model-space direction; absence uses the inward surface direction.
    pub direction: Option<[f32; 3]>,
    /// Acceleration strength relative to gravitational acceleration.
    pub strength: f32,
}
/// Jiggle response and its authored impulses.
#[derive(Clone, Debug, Default)]
pub struct Jiggle<'a> {
    /// Response parameter preset.
    pub preset: JigglePreset,
    /// Slowest oscillation frequency.
    pub looseness: f32,
    /// Rebound overshoot.
    pub bounce: f32,
    /// Gravity-induced sag.
    pub weight: f32,
    /// Coupling between neighboring points.
    pub hold_shape: f32,
    /// Volume-direction stiffness.
    pub keep_volume: f32,
    /// Displacement limit relative to the region size.
    pub limit: f32,
    /// Restrains overall displacement.
    pub keep_in_place: bool,
    /// Includes contact with the floor.
    pub meets_floor: bool,
    /// Includes world-force contributions.
    pub world_forces: bool,
    /// Authored impulse attachments and strengths.
    pub impulses: Values<'a, Impulse>,
}
/// Oscillatory deformation along a selected frame.
#[derive(Clone, Debug, Default)]
pub struct Wobble {
    /// Oscillation pattern.
    pub pattern: WobblePattern,
    /// Swings per second.
    pub speed: f32,
    /// Rolling wavelength relative to region size.
    pub size: f32,
    /// Phase delay between the region's ends.
    pub overlap: f32,
    /// Axis interpretation.
    pub axis: Axis,
    /// Actual hierarchy frame for a bone-defined axis, when supplied.
    pub axis_bone: Option<NodeId>,
    /// Compensates volume changes.
    pub keep_volume: bool,
}
/// Surface rounding and expansion behavior.
#[derive(Clone, Debug, Default)]
pub struct Blobify {
    /// Surface rounding amount.
    pub roundness: f32,
    /// Expansion amount.
    pub swell: f32,
    /// Compensates volume changes.
    pub keep_volume: bool,
}
/// An authored emission site on actual mesh geometry.
#[derive(Clone, Debug)]
pub struct DripSite {
    /// Surface point from which the strand is emitted.
    pub place: SurfacePoint,
    /// Relative thickness; zero uses the global thickness.
    pub thickness: f32,
}
/// Actual generated strand hierarchy and its bind-time dimensions.
#[derive(Clone, Debug)]
pub struct Strand {
    /// Hierarchy nodes for the base, neck, and tip.
    pub nodes: [NodeId; 3],
    /// Position at the bind.
    pub position: [f32; 3],
    /// Bind-time strand radius.
    pub width: f32,
    /// Detached-drop hierarchy node, when generated.
    pub drop: Option<NodeId>,
}
/// Drip preparation, progression, and generated geometry associations.
#[derive(Clone, Debug, Default)]
pub struct Drip<'a> {
    /// Response parameter preset.
    pub preset: DripPreset,
    /// Rule for narrowing the strand neck.
    pub neck: NeckLaw,
    /// Uses explicitly authored emission sites.
    pub sites_by_hand: bool,
    /// Requested automatic emission-site count.
    pub sites: u8,
    /// Face angle threshold for shedding, in radians.
    pub overhang: f32,
    /// Site separation relative to region size.
    pub spacing: f32,
    /// Extends strands toward the ground.
    pub to_ground: bool,
    /// Strand length relative to region size.
    pub length: f32,
    /// Progression in body-height fractions per second.
    pub speed: f32,
    /// Strand thickness relative to region size.
    pub thickness: f32,
    /// Neck width relative to strand thickness.
    pub neck_width: f32,
    /// Tip-width multiplier.
    pub tip: f32,
    /// Relative site timing offset.
    pub stagger: f32,
    /// Swing amount, with one corresponding to twenty degrees.
    pub sway: f32,
    /// Stretch amount.
    pub pull: f32,
    /// Seconds between detached drops; zero prevents detachment.
    pub pinch_every: f32,
    /// Appearance slot for generated geometry; absence inherits the surface material.
    pub material: Option<MaterialSlotId>,
    /// Authored emission sites.
    pub authored_sites: Values<'a, DripSite>,
    /// Actual generated strands and drop nodes.
    pub strands: Values<'a, Strand>,
}
/// Squash and stretch along an authored axis.
#[derive(Clone, Debug, Default)]
pub struct Squash {
    /// Automatic or explicitly keyed behavior.
    pub mode: SquashMode,
    /// Axis interpretation.
    pub axis: Axis,
    /// Actual hierarchy frame for a bone-defined axis, when supplied.
    pub axis_bone: Option<NodeId>,
    /// Longitudinal stretch amount.
    pub stretch: f32,
    /// Compression amount.
    pub squash: f32,
    /// Response to direction changes.
    pub turning: f32,
    /// Trailing-motion deformation.
    pub drag: f32,
    /// Anticipatory deformation.
    pub wind_up: f32,
    /// Axis-relative anchor interpretation.
    pub anchor: Anchor,
    /// Motion smoothing interval in seconds.
    pub smoothing: f32,
}
/// Melt progression and its floor interaction.
#[derive(Clone, Debug, Default)]
pub struct Melt {
    /// Progression order.
    pub order: MeltOrder,
    /// Resistance to deformation.
    pub solidity: f32,
    /// Uses grid height instead of the explicit floor.
    pub on_grid: bool,
    /// Explicit floor height.
    pub floor: f32,
    /// Spread multiplier relative to the body footprint.
    pub puddle: f32,
    /// Progression irregularity.
    pub uneven: f32,
    /// Reverses the progression.
    pub backward: bool,
    /// Retains the final state in decay animations.
    pub stays_in_decay: bool,
}
/// Triggered ripple propagation and its envelope.
#[derive(Clone, Debug, Default)]
pub struct Ripple<'a> {
    /// Displacement direction.
    pub along: Along,
    /// Repetition interpretation.
    pub plays: RipplePlays,
    /// Activation source.
    pub from: ComesFrom,
    /// Symbolic event name used for activation.
    pub event: Cow<'a, str>,
    /// Wavelength relative to region size.
    pub wavelength: f32,
    /// Propagation in region sizes per second.
    pub speed: f32,
    /// Propagation through the skin or the body.
    pub through: Through,
    /// Separates envelope speed from crest speed.
    pub water: bool,
    /// Envelope speed relative to crest speed.
    pub envelope: f32,
    /// Spatial attenuation distance.
    pub fade_distance: f32,
    /// Temporal fade duration in seconds.
    pub fade_time: f32,
}
/// Independently authored families retained when effect selection changes.
#[derive(Clone, Debug, Default)]
pub struct Settings<'a> {
    /// Inflation behavior.
    pub inflate: Option<Inflate>,
    /// Bulge behavior.
    pub bulge: Option<Bulge<'a>>,
    /// Jiggle behavior.
    pub jiggle: Option<Jiggle<'a>>,
    /// Wobble behavior.
    pub wobble: Option<Wobble>,
    /// Surface-rounding behavior.
    pub blobify: Option<Blobify>,
    /// Drip behavior and actual generated geometry.
    pub drip: Option<Drip<'a>>,
    /// Squash-and-stretch behavior.
    pub squash: Option<Squash>,
    /// Melt behavior.
    pub melt: Option<Melt>,
    /// Ripple behavior.
    pub ripple: Option<Ripple<'a>>,
}
