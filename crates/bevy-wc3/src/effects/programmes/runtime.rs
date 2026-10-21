//! Universal feature controls, independent instances and compact sprite extraction.
use super::prepared::{ParticleEffect, SpriteRenderer};
use super::sampling::{atlas_sample, lut_cursor};
use super::visibility::VisibilityGuide;
use crate::coordinates::render_basis;
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::age_sprites_gpu::{
    AgeSpriteData, AgeSpriteRecord, AgeSpriteUniform, SampleBuffers,
};
use crate::effects::emission::EffectChange;
use crate::effects::frame::EffectStart;
use crate::effects::render::SortPolicy;
use crate::effects::render::{effect_quad, EffectGeometry};
use crate::effects::EffectFrame;
use crate::layers::ModelOwnedRenderEntity;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use wc3::model::profiles::mdlx::Popcorn;
use wc3::model::scene::popcorn::{
    EffectAttributeId, EffectInstance, EffectRuntimeError, ParticleView, SceneInputs, Value,
    ValueType,
};
use wc3::model::{ClipId, FeatureId, Model, NodeFeature, NodeId, Property};

// The baked programme's numeric space is one hundred native MDLX units per unit.
// Convert the world translation on input and the complete generated position/size on output.
const MODEL_UNITS_PER_EFFECT_UNIT: f32 = 100.;

