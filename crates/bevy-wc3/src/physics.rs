//! Optional, deterministic rigid-body secondary motion in authored model space.
mod contact;
mod fixtures;
mod mass;
mod response;
use crate::compiled::RuntimeNode;
use crate::diagnostics::{RenderDiagnosticKind, RenderDiagnosticSource};
use crate::pose::{hierarchical, local_matrix};
use crate::RenderDiagnostic;
use bevy::math::Affine3A;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use fixtures::Fixture;
use mass::Mass;
use response::Body as ImpulseBody;
use std::collections::HashMap;
use wc3::model::profiles::Universal;
use wc3::model::systems::model_physics::ShapeGeometry;
use wc3::model::systems::model_physics::{Body, Constraint, Frame, Joint, Shape};
use wc3::model::systems::{model::System as ModelSystem, System};
use wc3::model::values::model::{PhysicsBodyKind, PhysicsKind};
use wc3::model::SystemId;
use wc3::model::{Model, NodeId, Transform as ModelTransform};

/// Motion frame used for secondary-motion integration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PhysicsFrame {
    /// Preserve world-space body motion when the rig or its mount moves.
    #[default]
    World,
    /// Carry simulated bodies with the model frame; forces remain model-local.
    Model,
}

/// Opt-in rigid-body secondary motion for a model instance.
///
/// Forces use authored model coordinates and units, before `model_to_bevy`.
/// This bounded position solver is an approximation of native physics; it is not
/// a cloth solver. Removing the component restores authored animation immediately.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct PhysicsSimulation {
    /// Whether scene-frame motion produces inertial secondary motion.
    pub frame: PhysicsFrame,
    /// Model-space acceleration, in model units per second squared.
    pub gravity: Vec3,
    /// Additional model-space acceleration, such as application-supplied wind.
    pub acceleration: Vec3,
    /// Fixed integration step in seconds. Defaults to 1/120 second.
    pub timestep: f64,
    /// Maximum fixed steps per frame; excess elapsed time resets to animation.
    pub max_steps: u32,
    /// Position-constraint projection iterations per fixed step.
    pub iterations: u32,
    /// Frequency in Hz used to interpret nonzero authored pose-follow factors.
    pub follow_frequency: f32,
    /// Stop integration while preserving the simulated pose.
    pub paused: bool,
    /// Changing this generation resets bodies to the next authored pose.
    pub reset_generation: u64,
}
impl Default for PhysicsSimulation {
    fn default() -> Self {
        Self {
            frame: PhysicsFrame::World,
            gravity: Vec3::new(0., 0., -9.81),
            acceleration: Vec3::ZERO,
            timestep: 1. / 120.,
            max_steps: 16,
            iterations: 8,
            follow_frequency: 4.,
            paused: false,
            reset_generation: 0,
        }
    }
}
impl PhysicsSimulation {
    /// Reset velocities and body placements to the next sampled animation pose.
    pub fn reset(&mut self) {
        self.reset_generation = self.reset_generation.wrapping_add(1);
    }
}

/// An authored body's relationship to the visible skeleton.
#[derive(Clone, Debug)]
pub struct PhysicsBodyInfo {
    /// Authored simulation domain, independent of the opt-in solver policy.
    pub domain: PhysicsKind,
    /// Complete owned authored coefficients and collision geometry, suitable for
    /// application physics adapters. Node identities belong to the compiled rig.
    pub body: Body<'static>,
    /// Skeleton node driven by the body. M2 body placements are children of bones.
    pub target: NodeId,
}

