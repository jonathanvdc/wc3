//! Semantic persistent trails backed by immutable GPU cross-section records.
use super::super::coordinates::{axis, basis};
use super::super::material::{effect_state, ModelMaterial, SurfaceAnimation, SurfacePass};
use super::atlas_sprites::retimed_selection;
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectStart;
use super::model_budget::ActorFrame;
use super::ribbon_definition::{TrailAtlasSlot, TrailBehavior, TrailEmitter, TrailRendering};
use super::EffectFrame;
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::blend::EffectBlend;
use crate::effects::records::RingCursor;
use crate::effects::render::{effect_quad, EffectGeometry};
use crate::effects::simulation::{split_time, EmissionPhase, EmissionRounding, LiveRecords};
use crate::effects::trails::{RibbonInstances, RibbonSection, RibbonSegment, RibbonUniform};
use crate::layers::ModelOwnedRenderEntity;
use crate::material::map_material_images;
use crate::textures::{TextureImages, TextureRole};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::{
    ClipId, FeatureId, MaterialId, Model, NodeFeature, NodeId,
    RibbonEmitter as SemanticRibbonEmitter, TextureTransform,
};

const CAPACITY: usize = 8192;
/// Source identity and bounded resident storage of a persistent trail.
#[derive(Component, Clone, Debug)]
pub struct RibbonEmitter {
    /// Semantic trail feature identity.
    pub feature: FeatureId,
    /// Hierarchy anchor carrying the trail.
    pub node: NodeId,
    /// Number of living resident cross-sections.
    pub sections: usize,
    /// Number of connected drawable segments.
    pub segments: usize,
    /// Cross-sections omitted by resident storage or long-update budgets.
    pub omitted_sections: u64,
}
#[derive(Clone)]
struct Pass {
    material: ModelMaterial,
    animation: SurfaceAnimation,
    ribbon_transform: Option<TextureTransform>,
    override_texture: bool,
    blend: EffectBlend,
    alpha_cutoff: f32,
    depth_test: bool,
    depth_write: bool,
    two_sided: bool,
    priority: i32,
    sort_far: bool,
    sort_near: bool,
}
#[derive(Clone)]
struct Emitter {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    definition: TrailEmitter,
    passes: Vec<Pass>,
}
#[derive(Clone, Default)]
pub(crate) struct RuntimeRibbons {
    emitters: Vec<Emitter>,
    gravity_axis: Vec3,
    clips: HashMap<ClipId, (f64, bool)>,
    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
struct Section {
    birth: f64,
    chain: u64,
}
struct State {
    entity: Entity,
    passes: Vec<(Entity, ModelMaterial)>,
    live: LiveRecords<RibbonSection, Section>,
    segments: Arc<Vec<RibbonSegment>>,
    version: Option<(RingCursor, usize)>,
    phase: EmissionPhase,
    chain: u64,
    connected: bool,
    previous_node: Mat4,
    previous_basis: Mat4,
    pose_scratch: Vec<usize>,
    omitted: u64,
}
#[derive(Default)]
pub(crate) struct RibbonInstancesState {
    emitters: Vec<State>,
    previous_time: Option<f64>,
    previous_clip: Option<ClipId>,
}
impl State {
    fn break_chain(&mut self) {
        if self.connected {
            self.chain = self.chain.wrapping_add(1);
            self.connected = false;
        }
    }
    fn rebuild(&mut self) {
        let version = (self.live.records.cursor(), self.live.len());
        if self.version == Some(version) {
            return;
        }
        self.version = Some(version);
        let segments = Arc::make_mut(&mut self.segments);
        segments.clear();
        let sections = self.live.metadata();
        let mut start = 0;
        while start < sections.len() {
            let mut end = start + 1;
            while end < sections.len() && sections[end].chain == sections[start].chain {
                end += 1;
            }
            for index in start..end - 1 {
                segments.push(RibbonSegment([
                    self.live.records.index(index),
                    self.live.records.index(index + 1),
                    (index - start) as u32,
                    (end - start - 1) as u32,
                ]));
            }
            start = end;
        }
    }
}
impl RuntimeRibbons {
    pub(super) fn seed_owned(
        &self,
        instances: &mut RibbonInstancesState,
        start: &EffectStart<'_, '_, '_>,
    ) {
        if instances.previous_time.is_some() {
            return;
        }
        instances.previous_time = Some(0.);
        instances.previous_clip = start.frame.selection.clip;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            state.previous_node = start.feature(emitter.anchor, &mut state.pose_scratch);
            state.previous_basis = start.frame.basis_world;
        }
    }

