//! Composed ribbon emission, geometry, appearance, and rendering programs.
use crate::model::semantic::profiles::{m2, m3, mdlx};
use crate::model::semantic::{Animated, MaterialId, TextureId};

/// A persistent trail assembled from successive cross sections.
#[derive(Clone, Debug)]
pub struct TrailEmitter {
    /// Authored generation rate and its semantic unit.
    pub emission_rate: f64,
    /// Persistence of a generated section or edge in seconds.
    pub lifetime: f32,
    /// Cross-section extents.
    pub geometry: TrailGeometry,
    /// Tint, opacity, and atlas selection.
    pub appearance: TrailAppearance,
    /// Material and texture bindings.
    pub rendering: TrailRendering,
    /// Independent physical and activation controls.
    pub controls: TrailControls,
}
/// Animated upper and lower extent of a generated trail.
#[derive(Clone, Debug)]
pub struct TrailGeometry {
    /// Extent above the center line.
    pub height_above: Animated<f32>,
    /// Extent below the center line.
    pub height_below: Animated<f32>,
}
/// Animated trail appearance and atlas layout.
#[derive(Clone, Debug)]
pub struct TrailAppearance {
    /// RGB tint.
    pub color: Animated<[f32; 3]>,
    /// Opacity multiplier.
    pub opacity: Animated<f32>,
    /// Atlas rows and columns; export checks the destination's integer width.
    pub atlas: [u32; 2],
    /// Atlas selector retaining its typed animated destination.
    pub texture_slot: TrailAtlasSlot,
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
    Layered {
        /// Complete layered surface.
        material: Option<MaterialId>,
    },
    /// Ordered texture and shader-surface bindings.
    Surfaces {
        /// Ordered texture resources.
        textures: Vec<TextureId>,
        /// Ordered surfaces.
        materials: Vec<MaterialId>,
    },
}
/// Physical and application controls with explicit meanings.
#[derive(Clone, Debug)]
pub enum TrailControls {
    /// Gravitational acceleration and fractional visibility.
    Gravity {
        /// Gravity acceleration.
        gravity: f32,
        /// Visibility multiplier.
        visibility: Animated<f32>,
    },
    /// Emission direction and discrete activation with texture controls.
    EmissionAngle {
        /// Emission angle; this is not acceleration.
        angle: f32,
        /// Runtime effect identifier.
        id: u32,
        /// Discrete activation.
        enabled: Animated<bool>,
        /// Signed pass-order plane.
        priority_plane: i16,
        /// External color-table selection.
        color_index: i8,
        /// Optional texture-coordinate transform.
        texture_transform: Option<Box<m2::TextureTransform>>,
    },
}
impl From<mdlx::Ribbon> for TrailEmitter {
    fn from(v: mdlx::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.life_span,
            geometry: TrailGeometry {
                height_above: v.height_above,
                height_below: v.height_below,
            },
            appearance: TrailAppearance {
                color: v.color,
                opacity: v.opacity,
                atlas: v.atlas,
                texture_slot: TrailAtlasSlot::Wide(v.texture_slot),
            },
            rendering: TrailRendering::Layered {
                material: v.material,
            },
            controls: TrailControls::Gravity {
                gravity: v.gravity,
                visibility: v.visibility,
            },
        }
    }
}
impl From<m2::Ribbon> for TrailEmitter {
    fn from(v: m2::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.edges_per_second),
            lifetime: v.edge_lifetime,
            geometry: TrailGeometry {
                height_above: v.height_above,
                height_below: v.height_below,
            },
            appearance: TrailAppearance {
                color: v.color,
                opacity: v.alpha,
                atlas: v.atlas.map(u32::from),
                texture_slot: TrailAtlasSlot::Compact(v.texture_slot),
            },
            rendering: TrailRendering::Surfaces {
                textures: v.textures,
                materials: v.materials,
            },
            controls: TrailControls::EmissionAngle {
                angle: v.emission_angle,
                id: v.id,
                enabled: v.enabled,
                priority_plane: v.priority_plane,
                color_index: v.color_index,
                texture_transform: v.texture_transform.map(Box::new),
            },
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
                    TrailRendering::Layered { .. },
                    TrailControls::Gravity { .. }
                )
            ))
        {
            return Err(Box::new(v));
        }
        let emission_rate = v.emission_rate as u32;
        let TrailAtlasSlot::Wide(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Layered { material } = v.rendering else {
            unreachable!()
        };
        let TrailControls::Gravity {
            gravity,
            visibility,
        } = v.controls
        else {
            unreachable!()
        };
        Ok(Self {
            height_above: v.geometry.height_above,
            height_below: v.geometry.height_below,
            opacity: v.appearance.opacity,
            color: v.appearance.color,
            life_span: v.lifetime,
            texture_slot,
            emission_rate,
            atlas: v.appearance.atlas,
            material,
            gravity,
            visibility,
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
                    TrailRendering::Surfaces { .. },
                    TrailControls::EmissionAngle { .. }
                )
            )
        {
            return Err(Box::new(v));
        }
        let edges_per_second = v.emission_rate as f32;
        let TrailAtlasSlot::Compact(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Surfaces {
            textures,
            materials,
        } = v.rendering
        else {
            unreachable!()
        };
        let TrailControls::EmissionAngle {
            angle: emission_angle,
            id,
            enabled,
            priority_plane,
            color_index,
            texture_transform,
        } = v.controls
        else {
            unreachable!()
        };
        Ok(Self {
            id,
            textures,
            materials,
            color: v.appearance.color,
            alpha: v.appearance.opacity,
            height_above: v.geometry.height_above,
            height_below: v.geometry.height_below,
            edges_per_second,
            edge_lifetime: v.lifetime,
            emission_angle,
            atlas: v.appearance.atlas.map(|n| n as u16),
            texture_slot,
            enabled,
            priority_plane,
            color_index,
            texture_transform: texture_transform.map(|value| *value),
        })
    }
}

