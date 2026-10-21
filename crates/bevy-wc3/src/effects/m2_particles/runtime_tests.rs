//! CPU-world checks for the actual occurrence observer and resource lifecycle.
use super::templates::{FrozenTemplate, TemplateCatalogue, TemplateSlot};
use super::*;
use crate::child_coordinates::CoordinatePolicy;
use crate::compile_model;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use wc3::model::profiles::Universal;
use wc3::model::{AnimationProgram, AnimationSample, ModelMetadata, PropertyValues};

type TransportQueries = (
    Commands<'static, 'static>,
    Query<'static, 'static, &'static mut AgeSpriteData>,
    Query<'static, 'static, &'static mut SpriteEmitter>,
);
struct Harness {
    world: World,
    system: SystemState<TransportQueries>,
    assets: Assets<ModelAsset>,
    handle: Handle<ModelAsset>,
    asset: Option<ModelAsset>,
    runtime: RuntimeM2Particles,
    instances: M2ParticleInstances,
    program: AnimationProgram,
    owner: Entity,
    node: Mat4,
}
impl Harness {
    fn new(parent_rate: f32, parent_lifetime: f32, ready: bool) -> Self {
        let mut values = PropertyValues::default();
        let mut parent = super::tests::emitter(&mut values);
        parent.speed = values.add(0.);
        parent.rate = values.add(parent_rate);
        parent.lifespan = values.add(parent_lifetime);
        let mut child = super::tests::emitter(&mut values);
        child.speed = values.add(0.);
        child.rate = values.add(120.);
        child.lifespan = values.add(1.);
        let child = Arc::new(child);
        let template = Arc::new(FrozenTemplate::new(&child, &values));
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        builder.import_values(&values).unwrap();
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        let mut compiled = compile_model(&model, |_| None).unwrap();
        Arc::make_mut(&mut compiled.particles).m2.templates = Arc::new(TemplateCatalogue {
            slots: vec![TemplateSlot {
                feature: child.feature,
                template: Ok(template),
            }],
        });
        let asset = ModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        };
        let mut assets = Assets::default();
        let handle = assets.reserve_handle();
        let mut asset = Some(asset);
        if ready {
            assets.insert(handle.id(), asset.take().unwrap()).unwrap();
        }
        let runtime = RuntimeM2Particles {
            emitters: vec![Arc::new(parent)],
            sources: HashMap::from([(0, ChildModelBinding::new(handle.clone()))]),
            ..Default::default()
        };
        let mut world = World::new();
        let basis = world.spawn_empty().id();
        let mut queue = CommandQueue::default();
        let instances = runtime.spawn(
            &mut Commands::new(&mut queue, &world),
            &mut Assets::default(),
            basis,
        );
        queue.apply(&mut world);
        let owner = instances.emitters[0].passes[0].entity;
        let system = SystemState::new(&mut world);
        Self {
            world,
            system,
            assets,
            handle,
            asset,
            runtime,
            instances,
            program,
            owner,
            node: Mat4::from_translation(Vec3::X * 7.),
        }
    }
    fn seed(&mut self) {
        let sample = self.program.sample(AnimationSample::default()).unwrap();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &self.world);
        self.runtime.seed_owned(
            &mut self.instances,
            &EffectStart {
                frame: EffectFrame {
                    traversal: Default::default(),
                    change: EffectChange::Continuous,
                    root_visible: true,
                    pose: None,
                    selection: Default::default(),
                    time: 0.,
                    sample: &sample,
                    node_poses: &[self.node],
                    node_visible: &[true],
                    basis_world: Mat4::IDENTITY,
                },
            },
            &self.assets,
            &mut commands,
        );
        queue.apply(&mut self.world);
    }
    fn geometry(&mut self) {
        Arc::make_mut(&mut self.runtime.emitters[0]).model = Some(GeometryEmitter {
            template: ModelTemplate {
                model: self.handle.clone(),
                coordinates: ChildCoordinates {
                    coordinates: CoordinatePolicy::Host,
                    parent_basis: Transform::IDENTITY,
                },
            },
            minimum: Vec3::ZERO,
            maximum: Vec3::ZERO,
        });
        self.instances.emitters[0].models = Some(GeometryInstances::default());
    }
    fn ready(&mut self) {
        self.assets
            .insert(self.handle.id(), self.asset.take().unwrap())
            .unwrap();
    }
    fn step(&mut self, time: f64, change: EffectChange) {
        self.advance(time, change, false, false);
    }
    fn seed_and_step(&mut self, time: f64, remove_owner: bool) {
        self.advance(time, EffectChange::Continuous, true, remove_owner);
    }
    fn advance(&mut self, time: f64, change: EffectChange, seed: bool, remove_owner: bool) {
        let sample = self.program.sample(AnimationSample::default()).unwrap();
        let (mut commands, mut gpu, mut statistics) = self.system.get_mut(&mut self.world).unwrap();
        if seed {
            self.runtime.seed_owned(
                &mut self.instances,
                &EffectStart {
                    frame: EffectFrame {
                        traversal: Default::default(),
                        change: EffectChange::Continuous,
                        root_visible: true,
                        pose: None,
                        selection: Default::default(),
                        time: 0.,
                        sample: &sample,
                        node_poses: &[self.node],
                        node_visible: &[true],
                        basis_world: Mat4::IDENTITY,
                    },
                },
                &self.assets,
                &mut commands,
            );
        }
        if remove_owner {
            commands.entity(self.owner).try_despawn();
        }
        self.runtime.update_bounded(
            &mut self.instances,
            EffectFrame {
                traversal: Default::default(),
                change,
                root_visible: true,
                pose: None,
                selection: Default::default(),
                time,
                sample: &sample,
                node_poses: &[self.node],
                node_visible: &[true],
                basis_world: Mat4::IDENTITY,
            },
            &mut gpu,
            &mut statistics,
            M2World {
                commands: &mut commands,
                models: &self.assets,
                geometry: None,
            },
            None,
        );
        self.system.apply(&mut self.world);
    }
    fn child_draws(&mut self) -> Vec<(Entity, usize)> {
        self.world
            .query_filtered::<(Entity, &SpriteEmitter), With<AdoptedParticleTemplate>>()
            .iter(&self.world)
            .map(|(entity, stats)| (entity, stats.alive))
            .collect()
    }
}
#[test]
fn actual_parent_loop_retains_children_from_sources_dead_at_endpoint() {
    let mut test = Harness::new(1., 0.2, true);
    test.step(0., EffectChange::Continuous);
    test.step(1.5, EffectChange::Continuous);
    assert!(test.instances.emitters[0].births.is_empty());
    assert_eq!(
        test.world.get::<SpriteEmitter>(test.owner).unwrap().alive,
        0
    );
    let draws = test.child_draws();
    assert_eq!(draws.len(), 1);
    assert!(
        draws[0].1 > 0,
        "the actual occurrence hook must retain subframe sources"
    );
    let data = test.world.get::<AgeSpriteData>(draws[0].0).unwrap();
    assert!(!data.records.is_empty());
    assert!(data
        .records
        .iter()
        .all(|record| (record.position[0] - 7.).abs() < 1e-5));
    let population = draws[0].1;
    let records = data.records.clone();
    test.step(1.5, EffectChange::Continuous);
    assert_eq!(
        test.child_draws()[0].1,
        population,
        "paused publication does not drive templates"
    );
    assert!(
        Arc::ptr_eq(
            &records,
            &test.world.get::<AgeSpriteData>(draws[0].0).unwrap().records
        ),
        "unchanged paused views reuse the published sample"
    );
    test.step(1.6, EffectChange::Continuous);
    assert_eq!(
        test.child_draws()[0].1,
        population,
        "children survive independently of the parent"
    );
}
#[test]
fn pending_readiness_does_not_replay_past_drives_and_reset_clears_pools() {
    let mut test = Harness::new(10., 2., false);
    test.step(0., EffectChange::Continuous);
    test.step(0.5, EffectChange::Continuous);
    assert!(test.child_draws().is_empty());
    assert_eq!(
        test.world
            .get::<AdoptedParticleSource>(test.owner)
            .unwrap()
            .status,
        AdoptedTemplateStatus::Pending
    );
    test.ready();
    test.step(0.5, EffectChange::Continuous);
    assert_eq!(
        test.child_draws()[0].1,
        0,
        "paused resource readiness has no historical drives"
    );
    let old = test.child_draws()[0].0;
    test.step(0.6, EffectChange::Continuous);
    assert!(test.child_draws()[0].1 > 0);
    test.step(0., EffectChange::Reset);
    assert_eq!(
        test.child_draws()[0],
        (old, 0),
        "reset clears simulation while retaining its draw"
    );
    let asset = test.assets.remove(test.handle.id()).unwrap();
    test.step(0., EffectChange::Continuous);
    assert!(
        test.world.get_entity(old).is_err(),
        "unloading the child destroys owned draws"
    );
    assert!(test.child_draws().is_empty());
    test.assets.insert(test.handle.id(), asset).unwrap();
    test.step(0., EffectChange::Continuous);
    let replacement = test.child_draws()[0];
    assert_ne!(replacement.0, old);
    assert_eq!(
        replacement.1, 0,
        "reloading never restores discarded child history"
    );
}

