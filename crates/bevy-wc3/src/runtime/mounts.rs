//! Independently rigged rigid equipment mounted beneath a host's animated nodes.
use super::Playback;
use crate::events::EventTraversal;
use crate::physics::PhysicsSimulation;
use crate::transition::TransitionRequest;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::ClipId;

#[derive(Component)]
pub(crate) struct MountedModel {
    pub(super) parent: Entity,
    pub(super) clips: Arc<HashMap<ClipId, ClipId>>,
    pub(super) inherited_physics: Option<PhysicsSimulation>,
    pub(super) inherited_from: Option<Entity>,
}
#[derive(Clone, Copy)]
struct PlaybackSnapshot {
    clip: Option<ClipId>,
    clip_time: f64,
    timeline_time: f64,
    shared_time: f64,
    speed: f64,
    paused: bool,
    driving_camera: Option<Entity>,
    transition_request: TransitionRequest,
    transition_clock: f64,
    event_traversal: EventTraversal,
}
impl From<&Playback> for PlaybackSnapshot {
    fn from(playback: &Playback) -> Self {
        Self {
            clip: playback.clip,
            clip_time: playback.clip_time,
            timeline_time: playback.timeline_time,
            shared_time: playback.shared_time,
            speed: playback.speed,
            paused: playback.paused,
            driving_camera: playback.driving_camera,
            transition_request: playback.transition_request,
            transition_clock: playback.transition_clock,
            event_traversal: playback.event_traversal,
        }
    }
}
#[derive(Clone)]
struct Snapshot {
    playback: PlaybackSnapshot,
    physics: Option<PhysicsSimulation>,
    force_source: Entity,
}
#[derive(Default)]
pub(crate) struct MountScratch {
    order: Vec<(Entity, Entity)>,
    parents: HashMap<Entity, Entity>,
    snapshots: HashMap<Entity, Snapshot>,
}
type MountQueries<'w, 's> = ParamSet<
    'w,
    's,
    (
        Query<
            'static,
            'static,
            (
                &'static Playback,
                Option<&'static PhysicsSimulation>,
                Option<&'static MountedModel>,
            ),
        >,
        Query<
            'static,
            'static,
            (
                Entity,
                &'static mut MountedModel,
                &'static mut Playback,
                Option<&'static PhysicsSimulation>,
            ),
        >,
    ),
>;

