//! Detaching checked models preserves caller identities and executable meaning.
use glam::{Mat4, Vec2, Vec3};
use mdlx::mdx::{Read as _, Write as _};
use std::borrow::Cow;
use wc3::model::common::{Detached, FileKey, IntoOwned, PackedArray, PackedValue};
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::{m2, mdlx, wmo};
use wc3::model::io;
use wc3::model::profiles::{m2 as mp, mdlx as profile, Mdlx, Profile, Universal, Wmo, M2};
use wc3::model::scene::{
    self, Body, ContactMaterial, ForceChannels, Friction, Joint, JointKind, Motion, PhysicsSystem,
    Shape,
};
use wc3::model::{
    AnimationId, AnimationSample, BoneOverrideResourceId, ChildModelDefinition, ClockId, Curve,
    DrawId, DynamicTrack, EffectResourceId, EventId, EventOccurrence, EventSourceId, FeatureId,
    ImageResourceId, InstanceId, LookId, MaterialId, MeshId, Model, ModelResourceId, NodeId,
    PhysicsResourceId, PoseInputs, Property, SkinId, SocketDefinition, SocketIdentity, System,
    SystemId, Time, TimeSource, Track, ValueSlot,
};

#[path = "fixtures/model_ownership.rs"]
mod fixtures;

#[derive(Debug, PartialEq, Eq)]
struct Identities {
    nodes: Vec<NodeId>,
    meshes: Vec<MeshId>,
    materials: Vec<MaterialId>,
    instances: Vec<InstanceId>,
    skins: Vec<SkinId>,
    clips: Vec<wc3::model::ClipId>,
    features: Vec<FeatureId>,
    systems: Vec<SystemId>,
    animations: Vec<AnimationId>,
    clocks: Vec<ClockId>,
    sources: Vec<EventSourceId>,
    events: Vec<EventId>,
    looks: Vec<LookId>,
    draws: Vec<DrawId>,
    images: Vec<ImageResourceId>,
    models: Vec<ModelResourceId>,
    effects: Vec<EffectResourceId>,
    physics: Vec<PhysicsResourceId>,
    overrides: Vec<BoneOverrideResourceId>,
    targets: Vec<ValueSlot>,
    values: usize,
    default_look: Option<LookId>,
    subsets: Vec<wc3::model::DrawSubsetId>,
}
fn identities<P: Profile>(model: &Model<'_, P>) -> Identities {
    Identities {
        nodes: model.nodes().iter().map(|(id, _)| id).collect(),
        meshes: model.meshes().iter().map(|(id, _)| id).collect(),
        materials: model.materials().iter().map(|(id, _)| id).collect(),
        instances: model.instances().iter().map(|(id, _)| id).collect(),
        skins: model.skins().iter().map(|(id, _)| id).collect(),
        clips: model.clips().iter().map(|(id, _)| id).collect(),
        features: model.features().iter().map(|(id, _)| id).collect(),
        systems: model.systems().iter().map(|(id, _)| id).collect(),
        animations: model.animations().iter().map(|(id, _)| id).collect(),
        clocks: model.clocks().iter().map(|(id, _)| id).collect(),
        sources: model.event_sources().iter().map(|(id, _)| id).collect(),
        events: model
            .animations()
            .values()
            .flat_map(|animation| animation.events.keys().copied())
            .collect(),
        looks: model.looks().map(|(id, _)| id).collect(),
        draws: model
            .instances()
            .values()
            .flat_map(|instance| instance.draws.iter().map(|draw| draw.id()))
            .collect(),
        images: model
            .dependencies()
            .images()
            .iter()
            .map(|(id, _)| id)
            .collect(),
        models: model
            .dependencies()
            .models()
            .iter()
            .map(|(id, _)| id)
            .collect(),
        effects: model
            .dependencies()
            .effects()
            .iter()
            .map(|(id, _)| id)
            .collect(),
        physics: model
            .dependencies()
            .physics()
            .iter()
            .map(|(id, _)| id)
            .collect(),
        overrides: model
            .dependencies()
            .bone_overrides()
            .iter()
            .map(|(id, _)| id)
            .collect(),
        targets: model.tracks().map(|track| track.target()).collect(),
        values: model.values().len(),
        default_look: model.default_look(),
        subsets: model
            .instances()
            .values()
            .flat_map(|instance| instance.draws.iter().map(|draw| draw.subset))
            .collect(),
    }
}
fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}
fn bytes<T: PackedValue>(values: &[T]) -> Vec<u8> {
    PackedArray::alloc_from_slice(values).as_bytes().to_vec()
}
fn animate<'a, P: Profile>(
    model: &mut Model<'a, P>,
    times: &'a [u8],
    values: &'a [u8],
) -> Property<Vec3> {
    let target = model.nodes().values().next().unwrap().transform.translation;
    model
        .edit(|draft| {
            draft.add_global_track(DynamicTrack::Vector3(Track {
                target,
                clock: TimeSource::Animation,
                curve: Curve::linear(
                    PackedArray::<Time>::parse_bytes(times).unwrap(),
                    PackedArray::<Vec3>::parse_bytes(values).unwrap(),
                )?,
            }))
        })
        .unwrap();
    target
}
fn positions(model: &Model<'_>, seconds: f64) -> Vec<Vec3> {
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(seconds),
            ..Default::default()
        })
        .unwrap();
    let layout = model.prepare_pose().unwrap();
    let pose = layout.evaluate(&sample, PoseInputs::default()).unwrap();
    let mut output = Vec::new();
    let mut joints = Vec::new();
    let mut matrices = Vec::new();
    for (_, instance) in model.instances() {
        let mesh = model.mesh(instance.mesh).unwrap();
        for draw in &instance.draws {
            for mut geometry in mesh
                .project(mesh.subset(draw.subset).unwrap(), 0, [None; 3])
                .unwrap()
            {
                if let Some(skin) = instance.skin.or(mesh.skin()) {
                    let skin = model.skin(skin).unwrap();
                    let weights = geometry.skin_weights(mesh, &skin.joints).unwrap();
                    layout
                        .joint_matrices_into(
                            skin,
                            &pose,
                            &weights.palette,
                            Mat4::IDENTITY,
                            &mut joints,
                        )
                        .unwrap();
                    weights.matrices_into(&joints, &mut matrices).unwrap();
                } else {
                    matrices.clear();
                    matrices.resize(
                        geometry.positions.len(),
                        pose.nodes[layout.index(instance.node).unwrap()].matrix,
                    );
                }
                geometry.deform(&matrices).unwrap();
                output.extend(geometry.positions.iter().copied());
            }
        }
    }
    output
}

