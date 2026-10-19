use super::*;
use crate::runtime::animate_instances;
use crate::{compile_model, spawn_model, ModelMaterial, ModelRig, Playback};
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::conversion::ImportOptions;
use wc3::model::io::mdlx;
use wc3::model::{
    AnimationSample, Curve, DynamicTrack, Model, Time as ModelTime, TimeSource, Track,
};

fn source() -> Model<'static> {
    mdlx::decode_mdl(
        include_str!("../tests/fixtures/geoset_capture.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen()
}
fn time(value: f64) -> ModelTime {
    ModelTime::seconds(value).unwrap()
}
#[test]
fn geometry_controllers_isolate_meshes_without_marking_constants_dynamic() {
    let model = source();
    let compiled = compile_model(&model, |_| None).unwrap();
    let [left, right] = compiled.draws.as_slice() else {
        panic!("expected two geometry draws")
    };
    assert!(!left.animation.is_animated());
    assert!(right.animation.is_animated());
    assert_ne!(left.mesh_state, right.mesh_state);
    let clip = model.clips().iter().next().unwrap().0;
    let sample = model
        .animation_sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(0.75),
            ..Default::default()
        })
        .unwrap();
    let mut material = right.material.base.clone();
    right.animation.apply(&sample, &mut material);
    assert!((material.base_color.alpha() - 0.5).abs() < 1e-5);
    assert_eq!(material.alpha_mode, AlphaMode::AlphaToCoverage);
    let first = material.base_color;
    assert!(!right.animation.apply(&sample, &mut material));
    assert_eq!(first, material.base_color);
    // Authored constant tint is independent of the second mesh's controller.
    assert_ne!(
        left.material.base.base_color.to_linear(),
        material.base_color.to_linear()
    );
}
#[test]
fn common_mesh_targets_override_feature_values_and_equal_constants_share_keys() {
    let mut model = source();
    let mesh = model.meshes().iter().next().unwrap().0;
    let mut states = MeshAnimation::compile(&model);
    let constant = states.remove(&mesh).unwrap();
    assert!(!constant.is_animated());
    let other_mesh = model.meshes().iter().nth(1).unwrap().0;
    assert_eq!(state(&constant, mesh), state(&constant, other_mesh));
    let definition = model
        .features()
        .values()
        .find_map(|feature| match &feature.payload {
            wc3::model::NodeFeature::GeometryAnimation(value) if value.mesh == Some(mesh) => {
                Some(value.clone())
            }
            _ => None,
        })
        .unwrap();
    model
        .edit(|builder| {
            builder.add_global_track(DynamicTrack::Vector3(Track {
                target: definition.color,
                clock: TimeSource::Animation,
                curve: Curve::linear(
                    [time(0.), time(1.)],
                    ([[0.2, 0.4, 0.6], [0.4, 0.6, 0.8]])
                        .into_iter()
                        .map(Vec3::from_array)
                        .collect::<Vec<_>>(),
                )?,
            }))?;
            builder.add_global_track(DynamicTrack::Scalar(Track {
                target: definition.opacity,
                clock: TimeSource::Animation,
                curve: Curve::linear([time(0.), time(1.)], [0.5, 1.])?,
            }))
        })
        .unwrap();
    let states = MeshAnimation::compile(&model);
    let value = &states[&mesh];
    assert!(value.is_animated());
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(0.5),
            ..Default::default()
        })
        .unwrap();
    let rgba = value.sample(&sample);
    assert!((rgba[0] - 0.3).abs() < 1e-5);
    assert!((rgba[3] - 0.75).abs() < 1e-5);
}
#[test]
fn geometry_alpha_hides_draw_and_materials_share_only_matching_mesh_states() {
    let model = source();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    app.add_systems(Update, animate_instances);
    let mut materials = Assets::<ModelMaterial>::default();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::IDENTITY,
    );
    let second = spawn_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::IDENTITY,
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    let clip = prepared.clips()[0].0;
    for root in [first, second] {
        let mut playback = app.world_mut().get_mut::<Playback>(root).unwrap();
        playback.clip = Some(clip);
        playback.clip_time = 1.;
        playback.paused = true;
    }
    app.update();
    let collect = |world: &World, root: Entity| {
        let instance = world.get::<ModelRig>(root).unwrap();
        compiled
            .draws
            .iter()
            .map(|draw| {
                let node = instance.node(compiled.nodes[draw.node].id).unwrap();
                world
                    .get::<Children>(node)
                    .unwrap()
                    .iter()
                    .find_map(|entity| {
                        Some((
                            world
                                .get::<MeshMaterial3d<ModelMaterial>>(entity)?
                                .0
                                .clone(),
                            *world.get::<Visibility>(entity)?,
                        ))
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>()
    };
    let first = collect(app.world(), first);
    let second = collect(app.world(), second);
    assert_eq!(first[0].0, second[0].0);
    assert_ne!(first[1].0, second[1].0);
    assert_eq!(first[1].1, Visibility::Hidden);
    assert_eq!(first[0].1, Visibility::Inherited);
}