#[derive(Clone)]
struct BodyDefinition {
    info: PhysicsBodyInfo,
    placement: usize,
    target: usize,
    offset_inverse: Mat4,
    gravity: f32,
    damping: f32,
    angular_damping: f32,
    follow: f32,
    inverse_mass: f32,
    inverse_inertia: Mat3,
    mass: Mass,
    fixtures: Vec<Fixture>,
    radius: f32,
}
#[derive(Clone)]
struct JointDefinition {
    a: usize,
    b: usize,
    anchor_a: Vec3,
    anchor_b: Vec3,
    rotation_a: Quat,
    rotation_b: Quat,
    authored: Joint,
}
#[derive(Clone, Default)]
pub(crate) struct PhysicsDefinition {
    bodies: Vec<BodyDefinition>,
    joints: Vec<JointDefinition>,
    pairs: Vec<(usize, usize)>,
}
fn diagnostic(
    diagnostics: &mut Vec<RenderDiagnostic>,
    system: SystemId,
    kind: RenderDiagnosticKind,
    message: impl Into<String>,
) {
    let message = message.into();
    let source = RenderDiagnosticSource::System(system);
    if diagnostics
        .iter()
        .any(|entry| entry.source == source && entry.kind == kind && entry.message == message)
    {
        return;
    }
    diagnostics.push(RenderDiagnostic::new(kind, source, message));
}
fn own_body(body: &Body<'_>) -> Body<'static> {
    Body {
        node: body.node,
        kind: body.kind,
        gravity_scale: body.gravity_scale,
        inertia_scale: body.inertia_scale,
        linear_damping: body.linear_damping,
        angular_damping: body.angular_damping,
        follow_factor: body.follow_factor,
        parent_body: body.parent_body,
        attachment_host: body.attachment_host,
        ragdoll_root: body.ragdoll_root,
        shapes: body
            .shapes
            .iter()
            .map(|shape| Shape {
                transform: shape.transform,
                geometry: match &shape.geometry {
                    ShapeGeometry::Box(value) => ShapeGeometry::Box(*value),
                    ShapeGeometry::Sphere(value) => ShapeGeometry::Sphere(*value),
                    ShapeGeometry::Capsule { radius, endpoints } => ShapeGeometry::Capsule {
                        radius: *radius,
                        endpoints: *endpoints,
                    },
                    ShapeGeometry::ConvexHull {
                        vertices,
                        planes,
                        faces,
                    } => ShapeGeometry::ConvexHull {
                        vertices: vertices.iter().collect(),
                        planes: planes.iter().collect(),
                        faces: faces.clone().into_owned(),
                    },
                },
                friction: shape.friction,
                restitution: shape.restitution,
                density: shape.density,
                mass_properties: shape.mass_properties,
                game_flags: shape.game_flags,
                collision_mask: shape.collision_mask,
            })
            .collect(),
    }
}
impl PhysicsDefinition {
    pub(crate) fn compile(
        model: &Model<'_, Universal>,
        nodes: &[RuntimeNode],
        diagnostics: &mut Vec<RenderDiagnostic>,
    ) -> Self {
        let indices: HashMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id, i))
            .collect();
        let mut output = Self::default();
        for (system_id, system) in model.systems() {
            let System::M2(ModelSystem::Physics(system)) = system else {
                continue;
            };
            if matches!(
                system.kind,
                PhysicsKind::Vegetation | PhysicsKind::Unknown(_)
            ) {
                diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, format!("Physics domain {:?} is not supported by the rigid-body solver; simulation remains application-owned", system.kind));
                continue;
            }
            if system.kind != PhysicsKind::WornItem {
                diagnostic(diagnostics, system_id, RenderDiagnosticKind::Approximation, format!("Physics domain {:?} uses opt-in local rigid constraints and declared body contacts; external-world collision and native ragdoll/game-state arbitration are not interpreted", system.kind));
            }
            if !system.follow_allow_list.is_empty()
                || system.wind_coefficients.is_some()
                || !system.vegetation.is_empty()
            {
                diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, "Native physics allow lists, wind coefficients and vegetation controls require an external physics integration");
            }
            diagnostic(diagnostics,system_id,RenderDiagnosticKind::Unverified,
                "Physics uses compound tensor inertia, point contacts, maximum restitution and geometric-mean Coulomb friction; native mass/contact fidelity and stable multipoint manifolds remain unverified");
            diagnostic(diagnostics,system_id,RenderDiagnosticKind::Approximation,
                "Animated nonuniform body scale uses conservative rounded contact bounds and fixed-mass affine inertia; kinematic targets interpolate over available fixed steps");
            let mut body_indices = HashMap::new();
            for body in &system.bodies {
                let Some(&placement) = indices.get(&body.node) else {
                    continue;
                };
                let node = &nodes[placement];
                // A placement beneath a bone carries a body offset, not another
                // skin joint. Driving only this child would never deform armor.
                let target = node.parent.unwrap_or(placement);
                if !hierarchical(&nodes[target]) {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, "Physics targets with billboarding or selective inheritance require an external solver integration");
                    continue;
                }
                let offset = if target == placement {
                    Mat4::IDENTITY
                } else {
                    local_matrix(node.rest)
                };
                if offset.determinant().abs() < 1e-8
                    || matches!(body.kind, PhysicsBodyKind::Unknown(_))
                {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, "Physics body with a singular offset or unknown motion kind is not simulated");
                    continue;
                }
                if output
                    .bodies
                    .iter()
                    .any(|candidate| candidate.target == target)
                {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, "Multiple physics bodies target the same skeleton node; subsequent bodies are not simulated");
                    continue;
                }
                if body.parent_body.is_some() || body.attachment_host || body.ragdoll_root {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unsupported, "Native attachment-host and ragdoll body controls need application physics policy");
                }
                let fixtures: Vec<_> = body.shapes.iter().map(Fixture::prepare).collect();
                if fixtures.iter().any(|(fixture, _)| fixture.approximate) {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Approximation,
                        "Physics convex hulls and nonuniform/sheared rounded fixtures use conservative per-fixture contact bounds; hull inertia uses its declared mass with a spherical tensor approximation");
                }
                if body
                    .shapes
                    .iter()
                    .any(|shape| shape.game_flags.bits() != 0 || shape.collision_mask.bits() != 0)
                {
                    diagnostic(diagnostics,system_id,RenderDiagnosticKind::Unsupported,
                        "Native physics game collision categories and contact masks need an external physics integration");
                }
                let mass = Mass::compound(fixtures.iter().map(|(_, mass)| *mass))
                    .filter(|mass| mass.mass.recip().is_finite())
                    .unwrap_or(Mass {
                        mass: 1e-3,
                        center: Vec3::ZERO,
                        second_moment: Mat3::from_diagonal(Vec3::splat(1e-7)),
                    });
                let center = mass.center;
                let radius = fixtures
                    .iter()
                    .map(|(f, _)| f.center.distance(center) + f.radius)
                    .fold(0., f32::max);
                let inverse_inertia = if body.kind == PhysicsBodyKind::Dynamic {
                    mass.inverse_inertia(body.inertia_scale.max(1e-3))
                        .unwrap_or(Mat3::ZERO)
                } else {
                    Mat3::ZERO
                };
                body_indices.insert(body.node, output.bodies.len());
                output.bodies.push(BodyDefinition {
                    info: PhysicsBodyInfo {
                        domain: system.kind,
                        body: own_body(body),
                        target: nodes[target].id,
                    },
                    placement,
                    target,
                    offset_inverse: offset.inverse(),
                    gravity: body.gravity_scale,
                    damping: body.linear_damping.max(0.),
                    angular_damping: body.angular_damping.max(0.),
                    follow: body.follow_factor.max(0.),
                    inverse_mass: if body.kind == PhysicsBodyKind::Dynamic {
                        mass.mass.recip()
                    } else {
                        0.
                    },
                    inverse_inertia,
                    mass,
                    fixtures: fixtures.into_iter().map(|(fixture, _)| fixture).collect(),
                    radius,
                });
            }
            for joint in &system.joints {
                diagnostic(diagnostics, system_id, RenderDiagnosticKind::Approximation, "Secondary-motion constraints approximate native physics: spring damping ratios, angular friction and motors are not interpreted");
                let (Some(&a), Some(&b)) = (
                    body_indices.get(&joint.body_a),
                    body_indices.get(&joint.body_b),
                ) else {
                    continue;
                };
                let frame_rotation = |frame: Frame| {
                    let matrix = Mat3::from_cols(frame.axis_x, frame.axis_y, frame.axis_z);
                    if matrix.determinant().abs() < 1e-6 {
                        Quat::IDENTITY
                    } else {
                        Quat::from_mat3(&matrix).normalize()
                    }
                };
                if matches!(joint.constraint, Constraint::Prismatic { .. }) {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Approximation, "Physics slider limits currently project anchor coincidence only; native slider behavior needs an external solver");
                }
                if matches!(
                    joint.constraint,
                    Constraint::Shoulder { .. } | Constraint::Revolute { .. }
                ) {
                    diagnostic(diagnostics, system_id, RenderDiagnosticKind::Unverified, "Physics cone, twist and hinge limits use the authored frame Z axis; native solver response remains unverified");
                }
                output.joints.push(JointDefinition {
                    a,
                    b,
                    anchor_a: joint.frame_a.origin,
                    anchor_b: joint.frame_b.origin,
                    rotation_a: frame_rotation(joint.frame_a),
                    rotation_b: frame_rotation(joint.frame_b),
                    authored: joint.clone(),
                });
            }
            // Empty does not mean all-to-all: this list explicitly grants contact.
            for &(a, b) in &system.collision_pairs {
                if let (Some(&a), Some(&b)) = (body_indices.get(&a), body_indices.get(&b)) {
                    if a != b {
                        if output.bodies[a]
                            .fixtures
                            .iter()
                            .any(|fa| output.bodies[b].fixtures.iter().any(|fb| !fa.supports(fb)))
                        {
                            diagnostic(diagnostics,system_id,RenderDiagnosticKind::Approximation,
                                "Physics box-box and capsule-box pairs use per-fixture bounding spheres; exact supported capsule contacts are retained elsewhere in the compound");
                        }
                        output.pairs.push((a, b));
                    }
                }
            }
        }
        output
    }
    /// Append bodies from another compiled part after its node identities are remapped.
    pub(crate) fn append_remapped(
        &mut self,
        source: &Self,
        nodes: &[usize],
        identities: &[RuntimeNode],
    ) {
        let mut remap = HashMap::new();
        let mut added = Vec::new();
        for (index, body) in source.bodies.iter().enumerate() {
            let mut body = body.clone();
            body.placement = nodes[body.placement];
            body.target = nodes[body.target];
            body.info.body.node = identities[body.placement].id;
            body.info.target = identities[body.target].id;
            // Composition may insert a rest-space correction anchor between the
            // bound host bone and this placement. Include that entire path so
            // activating physics never applies the correction to the host twice.
            let mut cursor = body.placement;
            let mut offset = Mat4::IDENTITY;
            while cursor != body.target {
                offset = local_matrix(identities[cursor].rest) * offset;
                let Some(parent) = identities[cursor].parent else {
                    break;
                };
                cursor = parent;
            }
            if cursor != body.target || offset.determinant().abs() < 1e-8 {
                continue;
            }
            body.offset_inverse = offset.inverse();
            // Shared host bones have one authority. Constraints can still refer
            // to that existing body through the remap, rather than disappearing.
            if let Some(existing) = self
                .bodies
                .iter()
                .position(|value| value.target == body.target)
            {
                remap.insert(index, existing);
                continue;
            }
            remap.insert(index, self.bodies.len());
            added.push((index, self.bodies.len()));
            self.bodies.push(body);
        }
        for (old, new) in added {
            let parent = source.bodies[old]
                .info
                .body
                .parent_body
                .and_then(|id| {
                    source
                        .bodies
                        .iter()
                        .position(|candidate| candidate.info.body.node == id)
                })
                .and_then(|index| remap.get(&index).copied())
                .map(|index| self.bodies[index].info.body.node);
            self.bodies[new].info.body.parent_body = parent;
        }
        for joint in &source.joints {
            if let (Some(&a), Some(&b)) = (remap.get(&joint.a), remap.get(&joint.b)) {
                if a == b {
                    continue;
                }
                let mut joint = joint.clone();
                joint.a = a;
                joint.b = b;
                joint.authored.body_a = self.bodies[a].info.body.node;
                joint.authored.body_b = self.bodies[b].info.body.node;
                self.joints.push(joint);
            }
        }
        for (a, b) in &source.pairs {
            if let (Some(&a), Some(&b)) = (remap.get(a), remap.get(b)) {
                if a != b {
                    self.pairs.push((a, b));
                }
            }
        }
    }
    pub(crate) fn bodies(&self) -> impl Iterator<Item = &PhysicsBodyInfo> {
        self.bodies.iter().map(|body| &body.info)
    }
    pub(crate) fn joints(&self) -> impl Iterator<Item = &Joint> {
        self.joints.iter().map(|joint| &joint.authored)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }
}