#[test]
fn mdlx_detach_outlives_native_bytes_and_keeps_ids_animation_and_skinning() {
    let (owned, ids, target, expected, native) = {
        let encoded = include_bytes!("fixtures/mdl/quad_model.mdx").to_vec();
        let source = mdlx::Document::<mdlx::V800>::decode_mdx(&encoded).unwrap();
        let mut model = io::mdlx::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        assert!(!model
            .meshes()
            .values()
            .next()
            .unwrap()
            .positions()
            .is_owned());
        let times = bytes(&[time(0.), time(1.)]);
        let values = bytes(&[Vec3::ZERO, Vec3::new(0., 0., 4.)]);
        let target = animate(&mut model, &times, &values);
        let expected = positions(&model.clone().widen::<Universal>(), 0.5);
        assert!(expected.iter().all(|position| position.z == 2.));
        let native = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .value
            .encode_mdx()
            .unwrap();
        let ids = identities(&model);
        (model.into_owned(), ids, target, expected, native)
    };
    let owned: Model<'static, Mdlx> = owned;
    owned.validate().unwrap();
    assert_eq!(identities(&owned), ids);
    assert!(owned
        .meshes()
        .values()
        .next()
        .unwrap()
        .positions()
        .is_owned());
    assert_eq!(
        owned
            .animation_sample(AnimationSample {
                clip_time: time(0.5),
                ..Default::default()
            })
            .unwrap()
            .animated(&target),
        Vec3::new(0., 0., 2.)
    );
    assert_eq!(
        positions(&owned.clone().widen::<Universal>(), 0.5),
        expected
    );
    assert_eq!(
        io::mdlx::plan::<mdlx::V800>(&owned, &ExportOptions::default())
            .unwrap()
            .execute()
            .value
            .encode_mdx()
            .unwrap(),
        native
    );
}