#[test]
fn seek_preserves_the_authorized_elapsed_interval_after_the_skipped_jump() {
    // EffectClock has already projected away the seek jump. This positive .25s
    // is the real interval following traversal.origin, exactly as for parents.
    let mut test = Harness::new(10., 2., true);
    test.step(0., EffectChange::Continuous);
    test.step(0.5, EffectChange::Continuous);
    let population = test.child_draws()[0].1;
    assert!(population > 0);
    test.step(0.75, EffectChange::Seek);
    assert!(
        test.child_draws()[0].1 > population,
        "existing parents must drive children during authorized post-seek time"
    );
}

#[test]
fn geometry_projection_rejection_cannot_create_phantom_adopted_sources() {
    let mut test = Harness::new(10., 0.05, true);
    test.geometry();
    // A valid particle frame can still have an invalid model projection.
    // Host unit scale is checked when constructing the child transform.
    assert!(ModelBasis::from_frame(test.node).is_some());
    Arc::make_mut(&mut test.runtime.emitters[0])
        .model
        .as_mut()
        .unwrap()
        .template
        .coordinates
        .parent_basis
        .scale = Vec3::NAN;
    test.step(0., EffectChange::Continuous);
    test.step(0.5, EffectChange::Continuous);
    assert!(test.instances.emitters[0].births.is_empty());
    assert!(test.instances.emitters[0].dropped > 0);
    assert_eq!(
        test.child_draws()[0].1,
        0,
        "a rejected model birth never becomes a template source, including endpoint-dead births"
    );
}

