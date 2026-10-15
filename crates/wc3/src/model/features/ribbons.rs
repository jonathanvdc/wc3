//! Composed ribbon emission, geometry, appearance, and rendering programs.
use crate::model::profiles::{m2, m3, mdlx};
use crate::model::{Animated, MaterialId, TextureId};

/// A persistent trail assembled from successive cross sections.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrailComposition<R, S, A, B, C> {
    /// Authored generation rate and its semantic unit.
    pub emission_rate: R,
    /// Persistence of a generated section or edge in seconds.
    pub lifetime: f32,
    /// Cross-section extents.
    pub geometry: TrailGeometry,
    /// Tint, opacity, and atlas selection.
    pub appearance: TrailAppearance<S, A>,
    /// Material and texture bindings.
    pub rendering: B,
    /// Independent physical and activation controls.
    pub controls: C,
}
/// Broad trail composition selected by the Universal profile.
pub type TrailEmitter = TrailComposition<f64, TrailAtlasSlot, u32, TrailRendering, TrailControls>;
/// Animated upper and lower extent of a generated trail.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrailGeometry {
    /// Extent above the center line.
    pub height_above: Animated<f32>,
    /// Extent below the center line.
    pub height_below: Animated<f32>,
}
/// Animated trail appearance and atlas layout.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrailAppearance<S = TrailAtlasSlot, A = u32> {
    /// RGB tint.
    pub color: Animated<[f32; 3]>,
    /// Opacity multiplier.
    pub opacity: Animated<f32>,
    /// Atlas rows and columns; export checks the destination's integer width.
    pub atlas: [A; 2],
    /// Atlas selector retaining its typed animated destination.
    pub texture_slot: S,
}
/// Typed atlas selectors retain animation identity and value domain.
#[derive(Clone, Debug)]
pub enum TrailAtlasSlot {
    /// Full-width cell selection.
    Wide(Animated<u32>),
    /// Compact cell selection.
    Compact(Animated<u16>),
}
/// Logical rendering alternatives rather than source-file layouts.
#[derive(Clone, Debug)]
pub enum TrailRendering {
    /// A layered material supplies the complete surface program.
    Layered(LayeredTrailRendering),
    /// Ordered texture and shader-surface bindings.
    Surfaces(SurfaceTrailRendering),
}
/// Physical and application controls with explicit meanings.
#[derive(Clone, Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Authored controls remain directly editable without an allocation."
)]
pub enum TrailControls {
    /// Gravitational acceleration and fractional visibility.
    Gravity(GravityTrailControls),
    /// Emission direction and discrete activation with texture controls.
    EmissionAngle(AngleTrailControls),
}
/// A trail rendered by one layered material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayeredTrailRendering {
    /// Complete layered surface.
    pub material: Option<MaterialId>,
}
/// Ordered textures and shader surfaces supplying a trail.
#[derive(Clone, Debug)]
pub struct SurfaceTrailRendering {
    /// Ordered texture resources.
    pub textures: Vec<TextureId>,
    /// Ordered surfaces.
    pub materials: Vec<MaterialId>,
}
/// Gravitational acceleration and fractional visibility.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GravityTrailControls {
    /// Gravity acceleration.
    pub gravity: f32,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
}
/// Emission direction, discrete activation, and texture controls.
#[derive(Clone, Debug)]
pub struct AngleTrailControls {
    /// Emission angle, independently of acceleration.
    pub angle: f32,
    /// Runtime effect identifier.
    pub id: u32,
    /// Discrete activation.
    pub enabled: Animated<bool>,
    /// Signed pass-order plane.
    pub priority_plane: i16,
    /// External color-table selection.
    pub color_index: i8,
    /// Optional texture-coordinate transform.
    pub texture_transform: Option<m2::TextureTransform>,
}
/// Trail composition with integral section generation and fractional visibility.
pub type GravityTrail =
    TrailComposition<u32, Animated<u32>, u32, LayeredTrailRendering, GravityTrailControls>;
/// Trail composition with fractional edge generation and discrete activation.
pub type AngleTrail =
    TrailComposition<f32, Animated<u16>, u16, SurfaceTrailRendering, AngleTrailControls>;