#[test]
fn m2_detach_preserves_skinned_geometry_and_hidden_particle_age_curves() {
    let (owned, ids, particle, expected) = {
        let encoded = fixtures::m2_bytes();
        let source = m2::Bundle::from_model(
            m2::Document::<m2::Flat<m2::V256>>::decode_m2(&encoded).unwrap(),
        );
        let mut model = io::m2::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        let times = bytes(&[time(0.), time(1.)]);
        let translations = bytes(&[Vec3::ZERO, Vec3::new(0., 0., 4.)]);
        animate(&mut model, &times, &translations);
        let colors = bytes(&[Vec3::X, Vec3::Y]);
        let spline = bytes(&[Vec3::ZERO, Vec3::new(1., 2., 3.)]);
        let (particle, mut feature) = model
            .features()
            .iter()
            .find_map(|(id, feature)| {
                matches!(&feature.payload, mp::ActorKind::Particle(_))
                    .then_some((id, feature.payload.clone()))
            })
            .unwrap();
        let mp::ActorKind::Particle(value) = &mut feature else {
            unreachable!()
        };
        value.appearance.color = Some(
            Curve::linear(
                PackedArray::<Time>::parse_bytes(&times).unwrap(),
                PackedArray::<Vec3>::parse_bytes(&colors).unwrap(),
            )
            .unwrap(),
        );
        value.details.shape.spline = PackedArray::parse_bytes(&spline).unwrap();
        model
            .edit(|draft| draft.set_feature(particle, feature))
            .unwrap();
        let expected = positions(&model.clone().widen::<Universal>(), 0.5);
        assert!(expected.iter().all(|position| position.z == 2.));
        let ids = identities(&model);
        (model.into_owned(), ids, particle, expected)
    };
    let owned: Model<'static, M2> = owned;
    owned.validate().unwrap();
    assert_eq!(identities(&owned), ids);
    assert_eq!(
        positions(&owned.clone().widen::<Universal>(), 0.5),
        expected
    );
    let mp::ActorKind::Particle(particle) = &owned.feature(particle).unwrap().payload else {
        panic!("particle kind changed")
    };
    let color = particle.appearance.color.as_ref().unwrap();
    assert!(color.values().is_owned() && color.times().is_owned());
    assert_eq!(color.sample(time(0.25)), Some(Vec3::new(0.75, 0.25, 0.)));
    assert!(particle.details.shape.spline.is_owned());
    assert_eq!(
        particle.details.shape.spline.get(1),
        Some(Vec3::new(1., 2., 3.))
    );
}

#[test]
fn wmo_detach_outlives_group_buffers_and_preserves_world_catalog_and_output() {
    let (owned, ids, expected) = {
        let (root, group) = fixtures::wmo_bytes();
        let source = wmo::Bundle::<wmo::Classic>::decode(&root, FileKey::Root, |_| {
            Ok(Some(group.as_slice()))
        })
        .unwrap();
        let model = io::wmo::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        assert_eq!(
            model.systems().values().next().unwrap().groups[0].name,
            "Hall"
        );
        assert_eq!(
            model
                .instances()
                .values()
                .next()
                .unwrap()
                .properties
                .description,
            "Stone chamber"
        );
        let ids = identities(&model);
        let expected = io::wmo::plan::<wmo::Classic>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .value
            .encode()
            .unwrap();
        (model.into_owned(), ids, expected)
    };
    let owned: Model<'static, Wmo> = owned;
    owned.validate().unwrap();
    assert_eq!(identities(&owned), ids);
    assert_eq!(
        owned.systems().values().next().unwrap().groups[0].name,
        "Hall"
    );
    assert_eq!(
        owned
            .instances()
            .values()
            .next()
            .unwrap()
            .properties
            .description,
        "Stone chamber"
    );
    assert_eq!(
        io::wmo::plan::<wmo::Classic>(&owned, &ExportOptions::default())
            .unwrap()
            .execute()
            .value
            .encode()
            .unwrap(),
        expected
    );
}

