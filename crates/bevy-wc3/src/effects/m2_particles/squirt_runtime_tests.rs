//! Original complete native inputs exercise the production M2 policy and transport.
use super::*;
use crate::effects::model_budget::{ActorBudget, ActorFrame, ModelActorTree, ModelParticleLimits};
use crate::effects::model_children::ModelParticleChild;
use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use std::collections::VecDeque;
use wc3::model::common::ZeroVec;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::m2::codec::{Array, Value};
use wc3::model::formats::m2::companions::{DynamicSkin, Skin};
use wc3::model::formats::m2::records::{
    Bone, BoneFlags, Interpolation, ModelFlags, ModernParticleLifetime, ModernTimeline,
    PackedParticleBlend, PackedParticleGenerator, ParticleEmitter as NativeEmitter, ParticleTrack,
    Sequence, Texture as NativeTexture, TextureSource, Track, TrackBase, TrackValues,
};
use wc3::model::formats::m2::{Bundle, Chunked, Document, DynamicDocument, File, FileKey, V274};
use wc3::model::io::m2 as semantic_m2;
use wc3::model::profiles::Universal;
use wc3::model::{AnimationProgram, AnimationSample};

fn track<T: Value>(times: &[u32], values: &[T], linear: bool) -> Track<'static, T, V274>
where
    [T; 3]: Value,
{
    Track {
        base: TrackBase {
            interpolation: if linear {
                Interpolation::Linear
            } else {
                Interpolation::Step
            },
            global_sequence: 0,
            timeline: ModernTimeline {
                timestamps: vec![Array::Inline(ZeroVec::alloc_from_slice(times))],
            },
        },
        values: TrackValues::Values(vec![Array::Inline(ZeroVec::alloc_from_slice(values))]),
    }
}
fn age<T: Value>(value: T) -> ParticleTrack<'static, T> {
    ParticleTrack {
        timestamps: ZeroVec::alloc_from_slice(&[0]),
        values: ZeroVec::alloc_from_slice(&[value]),
    }
}
fn native(
    times: &[u32],
    rates: &[f32],
    linear: bool,
    geometry: bool,
) -> Bundle<'static, Chunked<V274>> {
    let mut document = Document::<Chunked<V274>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"original sampled-squirt control\0");
    core.global_sequences = ZeroVec::alloc_from_slice(&[2000]);
    core.skins = 1;
    core.bones = vec![Bone::new()];
    core.bones[0].flags = BoneFlags::from_bits_retain(0x200);
    core.sequences = vec![Sequence {
        timing: 2000,
        frequency: 32767,
        ..Default::default()
    }];
    core.textures = vec![NativeTexture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"white.tga\0"),
        ..Default::default()
    }];
    let mut emitter = NativeEmitter::<V274>::new(ModelFlags::default());
    emitter.flags.set_squirt(true);
    emitter.flags.set_head(true);
    emitter.flags.set_disable_lighting(true);
    emitter.bone_id = 0;
    emitter.rows = 1;
    emitter.columns = 1;
    emitter.twinkle_percent = 1.;
    emitter.twinkle_scale = [1.; 2];
    emitter.kind.blending = PackedParticleBlend::Alpha;
    emitter.kind.emitter = PackedParticleGenerator::Plane;
    emitter.emission_rate = track(times, rates, linear);
    emitter.emission_speed = track(&[0], &[0.], false);
    emitter.lifespan = track(&[0], &[10.], false);
    emitter.gravity = track(&[0], &[0], false);
    emitter.enabled = track(&[0], &[1], false);
    emitter.lifetime = ModernParticleLifetime {
        color: age([255.; 3]),
        alpha: age(32767),
        scale: age([0.15; 2]),
        ..Default::default()
    };
    if geometry {
        emitter.particle_model_filename = ZeroVec::alloc_from_slice(b"geometry.m2\0");
    }
    core.particles.push(emitter);
    let mut bundle = Bundle::from_model(document).with_key(FileKey::Path("emitter.m2".into()));
    bundle.insert(
        FileKey::Path("emitter00.skin".into()),
        File::Skin(Box::new(DynamicSkin::V274(Skin::default()))),
    );
    if geometry {
        let child = super::super::fixture_tests::gravity_fixture(false, 0.0f32.to_bits()).model;
        bundle.insert(
            FileKey::Path("geometry.m2".into()),
            File::Document(Box::new(DynamicDocument::Flat256(child))),
        );
    }
    bundle.validate_complete().unwrap();
    bundle
}