impl From<mdlx::Ribbon> for TrailEmitter {
    fn from(v: mdlx::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas,
                texture_slot: TrailAtlasSlot::Wide(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Layered(v.rendering),
            controls: TrailControls::Gravity(v.controls),
        }
    }
}
impl From<m2::Ribbon> for TrailEmitter {
    fn from(v: m2::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas.map(u32::from),
                texture_slot: TrailAtlasSlot::Compact(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Surfaces(v.rendering),
            controls: TrailControls::EmissionAngle(v.controls),
        }
    }
}
impl TryFrom<TrailEmitter> for mdlx::Ribbon {
    type Error = Box<TrailEmitter>;
    fn try_from(v: TrailEmitter) -> Result<Self, Self::Error> {
        if !(v.emission_rate >= 0.0
            && v.emission_rate <= f64::from(u32::MAX)
            && v.emission_rate.fract() == 0.0
            && matches!(
                (&v.appearance.texture_slot, &v.rendering, &v.controls),
                (
                    TrailAtlasSlot::Wide(_),
                    TrailRendering::Layered(_),
                    TrailControls::Gravity(_)
                )
            ))
        {
            return Err(Box::new(v));
        }
        let emission_rate = v.emission_rate as u32;
        let TrailAtlasSlot::Wide(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Layered(rendering) = v.rendering else {
            unreachable!()
        };
        let TrailControls::Gravity(controls) = v.controls else {
            unreachable!()
        };
        Ok(Self {
            geometry: v.geometry,
            appearance: TrailAppearance {
                opacity: v.appearance.opacity,
                color: v.appearance.color,
                texture_slot,
                atlas: v.appearance.atlas,
            },
            lifetime: v.lifetime,
            emission_rate,
            rendering,
            controls,
        })
    }
}
impl TryFrom<TrailEmitter> for m2::Ribbon {
    type Error = Box<TrailEmitter>;
    fn try_from(v: TrailEmitter) -> Result<Self, Self::Error> {
        if f64::from(v.emission_rate as f32).to_bits() != v.emission_rate.to_bits()
            || v.appearance
                .atlas
                .iter()
                .any(|n| u16::try_from(*n).is_err())
            || !matches!(
                (&v.appearance.texture_slot, &v.rendering, &v.controls),
                (
                    TrailAtlasSlot::Compact(_),
                    TrailRendering::Surfaces(_),
                    TrailControls::EmissionAngle(_)
                )
            )
        {
            return Err(Box::new(v));
        }
        let edges_per_second = v.emission_rate as f32;
        let TrailAtlasSlot::Compact(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Surfaces(rendering) = v.rendering else {
            unreachable!()
        };
        let TrailControls::EmissionAngle(controls) = v.controls else {
            unreachable!()
        };
        Ok(Self {
            controls,
            rendering,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas.map(|n| n as u16),
                texture_slot,
            },
            geometry: v.geometry,
            emission_rate: edges_per_second,
            lifetime: v.lifetime,
        })
    }
}

use super::effect_components::{
    ProgrammedAge, ProgrammedDirection, ProgrammedForces, ProgrammedNoise,
};
use super::particles::{ParticleGravity, ParticleLifetime, ParticleMotion, ParticleScalar};
use super::programmed_effects::{SplinePoint, VariationControl};
use crate::model::animation::programmed::Property;
use crate::model::values::programmed::{RibbonAdditionalFlags, RibbonFlags, RibbonKind, Smoothing};
use crate::model::NodeId;
use std::borrow::Cow;

/// Ribbon simulation composed from shared particle motion and age programs.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedRibbonComposition<'a, M, L> {
    /// Shared motion controls.
    pub motion: M,
    /// Initial emission direction and drag.
    pub direction: ProgrammedDirection,
    /// Shared lifetime controls.
    pub lifetime: L,
    /// Shared appearance controls.
    pub appearance: ProgrammedAge,
    /// Shared forces controls.
    pub forces: ProgrammedForces,
    /// Shared noise controls.
    pub noise: ProgrammedNoise,
    /// Secondary bone used by the authored ribbon fallback attachment.
    pub fallback_node: NodeId,
    /// Material drawn by this emitter.
    pub material: Option<MaterialId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: RibbonAdditionalFlags,
    /// Source ribbon kill-radius word; its integer encoding is retained without assigning distance units.
    pub kill_radius: u32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: u32,
    /// StarCraft II ribbon geometry mode.
    pub ribbon_type: RibbonKind,
    /// Divisions parameter stored as a floating-point value.
    pub divisions: f32,
    /// Ribbon edge-count control.
    pub edges: u32,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: f32,
    /// Animated maximum-length control for the ribbon.
    pub max_length: Property<f32>,
    /// Authored spline control points defining the emission curve.
    pub spline_points: Cow<'a, [SplinePoint]>,
    /// Animated ribbon activation control.
    pub active: Property<u32>,
    /// Stored state bits; unknown bits are retained.
    pub flags: RibbonFlags,
    /// Interpolation mode for the particle size controls.
    pub size_smoothing: Smoothing,
    /// Interpolation mode for the particle color controls.
    pub color_smoothing: Smoothing,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: VariationControl,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: VariationControl,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: VariationControl,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: VariationControl,
    /// Opacity control for the emitted geometry.
    pub alpha: VariationControl,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Property<f32>,
    /// Animated ribbon overlay control; its detailed source semantics are unresolved.
    pub overlay: Property<f32>,
    /// Unresolved deprecated unknown data retained in its original position and representation.
    pub deprecated_unknown: i32,
}
/// Broad programmed trail composition selected by the Universal profile.
pub type ProgrammedRibbon<'a> = ProgrammedRibbonComposition<'a, ParticleMotion, ParticleLifetime>;
impl<'a> From<m3::Ribbon<'a>> for ProgrammedRibbon<'a> {
    fn from(v: m3::Ribbon<'a>) -> Self {
        Self {
            motion: ParticleMotion {
                speed: ParticleScalar::Programmed(v.motion.speed),
                variation: ParticleScalar::Programmed(v.motion.variation),
                gravity: ParticleGravity::ProgrammedVector {
                    x: v.motion.gravity.x,
                    y: v.motion.gravity.y,
                    magnitude: v.motion.gravity.magnitude,
                },
            },
            direction: v.direction,
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Programmed(v.lifetime.duration),
                variation: ParticleScalar::Programmed(v.lifetime.variation),
            },
            appearance: v.appearance,
            forces: v.forces,
            noise: v.noise,
            fallback_node: v.fallback_node,
            material: v.material,
            additional_flags: v.additional_flags,
            kill_radius: v.kill_radius,
            index_plus_length: v.index_plus_length,
            emitter_shape: v.emitter_shape,
            ribbon_type: v.ribbon_type,
            divisions: v.divisions,
            edges: v.edges,
            inner_radius: v.inner_radius,
            max_length: v.max_length,
            spline_points: v.spline_points,
            active: v.active,
            flags: v.flags,
            size_smoothing: v.size_smoothing,
            color_smoothing: v.color_smoothing,
            friction: v.friction,
            bounce: v.bounce,
            lod_reduce: v.lod_reduce,
            lod_cut: v.lod_cut,
            yaw: v.yaw,
            pitch: v.pitch,
            speed: v.speed,
            size: v.size,
            alpha: v.alpha,
            particle_velocity: v.particle_velocity,
            overlay: v.overlay,
            deprecated_unknown: v.deprecated_unknown,
        }
    }
}
impl<'a> TryFrom<ProgrammedRibbon<'a>> for m3::Ribbon<'a> {
    type Error = Box<ProgrammedRibbon<'a>>;
    fn try_from(v: ProgrammedRibbon<'a>) -> Result<Self, Self::Error> {
        if !matches!(
            (
                &v.motion.speed,
                &v.motion.variation,
                &v.motion.gravity,
                &v.lifetime.duration,
                &v.lifetime.variation
            ),
            (
                ParticleScalar::Programmed(_),
                ParticleScalar::Programmed(_),
                ParticleGravity::ProgrammedVector { .. },
                ParticleScalar::Programmed(_),
                ParticleScalar::Programmed(_)
            )
        ) {
            return Err(Box::new(v));
        }
        let ParticleScalar::Programmed(initial_speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Programmed(initial_speed_random) = v.motion.variation else {
            unreachable!()
        };
        let ParticleGravity::ProgrammedVector {
            x: gravity_x,
            y: gravity_y,
            magnitude: gravity,
        } = v.motion.gravity
        else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime_random) = v.lifetime.variation else {
            unreachable!()
        };
        Ok(Self {
            motion: ParticleMotion {
                gravity: ProgrammedTrailGravity {
                    x: gravity_x,
                    y: gravity_y,
                    magnitude: gravity,
                },
                speed: initial_speed,
                variation: initial_speed_random,
            },
            fallback_node: v.fallback_node,
            material: v.material,
            additional_flags: v.additional_flags,
            direction: v.direction,
            lifetime: ParticleLifetime {
                duration: lifetime,
                variation: lifetime_random,
            },
            kill_radius: v.kill_radius,
            appearance: v.appearance,
            forces: v.forces,
            noise: v.noise,
            index_plus_length: v.index_plus_length,
            emitter_shape: v.emitter_shape,
            ribbon_type: v.ribbon_type,
            divisions: v.divisions,
            edges: v.edges,
            inner_radius: v.inner_radius,
            max_length: v.max_length,
            spline_points: v.spline_points,
            active: v.active,
            flags: v.flags,
            size_smoothing: v.size_smoothing,
            color_smoothing: v.color_smoothing,
            friction: v.friction,
            bounce: v.bounce,
            lod_reduce: v.lod_reduce,
            lod_cut: v.lod_cut,
            yaw: v.yaw,
            pitch: v.pitch,
            speed: v.speed,
            size: v.size,
            alpha: v.alpha,
            particle_velocity: v.particle_velocity,
            overlay: v.overlay,
            deprecated_unknown: v.deprecated_unknown,
        })
    }
}