#[test]
fn generic_child_socket_keeps_named_identity_and_independent_control() {
    let child = {
        let resource = vec![0xff_u8, b'.', b'm', b'2'];
        let name = String::from("Weapon socket");
        ChildModelDefinition {
            resource: Cow::Borrowed(resource.as_slice()),
            socket: Some(SocketDefinition {
                identity: SocketIdentity::Named(Cow::Borrowed(&name)),
                visibility: 0.375_f32,
                control: 42_u32,
            }),
            appearance: (),
        }
        .into_owned()
    };
    assert!(matches!(child.resource, Cow::Owned(_)));
    let socket = child.socket.unwrap();
    assert!(
        matches!(socket.identity, SocketIdentity::Named(Cow::Owned(ref value)) if value == "Weapon socket")
    );
    assert_eq!(socket.visibility, 0.375);
    assert_eq!(socket.control, 42);
}

fn physics_body(node: NodeId, simulates: Property<bool>, shape: scene::ShapeId) -> Body {
    Body {
        node,
        motion: Motion::Dynamic,
        simulates,
        shapes: vec![shape],
        damping: Vec2::new(0.1, 0.2),
        inertia_scale: 1.,
        gravity_scale: 1.,
        inherit_dynamic: false,
        exempt_from_ragdoll: false,
        local_force_channels: ForceChannels::from_bits_retain(3),
        world_force_channels: ForceChannels::from_bits_retain(5),
        collision_filter: None,
        follow: None,
        locked: false,
    }
}
fn physics_joint(bodies: [scene::BodyId; 2]) -> Joint {
    Joint {
        bodies,
        kind: JointKind::Distance,
        frames: [Mat4::IDENTITY; 2],
        collide_connected: false,
        limit_enabled: false,
        limits: Vec2::ZERO,
        cone: 0.,
        friction: Friction::None,
        friction_amount: 0.,
        angular_spring: Vec2::ZERO,
        linear_spring: Vec2::ZERO,
        rest_length: 1.,
        breaking: Vec2::ZERO,
        motor: None,
        max_motor_force: None,
        motor_speed: None,
        reference_translation: None,
        locked: false,
    }
}
fn sphere() -> Shape<'static> {
    Shape {
        geometry: scene::Geometry::Sphere(2.),
        transform: Mat4::IDENTITY,
        material: ContactMaterial::default(),
        callback_categories: 17,
    }
}