pub(crate) fn synchronize_mounted_models(
    mut queries: MountQueries<'_, '_>,
    mut scratch: Local<MountScratch>,
    mut commands: Commands,
) {
    if queries.p1().is_empty() {
        return;
    }
    scratch.order.clear();
    scratch.parents.clear();
    scratch.snapshots.clear();
    for (entity, mount, _, _) in &queries.p1() {
        scratch.order.push((entity, mount.parent));
        scratch.parents.insert(entity, mount.parent);
    }
    // Nested equipment consumes its already-synchronized parent's clocks. The
    // marker is private and only created through validated model composition.
    let MountScratch {
        order,
        parents,
        snapshots,
    } = &mut *scratch;
    order.sort_unstable_by_key(|(entity, _)| {
        let mut cursor = *entity;
        let mut depth = 0;
        while let Some(parent) = parents.get(&cursor) {
            depth += 1;
            if depth > parents.len() {
                break;
            }
            cursor = *parent;
        }
        depth
    });
    let sources = queries.p0();
    for (_, parent) in order.iter() {
        if let Ok((playback, physics, mount)) = sources.get(*parent) {
            snapshots.entry(*parent).or_insert_with(|| Snapshot {
                playback: PlaybackSnapshot::from(playback),
                physics: physics.cloned(),
                force_source: mount
                    .filter(|mount| physics == mount.inherited_physics.as_ref())
                    .and_then(|mount| mount.inherited_from)
                    .unwrap_or(*parent),
            });
        }
    }
    let mut mounts = queries.p1();
    for &(entity, parent) in order.iter() {
        let Some(source) = snapshots.get(&parent).cloned() else {
            continue;
        };
        let Ok((_, mut mount, mut playback, physics)) = mounts.get_mut(entity) else {
            continue;
        };
        {
            let playback = playback.bypass_change_detection();
            playback.speed = source.playback.speed;
            playback.paused = source.playback.paused;
            playback.driving_camera = source.playback.driving_camera;
        }
        if let Some(clip) = source
            .playback
            .clip
            .and_then(|clip| mount.clips.get(&clip).copied())
        {
            let playback = playback.bypass_change_detection();
            playback.clip = Some(clip);
            playback.clip_time = source.playback.clip_time;
            playback.timeline_time = source.playback.timeline_time;
            playback.shared_time = source.playback.shared_time;
            playback.transition_request = source.playback.transition_request;
            playback.transition_clock = source.playback.transition_clock;
            playback.event_traversal = source.playback.event_traversal;
        }
        // Inherit only while the component still equals the last inherited
        // value. An application edit or removal establishes child authority.
        let inherited = physics == mount.inherited_physics.as_ref();
        let next_physics = if inherited {
            source.physics.clone()
        } else {
            physics.cloned()
        };
        if inherited && next_physics.as_ref() != physics {
            if let Some(settings) = &next_physics {
                commands.entity(entity).insert(settings.clone());
            } else {
                commands.entity(entity).remove::<PhysicsSimulation>();
            }
        }
        if inherited {
            mount.inherited_physics = next_physics.clone();
            mount.inherited_from = next_physics.as_ref().map(|_| source.force_source);
        }
        snapshots.insert(
            entity,
            Snapshot {
                playback: PlaybackSnapshot::from(&*playback),
                physics: next_physics,
                force_source: if inherited {
                    source.force_source
                } else {
                    entity
                },
            },
        );
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use wc3::model::profiles::Universal;
    use wc3::model::{Clip, Model};

    #[test]
    fn nested_mounts_share_clocks_and_preserve_child_physics_edits() {
        let mut builder = Model::<Universal>::builder("mount clips");
        let clips: Vec<_> = (0..3)
            .map(|i| builder.add_clip(Clip::named(format!("clip{i}"))).unwrap())
            .collect();
        let mut app = App::new();
        app.add_systems(Update, synchronize_mounted_models);
        let mut parent_playback = Playback::for_clip(clips[0]);
        parent_playback.clip_time = 0.75;
        parent_playback.shared_time = 2.;
        parent_playback.paused = true;
        let parent = app
            .world_mut()
            .spawn((parent_playback, PhysicsSimulation::default()))
            .id();
        let child = app
            .world_mut()
            .spawn((
                Playback::default(),
                MountedModel {
                    parent,
                    clips: Arc::new(HashMap::from([(clips[0], clips[1])])),
                    inherited_physics: None,
                    inherited_from: None,
                },
            ))
            .id();
        let nested = app
            .world_mut()
            .spawn((
                Playback::default(),
                MountedModel {
                    parent: child,
                    clips: Arc::new(HashMap::from([(clips[1], clips[2])])),
                    inherited_physics: None,
                    inherited_from: None,
                },
            ))
            .id();
        let independent = app
            .world_mut()
            .spawn((
                Playback::for_clip(clips[1]),
                MountedModel {
                    parent,
                    clips: Arc::new(HashMap::new()),
                    inherited_physics: None,
                    inherited_from: None,
                },
            ))
            .id();
        app.update();
        let independent_playback = app.world().get::<Playback>(independent).unwrap();
        assert_eq!(independent_playback.clip, Some(clips[1]));
        assert_eq!(independent_playback.clip_time, 0.);
        assert!(independent_playback.paused);
        let playback = app.world().get::<Playback>(nested).unwrap();
        assert_eq!(playback.clip, Some(clips[2]));
        assert_eq!(playback.clip_time, 0.75);
        assert_eq!(playback.shared_time, 2.);
        assert!(playback.paused);
        assert_eq!(
            app.world()
                .get::<PhysicsSimulation>(nested)
                .unwrap()
                .gravity,
            Vec3::new(0., 0., -9.81)
        );
        app.world_mut()
            .get_mut::<PhysicsSimulation>(child)
            .unwrap()
            .gravity = Vec3::X;
        app.world_mut()
            .get_mut::<PhysicsSimulation>(parent)
            .unwrap()
            .gravity = Vec3::Y;
        app.update();
        assert_eq!(
            app.world().get::<PhysicsSimulation>(child).unwrap().gravity,
            Vec3::X
        );
        assert_eq!(
            app.world()
                .get::<PhysicsSimulation>(nested)
                .unwrap()
                .gravity,
            Vec3::X
        );
        app.world_mut()
            .entity_mut(child)
            .remove::<PhysicsSimulation>();
        app.update();
        assert!(app.world().get::<PhysicsSimulation>(child).is_none());
        assert!(app.world().get::<PhysicsSimulation>(nested).is_none());
    }
}

#[cfg(test)]
mod camera_tests {
    use super::super::animate_instances;
    use super::*;
    use crate::{compile_model, spawn_model, ModelMaterial, ModelPart, ModelRig, PartBinding};
    use bevy::ecs::world::CommandQueue;
    use wc3::model::profiles::Universal;
    use wc3::model::{
        Billboard, Model, ModelMount, MountTarget, Node, Quaternion, Transform as ModelTransform,
    };
    use wc3::model::{Construct as SlotConstruct, Values as SlotValues};

    #[test]
    fn mounted_billboards_resolve_current_carrier_frame_without_transform_propagation() {
        let mut slot_values = SlotValues::default();
        let mut host = Model::<Universal>::builder("billboard host");
        let carrier = host
            .add_node(
                Node::named("carrier", &mut slot_values)
                    .with_transform(
                        ModelTransform {
                            rotation: Quaternion(Quat::from_rotation_y(1.).to_array()),
                            ..SlotConstruct::construct(&mut slot_values)
                        },
                        &mut slot_values,
                    )
                    .unwrap(),
            )
            .unwrap();
        let host = compile_model(
            &{
                host.import_values(&slot_values).unwrap();
                host.finish()
            }
            .unwrap(),
            |_| None,
        )
        .unwrap();
        let mut part = Model::<Universal>::builder("billboard part");
        let mut node = Node::named("billboard", &mut slot_values);
        node.billboard = Billboard::Spherical;
        let billboard = part.add_node(node).unwrap();
        let part = compile_model(
            &{
                part.import_values(&slot_values).unwrap();
                part.finish()
            }
            .unwrap(),
            |_| None,
        )
        .unwrap();
        let composed = host
            .compose([ModelPart::new(
                &part,
                PartBinding::Mounted(ModelMount::new(MountTarget::Node(carrier))),
            )])
            .unwrap();
        let mut meshes = Assets::default();
        let mut binds = Assets::default();
        let prepared = composed.prepare(&mut meshes, &mut binds);
        let mut materials = Assets::<ModelMaterial>::default();
        let mut app = App::new();
        app.add_systems(
            Update,
            (synchronize_mounted_models, animate_instances).chain(),
        );
        let mut queue = CommandQueue::default();
        let root = spawn_model(
            &mut Commands::new(&mut queue, app.world()),
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        );
        queue.apply(app.world_mut());
        app.insert_resource(materials);
        let driver = app
            .world_mut()
            .spawn(Transform::from_xyz(5., 4., 6.).looking_at(Vec3::ZERO, Vec3::Y))
            .id();
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .driving_camera = Some(driver);
        app.update();
        let child = app.world().get::<ModelRig>(root).unwrap().mounts()[0];
        let node = app
            .world()
            .get::<ModelRig>(child)
            .unwrap()
            .node(billboard)
            .unwrap();
        let initial = app.world().get::<Transform>(node).unwrap().rotation;
        {
            let mut driver = app.world_mut().get_mut::<Transform>(driver).unwrap();
            driver.translation.x = -5.;
            driver.look_at(Vec3::ZERO, Vec3::Y);
        }
        app.update();
        let next = app.world().get::<Transform>(node).unwrap().rotation;
        assert!(
            initial.angle_between(next) > 0.1,
            "mounted billboard ignored current driving camera"
        );
    }
}

#[cfg(test)]
mod activation_tests {
    use super::super::animate_instances;
    use super::*;
    use crate::effects::model_particles::hierarchy_visible;
    use crate::{
        compile_model, spawn_model, AttachmentState, ModelAttachment, ModelMaterial, ModelPart,
        ModelRig, PartBinding,
    };
    use bevy::ecs::system::SystemState;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::profiles::Universal;

    use wc3::model::Values as SlotValues;
    use wc3::model::{
        Attachment, Channel, Clip, Clock, Curve, Model, NodeFeature, Socket as SemanticSocket,
        SocketIdentity, Time, Track,
    };
    use wc3::model::{ModelMount, MountTarget};

    #[test]
    fn authored_attachment_activation_gates_composed_equipment_through_ancestor() {
        let mut slot_values = SlotValues::default();
        let mut host = Model::<Universal>::builder("attachment host");
        let carrier = host.add_named_node("carrier").unwrap();
        let point = host.reserve_feature(carrier);
        let visibility = slot_values.add(1.);
        host.set_feature(
            point,
            NodeFeature::Attachment(Attachment::Socket(SemanticSocket {
                identity: SocketIdentity::Role(11),
                visibility,
                control: Some(23),
            })),
        )
        .unwrap();
        let clip = host
            .add_clip(Clip::named("hide equipment").with_duration(Time::seconds(1.).unwrap()))
            .unwrap();
        host.add_channel(
            clip,
            Channel::Scalar(Track {
                target: visibility,
                curve: Curve::step(
                    vec![Time::seconds(0.).unwrap(), Time::seconds(0.5).unwrap()],
                    vec![1., 0.],
                )
                .unwrap(),
                clock: Clock::Clip,
            }),
        )
        .unwrap();
        let host = compile_model(
            &{
                host.import_values(&slot_values).unwrap();
                host.finish()
            }
            .unwrap(),
            |_| None,
        )
        .unwrap();
        let mut part = Model::<Universal>::builder("helmet");
        part.add_named_node("helmet root").unwrap();
        let part = compile_model(
            &{
                part.import_values(&slot_values).unwrap();
                part.finish()
            }
            .unwrap(),
            |_| None,
        )
        .unwrap();
        let composed = host
            .compose([ModelPart::new(
                &part,
                PartBinding::Mounted(ModelMount::new(MountTarget::Socket(point))),
            )])
            .unwrap();
        let mut meshes = Assets::default();
        let mut binds = Assets::default();
        let prepared = composed.prepare(&mut meshes, &mut binds);
        let mut materials = Assets::<ModelMaterial>::default();
        let mut app = App::new();
        app.add_systems(
            Update,
            (synchronize_mounted_models, animate_instances).chain(),
        );
        let mut queue = CommandQueue::default();
        let root = spawn_model(
            &mut Commands::new(&mut queue, app.world()),
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        );
        queue.apply(app.world_mut());
        app.insert_resource(materials);
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .play_immediately(clip);
        app.update();
        let child = app.world().get::<ModelRig>(root).unwrap().mounts()[0];
        let gate = app.world().get::<ChildOf>(child).unwrap().parent();
        assert!(
            app.world().get::<ModelAttachment>(gate).is_some(),
            "equipment bypassed the authored attachment gate"
        );
        let definition = &app.world().get::<ModelAttachment>(gate).unwrap().definition;
        assert_eq!(definition.socket().unwrap().control, Some(23));
        assert_eq!(1., 1.);
        assert_eq!(
            app.world().get::<AttachmentState>(gate).unwrap().visibility,
            1.
        );
        let carrier = app
            .world()
            .get::<ModelRig>(root)
            .unwrap()
            .node(carrier)
            .unwrap();
        let mut hierarchy =
            SystemState::<Query<(&Visibility, Option<&ChildOf>)>>::new(app.world_mut());
        assert!(hierarchy_visible(
            child,
            &hierarchy.get(app.world()).unwrap()
        ));
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .seek(0.75)
            .unwrap();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(carrier).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            *app.world().get::<Visibility>(gate).unwrap(),
            Visibility::Hidden
        );
        assert!(!hierarchy_visible(
            child,
            &hierarchy.get(app.world()).unwrap()
        ));
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .seek(0.25)
            .unwrap();
        app.update();
        assert!(hierarchy_visible(
            child,
            &hierarchy.get(app.world()).unwrap()
        ));
    }
}