/// Runtime population, bounded-work accounting and execution failures for an external effect.
#[derive(Component, Clone, Debug)]
pub struct ParticleProgramEmitter {
    /// Authored model feature.
    pub feature: FeatureId,
    /// Carrying hierarchy node.
    pub node: NodeId,
    /// Living particles, including structural controllers.
    pub alive: usize,
    /// Sprite records prepared for all renderer declarations.
    pub sprites: usize,
    /// Omitted births and particle executions from finite work budgets.
    pub omitted: u64,
    /// Births intentionally suppressed while emission is inactive.
    pub suppressed: u64,
    /// Time discarded by the simulation step budget.
    pub skipped_seconds: f64,
    /// A checked execution failure; the instance remains stopped until playback resets.
    pub error: Option<String>,
}
#[derive(Clone)]
struct Emitter {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    world_from_space: Mat4,
    space_from_world: Mat4,
    effect: ParticleEffect,
    activation: Property<f32>,
    rate: Property<f32>,
    speed: Property<f32>,
    lifetime: Property<f32>,
    color: Property<Vec3>,
    opacity: Property<f32>,
    scaling: bool,
    unfogged: bool,
    unlit: bool,
    sort: bool,
    guide: VisibilityGuide,
    attributes: Vec<(EffectAttributeId, Attribute)>,
}
#[derive(Clone, Copy)]
enum Attribute {
    Rate,
    Speed,
    Lifetime,
    Color,
    Scale,
    Target,
}
#[derive(Clone, Default)]
pub(crate) struct RuntimeProgramParticles {
    emitters: Vec<Emitter>,
    clips: HashMap<ClipId, String>,
    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
struct RendererState {
    entity: Entity,
    samples: SampleBuffers,
}
struct State {
    entity: Entity,
    instance: Option<EffectInstance>,
    renderers: Vec<RendererState>,
    started: bool,
    sampled: bool,
    sprites: usize,
    error: Option<String>,
}
#[derive(Default)]
pub(crate) struct ProgramInstances {
    states: Vec<State>,
    previous: Option<f64>,
}
impl RuntimeProgramParticles {
    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }
    pub fn diagnostics(&self) -> &[(FeatureId, RenderIssue)] {
        &self.diagnostics
    }
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        effects: &HashMap<FeatureId, ParticleEffect>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .iter()
                .map(|(id, clip)| (id, clip.name().to_owned()))
                .collect(),
            ..Default::default()
        };
        for (feature, source) in model.features() {
            let NodeFeature::ParticleProgram(source_effect) = &source.payload else {
                continue;
            };
            let Some(effect) = effects.get(&feature) else {
                result.diagnostics.push((feature,RenderIssue::new(RenderDiagnosticKind::Resource,"external particle programme has no prepared interpreter; provide its baked companion through ResourceResolver or CompileOptions::particle_effects")));
                continue;
            };
            for issue in effect.diagnostics() {
                result
                    .diagnostics
                    .push((feature, RenderIssue::new(issue.kind, issue.message.clone())));
            }
            for limitation in effect.program.limitations() {
                result.diagnostics.push((feature,RenderIssue::new(RenderDiagnosticKind::Unverified,format!("external particle programme uses explicit {limitation:?} policy; native game fidelity remains unverified"))));
            }
            result.diagnostics.push((feature,RenderIssue::new(RenderDiagnosticKind::Budget,"external programmes use finite particle pools, bounded event queues and per-update execution budgets; omissions are reported by ParticleProgramEmitter")));
            let mut attributes = Vec::new();
            for (index, attribute) in effect.program.attributes().iter().enumerate() {
                let id = EffectAttributeId(index as u32);
                if effect.overrides.iter().any(|(key, _)| *key == id) {
                    continue;
                }
                let (binding, kind) = match attribute.name.as_ref() {
                    b"Game.EmissionRateMultiplier" => (Attribute::Rate, ValueType::Float),
                    b"Game.SpeedMultiplier" => (Attribute::Speed, ValueType::Float),
                    b"Game.LifespanMultiplier" => (Attribute::Lifetime, ValueType::Float),
                    b"Game.ColorMultiplier" => (Attribute::Color, ValueType::Float4),
                    b"Game.Scale" => (Attribute::Scale, ValueType::Float),
                    b"Game.TargetPosition" => (Attribute::Target, ValueType::Float3),
                    _ => continue,
                };
                if attribute.default.kind() != kind {
                    let name = String::from_utf8_lossy(&attribute.name);
                    result.diagnostics.push((feature, RenderIssue::new(
                        RenderDiagnosticKind::Unsupported,
                        format!("external particle attribute {name:?} has an incompatible model-control layout"),
                    )));
                    continue;
                }
                attributes.push((id, binding));
            }
            result.emitters.push(Emitter::new(
                feature,
                source.node,
                nodes[&source.node],
                model.basis().conversion_to(render_basis()),
                source_effect,
                effect.clone(),
                attributes,
            ));
        }
        result
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        basis: Entity,
    ) -> ProgramInstances {
        let quad = (!self.emitters.is_empty()).then(|| effect_quad(meshes));
        let states =
            self.emitters
                .iter()
                .enumerate()
                .map(|(ordinal, emitter)| {
                    let entity = commands
                        .spawn((
                            ParticleProgramEmitter {
                                feature: emitter.feature,
                                node: emitter.node,
                                alive: 0,
                                sprites: 0,
                                omitted: 0,
                                suppressed: 0,
                                skipped_seconds: 0.,
                                error: None,
                            },
                            ChildOf(basis),
                        ))
                        .id();
                    let renderers = emitter
                        .effect
                        .renderers
                        .iter()
                        .map(|renderer| {
                            let mut style = renderer.style.clone();
                            style.sort = if emitter.sort {
                                SortPolicy::Far
                            } else {
                                SortPolicy::None
                            };
                            style.unlit |= emitter.unlit;
                            let entity = commands
                                .spawn((
                                    EffectGeometry(quad.as_ref().unwrap().clone()),
                                    ModelOwnedRenderEntity,
                                    AgeSpriteData {
                                        records: Default::default(),
                                        live_indices: Default::default(),
                                        uniform: Default::default(),
                                        style,
                                    },
                                    Transform::default(),
                                    Visibility::default(),
                                    NoFrustumCulling,
                                    NotShadowCaster,
                                    ChildOf(basis),
                                ))
                                .id();
                            RendererState {
                                entity,
                                samples: Default::default(),
                            }
                        })
                        .collect();
                    let (instance, error) = match emitter
                        .effect
                        .program
                        .instantiate(emitter.effect.limits, ordinal as u64)
                    {
                        Ok(mut instance) => {
                            let configured =
                                emitter.effect.overrides.iter().try_for_each(|(id, value)| {
                                    instance.set_attribute(*id, *value)
                                });
                            match configured {
                                Ok(()) => (Some(instance), None),
                                Err(error) => (None, Some(error.to_string())),
                            }
                        }
                        Err(error) => (None, Some(error.to_string())),
                    };
                    State {
                        entity,
                        instance,
                        renderers,
                        started: false,
                        sampled: false,
                        sprites: 0,
                        error,
                    }
                })
                .collect();
        ProgramInstances {
            states,
            previous: None,
        }
    }
    pub(crate) fn seed_owned(
        &self,
        instances: &mut ProgramInstances,
        start: &EffectStart<'_, '_, '_>,
    ) {
        if instances.previous.is_some() {
            return;
        }
        instances.previous = Some(0.);
        let frame = &start.frame;
        let clip = frame
            .selection
            .clip
            .and_then(|id| self.clips.get(&id))
            .map_or("", String::as_str);
        let mut scratch = Vec::new();
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            let (matrix, visible) = start.sample_feature(emitter.anchor, &mut scratch);
            if state.error.is_some()
                || !frame.root_visible
                || !visible
                || frame.sample.animated(&emitter.activation) <= 0.
                || !emitter.guide.visible(clip)
            {
                continue;
            }
            let (inputs, scale) = emitter.inputs_at(matrix);
            if let Some(instance) = &mut state.instance {
                let result = emitter
                    .controls(instance, frame, scale, inputs)
                    .and_then(|()| {
                        instance.start(
                            &emitter.effect.program,
                            emitter.effect.program.default_entry(),
                            inputs,
                        )
                    });
                match result {
                    Ok(()) => state.started = true,
                    Err(error) => {
                        instance.stop();
                        state.error = Some(error.to_string());
                    }
                }
            }
        }
    }
    pub fn update(
        &self,
        instances: &mut ProgramInstances,
        frame: &EffectFrame<'_, '_, '_>,
        sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut ParticleProgramEmitter>,
    ) {
        let previous = instances.previous.unwrap_or(frame.time);
        let delta = (frame.time - previous).max(0.);
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            let (inputs, scale) = emitter.inputs(frame);
            if frame.change == EffectChange::Reset {
                state.started = false;
                if state.instance.is_some() {
                    state.error = None;
                }
            }
            let was_started = state.started;
            if state.error.is_none() {
                if let Some(instance) = &mut state.instance {
                    let result = (|| {
                        emitter.controls(instance, frame, scale, inputs)?;
                        let clip = frame
                            .selection
                            .clip
                            .and_then(|id| self.clips.get(&id))
                            .map_or("", String::as_str);
                        let active = frame.root_visible
                            && frame.node_visible[emitter.anchor]
                            && frame.sample.animated(&emitter.activation) > 0.
                            && emitter.guide.visible(clip);
                        if !state.started && active {
                            instance.start(
                                &emitter.effect.program,
                                emitter.effect.program.default_entry(),
                                inputs,
                            )?;
                            state.started = true;
                        }
                        if state.started {
                            instance.set_emitting(active);
                            instance.advance(
                                &emitter.effect.program,
                                if frame.change == EffectChange::Reset {
                                    0.
                                } else {
                                    delta
                                },
                                inputs,
                            )?;
                        }
                        Ok::<_, EffectRuntimeError>(())
                    })();
                    if let Err(error) = result {
                        instance.stop();
                        state.error = Some(error.to_string());
                    }
                }
            }
            if !state.sampled
                || state.started != was_started
                || delta > 0.
                || frame.change == EffectChange::Reset
            {
                let mut sprite_count = 0;
                for (renderer, output) in emitter.effect.renderers.iter().zip(&mut state.renderers)
                {
                    if let Ok(mut data) = sprites.get_mut(output.entity) {
                        let (records, indices) = output.samples.begin();
                        if state.error.is_none() {
                            if let Some(instance) = &state.instance {
                                let record_values = Arc::get_mut(records).unwrap();
                                let order = Arc::get_mut(indices).unwrap();
                                for particle in instance.particles(renderer.layer) {
                                    if let Some(record) =
                                        sprite_record(renderer, &particle, emitter.world_from_space)
                                    {
                                        order.push(record_values.len() as u32);
                                        record_values.push(record);
                                    }
                                }
                            }
                        }
                        sprite_count += records.len();
                        data.records = records.clone();
                        data.live_indices = indices.clone();
                        data.uniform = AgeSpriteUniform {
                            axes: [[0.; 4]; 2],
                            flags: [
                                renderer.facing.code(),
                                u32::from(renderer.axes[0].is_some())
                                    | (u32::from(renderer.axes[1].is_some()) << 1),
                                u32::from(data.style.unlit),
                                u32::from(!emitter.unfogged),
                            ],
                            atlas: [
                                renderer.atlas[0],
                                renderer.atlas[1],
                                data.style.composition.flags()
                                    | (u32::from(data.style.uv_flip_v) << 3),
                                u32::from(data.style.blend.raw()),
                            ],
                        };
                    }
                }
                state.sprites = sprite_count;
                state.sampled = true;
            }
            if let Ok(mut output) = statistics.get_mut(state.entity) {
                output.sprites = state.sprites;
                if output.error.as_deref() != state.error.as_deref() {
                    output.error = state.error.clone();
                }
                if let Some(instance) = &state.instance {
                    output.alive = instance.active_count();
                    let stats = instance.stats();
                    output.omitted = stats
                        .omitted_births
                        .saturating_add(stats.omitted_executions);
                    output.suppressed = stats.suppressed_births;
                    output.skipped_seconds = stats.skipped_seconds;
                }
            }
        }
        instances.previous = Some(frame.time);
    }
    pub fn map_images(&mut self, map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>) {
        for emitter in &mut self.emitters {
            emitter.effect.map_images(map);
        }
    }
    pub fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        self.emitters
            .extend(source.emitters.iter().cloned().map(|mut emitter| {
                emitter.node = nodes[&emitter.node];
                emitter.anchor = indices[emitter.anchor];
                emitter
            }));
        self.clips.extend(
            source
                .clips
                .iter()
                .filter_map(|(id, name)| clips.get(id).map(|mapped| (*mapped, name.clone()))),
        );
        self.diagnostics.extend(source.diagnostics.iter().cloned());
    }
}
impl Emitter {
    fn new(
        feature: FeatureId,
        node: NodeId,
        anchor: usize,
        world_from_space: Mat4,
        source: &Popcorn<'_>,
        effect: ParticleEffect,
        attributes: Vec<(EffectAttributeId, Attribute)>,
    ) -> Self {
        Self {
            feature,
            node,
            anchor,
            world_from_space,
            space_from_world: world_from_space.inverse(),
            effect,
            activation: source.visibility,
            rate: source.emission_rate,
            speed: source.speed,
            lifetime: source.life_span,
            color: source.color,
            opacity: source.opacity,
            scaling: source.scaling,
            unfogged: source.unfogged,
            unlit: source.unlit,
            sort: source.sort_far_z,
            guide: VisibilityGuide::compile(&source.visibility_guide),
            attributes,
        }
    }
    fn inputs(&self, frame: &EffectFrame<'_, '_, '_>) -> (SceneInputs, f32) {
        self.inputs_at(frame.basis_world * frame.feature_matrix(self.anchor))
    }
    fn inputs_at(&self, matrix: Mat4) -> (SceneInputs, f32) {
        let mut matrix = self.space_from_world * matrix * Mat4::from_rotation_z(FRAC_PI_2);
        let lengths = Vec3::new(
            matrix.x_axis.truncate().length(),
            matrix.y_axis.truncate().length(),
            matrix.z_axis.truncate().length(),
        );
        let scale = (lengths.x + lengths.y + lengths.z) / 3.;
        if self.scaling {
            matrix.x_axis /= lengths.x.max(f32::MIN_POSITIVE);
            matrix.y_axis /= lengths.y.max(f32::MIN_POSITIVE);
            matrix.z_axis /= lengths.z.max(f32::MIN_POSITIVE);
        }
        matrix.w_axis = (matrix.w_axis.truncate() / MODEL_UNITS_PER_EFFECT_UNIT).extend(1.);
        (
            SceneInputs {
                world_from_effect: matrix,
            },
            if self.scaling { scale } else { 1. },
        )
    }
    fn controls(
        &self,
        instance: &mut EffectInstance,
        frame: &EffectFrame<'_, '_, '_>,
        scale: f32,
        inputs: SceneInputs,
    ) -> Result<(), EffectRuntimeError> {
        for (id, binding) in &self.attributes {
            let value = match binding {
                Attribute::Rate => scalar(frame.sample.animated(&self.rate)),
                Attribute::Speed => scalar(frame.sample.animated(&self.speed)),
                Attribute::Lifetime => scalar(frame.sample.animated(&self.lifetime)),
                Attribute::Scale => scalar(scale),
                Attribute::Color => Value::from_words(
                    ValueType::Float4,
                    frame
                        .sample
                        .animated(&self.color)
                        .extend(frame.sample.animated(&self.opacity))
                        .to_array()
                        .map(f32::to_bits),
                ),
                Attribute::Target => Value::from_words(
                    ValueType::Float3,
                    inputs.world_from_effect.w_axis.to_array().map(f32::to_bits),
                ),
            };
            instance.set_attribute(*id, value)?;
        }
        Ok(())
    }
}
fn scalar(value: f32) -> Value {
    Value::from_words(ValueType::Float, [value.to_bits(), 0, 0, 0])
}
fn floats(value: Value) -> [f32; 4] {
    value.words().map(f32::from_bits)
}
fn sprite_record(
    renderer: &SpriteRenderer,
    particle: &ParticleView<'_>,
    world_from_space: Mat4,
) -> Option<AgeSpriteRecord> {
    if renderer
        .enabled
        .and_then(|field| particle.field(field))
        .is_some_and(|value| value.words()[0] == 0)
    {
        return None;
    }
    let position = world_from_space.transform_point3(
        Vec3::from_slice(&floats(particle.field(renderer.position)?)[..3])
            * MODEL_UNITS_PER_EFFECT_UNIT,
    );
    let size = floats(particle.field(renderer.size)?)[0] * MODEL_UNITS_PER_EFFECT_UNIT * 2.;
    let rotation = renderer
        .rotation
        .and_then(|field| particle.field(field))
        .map_or(0., |value| floats(value)[0]);
    let mut axes = renderer.axes.map(|field| {
        field
            .and_then(|field| particle.field(field))
            .map_or(Vec3::ZERO, |value| {
                world_from_space.transform_vector3(Vec3::from_slice(&floats(value)[..3]))
            })
    });
    if renderer.facing.longitudinal_axis() {
        axes[0] *= MODEL_UNITS_PER_EFFECT_UNIT;
    }
    let color = renderer
        .color
        .and_then(|field| particle.field(field))
        .map_or([1.; 4], floats);
    let cursor = renderer
        .cursor
        .and_then(|field| particle.field(field))
        .map_or(0., |value| floats(value)[0]);
    if !position.is_finite()
        || !size.is_finite()
        || !rotation.is_finite()
        || size <= 0.
        || axes.iter().any(|axis| !axis.is_finite())
        || color.iter().any(|component| !component.is_finite())
    {
        return None;
    }
    let count = renderer.atlas[0] * renderer.atlas[1];
    let (cell, next, fraction) = atlas_sample(cursor, count);
    let lut_v = lut_cursor(particle.id.slot, particle.id.generation, particle.layer.0);
    let mut record = AgeSpriteRecord {
        position: position.extend(1.).to_array(),
        velocity_tail: axes[0].extend(0.).to_array(),
        size_spin: [size, size, rotation.cos(), rotation.sin()],
        color: color.map(|value| value.max(0.)),
        offsets: axes[1].extend(0.).to_array(),
        scales: [1.; 4],
        cells: [cell, 0, 0, 0],
    };
    record.set_sampled_appearance(next, fraction, lut_v);
    Some(record)
}
