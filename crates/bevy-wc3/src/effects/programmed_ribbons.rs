//! Endpoint-anchored M3 beams, without guessing emitted ribbon simulation.
use super::super::material::{ModelMaterial, SurfaceAnimation, SurfacePass};
use super::frame::EffectFrame;
use super::programmed::{animated, nondefault_scalar, nondefault_variation, ProgrammedAppearance};
use super::RibbonEmitter;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::values::programmed::{RibbonKind, Smoothing};
use wc3::model::{
    Animated, ClipId, FeatureId, MaterialId, Model, NodeFeature, NodeId, ProgrammedAge,
    RibbonEmitter as SemanticRibbonEmitter,
};

const MAX_ANCHORS: usize = 256;
const MAX_SEGMENTS: usize = 256;
#[derive(Clone)]
struct Definition {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    points: Vec<(usize, Vec3)>,
    appearance: ProgrammedAge,
    smoothing: [Smoothing; 2],
    kind: RibbonKind,
    segments: usize,
    max_length: Animated<f32>,
    active: Animated<u32>,
    passes: Vec<Pass>,
}
#[derive(Clone)]
struct Pass {
    material: ModelMaterial,
    animation: Arc<SurfaceAnimation>,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeProgrammedRibbons {
    definitions: Vec<Definition>,
    diagnostics: Vec<(FeatureId, String)>,
}
struct State {
    mesh: Handle<Mesh>,
    passes: Vec<PassState>,
    points: Vec<Vec3>,
    sampled: Vec<(Vec3, f32)>,
    appearance: Option<ProgrammedAppearance>,
    camera: Option<Mat4>,
    basis: Option<Mat4>,
    anchor: Option<Mat4>,
    max_length: f32,
    visible: bool,
}
struct PassState {
    entity: Entity,
    material: Handle<ModelMaterial>,
}
#[derive(Default)]
pub(super) struct ProgrammedRibbonInstances {
    states: Vec<State>,
}
impl RuntimeProgrammedRibbons {
    pub(super) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        self.definitions
            .extend(source.definitions.iter().cloned().map(|mut value| {
                value.node = nodes[&value.node];
                value.anchor = indices[value.anchor];
                for point in &mut value.points {
                    point.0 = indices[point.0];
                }
                value
            }));
        self.diagnostics.extend(source.diagnostics.iter().cloned());
        let _ = clips;
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
    pub fn diagnostics(&self) -> &[(FeatureId, String)] {
        &self.diagnostics
    }
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    ) -> Self {
        let mut result = Self::default();
        for (feature, attached) in model.features() {
            let NodeFeature::Ribbon(SemanticRibbonEmitter::Programmed(definition)) =
                &attached.payload
            else {
                continue;
            };
            let mut diagnose =
                |message: &str| result.diagnostics.push((feature, message.to_owned()));
            if definition.spline_points.is_empty() {
                diagnose(
                    "M3 ribbons without endpoint anchors require emitted-section motion and lifespan simulation; this mode is not implemented",
                );
                continue;
            }
            if !matches!(
                definition.ribbon_type,
                RibbonKind::Billboard | RibbonKind::Planar
            ) {
                diagnose("M3 cylinder/star/unknown ribbon geometry is not implemented");
                continue;
            }
            if definition.spline_points.len() >= MAX_ANCHORS {
                diagnose("M3 endpoint ribbon exceeds the explicit 256-anchor geometry budget");
                continue;
            }
            if ![definition.size_smoothing, definition.color_smoothing]
                .iter()
                .all(|mode| matches!(mode, Smoothing::Linear | Smoothing::LinearSmooth))
            {
                diagnose("M3 endpoint ribbon Bezier/hold/unknown age smoothing is not implemented");
                continue;
            }
            let Some(passes) = definition.material.and_then(|id| materials.get(&id)) else {
                diagnose("M3 endpoint ribbon material does not resolve to a compiled PBR surface");
                continue;
            };
            if passes.is_empty() {
                continue;
            }
            if [
                &definition.yaw,
                &definition.pitch,
                &definition.speed,
                &definition.size,
                &definition.alpha,
            ]
            .into_iter()
            .any(|value| nondefault_variation(value, model))
            {
                diagnose(
                    "M3 endpoint ribbon yaw/pitch/speed/size/alpha variation controls are not implemented",
                );
            }
            if nondefault_scalar(&definition.overlay, model)
                || nondefault_scalar(&definition.particle_velocity, model)
            {
                diagnose(
                    "M3 endpoint ribbon overlay and particle_velocity controls are not implemented",
                );
            }
            if definition.flags.edge_falloff() || definition.flags.vertex_alpha() {
                diagnose(
                    "M3 endpoint ribbon edge_falloff and vertex_alpha flags are not implemented",
                );
            }
            diagnose(
                "M3 endpoint ribbons use piecewise-linear anchor paths with normalized width/color/rotation, bounded to 256 segments; authored tangent/velocity variations, noise and exact game fidelity remain unimplemented or unverified",
            );
            result.definitions.push(Definition {
                feature,
                node: attached.node,
                anchor: nodes[&attached.node],
                points: definition
                    .spline_points
                    .iter()
                    .map(|point| (nodes[&point.node], Vec3::from(point.emission_offset)))
                    .collect(),
                appearance: definition.appearance.clone(),
                smoothing: [definition.size_smoothing, definition.color_smoothing],
                kind: definition.ribbon_type,
                segments: (definition.divisions.ceil().max(1.) as usize).min(MAX_SEGMENTS),
                max_length: animated(&definition.max_length),
                active: animated(&definition.active),
                passes: passes
                    .iter()
                    .map(|pass| {
                        let mut material = ModelMaterial {
                            base: pass.material.clone(),
                            extension: pass.state.clone(),
                        };
                        material.extension.enable_effect_color();
                        material.base.cull_mode = None;
                        Pass {
                            material,
                            animation: Arc::new(pass.animation.clone()),
                        }
                    })
                    .collect(),
            });
        }
        result
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ModelMaterial>,
        basis: Entity,
    ) -> ProgrammedRibbonInstances {
        ProgrammedRibbonInstances {
            states: self
                .definitions
                .iter()
                .map(|definition| {
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; 3]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; 3]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.; 2]; 3]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.; 4]; 3]);
                    let mesh = meshes.add(mesh);
                    let passes = definition
                        .passes
                        .iter()
                        .enumerate()
                        .map(|(index, pass)| {
                            let material = materials.add(pass.material.clone());
                            let entity = commands
                                .spawn((
                                    Mesh3d(mesh.clone()),
                                    MeshMaterial3d(material.clone()),
                                    Transform::default(),
                                    Visibility::default(),
                                    NoFrustumCulling,
                                    ChildOf(basis),
                                ))
                                .id();
                            if index == 0 {
                                commands.entity(entity).insert(RibbonEmitter {
                                    feature: definition.feature,
                                    node: definition.node,
                                    sections: 0,
                                    segments: 0,
                                    omitted_sections: 0,
                                });
                            }
                            if !pass.material.extension.casts_shadows() {
                                commands.entity(entity).insert(NotShadowCaster);
                            }
                            if !pass.material.extension.receives_shadows() {
                                commands.entity(entity).insert(NotShadowReceiver);
                            }
                            PassState { entity, material }
                        })
                        .collect();
                    State {
                        mesh,
                        passes,
                        points: Vec::new(),
                        sampled: Vec::new(),
                        appearance: None,
                        camera: None,
                        basis: None,
                        anchor: None,
                        max_length: 0.,
                        visible: false,
                    }
                })
                .collect(),
        }
    }
    pub fn update(
        &self,
        states: &mut ProgrammedRibbonInstances,
        frame: &EffectFrame<'_, '_, '_>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ModelMaterial>,
        statistics: &mut Query<&mut RibbonEmitter>,
    ) {
        let sample = frame.sample.at_times(
            frame.selection.clip_time,
            frame.selection.timeline_time,
            frame.selection.shared_time,
        );
        for (definition, state) in self.definitions.iter().zip(&mut states.states) {
            for (pass, state) in definition.passes.iter().zip(&state.passes) {
                if pass.animation.is_animated() {
                    pass.animation
                        .apply_to_assets(frame.sample, &state.material, materials);
                }
            }
            let anchor = frame.basis_world * frame.feature_matrix(definition.anchor);
            let appearance = ProgrammedAppearance::from_age(
                &definition.appearance,
                &sample,
                definition.smoothing[0],
                definition.smoothing[1],
            );
            let max_length = frame.sample.animated(&definition.max_length);
            let visible = frame.sample.animated(&definition.active) != 0
                && frame
                    .node_visible
                    .get(definition.anchor)
                    .copied()
                    .unwrap_or(true);
            let mut changed = state.visible != visible
                || state.basis != Some(frame.basis_world)
                || state.anchor != Some(anchor)
                || state.camera != frame.camera_world
                || state.appearance.as_ref() != Some(&appearance)
                || state.max_length != max_length;
            let count = definition.points.len() + 1;
            changed |= state.points.len() != count;
            state.points.resize(count, Vec3::ZERO);
            let origin = anchor.w_axis.truncate();
            changed |= state.points[0] != origin;
            state.points[0] = origin;
            for (index, (node, offset)) in definition.points.iter().enumerate() {
                let position =
                    (frame.basis_world * frame.feature_matrix(*node)).transform_point3(*offset);
                changed |= state.points[index + 1] != position;
                state.points[index + 1] = position;
            }
            if !changed {
                continue;
            }
            state.basis = Some(frame.basis_world);
            state.anchor = Some(anchor);
            state.camera = frame.camera_world;
            state.max_length = max_length;
            state.visible = visible;
            let width_scale = anchor.y_axis.truncate().length();
            let limit = if max_length.is_finite() && max_length >= 0. {
                max_length * width_scale
            } else {
                f32::INFINITY
            };
            if visible {
                sample_path(
                    &state.points,
                    definition.segments,
                    limit,
                    &mut state.sampled,
                );
            } else {
                state.sampled.clear();
            }
            if let Some(mut mesh) = meshes.get_mut(&state.mesh) {
                draw_mesh(
                    &mut mesh,
                    &state.sampled,
                    &appearance,
                    definition.kind,
                    anchor,
                    frame.basis_world,
                    frame.camera_world,
                );
            }
            state.appearance = Some(appearance);
            if let Some(pass) = state.passes.first() {
                if let Ok(mut statistics) = statistics.get_mut(pass.entity) {
                    let sections = state.sampled.len();
                    let segments = sections.saturating_sub(1);
                    if statistics.sections != sections {
                        statistics.sections = sections;
                    }
                    if statistics.segments != segments {
                        statistics.segments = segments;
                    }
                }
            }
        }
    }
}
fn sample_path(points: &[Vec3], segments: usize, limit: f32, output: &mut Vec<(Vec3, f32)>) {
    output.clear();
    let length = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum::<f32>()
        .min(limit);
    if !length.is_finite() || length <= 1e-8 {
        return;
    }
    let mut edge = 0;
    let mut prefix = 0.;
    for index in 0..=segments {
        let t = index as f32 / segments as f32;
        let distance = t * length;
        while edge + 2 < points.len() && prefix + points[edge].distance(points[edge + 1]) < distance
        {
            prefix += points[edge].distance(points[edge + 1]);
            edge += 1;
        }
        let edge_length = points[edge].distance(points[edge + 1]);
        let weight = if edge_length > 1e-8 {
            ((distance - prefix) / edge_length).clamp(0., 1.)
        } else {
            0.
        };
        output.push((points[edge].lerp(points[edge + 1], weight), t));
    }
}
fn draw_mesh(
    mesh: &mut Mesh,
    sections: &[(Vec3, f32)],
    appearance: &ProgrammedAppearance,
    kind: RibbonKind,
    anchor: Mat4,
    basis: Mat4,
    camera: Option<Mat4>,
) {
    let mut positions = match mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => Vec::new(),
    };
    positions.clear();
    let mut normals = match mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => Vec::new(),
    };
    normals.clear();
    let mut uv = match mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => values,
        _ => Vec::new(),
    };
    uv.clear();
    let mut colors = match mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => Vec::new(),
    };
    colors.clear();
    if basis.determinant().abs() > 1e-12 {
        let inverse = basis.inverse();
        let camera = camera.unwrap_or(basis);
        let width_scale = anchor.y_axis.truncate().length();
        let vertex = |index: usize| {
            let (center, t) = sections[index];
            let direction = if index + 1 < sections.len() {
                sections[index + 1].0 - center
            } else {
                center - sections[index - 1].0
            };
            let direction = direction.normalize_or_zero();
            let style = appearance.sample(t);
            let side = if kind == RibbonKind::Billboard {
                direction
                    .cross(camera.w_axis.truncate() - center)
                    .try_normalize()
                    .unwrap_or(camera.x_axis.truncate().normalize_or_zero())
            } else {
                anchor.y_axis.truncate().normalize_or_zero()
            };
            let side =
                Quat::from_axis_angle(direction.try_normalize().unwrap_or(Vec3::Z), style.rotation)
                    * side;
            let normal = side.cross(direction).normalize_or_zero();
            (
                center,
                side * (style.size[0] * width_scale * 0.5),
                normal,
                style.color,
                t,
            )
        };
        for index in 0..sections.len().saturating_sub(1) {
            let a = vertex(index);
            let b = vertex(index + 1);
            for (which, right) in [
                (false, false),
                (false, true),
                (true, true),
                (false, false),
                (true, true),
                (true, false),
            ] {
                let (center, side, normal, color, t) = if which { b } else { a };
                positions.push(
                    inverse
                        .transform_point3(center + if right { side } else { -side })
                        .to_array(),
                );
                normals.push(
                    basis
                        .transpose()
                        .transform_vector3(normal)
                        .normalize_or_zero()
                        .to_array(),
                );
                uv.push([t, if right { 1. } else { 0. }]);
                colors.push(color);
            }
        }
    }
    if positions.is_empty() {
        positions.resize(3, [0.; 3]);
        normals.resize(3, [0., 0., 1.]);
        uv.resize(3, [0.; 2]);
        colors.resize(3, [0.; 4]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}
#[cfg(test)]
mod tests {
    use wc3::model::{Construct as SlotConstruct, Values as SlotValues};

    use super::*;
    #[test]
    fn endpoint_paths_follow_arc_length_and_explicit_length_cap() {
        let points = [Vec3::ZERO, Vec3::X * 2., Vec3::new(2., 2., 0.)];
        let mut output = Vec::new();
        sample_path(&points, 4, 3., &mut output);
        assert_eq!(output.len(), 5);
        assert_eq!(output[0], (Vec3::ZERO, 0.));
        assert_eq!(output[4], (Vec3::new(2., 1., 0.), 1.));
        let pointer = output.as_ptr();
        sample_path(&points, 4, 3., &mut output);
        assert_eq!(pointer, output.as_ptr());
    }
    #[test]
    fn endpoint_material_passes_share_one_mesh_and_keep_distinct_materials() {
        let mut slot_values = SlotValues::default();
        use bevy::ecs::world::CommandQueue;
        use wc3::model::profiles::Universal;
        use wc3::model::{Attachment, Socket, SocketIdentity};
        let mut builder = Model::<Universal>::builder("ribbon passes");
        let node = builder.add_named_node("anchor").unwrap();
        let feature = builder
            .add_feature(
                node,
                NodeFeature::Attachment(Attachment::Socket(Socket {
                    identity: SocketIdentity::Role(11),
                    visibility: slot_values.add(1.),
                    control: None,
                })),
            )
            .unwrap();
        let runtime = RuntimeProgrammedRibbons {
            definitions: vec![Definition {
                feature,
                node,
                anchor: 0,
                points: vec![],
                appearance: ProgrammedAge {
                    size_mid_time: 0.5,
                    color_mid_time: 0.5,
                    alpha_mid_time: 0.5,
                    rotation_mid_time: 0.5,
                    size_mid_hold_time: 0.,
                    color_mid_hold_time: 0.,
                    alpha_mid_hold_time: 0.,
                    rotation_mid_hold_time: 0.,
                    size_animation: SlotConstruct::construct(&mut slot_values),
                    rotation_animation: SlotConstruct::construct(&mut slot_values),
                    color_start: SlotConstruct::construct(&mut slot_values),
                    color_mid: SlotConstruct::construct(&mut slot_values),
                    color_end: SlotConstruct::construct(&mut slot_values),
                },
                smoothing: [Smoothing::Linear; 2],
                kind: RibbonKind::Billboard,
                segments: 1,
                max_length: slot_values.add(10.),
                active: slot_values.add(1),
                passes: [Color::WHITE, Color::BLACK]
                    .map(|color| {
                        let mut material = if color == Color::BLACK {
                            super::super::programmed::no_shadow_material()
                        } else {
                            ModelMaterial {
                                base: StandardMaterial::default(),
                                extension: Default::default(),
                            }
                        };
                        material.base.base_color = color;
                        Pass {
                            material,
                            animation: Arc::default(),
                        }
                    })
                    .into(),
            }],
            diagnostics: vec![],
        };
        let mut world = World::new();
        let basis = world.spawn_empty().id();
        let mut queue = CommandQueue::default();
        let mut meshes = Assets::default();
        let mut materials = Assets::default();
        let states = runtime.spawn(
            &mut Commands::new(&mut queue, &world),
            &mut meshes,
            &mut materials,
            basis,
        );
        queue.apply(&mut world);
        assert_eq!(meshes.len(), 1);
        assert_eq!(materials.len(), 2);
        let state = &states.states[0];
        assert_eq!(state.passes.len(), 2);
        assert_eq!(world.query::<&RibbonEmitter>().iter(&world).count(), 1);
        for pass in &state.passes {
            assert_eq!(world.get::<Mesh3d>(pass.entity).unwrap().0, state.mesh);
            assert_eq!(
                world
                    .get::<MeshMaterial3d<ModelMaterial>>(pass.entity)
                    .unwrap()
                    .0,
                pass.material
            );
        }
        assert_ne!(state.passes[0].material, state.passes[1].material);
        assert!(!world
            .entity(state.passes[0].entity)
            .contains::<NotShadowCaster>());
        assert!(!world
            .entity(state.passes[0].entity)
            .contains::<NotShadowReceiver>());
        assert!(world
            .entity(state.passes[1].entity)
            .contains::<NotShadowCaster>());
        assert!(world
            .entity(state.passes[1].entity)
            .contains::<NotShadowReceiver>());
        assert_eq!(
            materials
                .get(&state.passes[0].material)
                .unwrap()
                .base
                .base_color,
            Color::WHITE
        );
        assert_eq!(
            materials
                .get(&state.passes[1].material)
                .unwrap()
                .base
                .base_color,
            Color::BLACK
        );
    }
}