#[derive(Clone, Copy)]
struct BodyState {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    velocity: Vec3,
    angular_velocity: Vec3,
    center: Vec3,
}
impl BodyState {
    fn from_matrix(matrix: Mat4, center: Vec3) -> Self {
        let (scale, rotation, position) = matrix.to_scale_rotation_translation();
        Self {
            position: position + rotation * (scale * center),
            rotation,
            scale,
            center,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }
    fn matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.position - self.rotation * (self.scale * self.center),
        )
    }
}
// Swing/twist decomposition about the authored joint frame's Z axis. The
// antiparallel swing case has undefined twist; choose zero twist deterministically.
fn limited_rotation(relative: Quat, limits: [f64; 2], cone: f32) -> Quat {
    let relative = if relative.w < 0. { -relative } else { relative };
    let twist_length = (relative.z * relative.z + relative.w * relative.w).sqrt();
    let twist = if twist_length > 1e-8 {
        Quat::from_xyzw(0., 0., relative.z / twist_length, relative.w / twist_length)
    } else {
        Quat::IDENTITY
    };
    let angle = 2. * twist.z.atan2(twist.w);
    let lower = (limits[0] as f32).clamp(-std::f32::consts::PI, std::f32::consts::PI);
    let upper = (limits[1] as f32).clamp(lower, std::f32::consts::PI);
    let swing = relative * twist.inverse();
    let (axis, swing_angle) = swing.to_axis_angle();
    let swing = Quat::from_axis_angle(axis, swing_angle.min(cone.clamp(0., std::f32::consts::PI)));
    (swing * Quat::from_rotation_z(angle.clamp(lower, upper))).normalize()
}