use crate::model::{Link, ValidateLinks};
impl<M: ValidateLinks, L: ValidateLinks> ValidateLinks for ProgrammedRibbonComposition<'_, M, L> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.fallback_node));
        if let Some(id) = self.material {
            visitor(Link::Material(id));
        }
        self.motion.visit_links(visitor);
        self.lifetime.visit_links(visitor);
        self.direction.initial_yaw.visit_links(visitor);
        self.direction.initial_pitch.visit_links(visitor);
        self.direction.initial_horizontal.visit_links(visitor);
        self.direction.initial_vertical.visit_links(visitor);
        if !self.appearance.size_mid_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.color_mid_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.alpha_mid_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.rotation_mid_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.size_mid_hold_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.color_mid_hold_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.alpha_mid_hold_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.appearance.rotation_mid_hold_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.appearance.size_animation.visit_links(visitor);
        self.appearance.rotation_animation.visit_links(visitor);
        self.appearance.color_start.visit_links(visitor);
        self.appearance.color_mid.visit_links(visitor);
        self.appearance.color_end.visit_links(visitor);
        if !self.direction.drag.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.forces.mass.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.forces.mass_random.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.forces.mass_size_multiplier.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.forces.world_forces_mass_multiplier.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.noise.noise_amplitude.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.noise.noise_frequency.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.noise.noise_coherence.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.noise.noise_edge.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.divisions.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.inner_radius.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.max_length.visit_links(visitor);
        for child in self.spline_points.iter() {
            child.visit_links(visitor);
        }
        self.active.visit_links(visitor);
        if !self.friction.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.bounce.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.yaw.visit_links(visitor);
        self.pitch.visit_links(visitor);
        self.speed.visit_links(visitor);
        self.size.visit_links(visitor);
        self.alpha.visit_links(visitor);
        self.particle_velocity.visit_links(visitor);
        self.overlay.visit_links(visitor);
    }
}