use super::effect_components::{
    ProgrammedAge, ProgrammedDirection, ProgrammedForces, ProgrammedNoise,
};
use super::particles::{ParticleGravity, ParticleLifetime, ParticleMotion, ParticleScalar};
use crate::model::semantic::NodeId;
use m3::values::{RibbonAdditionalFlags, RibbonFlags, RibbonKind, Smoothing};
use m3::{Property, SplinePoint, VariationControl};
use std::borrow::Cow;

/// Ribbon simulation composed from shared particle motion and age programs.
#[derive(Clone, Debug)]
pub struct ProgrammedRibbon<'a> {
    /// Shared motion controls.
    pub motion: ParticleMotion,
    /// Initial emission direction and drag.
    pub direction: ProgrammedDirection,
    /// Shared lifetime controls.
    pub lifetime: ParticleLifetime,
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
impl<'a> From<m3::Ribbon<'a>> for ProgrammedRibbon<'a> {
    fn from(v: m3::Ribbon<'a>) -> Self {
        Self {
            motion: ParticleMotion {
                speed: ParticleScalar::Programmed(v.initial_speed),
                variation: ParticleScalar::Programmed(v.initial_speed_random),
                gravity: ParticleGravity::ProgrammedVector {
                    x: v.gravity_x,
                    y: v.gravity_y,
                    magnitude: v.gravity,
                },
            },
            direction: ProgrammedDirection {
                initial_yaw: v.initial_yaw,
                initial_pitch: v.initial_pitch,
                initial_horizontal: v.initial_horizontal,
                initial_vertical: v.initial_vertical,
                drag: v.drag,
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Programmed(v.lifetime),
                variation: ParticleScalar::Programmed(v.lifetime_random),
            },
            appearance: ProgrammedAge {
                size_mid_time: v.size_mid_time,
                color_mid_time: v.color_mid_time,
                alpha_mid_time: v.alpha_mid_time,
                rotation_mid_time: v.rotation_mid_time,
                size_mid_hold_time: v.size_mid_hold_time,
                color_mid_hold_time: v.color_mid_hold_time,
                alpha_mid_hold_time: v.alpha_mid_hold_time,
                rotation_mid_hold_time: v.rotation_mid_hold_time,
                size_animation: v.size_animation,
                rotation_animation: v.rotation_animation,
                color_start: v.color_start,
                color_mid: v.color_mid,
                color_end: v.color_end,
            },
            forces: ProgrammedForces {
                mass: v.mass,
                mass_random: v.mass_random,
                mass_size_multiplier: v.mass_size_multiplier,
                local_forces: v.local_forces,
                world_forces: v.world_forces,
                local_forces_fallback: v.local_forces_fallback,
                world_forces_fallback: v.world_forces_fallback,
                world_forces_mass_multiplier: v.world_forces_mass_multiplier,
            },
            noise: ProgrammedNoise {
                noise_amplitude: v.noise_amplitude,
                noise_frequency: v.noise_frequency,
                noise_coherence: v.noise_coherence,
                noise_edge: v.noise_edge,
            },
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
            gravity_x,
            gravity_y,
            gravity,
            fallback_node: v.fallback_node,
            material: v.material,
            additional_flags: v.additional_flags,
            initial_speed,
            initial_speed_random,
            initial_yaw: v.direction.initial_yaw,
            initial_pitch: v.direction.initial_pitch,
            initial_horizontal: v.direction.initial_horizontal,
            initial_vertical: v.direction.initial_vertical,
            lifetime,
            lifetime_random,
            kill_radius: v.kill_radius,
            size_mid_time: v.appearance.size_mid_time,
            color_mid_time: v.appearance.color_mid_time,
            alpha_mid_time: v.appearance.alpha_mid_time,
            rotation_mid_time: v.appearance.rotation_mid_time,
            size_mid_hold_time: v.appearance.size_mid_hold_time,
            color_mid_hold_time: v.appearance.color_mid_hold_time,
            alpha_mid_hold_time: v.appearance.alpha_mid_hold_time,
            rotation_mid_hold_time: v.appearance.rotation_mid_hold_time,
            size_animation: v.appearance.size_animation,
            rotation_animation: v.appearance.rotation_animation,
            color_start: v.appearance.color_start,
            color_mid: v.appearance.color_mid,
            color_end: v.appearance.color_end,
            drag: v.direction.drag,
            mass: v.forces.mass,
            mass_random: v.forces.mass_random,
            mass_size_multiplier: v.forces.mass_size_multiplier,
            local_forces: v.forces.local_forces,
            world_forces: v.forces.world_forces,
            local_forces_fallback: v.forces.local_forces_fallback,
            world_forces_fallback: v.forces.world_forces_fallback,
            world_forces_mass_multiplier: v.forces.world_forces_mass_multiplier,
            noise_amplitude: v.noise.noise_amplitude,
            noise_frequency: v.noise.noise_frequency,
            noise_coherence: v.noise.noise_coherence,
            noise_edge: v.noise.noise_edge,
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

use crate::model::semantic::{Link, ValidateLinks};
impl ValidateLinks for ProgrammedRibbon<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.fallback_node));
        if let Some(id) = self.material {
            visitor(Link::Material(id));
        }
        self.motion.speed.visit_links(visitor);
        self.motion.variation.visit_links(visitor);
        self.motion.gravity.visit_links(visitor);
        self.lifetime.duration.visit_links(visitor);
        self.lifetime.variation.visit_links(visitor);
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
impl ValidateLinks for TrailEmitter {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.geometry.height_above.visit_links(visitor);
        self.geometry.height_below.visit_links(visitor);
        self.appearance.color.visit_links(visitor);
        self.appearance.opacity.visit_links(visitor);
        match &self.appearance.texture_slot {
            TrailAtlasSlot::Wide(value) => value.visit_links(visitor),
            TrailAtlasSlot::Compact(value) => value.visit_links(visitor),
        }
        match &self.rendering {
            TrailRendering::Layered { material } => material.visit_links(visitor),
            TrailRendering::Surfaces {
                textures,
                materials,
            } => {
                textures.visit_links(visitor);
                materials.visit_links(visitor);
            }
        }
        match &self.controls {
            TrailControls::Gravity {
                gravity,
                visibility,
            } => {
                visibility.visit_links(visitor);
                if !gravity.is_finite() {
                    visitor(Link::InvalidValue);
                }
            }
            TrailControls::EmissionAngle {
                angle,
                enabled,
                texture_transform,
                ..
            } => {
                if !angle.is_finite() {
                    visitor(Link::InvalidValue);
                }
                enabled.visit_links(visitor);
                if let Some(transform) = texture_transform {
                    transform.visit_links(visitor);
                }
            }
        }
        if !self.emission_rate.is_finite() || self.emission_rate < 0.0 || !self.lifetime.is_finite()
        {
            visitor(Link::InvalidValue);
        }
    }
}