/// Per-instance storage; no allocation occurs after initializing a fixed definition.
#[derive(Default)]
pub(crate) struct PhysicsState {
    bodies: Vec<BodyState>,
    targets: Vec<BodyState>,
    previous: Vec<Vec3>,
    previous_rotations: Vec<Quat>,
    output: Vec<Option<Mat4>>,
    weights: Vec<f32>,
    local_inertias: Vec<Mat3>,
    time: Option<f64>,
    remainder: f64,
    reset: u64,
    traversal: Option<u64>,
    frame: Option<Affine3A>,
    frame_failure: Option<&'static str>,
}
impl PhysicsState {
    pub(crate) fn synchronize_playback(&mut self, revision: u64) {
        if self.traversal != Some(revision) {
            self.clear();
            self.traversal = Some(revision);
        }
    }
    pub(crate) fn is_initialized(&self) -> bool {
        self.time.is_some()
    }
    pub(crate) fn clear(&mut self) {
        self.time = None;
        self.frame = None;
        self.output.clear();
    }
    fn frame_error(&mut self, message: &'static str) {
        if self.frame_failure != Some(message) {
            warn!("Model secondary motion: {message}");
        }
        self.frame_failure = Some(message);
        self.clear();
    }
    /// Re-express inertial body state in the current authored frame without
    /// estimating acceleration or introducing a force from rendering warmup.
    pub(crate) fn rebase(
        &mut self,
        definition: &PhysicsDefinition,
        settings: &PhysicsSimulation,
        time: f64,
        current: Option<Affine3A>,
    ) {
        if settings.frame == PhysicsFrame::Model {
            self.frame = None;
            self.frame_failure = None;
            return;
        }
        let Some(current) = current else {
            self.frame_error("current rig frame could not be resolved; bodies reset to animation");
            return;
        };
        let determinant = current.matrix3.determinant();
        if !Mat4::from(current).is_finite() || determinant.abs() < 1e-20 {
            self.frame_error("nonfinite or singular rig frame; bodies reset to animation");
            return;
        }
        let continuous = self.time.is_some_and(|previous| {
            time >= previous
                && time - previous <= settings.timestep * f64::from(settings.max_steps.min(1024))
        }) && self.reset == settings.reset_generation;
        if let Some(previous) = self
            .frame
            .filter(|previous| continuous && *previous != current)
        {
            let delta = current.inverse() * previous;
            let x = Vec3::from(delta.matrix3.x_axis);
            let y = Vec3::from(delta.matrix3.y_axis);
            let z = Vec3::from(delta.matrix3.z_axis);
            let rigid = Mat4::from(delta).is_finite()
                && (delta.matrix3.determinant() - 1.).abs() < 1e-4
                && (x.length_squared() - 1.).abs() < 1e-4
                && (y.length_squared() - 1.).abs() < 1e-4
                && (z.length_squared() - 1.).abs() < 1e-4
                && x.dot(y).abs() < 1e-4
                && x.dot(z).abs() < 1e-4
                && y.dot(z).abs() < 1e-4;
            if !rigid {
                self.frame_error(
                    "changing scale, reflection or shear in rig frame; bodies reset to animation",
                );
                self.frame = Some(current);
                return;
            }
            for (body, definition) in self.bodies.iter_mut().zip(&definition.bodies) {
                let velocity = delta.transform_vector3(body.velocity);
                let angular_velocity = delta.transform_vector3(body.angular_velocity);
                *body = BodyState::from_matrix(
                    Mat4::from(delta) * body.matrix(),
                    definition.mass.center,
                );
                body.velocity = velocity;
                body.angular_velocity = angular_velocity;
            }
        }
        self.frame = Some(current);
        self.frame_failure = None;
    }
    pub(crate) fn step<'a>(
        &'a mut self,
        definition: &PhysicsDefinition,
        settings: &PhysicsSimulation,
        time: f64,
        authored: &[Mat4],
        overrides: &HashMap<NodeId, ModelTransform>,
    ) -> &'a [Option<Mat4>] {
        self.output.resize(authored.len(), None);
        self.output.fill(None);
        if definition.is_empty() {
            return &self.output;
        }
        let valid = settings.timestep.is_finite()
            && (1e-5..=0.1).contains(&settings.timestep)
            && settings.gravity.is_finite()
            && settings.acceleration.is_finite()
            && settings.follow_frequency.is_finite()
            && time.is_finite();
        if !valid {
            self.clear();
            return &self.output;
        }
        let elapsed = self.time.map_or(0., |previous| time - previous);
        let reset = self.time.is_none()
            || self.reset != settings.reset_generation
            || self.bodies.len() != definition.bodies.len()
            || elapsed < 0.
            || (!settings.paused
                && elapsed > settings.timestep * f64::from(settings.max_steps.min(1024)));
        self.time = Some(time);
        self.reset = settings.reset_generation;
        self.weights.clear();
        self.weights.extend(definition.bodies.iter().map(|body| {
            if overrides.contains_key(&body.info.target) {
                0.
            } else {
                body.inverse_mass
            }
        }));
        self.targets.clear();
        self.targets.extend(
            definition
                .bodies
                .iter()
                .map(|body| BodyState::from_matrix(authored[body.placement], body.mass.center)),
        );
        if self.targets.iter().any(|body| {
            !body.position.is_finite() || !body.rotation.is_finite() || !body.scale.is_finite()
        }) {
            self.clear();
            return &self.output;
        }
        if reset {
            self.bodies.clone_from(&self.targets);
            self.previous.resize(self.bodies.len(), Vec3::ZERO);
            self.previous_rotations
                .resize(self.bodies.len(), Quat::IDENTITY);
            self.local_inertias.resize(self.bodies.len(), Mat3::ZERO);
            self.remainder = 0.;
        } else if !settings.paused {
            self.remainder += elapsed;
            let step = settings.timestep as f32;
            let iterations = settings.iterations.clamp(1, 64);
            let frequency = settings.follow_frequency.clamp(0., 100.);
            let mut remaining_steps = ((self.remainder + 1e-12) / settings.timestep)
                .floor()
                .max(1.) as u32;
            while self.remainder + 1e-12 >= settings.timestep {
                self.remainder -= settings.timestep;
                for (index, ((body, target), def)) in self
                    .bodies
                    .iter_mut()
                    .zip(&self.targets)
                    .zip(&definition.bodies)
                    .enumerate()
                {
                    self.previous[index] = body.position;
                    self.previous_rotations[index] = body.rotation;
                    if self.weights[index] == 0. {
                        if def.info.body.kind != PhysicsBodyKind::Static {
                            let fraction = 1. / remaining_steps as f32;
                            body.position = body.position.lerp(target.position, fraction);
                            body.rotation =
                                body.rotation.slerp(target.rotation, fraction).normalize();
                            body.scale = body.scale.lerp(target.scale, fraction);
                            body.velocity = (body.position - self.previous[index]) / step;
                            let delta = body.rotation * self.previous_rotations[index].inverse();
                            body.angular_velocity =
                                if delta.w < 0. { -delta } else { delta }.to_scaled_axis() / step;
                        }
                        continue;
                    }
                    let follow = (def.follow * frequency * std::f32::consts::TAU).min(100.);
                    // Implicit critically damped spring avoids stiffness-dependent
                    // explosion when native follow coefficients are large.
                    let acceleration = settings.gravity * def.gravity + settings.acceleration;
                    body.velocity = (body.velocity
                        + step
                            * (acceleration + follow * follow * (target.position - body.position)))
                        / (1. + step * (def.damping + 2. * follow) + step * step * follow * follow);
                    body.position += body.velocity * step;
                    body.angular_velocity *= (-def.angular_damping * step).exp();
                    body.rotation = (Quat::from_scaled_axis(body.angular_velocity * step)
                        * body.rotation)
                        .normalize();
                    body.rotation = body
                        .rotation
                        .slerp(target.rotation, 1. - (-follow * step).exp());
                    body.scale = target.scale;
                }
                remaining_steps = remaining_steps.saturating_sub(1).max(1);
                self.local_inertias.clear();
                self.local_inertias.extend(
                    self.bodies
                        .iter()
                        .zip(&definition.bodies)
                        .zip(&self.weights)
                        .map(|((body, def), weight)| {
                            if *weight == 0. {
                                Mat3::ZERO
                            } else if body.scale == Vec3::ONE {
                                def.inverse_inertia
                            } else {
                                let scale = Mat3::from_diagonal(body.scale);
                                let mass = Mass {
                                    second_moment: scale
                                        * def.mass.second_moment
                                        * scale.transpose(),
                                    ..def.mass
                                };
                                mass.inverse_inertia(def.info.body.inertia_scale.max(1e-3))
                                    .unwrap_or(Mat3::ZERO)
                            }
                        }),
                );
                for _ in 0..iterations {
                    for joint in &definition.joints {
                        self.project_joint(joint, step, iterations);
                    }
                    for &(a, b) in &definition.pairs {
                        self.project_contact(definition, a, b);
                    }
                }
                for (index, (body, def)) in
                    self.bodies.iter_mut().zip(&definition.bodies).enumerate()
                {
                    if def.inverse_mass != 0. {
                        body.velocity = (body.position - self.previous[index]) / step;
                        let delta = body.rotation * self.previous_rotations[index].inverse();
                        body.angular_velocity =
                            if delta.w < 0. { -delta } else { delta }.to_scaled_axis() / step;
                    }
                }
                for &(a, b) in &definition.pairs {
                    self.resolve_contact_velocity(definition, a, b);
                }
            }
        } else {
            // Pause discards kinematic target history; resuming must not replay
            // displacement accumulated while contact integration was disabled.
            for ((body, target), (def, weight)) in self
                .bodies
                .iter_mut()
                .zip(&self.targets)
                .zip(definition.bodies.iter().zip(&self.weights))
            {
                if *weight == 0. && def.info.body.kind != PhysicsBodyKind::Static {
                    *body = *target;
                }
            }
        }
        if self.bodies.iter().any(|body| {
            !body.position.is_finite()
                || !body.rotation.is_finite()
                || !body.velocity.is_finite()
                || !body.angular_velocity.is_finite()
        }) {
            self.frame_error("nonfinite secondary-motion state; bodies reset to animation");
            return &self.output;
        }
        for (body, def) in self.bodies.iter().zip(&definition.bodies) {
            if def.info.body.kind != PhysicsBodyKind::Kinematic
                && !overrides.contains_key(&def.info.target)
            {
                let matrix = body.matrix() * def.offset_inverse;
                if matrix.is_finite() {
                    self.output[def.target] = Some(matrix);
                }
            }
        }
        &self.output
    }
    fn project_joint(&mut self, joint: &JointDefinition, step: f32, iterations: u32) {
        let a = self.bodies[joint.a];
        let b = self.bodies[joint.b];
        let wa = self.weights[joint.a];
        let wb = self.weights[joint.b];
        if wa + wb == 0. {
            return;
        }
        let pa = a.position + a.rotation * (a.scale * (joint.anchor_a - a.center));
        let pb = b.position + b.rotation * (b.scale * (joint.anchor_b - b.center));
        let difference = pb - pa;
        let (distance, frequency) = match &joint.authored.constraint {
            Constraint::Distance { distance, spring } => (distance.max(0.), spring.frequency_hz),
            Constraint::Weld { linear, .. } => (0., linear.frequency_hz),
            _ => (0., 0.),
        };
        let length = difference.length();
        let compliance = if frequency > 0. {
            1. - (-(std::f32::consts::TAU * frequency * step) / iterations as f32).exp()
        } else {
            1.
        };
        let normal = difference.try_normalize().unwrap_or(Vec3::X);
        let ra = pa - a.position;
        let rb = pb - b.position;
        let ia = self.world_inverse_inertia(joint.a);
        let ib = self.world_inverse_inertia(joint.b);
        let effective_mass = wa
            + wb
            + ra.cross(normal).dot(ia * ra.cross(normal))
            + rb.cross(normal).dot(ib * rb.cross(normal));
        let impulse = normal * ((length - distance) * compliance / effective_mass.max(1e-8));
        self.bodies[joint.a].position += impulse * wa;
        self.bodies[joint.b].position -= impulse * wb;
        self.bodies[joint.a].rotation =
            (Quat::from_scaled_axis(ia * ra.cross(impulse)) * a.rotation).normalize();
        self.bodies[joint.b].rotation =
            (Quat::from_scaled_axis(-(ib * rb.cross(impulse))) * b.rotation).normalize();
        let a = self.bodies[joint.a];
        let b = self.bodies[joint.b];
        let relative = (a.rotation * joint.rotation_a).inverse() * b.rotation * joint.rotation_b;
        let (desired, response) = match joint.authored.constraint {
            Constraint::Weld { angular, .. } => {
                let response = if angular.frequency_hz > 0. {
                    1. - (-(std::f32::consts::TAU * angular.frequency_hz * step)
                        / iterations as f32)
                        .exp()
                } else {
                    1.
                };
                (Quat::IDENTITY, response)
            }
            Constraint::Shoulder { twist, cone, .. } => {
                (limited_rotation(relative, twist, cone as f32), 1.)
            }
            Constraint::Revolute { angles, .. } => (limited_rotation(relative, angles, 0.), 1.),
            _ => return,
        };
        let ia = self.world_inverse_inertia(joint.a);
        let ib = self.world_inverse_inertia(joint.b);
        {
            let desired_b = a.rotation * joint.rotation_a * desired * joint.rotation_b.inverse();
            let difference = desired_b * b.rotation.inverse();
            let correction = if difference.w < 0. {
                -difference
            } else {
                difference
            }
            .to_scaled_axis()
                * response;
            let Some(axis) = correction.try_normalize() else {
                return;
            };
            let effective = axis.dot((ia + ib) * axis);
            if effective <= 1e-12 {
                return;
            }
            let impulse = axis * (correction.length() / effective);
            self.bodies[joint.a].rotation =
                (Quat::from_scaled_axis(-(ia * impulse)) * a.rotation).normalize();
            self.bodies[joint.b].rotation =
                (Quat::from_scaled_axis(ib * impulse) * b.rotation).normalize();
        }
    }

    fn world_inverse_inertia(&self, index: usize) -> Mat3 {
        let rotation = Mat3::from_quat(self.bodies[index].rotation);
        rotation * self.local_inertias[index] * rotation.transpose()
    }
    fn near(&self, definition: &PhysicsDefinition, a: usize, b: usize, margin: f32) -> bool {
        let radius = definition.bodies[a].radius * self.bodies[a].scale.abs().max_element()
            + definition.bodies[b].radius * self.bodies[b].scale.abs().max_element()
            + margin;
        self.bodies[a]
            .position
            .distance_squared(self.bodies[b].position)
            <= radius * radius
    }
    fn project_contact(&mut self, definition: &PhysicsDefinition, a: usize, b: usize) {
        if self.weights[a] + self.weights[b] == 0. || !self.near(definition, a, b, 0.) {
            return;
        }
        for left in &definition.bodies[a].fixtures {
            for right in &definition.bodies[b].fixtures {
                let Some(contact) =
                    left.contact(self.bodies[a].matrix(), right, self.bodies[b].matrix(), 0.)
                else {
                    continue;
                };
                if contact.depth <= 0. {
                    continue;
                }
                let point = (contact.points[0] + contact.points[1]) * 0.5;
                let ra = point - self.bodies[a].position;
                let rb = point - self.bodies[b].position;
                let ia = self.world_inverse_inertia(a);
                let ib = self.world_inverse_inertia(b);
                let normal = contact.normal;
                let denominator = self.weights[a]
                    + self.weights[b]
                    + ra.cross(normal).dot(ia * ra.cross(normal))
                    + rb.cross(normal).dot(ib * rb.cross(normal));
                if denominator <= 1e-12 {
                    continue;
                }
                let impulse = normal * (contact.depth / denominator);
                for (index, linear, angular) in [
                    (a, -impulse * self.weights[a], -(ia * ra.cross(impulse))),
                    (b, impulse * self.weights[b], ib * rb.cross(impulse)),
                ] {
                    let rotation = Quat::from_scaled_axis(angular);
                    self.bodies[index].position += linear;
                    self.bodies[index].rotation =
                        (rotation * self.bodies[index].rotation).normalize();
                    // Repair both pose histories so geometric separation does not inject energy.
                    self.previous[index] += linear;
                    self.previous_rotations[index] =
                        (rotation * self.previous_rotations[index]).normalize();
                }
            }
        }
    }
    fn resolve_contact_velocity(&mut self, definition: &PhysicsDefinition, a: usize, b: usize) {
        if self.weights[a] + self.weights[b] == 0. || !self.near(definition, a, b, 1e-5) {
            return;
        }
        for left in &definition.bodies[a].fixtures {
            for right in &definition.bodies[b].fixtures {
                let Some(contact) = left.contact(
                    self.bodies[a].matrix(),
                    right,
                    self.bodies[b].matrix(),
                    1e-5,
                ) else {
                    continue;
                };
                let impulse_body = |index: usize| ImpulseBody {
                    center: self.bodies[index].position,
                    velocity: self.bodies[index].velocity,
                    angular_velocity: self.bodies[index].angular_velocity,
                    inverse_mass: self.weights[index],
                    inverse_inertia: self.world_inverse_inertia(index),
                };
                let (mut va, mut vb) = (impulse_body(a), impulse_body(b));
                let friction = left.friction.sqrt() * right.friction.sqrt();
                let Some(impulse) = response::resolve(
                    &mut va,
                    &mut vb,
                    contact,
                    left.restitution.max(right.restitution),
                    friction,
                ) else {
                    continue;
                };
                debug_assert!(impulse.normal.is_finite() && impulse.tangent.is_finite());
                self.bodies[a].velocity = va.velocity;
                self.bodies[a].angular_velocity = va.angular_velocity;
                self.bodies[b].velocity = vb.velocity;
                self.bodies[b].angular_velocity = vb.angular_velocity;
            }
        }
    }
}

#[cfg(test)]
#[path = "physics/tests.rs"]
mod tests;