type Parameters = (
    Commands<'static, 'static>,
    Query<'static, 'static, &'static mut AgeSpriteData>,
    Query<'static, 'static, &'static mut SpriteEmitter>,
    Query<'static, 'static, &'static mut ModelEmitter>,
    ModelOutputs<'static, 'static>,
);
struct NativeHarness {
    world: World,
    query: SystemState<Parameters>,
    runtime: RuntimeM2Particles,
    instances: M2ParticleInstances,
    program: AnimationProgram,
    node_poses: Vec<Mat4>,
    node_visible: Vec<bool>,
    root: Entity,
    models: Assets<ModelAsset>,
    budget: Option<Arc<ActorBudget>>,
    frame: u32,
}
impl NativeHarness {
    fn new(source: &Bundle<'_, Chunked<V274>>) -> Self {
        let imported = semantic_m2::import(source, &ImportOptions::default()).unwrap();
        assert!(imported.report.issues.is_empty());
        let model: Model<'_, Universal> = imported.value.widen();
        let program = AnimationProgram::compile(&model);
        let indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (node, _))| (node, index))
            .collect();
        let models = Assets::default();
        let children = model
            .features()
            .iter()
            .filter_map(|(feature, value)| {
                let NodeFeature::Particle(ParticleEmitter::M2(particle)) = &value.payload else {
                    return None;
                };
                particle.details.rendering.model.map(|_| {
                    (
                        ChildModelKey::particle_geometry(feature),
                        ChildModelBinding::new(models.reserve_handle()),
                    )
                })
            })
            .collect();
        let runtime = RuntimeM2Particles::compile_with_children(
            &model,
            &indices,
            &TextureImages::default(),
            &HashMap::new(),
            &children,
        );
        assert_eq!(runtime.emitters.len(), 1);
        assert_eq!(
            runtime.emitters[0].emission,
            if source.model.core().particles[0].flags.squirt() {
                EmissionMode::Squirt
            } else {
                EmissionMode::Continuous
            }
        );
        let mut world = World::new();
        let root = world.spawn_empty().id();
        let mut queue = CommandQueue::default();
        let instances = runtime.spawn(
            &mut Commands::new(&mut queue, &world),
            &mut Assets::default(),
            root,
        );
        queue.apply(&mut world);
        let query = SystemState::new(&mut world);
        Self {
            world,
            query,
            runtime,
            instances,
            program,
            node_poses: vec![Mat4::IDENTITY; model.nodes().len()],
            node_visible: vec![true; model.nodes().len()],
            root,
            models,
            budget: None,
            frame: 0,
        }
    }
    fn limits(&mut self, limits: ModelParticleLimits) {
        if let Some(budget) = &self.budget {
            budget.set_limits(limits);
        } else {
            let budget = ActorBudget::new(self.root, limits);
            self.world.entity_mut(self.root).insert(ModelActorTree {
                budget: budget.clone(),
                pending: VecDeque::new(),
            });
            self.budget = Some(budget);
        }
    }
    fn step(&mut self, time: f64) {
        self.selected(time, time, EffectChange::Continuous);
    }
    fn selected(&mut self, time: f64, selected: f64, change: EffectChange) {
        self.frame += 1;
        let selection = AnimationSample {
            clip: None,
            clip_time: ModelTime::seconds(selected).unwrap(),
            shared_time: ModelTime::seconds(selected).unwrap(),
        };
        let sample = self.program.sample(selection).unwrap();
        let actor = self.budget.as_ref().map(|budget| ActorFrame {
            budget: budget.clone(),
            frame: FrameCount(self.frame),
            owned: true,
        });
        let (mut commands, mut data, mut statistics, mut model_statistics, mut outputs) =
            self.query.get_mut(&mut self.world).unwrap();
        self.runtime.update_bounded(
            &mut self.instances,
            EffectFrame {
                traversal: Default::default(),
                change,
                root_visible: true,
                pose: None,
                selection,
                time,
                sample: &sample,
                node_poses: &self.node_poses,
                node_visible: &self.node_visible,
                basis_world: Mat4::from_translation(Vec3::X * selected as f32),
            },
            &mut data,
            &mut statistics,
            M2World {
                commands: &mut commands,
                models: &self.models,
                geometry: Some(GeometryContext {
                    outputs: &mut outputs,
                    statistics: &mut model_statistics,
                    root: self.root,
                    driving_camera: None,
                    allow_births: true,
                }),
            },
            actor.as_ref(),
        );
        self.query.apply(&mut self.world);
    }
    fn state(&self) -> &EmitterInstance {
        &self.instances.emitters[0]
    }
}