    pub(super) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        self.emitters
            .extend(source.emitters.iter().cloned().map(|mut value| {
                value.node = nodes[&value.node];
                value.anchor = indices[value.anchor];

                value
            }));
        self.diagnostics.extend(source.diagnostics.iter().cloned());
        for (&host, &part) in clips {
            if let Some(&policy) = source.clips.get(&part) {
                self.clips.entry(host).or_insert(policy);
            }
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }

    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &TextureImages,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    ) -> Self {
        let basis = basis(model)
            .map(|v| v.to_matrix())
            .unwrap_or(Mat4::IDENTITY);
        let mut result = Self {
            gravity_axis: basis.transform_vector3(axis(model.basis().up())),
            clips: model
                .clips()
                .iter()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Default::default()
        };
        for (feature, value) in model.features() {
            let definition: TrailEmitter = match &value.payload {
                NodeFeature::Ribbon(SemanticRibbonEmitter::Mdlx(value)) => (**value).clone().into(),
                NodeFeature::Ribbon(SemanticRibbonEmitter::M2(value)) => (**value).clone().into(),
                _ => continue,
            };
            let mut diagnostic = |kind, message: &str| {
                result
                    .diagnostics
                    .push((feature, RenderIssue::new(kind, message)))
            };
            let ids = match &definition.rendering {
                TrailRendering::Layered(value) => value.material.into_iter().collect::<Vec<_>>(),
                TrailRendering::Surfaces(value) => value.materials.clone(),
            };
            let mut passes = Vec::new();
            for id in ids {
                let Some(surfaces) = materials.get(&id) else {
                    diagnostic(
                        RenderDiagnosticKind::Resource,
                        "Trail material does not resolve to a compiled surface",
                    );
                    continue;
                };
                for surface in surfaces {
                    let mut material = ModelMaterial {
                        base: surface.material.clone(),
                        extension: surface.state.clone(),
                    };
                    let ribbon_transform =
                        if let TrailRendering::Surfaces(rendering) = &definition.rendering {
                            if let Some(texture) = rendering.textures.first() {
                                material.base.base_color_texture = images
                                    .ribbon(feature, 0)
                                    .for_role(*texture, TextureRole::Color)
                                    .cloned();
                            }
                            rendering.texture_transform.clone()
                        } else {
                            None
                        };
                    let Some(state) = effect_state(&material) else {
                        diagnostic(
                            RenderDiagnosticKind::Unsupported,
                            "Trail blend equation is unsupported by the transport and is not drawn",
                        );
                        continue;
                    };
                    if surface.coordinate != 0 {
                        diagnostic(RenderDiagnosticKind::Approximation, "Trail uses its procedural atlas coordinates; alternate geometry UV sets have no trail correspondence");
                    }
                    passes.push(Pass {
                        priority: material.base.depth_bias as i32,
                        sort_far: surface.sort_far,
                        sort_near: surface.sort_near,
                        two_sided: material.base.cull_mode.is_none(),
                        material,
                        animation: surface.animation.clone(),
                        ribbon_transform,
                        override_texture: matches!(&definition.rendering, TrailRendering::Surfaces(rendering) if !rendering.textures.is_empty()),
                        blend: state.blend,
                        alpha_cutoff: state.alpha_cutoff,
                        depth_test: state.depth_test,
                        depth_write: state.depth_write,
                    });
                }
            }
            if passes.is_empty() {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "Trail has no compiled material passes and is not drawn",
                );
                continue;
            }
            if let TrailBehavior::EmissionAngle(controls) = &definition.behavior {
                diagnostic(RenderDiagnosticKind::Unverified, "M2 trail cross-sections use an explicit local positive Z convention; native game fidelity remains unverified");
                if controls.angle != 0. {
                    diagnostic(RenderDiagnosticKind::Unsupported, "M2 trail emission angle is retained; its game-space cross-section convention remains unverified");
                }
                let TrailRendering::Surfaces(rendering) = &definition.rendering else {
                    unreachable!("M2 ribbon rendering")
                };
                if rendering.texture_transform.is_some() {
                    diagnostic(RenderDiagnosticKind::Unverified, "M2 trail UV transforms are evaluated; native global-flag applicability and game fidelity remain unverified");
                }
                if rendering.color_index != 0 {
                    diagnostic(
                        RenderDiagnosticKind::Resource,
                        "External M2 trail color tables require application-supplied colors",
                    );
                }
                if rendering.textures.len() > 1 {
                    diagnostic(RenderDiagnosticKind::Unsupported, "M2 trails sample their first texture; additional native texture bindings are retained but not drawn");
                }
                if rendering.materials.len() > 1 {
                    diagnostic(RenderDiagnosticKind::Unverified, "M2 trail material entries render as ordered Bevy passes; native game use beyond the first entry remains unverified");
                }
                for pass in &mut passes {
                    pass.priority = i32::from(rendering.priority_plane);
                }
            }
            diagnostic(RenderDiagnosticKind::Budget, "GPU trails retain 8192 resident cross-sections; skipped old sections are counted and never connected across omitted intervals");
            diagnostic(RenderDiagnosticKind::Approximation, "GPU trails use Bevy lighting; material surface channels beyond primary color/opacity/texture and UV transform remain approximations");
            result.emitters.push(Emitter {
                feature,
                node: value.node,
                anchor: nodes[&value.node],
                definition,
                passes,
            });
        }
        result
    }
    pub fn diagnostics(&self) -> &[(FeatureId, RenderIssue)] {
        &self.diagnostics
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        root: Entity,
    ) -> RibbonInstancesState {
        let quad = (!self.emitters.is_empty()).then(|| effect_quad(meshes));
        RibbonInstancesState {
            emitters: self
                .emitters
                .iter()
                .map(|emitter| {
                    let entity = commands
                        .spawn((
                            RibbonEmitter {
                                feature: emitter.feature,
                                node: emitter.node,
                                sections: 0,
                                segments: 0,
                                omitted_sections: 0,
                            },
                            Transform::default(),
                            Visibility::default(),
                            ChildOf(root),
                        ))
                        .id();
                    let passes = emitter
                        .passes
                        .iter()
                        .enumerate()
                        .map(|(index, pass)| {
                            let render = RibbonInstances {
                                records: Default::default(),
                                live_indices: Default::default(),
                                uniform: RibbonUniform {
                                    alpha_cutoff: pass.alpha_cutoff,
                                    ..Default::default()
                                },
                                texture: pass.material.base.base_color_texture.clone(),
                                filter: pass.blend,
                                priority_plane: pass.priority,
                                emitter: entity,
                                layer_index: index,
                                sort_far: pass.sort_far,
                                sort_near: pass.sort_near,
                                no_depth_test: !pass.depth_test,
                                no_depth_set: !pass.depth_write,
                                two_sided: pass.two_sided,
                            };
                            (
                                commands
                                    .spawn((
                                        ModelOwnedRenderEntity,
                                        EffectGeometry(quad.as_ref().unwrap().clone()),
                                        render,
                                        Transform::default(),
                                        Visibility::default(),
                                        NoFrustumCulling,
                                        ChildOf(entity),
                                    ))
                                    .id(),
                                pass.material.clone(),
                            )
                        })
                        .collect();
                    State {
                        entity,
                        passes,
                        live: Default::default(),
                        segments: Arc::default(),
                        version: None,
                        phase: Default::default(),
                        chain: 0,
                        connected: false,
                        previous_node: Mat4::IDENTITY,
                        previous_basis: Mat4::IDENTITY,
                        pose_scratch: Vec::new(),
                        omitted: 0,
                    }
                })
                .collect(),
            previous_time: None,
            previous_clip: None,
        }
    }
    pub fn update_bounded(
        &self,
        instances: &mut RibbonInstancesState,
        frame: &EffectFrame<'_, '_, '_>,
        render: &mut Query<&mut RibbonInstances>,
        stats: &mut Query<&mut RibbonEmitter>,
        actor: Option<&ActorFrame>,
    ) {
        let previous = instances.previous_time.unwrap_or(frame.time);
        let dt = (frame.time - previous).max(0.);
        let reset = frame.time < previous || frame.change == EffectChange::Reset;
        let switched = instances.previous_clip != frame.selection.clip;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            let definition = &emitter.definition;
            let node = frame.basis_world * frame.feature_matrix(emitter.anchor);
            if reset {
                state.live = Default::default();
                state.version = None;
            }
            if frame.change == EffectChange::Seek {
                state.break_chain();
            }
            if reset || switched || instances.previous_time.is_none() {
                state.break_chain();
                state.phase.reset();
                state.previous_node = node;
                state.previous_basis = frame.basis_world;
            }
            state
                .live
                .retire(|section| frame.time - section.birth >= f64::from(definition.lifetime));
            if dt > 0. && definition.emission_rate > 0. && definition.lifetime > 0. {
                let mut clock = EmissionClock::new(previous, frame.time);
                let mut remaining = CAPACITY;
                let mut checked_end = previous;
                for step in clock.by_ref() {
                    if actor.is_some_and(|actor| !actor.step()) {
                        break;
                    }
                    checked_end = step.end;
                    let schedule = state.phase.continuous(
                        definition.emission_rate,
                        step.end - step.start,
                        CAPACITY,
                        EmissionRounding::Ribbon,
                    );
                    let first = schedule.first_birth(step.start);
                    let live_window = f64::from(definition.lifetime)
                        .min(CAPACITY as f64 / definition.emission_rate);
                    let skip = schedule.skip(first, frame.time, live_window, remaining);
                    remaining = remaining.saturating_sub((schedule.count - skip) as usize);
                    if skip > 0. {
                        state.break_chain();
                        state.omitted = state.omitted.saturating_add(skip as u64);
                    }
                    for index in 0..(schedule.count - skip) as usize {
                        let time = schedule.birth(first, skip + index as f64);
                        let Some((clip, shared)) = retimed_selection(frame, time) else {
                            continue;
                        };
                        let sample = frame.sample.at_times(clip, shared);
                        let enabled = match &definition.behavior {
                            TrailBehavior::Gravity(controls) => {
                                sample.animated(&controls.visibility) > 0.
                            }
                            TrailBehavior::EmissionAngle(controls) => {
                                sample.animated(&controls.enabled)
                            }
                        } && (frame.pose.is_some()
                            || frame
                                .node_visible
                                .get(emitter.anchor)
                                .copied()
                                .unwrap_or(true));
                        let ended = frame
                            .selection
                            .clip
                            .and_then(|id| self.clips.get(&id))
                            .is_some_and(|(duration, looping)| {
                                !*looping && clip.as_seconds() >= *duration
                            });
                        if !enabled || ended {
                            state.break_chain();
                            continue;
                        }
                        let alpha = ((time - previous) / dt).clamp(0., 1.) as f32;
                        let transform = if let Some(pose) = frame.pose {
                            let (authored, visible) =
                                pose.at_offset(time - frame.time).sample_feature_node(
                                    emitter.anchor,
                                    &sample,
                                    &mut state.pose_scratch,
                                );
                            if !visible {
                                state.break_chain();
                                continue;
                            }
                            interpolated(state.previous_basis, frame.basis_world, alpha) * authored
                        } else {
                            interpolated(state.previous_node, node, alpha)
                        };
                        let above = transform.transform_point3(
                            axis(definition.geometry.axis)
                                * sample.animated(&definition.geometry.height_above),
                        );
                        let below = transform.transform_point3(
                            -axis(definition.geometry.axis)
                                * sample.animated(&definition.geometry.height_below),
                        );
                        if !above.is_finite() || !below.is_finite() {
                            state.break_chain();
                            continue;
                        }
                        while state.live.len() >= CAPACITY {
                            state.live.pop_front();
                            state.omitted = state.omitted.saturating_add(1);
                        }
                        state.live.push(
                            RibbonSection::new(above, below, time),
                            Section {
                                birth: time,
                                chain: state.chain,
                            },
                            CAPACITY,
                        );
                        state.connected = true;
                    }
                }
                let skipped = (frame.time - checked_end).max(0.);
                if let Some(actor) = actor {
                    actor.budget.skip_seconds(skipped);
                }
                if skipped > 0. {
                    state.break_chain();
                    state.omitted = state
                        .omitted
                        .saturating_add((skipped * definition.emission_rate).ceil() as u64);
                }
            } else if dt > 0. {
                state.break_chain();
            }
            state.previous_node = node;
            state.previous_basis = frame.basis_world;
            state.rebuild();
            let rgb = frame.sample.animated(&definition.appearance.color);
            let opacity = frame.sample.animated(&definition.appearance.opacity);
            let rows = definition.appearance.atlas[0].max(1);
            let columns = definition.appearance.atlas[1].max(1);
            let slot = match &definition.appearance.texture_slot {
                TrailAtlasSlot::Wide(value) => frame.sample.animated(value),
                TrailAtlasSlot::Compact(value) => u32::from(frame.sample.animated(value)),
            }
            .min(rows.saturating_mul(columns).saturating_sub(1));
            let gravity = match &definition.behavior {
                TrailBehavior::Gravity(controls) => controls.gravity,
                TrailBehavior::EmissionAngle(_) => 0.,
            };
            let [high, low] = split_time(frame.time);
            for (pass, (entity, material)) in emitter.passes.iter().zip(&mut state.passes) {
                pass.animation.apply_to_material(frame.sample, material);
                if pass.override_texture
                    && material.base.base_color_texture != pass.material.base.base_color_texture
                {
                    material
                        .base
                        .base_color_texture
                        .clone_from(&pass.material.base.base_color_texture);
                }
                let uv = pass
                    .ribbon_transform
                    .as_ref()
                    .map_or(material.base.uv_transform, |transform| {
                        transform.m2_affine(frame.sample) * material.base.uv_transform
                    });
                let color = material.base.base_color.to_linear();
                if let Ok(mut data) = render.get_mut(*entity) {
                    let Some(render_state) = effect_state(material) else {
                        Arc::make_mut(&mut data.live_indices).clear();
                        continue;
                    };
                    data.filter = render_state.blend;
                    data.no_depth_test = !render_state.depth_test;
                    data.no_depth_set = !render_state.depth_write;
                    data.records = state.live.records.clone();
                    data.live_indices = state.segments.clone();
                    data.texture = material.base.base_color_texture.clone();
                    data.uniform = RibbonUniform {
                        color: [
                            rgb[0] * color.red,
                            rgb[1] * color.green,
                            rgb[2] * color.blue,
                            (opacity * color.alpha).clamp(0., 1.),
                        ],
                        clock_gravity: [high, low, gravity],
                        alpha_cutoff: render_state.alpha_cutoff,
                        atlas_flags: [rows, columns, slot, u32::from(material.base.unlit)],
                        uv_transform: affine_uv(uv).to_cols_array_2d(),
                        gravity_axis: self.gravity_axis.extend(0.).to_array(),
                        render_flags: [
                            u32::from(material.base.fog_enabled),
                            render_state.blend.fog_blend(),
                            0,
                            0,
                        ],
                    };
                }
            }
            if let Ok(mut value) = stats.get_mut(state.entity) {
                if value.sections != state.live.len() {
                    value.sections = state.live.len();
                }
                if value.segments != state.segments.len() {
                    value.segments = state.segments.len();
                }
                if value.omitted_sections != state.omitted {
                    value.omitted_sections = state.omitted;
                }
            }
        }
        instances.previous_time = Some(frame.time);
        instances.previous_clip = frame.selection.clip;
    }
}
fn affine_uv(uv: Affine2) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(uv.matrix2.x_axis.x, uv.matrix2.x_axis.y, 0., 0.),
        Vec4::new(uv.matrix2.y_axis.x, uv.matrix2.y_axis.y, 0., 0.),
        Vec4::Z,
        Vec4::new(uv.translation.x, uv.translation.y, 0., 1.),
    )
}
fn interpolated(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}