#[test]
fn universal_detach_keeps_boolean_aliases_events_and_physics_scopes_editable() {
    let (
        mut owned,
        ids,
        alias,
        active,
        system_id,
        shape_id,
        body_ids,
        joint_id,
        source_id,
        light_id,
    ) = {
        let name = String::from("Borrowed hierarchy");
        let event_name = String::from("Footstep event");
        let points = bytes(&[Vec3::ZERO, Vec3::X, Vec3::Y]);
        let triangles = bytes(&[[0_u32, 1, 2]]);
        let times = bytes(&[time(0.), time(1.)]);
        let switches = bytes(&[true, false]);
        let mut draft = Model::<Mdlx>::builder(name.as_str());
        let node = draft.add_named_node(name.as_str()).unwrap();
        let active = draft.add_value(true);
        let alias = draft.values_mut().scalar_from_boolean(active).unwrap();
        let mut light: profile::Light = draft.construct();
        light.diffuse.intensity = alias;
        let light_id = draft
            .add_feature(node, profile::Node::Light(light))
            .unwrap();
        let source_id = draft.add_event_source(profile::EventSource {
            node,
            clock: TimeSource::Animation,
            name: Cow::Borrowed(&event_name),
        });
        draft
            .add_global_event(EventOccurrence {
                source: source_id,
                time: time(0.25),
            })
            .unwrap();
        let mut model = draft.finish().unwrap().widen::<Universal>();
        model
            .edit(|draft| {
                draft.add_global_track(DynamicTrack::Boolean(Track {
                    target: active,
                    clock: TimeSource::Animation,
                    curve: Curve::step(
                        PackedArray::parse_bytes(&times).unwrap(),
                        PackedArray::parse_bytes(&switches).unwrap(),
                    )
                    .unwrap(),
                }))
            })
            .unwrap();
        let mut physics = PhysicsSystem::default();
        let shape_id = physics
            .add_shape(Shape {
                geometry: scene::Geometry::Triangles {
                    positions: PackedArray::parse_bytes(&points).unwrap(),
                    triangles: PackedArray::parse_bytes(&triangles).unwrap(),
                },
                transform: Mat4::IDENTITY,
                material: ContactMaterial::default(),
                callback_categories: 23,
            })
            .unwrap();
        let first = physics
            .add_body(physics_body(node, active, shape_id))
            .unwrap();
        let second = physics
            .add_body(physics_body(node, active, shape_id))
            .unwrap();
        let joint_id = physics.add_joint(physics_joint([first, second])).unwrap();
        let system_id = model
            .edit(|draft| draft.add_system(System::Physics(physics)))
            .unwrap();
        let ids = identities(&model);
        (
            model.into_owned(),
            ids,
            alias,
            active,
            system_id,
            shape_id,
            [first, second],
            joint_id,
            source_id,
            light_id,
        )
    };
    owned.validate().unwrap();
    assert_eq!(identities(&owned), ids);
    assert!(owned.is_animated(alias));
    for (seconds, value) in [(0.5, 1.), (1.25, 0.)] {
        let sample = owned
            .animation_sample(AnimationSample {
                clip_time: time(seconds),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(sample.animated(&alias), value);
        assert_eq!(sample.animated(&active), value != 0.);
    }
    let wc3::model::EventSource::Mdlx(source) = owned.event_source(source_id).unwrap() else {
        panic!("event family changed")
    };
    assert!(matches!(source.name, Cow::Owned(_)));
    assert_eq!(source.name, "Footstep event");
    assert_eq!(owned.global_events().next().unwrap().source, source_id);
    assert!(owned.feature(light_id).is_some());
    let System::Physics(physics) = owned.system(system_id).unwrap() else {
        panic!("physics family changed")
    };
    assert_eq!(
        physics.shapes().map(|(id, _)| id).collect::<Vec<_>>(),
        [shape_id]
    );
    assert_eq!(
        physics.bodies().map(|(id, _)| id).collect::<Vec<_>>(),
        body_ids
    );
    assert_eq!(
        physics.joints().map(|(id, _)| id).collect::<Vec<_>>(),
        [joint_id]
    );
    assert_eq!(physics.body(body_ids[0]).unwrap().shapes, [shape_id]);
    assert_eq!(physics.joint(joint_id).unwrap().bodies, body_ids);
    let scene::Geometry::Triangles {
        positions,
        triangles,
    } = &physics.shape(shape_id).unwrap().geometry
    else {
        panic!("shape changed")
    };
    assert!(positions.is_owned() && triangles.is_owned());
    assert_eq!(positions.get(2), Some(Vec3::Y));
    let mut edited = physics.clone();
    let next_shape = edited.add_shape(sphere()).unwrap();
    let next_body = edited
        .add_body(physics_body(
            physics.body(body_ids[0]).unwrap().node,
            active,
            next_shape,
        ))
        .unwrap();
    let next_joint = edited
        .add_joint(physics_joint([body_ids[1], next_body]))
        .unwrap();
    assert_ne!(next_shape, shape_id);
    assert!(!body_ids.contains(&next_body));
    assert_ne!(next_joint, joint_id);
    let foreign_shape = PhysicsSystem::default().add_shape(sphere()).unwrap();
    assert!(edited.shape(foreign_shape).is_none());
    assert!(edited.set_shape(foreign_shape, sphere()).is_err());
    assert!(edited.shape(shape_id).is_some());
    assert!(edited.body(body_ids[0]).is_some());
    assert!(edited.joint(joint_id).is_some());
    edited.validate().unwrap();
    let old_nodes = ids.nodes;
    let new_node = owned
        .edit(|draft| {
            draft.set_system(system_id, System::Physics(edited))?;
            draft.add_named_node("After detach")
        })
        .unwrap();
    assert!(!old_nodes.contains(&new_node));
    assert!(old_nodes.into_iter().all(|node| owned.node(node).is_some()));
    owned.validate().unwrap();
}

#[test]
fn detaching_already_owned_model_moves_existing_geometry_buffers() {
    let source =
        mdlx::Document::<mdlx::V800>::decode_mdx(include_bytes!("fixtures/mdl/quad_model.mdx"))
            .unwrap();
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .into_owned();
    let ids = identities(&model);
    let positions = model
        .meshes()
        .values()
        .next()
        .unwrap()
        .positions()
        .as_bytes()
        .as_ptr();
    let owned = model.into_owned();
    assert_eq!(identities(&owned), ids);
    assert_eq!(
        owned
            .meshes()
            .values()
            .next()
            .unwrap()
            .positions()
            .as_bytes()
            .as_ptr(),
        positions
    );
}

// The profile still implements the original required vocabulary only. Its custom
// metadata opts into ownership independently through the field-wise derive.
#[derive(Clone, Debug)]
struct CustomProfile;
#[derive(Clone, Debug)]
struct Wrapper<T>(T);
impl<'owned, T: Detached<'owned>> Detached<'owned> for Wrapper<T> {
    type Detached = Wrapper<T::Detached>;
    fn into_detached(self) -> Self::Detached {
        Wrapper(self.0.into_detached())
    }
}
#[derive(Clone, Debug, IntoOwned)]
struct CustomMetadata<'a> {
    payload: Wrapper<Cow<'a, [u8]>>,
}
impl wc3::model::ValidateLinks for CustomMetadata<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(wc3::model::Link)) {}
}
macro_rules! mdlx_payloads {
    ($($name:ident),* $(,)?) => { $(type $name<'a> = <Mdlx as Profile>::$name<'a>;)* };
}
impl Profile for CustomProfile {
    mdlx_payloads!(
        ImageResource,
        ModelResource,
        EffectResource,
        PhysicsResource,
        BoneOverrideResource,
        Appearance,
        Binding,
        Geometry,
        EventSource,
        System,
        Surface,
        Feature,
        NodeProperties,
        MeshProperties,
        InstanceProperties,
        ClipProperties
    );
    type DrawProperties = <Mdlx as Profile>::DrawProperties;
    type ModelMetadata<'a> = CustomMetadata<'a>;
}

#[test]
fn custom_profile_opts_into_ownership_without_new_profile_methods() {
    let (owned, node, ids) = {
        let payload = vec![0xff, 0x00, 0xa5, 0x5a];
        let name = String::from("Custom payload");
        let mut draft = Model::<CustomProfile>::builder(CustomMetadata {
            payload: Wrapper(Cow::Borrowed(&payload)),
        });
        let node = draft.add_named_node(name.as_str()).unwrap();
        let model = draft.finish().unwrap();
        let ids = identities(&model);
        (model.into_owned(), node, ids)
    };
    let owned: Model<'static, CustomProfile> = owned;
    assert_eq!(identities(&owned), ids);
    assert!(matches!(owned.metadata().payload.0, Cow::Owned(_)));
    assert_eq!(owned.metadata().payload.0.as_ref(), [0xff, 0, 0xa5, 0x5a]);
    assert_eq!(owned.node(node).unwrap().name, "Custom payload");
    owned.validate().unwrap();
}