#[test]
fn admitted_endpoint_dead_geometry_parents_still_leave_living_template_children() {
    let mut test = Harness::new(1., 0.2, true);
    test.geometry();
    test.step(0., EffectChange::Continuous);
    test.step(1.5, EffectChange::Continuous);
    assert!(test.instances.emitters[0].births.is_empty());
    assert_eq!(test.instances.emitters[0].dropped, 0);
    assert!(
        test.child_draws()[0].1 > 0,
        "successful model admission must not discard its earlier surviving source intervals"
    );
}

#[test]
fn ready_owned_templates_consume_the_captured_startup_interval_exactly_once() {
    let mut test = Harness::new(10., 2., true);
    test.seed();
    assert_eq!(
        test.world
            .get::<AdoptedParticleSource>(test.owner)
            .unwrap()
            .status,
        AdoptedTemplateStatus::Ready,
        "catalogue availability is captured before advancing"
    );
    test.step(0.5, EffectChange::Continuous);
    assert!(!test.instances.emitters[0].births.is_empty());
    let population = test.child_draws()[0].1;
    assert!(
        population > 0,
        "ready resources participate in the accepted initial interval"
    );
    test.step(0.5, EffectChange::Continuous);
    assert_eq!(test.child_draws()[0].1, population);
}
#[test]
fn pending_at_captured_birth_does_not_backdate_later_first_update_readiness() {
    let mut test = Harness::new(10., 2., false);
    test.seed();
    assert_eq!(
        test.world
            .get::<AdoptedParticleSource>(test.owner)
            .unwrap()
            .status,
        AdoptedTemplateStatus::Pending
    );
    // Availability changes after the birth capture, but BEFORE the first owner
    // simulation. previous_time==0 must never be treated as evidence of readiness.
    test.ready();
    test.step(0.5, EffectChange::Continuous);
    assert_eq!(test.child_draws()[0].1, 0);
    test.step(0.6, EffectChange::Continuous);
    assert!(test.child_draws()[0].1 > 0);
}

#[test]
fn captured_ready_draw_is_fully_published_in_the_same_unflushed_commands_batch() {
    let mut test = Harness::new(10., 2., true);
    test.seed_and_step(0.5, false);
    let draw = test.child_draws()[0];
    assert!(draw.1 > 0);
    let data = test.world.get::<AgeSpriteData>(draw.0).unwrap();
    assert_eq!(
        data.records.len(),
        draw.1,
        "first rendered frame has the exact simulation population"
    );
    assert_eq!(data.live_indices.len(), draw.1);
    let provenance = test.world.get::<AdoptedParticleTemplate>(draw.0).unwrap();
    assert_eq!(provenance.model, test.handle);
    let records = data.records.clone();
    test.step(0.5, EffectChange::Continuous);
    assert!(
        Arc::ptr_eq(
            &records,
            &test.world.get::<AgeSpriteData>(draw.0).unwrap().records
        ),
        "deferred publication participates in the ordinary unchanged-sample cache"
    );
}
#[test]
fn queued_owner_despawn_prevents_deferred_publication_from_resurrecting_draws() {
    let mut test = Harness::new(10., 2., true);
    test.seed_and_step(0.5, true);
    assert!(test.world.get_entity(test.owner).is_err());
    assert!(
        test.child_draws().is_empty(),
        "try_insert cannot resurrect a cascaded draw entity"
    );
}