#[test]
fn native_constant_squirt_has_one_endpoint_burst_not_streaming_or_substep_bursts() {
    let source = native(&[0], &[5.9], false, false);
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    for _ in 0..120 {
        test.step(0.);
    }
    assert_eq!(test.state().squirt.packets(), 0);
    test.step(0.5);
    assert_eq!(test.state().births.len(), 5);
    assert!(test
        .state()
        .births
        .iter()
        .all(|birth| birth.time == 0.5 && (birth.origin.x - 0.5).abs() < 1e-6));
    test.step(1.);
    test.step(2.1);
    assert_eq!(
        test.state().births.len(),
        5,
        "natural global looping does not repeat a constant positive startup"
    );
    assert_eq!(test.state().squirt.packets(), 1);
    test.selected(0., 0., EffectChange::Reset);
    assert!(test.state().births.is_empty());
    test.step(0.25);
    assert_eq!(test.state().births.len(), 5);
    assert_eq!(test.state().squirt.packets(), 1);
}
#[test]
fn native_sampled_edges_ignore_positive_changes_and_unobserved_pulses() {
    let source = native(
        &[0, 200, 400, 600, 800, 1999],
        &[0., 3.9, 7.9, 0., 2.9, 0.],
        false,
        false,
    );
    let mut test = NativeHarness::new(&source);
    for (time, expected) in [
        (0., 0),
        (0.19, 0),
        (0.21, 3),
        (0.41, 3),
        (0.61, 3),
        (0.81, 5),
    ] {
        test.step(time);
        assert_eq!(test.state().births.len(), expected, "actor sample {time}");
    }
    let source = native(&[0, 201, 202, 1999], &[0., 5., 0., 0.], false, false);
    let mut missed = NativeHarness::new(&source);
    missed.step(0.);
    missed.step(0.19);
    missed.step(0.21);
    assert!(
        missed.state().births.is_empty(),
        "internal quanta do not detect extra actor edges"
    );
    let mut observed = NativeHarness::new(&source);
    observed.step(0.);
    observed.step(0.19);
    observed.step(0.2015);
    assert_eq!(observed.state().births.len(), 5);
}
#[test]
fn native_fractional_rising_edge_is_consumed_without_later_ramp_retry() {
    let source = native(&[0, 1000, 1999], &[0., 10., 10.], true, false);
    let mut test = NativeHarness::new(&source);
    for time in [0., 0.001, 0.003, 0.1, 0.9] {
        test.step(time);
    }
    assert!(test.state().births.is_empty());
    assert_eq!(
        test.state().squirt.packets(),
        2,
        "empty startup and threshold-edge packets each advance once"
    );
    assert_eq!(test.state().ordinal, 0);
}
#[test]
fn native_seek_rebases_destination_then_honors_only_positive_authorized_time() {
    let source = native(
        &[0, 200, 600, 800, 1999],
        &[0., 3.9, 0., 2.9, 0.],
        false,
        false,
    );
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    test.step(0.1);
    test.selected(0.1, 0.21, EffectChange::Seek);
    test.selected(0.2, 0.31, EffectChange::Continuous);
    assert!(
        test.state().births.is_empty(),
        "cursor jump high does not become an emitted edge"
    );
    test.selected(0.3, 0.81, EffectChange::Seek);
    assert_eq!(
        test.state().births.len(),
        2,
        "destination low to endpoint high has real authorized elapsed time"
    );
    assert!(test.state().births.iter().all(|birth| birth.time == 0.3));
}
#[test]
fn native_capacity_omissions_advance_birth_identity_and_never_queue_a_burst() {
    let mut source = native(&[0], &[9000.], false, false);
    source.model.core_mut().particles[0].lifespan = track(&[0], &[0.05], false);
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    test.step(0.1);
    assert_eq!(test.state().births.len(), MAX_PARTICLES);
    assert_eq!(test.state().dropped, 9000 - MAX_PARTICLES as u64);
    assert_eq!(test.state().ordinal, 9000);
    test.step(0.2);
    assert!(test.state().births.is_empty());
    assert_eq!(test.state().ordinal, 9000);
    assert_eq!(test.state().squirt.packets(), 1);
}
#[test]
fn native_model_work_denial_consumes_the_same_burst_and_shared_ordinal() {
    let source = native(&[0], &[3.], false, true);
    let mut test = NativeHarness::new(&source);
    test.limits(ModelParticleLimits {
        birth_work_per_frame: 0,
        ..Default::default()
    });
    test.step(0.);
    test.step(0.1);
    assert!(test.state().births.is_empty());
    assert_eq!(test.state().dropped, 3);
    assert_eq!(test.state().ordinal, 3);
    let budget = test.budget.as_ref().unwrap();
    assert_eq!(budget.stats(FrameCount(test.frame)).omitted_birth_work, 3);
    test.limits(ModelParticleLimits::default());
    test.step(0.2);
    assert!(
        test.state().births.is_empty(),
        "a raised allowance does not replay an omitted request"
    );
    assert_eq!(test.state().squirt.packets(), 1);
    assert_eq!(
        test.world
            .query::<&ModelParticleChild>()
            .iter(&test.world)
            .count(),
        0
    );
}
#[test]
fn denied_emission_step_consumes_known_count_and_reports_skipped_time_once() {
    let source = native(&[0], &[3.], false, false);
    let mut test = NativeHarness::new(&source);
    test.limits(ModelParticleLimits {
        emission_steps_per_frame: 0,
        ..Default::default()
    });
    test.step(0.);
    test.step(0.1);
    assert_eq!(test.state().dropped, 3);
    assert_eq!(test.state().ordinal, 3);
    assert_eq!(test.state().skipped_seconds, 0.1);
    test.limits(ModelParticleLimits::default());
    test.step(0.2);
    assert!(test.state().births.is_empty());
    assert_eq!(test.state().squirt.packets(), 1);
}
#[test]
fn unrepresentable_native_count_consumes_request_without_inventing_birth_counts() {
    let source = native(&[0], &[1e30], false, false);
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    test.step(0.1);
    assert!(test.state().births.is_empty());
    assert_eq!(
        test.state().ordinal,
        0,
        "uninterpreted counts have no invented ordinal history"
    );
    assert_eq!(test.state().skipped_seconds, 0.1);
    test.step(0.2);
    assert_eq!(test.state().squirt.packets(), 1);
    assert_eq!(test.state().skipped_seconds, 0.1);
}
#[test]
fn native_activation_and_exact_threshold_are_sampled_before_burst_counts() {
    let mut source = native(&[0], &[5.], false, false);
    source.model.core_mut().particles[0].enabled = track(&[0, 500, 1999], &[0, 1, 1], false);
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    test.step(0.1);
    assert!(test.state().births.is_empty());
    assert_eq!(
        test.state().squirt.packets(),
        0,
        "disabled startup consumes no random packet"
    );
    test.step(0.51);
    assert_eq!(test.state().births.len(), 5);
    let source = native(
        &[0, 200, 400, 600, 800, 1000, 1200, 1400, 1600, 1999],
        &[0., 0.02, 0.021, 4., 0., 0.5, 3., 0., 1.9, 1.9],
        false,
        false,
    );
    let mut test = NativeHarness::new(&source);
    for time in [0., 0.19, 0.21, 0.41, 0.61, 0.81, 1.01, 1.21, 1.41, 1.61] {
        test.step(time);
    }
    assert_eq!(
        test.state().births.len(),
        1,
        "fractional threshold edges do not accumulate or retry"
    );
}
#[test]
fn native_geometry_squirts_publish_the_same_exact_count_and_endpoint_age() {
    let source = native(&[0], &[5.9], false, true);
    let mut test = NativeHarness::new(&source);
    test.step(0.);
    test.step(0.5);
    assert_eq!(test.state().births.len(), 5);
    assert!(test.state().births.iter().all(|birth| birth.time == 0.5));
    let emitter = test.instances.emitters[0].passes[0].entity;
    assert_eq!(test.world.get::<ModelEmitter>(emitter).unwrap().alive, 5);
    assert_eq!(
        test.world
            .query::<&ModelParticleChild>()
            .iter(&test.world)
            .count(),
        5
    );
}
#[test]
fn native_continuous_control_retains_its_integrated_rate_policy() {
    let mut source = native(
        &[0, 200, 400, 600, 800, 1000, 1999],
        &[0., 3.9, 7.9, 0., 2.9, 0., 0.],
        false,
        false,
    );
    source.model.core_mut().particles[0].flags.set_squirt(false);
    let mut test = NativeHarness::new(&source);
    assert_eq!(test.runtime.emitters[0].emission, EmissionMode::Continuous);
    test.step(0.);
    test.step(1.61);
    assert_eq!(test.state().births.len(), 2);
    assert!(test.state().births.iter().all(|birth| birth.time < 1.61));
    assert_eq!(test.state().squirt.packets(), 0);
}