/// Numeric domain for a trail generation rate.
pub trait TrailRate {
    /// Whether this value describes a finite, nonnegative generation rate.
    fn valid_rate(&self) -> bool;
}
impl TrailRate for u32 {
    fn valid_rate(&self) -> bool {
        true
    }
}
impl TrailRate for f32 {
    fn valid_rate(&self) -> bool {
        self.is_finite() && *self >= 0.0
    }
}
impl TrailRate for f64 {
    fn valid_rate(&self) -> bool {
        self.is_finite() && *self >= 0.0
    }
}
impl<R: TrailRate, S: ValidateLinks, A, B: ValidateLinks, C: ValidateLinks> ValidateLinks
    for TrailComposition<R, S, A, B, C>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.geometry.height_above.visit_links(visitor);
        self.geometry.height_below.visit_links(visitor);
        self.appearance.color.visit_links(visitor);
        self.appearance.opacity.visit_links(visitor);
        self.appearance.texture_slot.visit_links(visitor);
        self.rendering.visit_links(visitor);
        self.controls.visit_links(visitor);
        if !self.emission_rate.valid_rate() || !self.lifetime.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for TrailAtlasSlot {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wide(value) => value.visit_links(visitor),
            Self::Compact(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for TrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Layered(value) => value.visit_links(visitor),
            Self::Surfaces(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for LayeredTrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.material.visit_links(visitor);
    }
}
impl ValidateLinks for SurfaceTrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.textures.visit_links(visitor);
        self.materials.visit_links(visitor);
    }
}
impl ValidateLinks for GravityTrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.gravity.visit_links(visitor);
        self.visibility.visit_links(visitor);
    }
}
impl ValidateLinks for AngleTrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.angle.visit_links(visitor);
        self.enabled.visit_links(visitor);
        self.texture_transform.visit_links(visitor);
    }
}
impl ValidateLinks for TrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Gravity(value) => value.visit_links(visitor),
            Self::EmissionAngle(value) => value.visit_links(visitor),
        }
    }
}
/// Floating horizontal gravity controls for programmed trail simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedTrailGravity {
    /// Authored horizontal gravity parameter.
    pub x: f32,
    /// Authored horizontal gravity parameter.
    pub y: f32,
    /// Authored gravity parameter.
    pub magnitude: f32,
}
impl ValidateLinks for ProgrammedTrailGravity {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.x.visit_links(visitor);
        self.y.visit_links(visitor);
        self.magnitude.visit_links(visitor);
    }
}
/// Programmed trail composition with reset-bearing controls.
pub type ProgrammedTrail<'a> = ProgrammedRibbonComposition<
    'a,
    ParticleMotion<Property<f32>, Property<f32>, ProgrammedTrailGravity>,
    ParticleLifetime<Property<f32>, Property<f32>>,
>;