impl RuntimeRibbons {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for emitter in &mut self.emitters {
            for pass in &mut emitter.passes {
                map_material_images(&mut pass.material, map);
                pass.animation.map_images(map);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::material::compile_material_with_images;
    use crate::textures::{texture_requests, TextureInput};
    use wc3::model::common::ZeroVec;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::m2::records::{
        Bone, Material, RibbonEmitter as NativeRibbon, Texture as NativeTexture, TextureSource,
    };
    use wc3::model::formats::m2::{Bundle, Document, Flat, V274};
    use wc3::model::{io, profiles::Universal, Construct, PropertyValues};

    #[test]
    fn m2_primary_texture_uses_feature_scoped_image_for_every_pass() {
        let mut slots = PropertyValues::default();
        let mut source = Bundle::from_model(Document::<Flat<V274>>::new());
        let core = source.model.core_mut();
        core.bones.push(Bone::new());
        core.materials = ZeroVec::alloc_from_slice(&[Material::default()]);
        core.textures.push(NativeTexture {
            filename: ZeroVec::alloc_from_slice(b"ribbon.png\0"),
            kind: TextureSource::File,
            ..Default::default()
        });
        core.ribbons.push(NativeRibbon {
            bone_id: 0,
            texture_indices: ZeroVec::alloc_from_slice(&[0]),
            material_indices: ZeroVec::alloc_from_slice(&[0]),
            ..Construct::construct(&mut slots)
        });
        let model = io::m2::import(&source, &ImportOptions::default())
            .unwrap()
            .value
            .widen::<Universal>();
        let feature = model
            .features()
            .iter()
            .find_map(|(id, feature)| {
                matches!(&feature.payload, NodeFeature::Ribbon(_)).then_some(id)
            })
            .unwrap();
        assert!(texture_requests(&model)
            .iter()
            .any(|request| request.input == Some(TextureInput::Ribbon { feature, slot: 0 })));
        let mut assets = Assets::<Image>::default();
        let scoped = assets.add(Image::default());
        let fallback = assets.add(Image::default());
        let images = TextureImages::resolve(&model, |request| {
            Some(
                if request.input == Some(TextureInput::Ribbon { feature, slot: 0 }) {
                    scoped.clone()
                } else {
                    fallback.clone()
                },
            )
        });
        let materials = model
            .materials()
            .keys()
            .map(|id| {
                (
                    id,
                    compile_material_with_images(&model, id, Arc::new(images.clone())).unwrap(),
                )
            })
            .collect();
        let nodes = model
            .nodes()
            .keys()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        let runtime = RuntimeRibbons::compile(&model, &nodes, &images, &materials);
        assert_eq!(runtime.emitters.len(), 1);
        assert!(runtime.emitters[0].passes.iter().all(|pass| pass
            .material
            .base
            .base_color
            .to_linear()
            == LinearRgba::WHITE));
        assert!(runtime.emitters[0].passes.iter().all(|pass| pass
            .material
            .base
            .base_color_texture
            .as_ref()
            == Some(&scoped)));
    }
    fn state() -> State {
        State {
            entity: Entity::PLACEHOLDER,
            passes: Vec::new(),
            live: Default::default(),
            segments: Arc::default(),
            version: None,
            phase: Default::default(),
            chain: 0,
            connected: false,
            previous_node: Mat4::IDENTITY,
            previous_basis: Mat4::IDENTITY,
            pose_scratch: Vec::new(),
            omitted: 0,
        }
    }
    #[test]
    fn segment_connectivity_never_crosses_a_visibility_gap() {
        let mut state = state();
        for (index, chain) in [0, 0, 1, 1, 1].into_iter().enumerate() {
            state.live.push(
                RibbonSection::new(
                    Vec3::Y + Vec3::X * index as f32,
                    Vec3::NEG_Y + Vec3::X * index as f32,
                    index as f64,
                ),
                Section {
                    birth: index as f64,
                    chain,
                },
                CAPACITY,
            );
        }
        state.rebuild();
        assert_eq!(
            state.segments.as_slice(),
            &[
                RibbonSegment([0, 1, 0, 1]),
                RibbonSegment([2, 3, 0, 2]),
                RibbonSegment([3, 4, 1, 2])
            ]
        );
        let snapshot = state.segments.clone();
        let pointer = Arc::as_ptr(&state.segments);
        state.rebuild();
        assert_eq!(pointer, Arc::as_ptr(&state.segments));
        state.live.retire(|section| section.birth < 2.);
        state.rebuild();
        assert_eq!(snapshot.len(), 3);
        assert_eq!(
            state.segments.as_slice(),
            &[RibbonSegment([2, 3, 0, 2]), RibbonSegment([3, 4, 1, 2])]
        );
    }
}
