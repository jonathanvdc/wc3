use glam::Vec3;
use wc3::model::common::IntoZeroVec;
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::io;
use wc3::model::IntoPackedArray;
use wc3::model::TimeSource;
use wc3::model::{
    conversion::{ExportOptions, ImportOptions},
    formats::mdlx::{
        self,
        animation::{SequenceFlags, Track, ValueKeyframe},
    },
    profiles::{mdlx as profile, Mdlx},
    Bounds, DynamicTrack, Model,
};

#[test]
fn additive_camera_keys_remain_exact_and_borrowed() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut camera = mdlx::scene::Camera::<mdlx::V800>::new("Camera").unwrap();
    camera.position = [10_000_000., 20., 30.];
    camera.target_position = [30_000_000., 40., 50.];
    camera.translation = Some(
        Track::linear(
            [
                ValueKeyframe {
                    frame: 0,
                    value: [0.1, 0.2, 0.3],
                },
                ValueKeyframe {
                    frame: 1000,
                    value: [0.7, 0.8, 0.9],
                },
            ],
            None,
        )
        .unwrap(),
    );
    camera.target_translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 0,
                value: [0.4, 0.5, 0.6],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_cameras([camera]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (_, feature) = graph.features().iter().next().unwrap();
    let profile::Node::Camera(camera) = &feature.payload else {
        panic!("camera feature missing")
    };
    assert_eq!(camera.target, Vec3::from_array([30_000_000., 40., 50.]));
    let property = camera.eye_offset;
    let curve = graph
        .tracks()
        .filter(|v| v.clock() == TimeSource::Animation)
        .find_map(|channel| match channel {
            DynamicTrack::Vector3(track) if track.target == property => Some(&track.curve),
            _ => None,
        })
        .unwrap();
    assert!(curve.values().is_owned());
    assert_eq!(
        curve.values().get(0),
        Some(Vec3::from_array([0.1, 0.2, 0.3]))
    );
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let camera = output.cameras().next().unwrap();
    assert_eq!(
        camera
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .value,
        [0.1, 0.2, 0.3]
    );
    assert_eq!(
        camera
            .target_translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .value,
        [0.4, 0.5, 0.6]
    );
}

#[test]
fn native_integer_effect_curves_keep_full_width() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_materials([mdlx::materials::Material::new()]);
    let node = mdlx::scene::Node::new("Ribbon", 0).unwrap();
    let mut ribbon = mdlx::emitters::RibbonEmitter::new(node);
    ribbon.texture_slot.set_track(
        Track::step(
            [ValueKeyframe {
                frame: 0,
                value: 16_777_217,
            }],
            None,
        )
        .unwrap(),
    );
    source.set_ribbon_emitters([ribbon]);
    source.set_pivot_points(&[[0.; 3]]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let curve = graph
        .tracks()
        .filter(|v| v.clock() == TimeSource::Animation)
        .find_map(|channel| match channel {
            DynamicTrack::Unsigned(track) => Some(&track.curve),
            _ => None,
        })
        .unwrap();
    assert_eq!(curve.values().get(0), Some(16_777_217));
    assert!(curve.values().is_owned());
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output
            .ribbon_emitters()
            .next()
            .unwrap()
            .texture_slot
            .track()
            .unwrap()
            .step_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .value,
        16_777_217
    );
}

#[test]
fn unknown_native_blend_remains_its_own_operation() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut material = mdlx::materials::Material::new();
    let mut layer = mdlx::materials::Layer::new();
    layer.texture_id = mdlx::animation::Animatable::Static(u32::MAX);
    layer.filter_mode = mdlx::materials::LayerFilterMode::Unknown(8);
    material.layers.push(layer);
    source.set_materials([material]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let surface = &imported.value.materials().iter().next().unwrap().1.surface;
    let material = surface;
    assert_eq!(material.steps[0].state.blend, profile::Blend::Unknown(8));
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.materials().next().unwrap().layers[0].filter_mode,
        mdlx::materials::LayerFilterMode::Unknown(8)
    );
}

#[test]
fn unsigned_native_intervals_and_blend_duration_are_not_keyframe_limited() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_global_sequences(&[u32::MAX]);
    source.set_sequences([
        mdlx::animation::Sequence::new("Long", [3_000_000_000, 4_000_000_000]).unwrap(),
    ]);
    let mut info = mdlx::scene::ModelInfo::new("Long").unwrap();
    info.blend_time = u32::MAX;
    source.set_model_info(&info);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.sequences().next().unwrap().interval,
        [3_000_000_000, 4_000_000_000]
    );
    assert_eq!(output.global_sequences(), [u32::MAX]);
    assert_eq!(output.model_info().unwrap().blend_time, u32::MAX);
}

#[test]
fn borrowed_mesh_authoring_edits_rebuild_native_geometry() {
    let source = mdlx::Document::<mdlx::V800>::decode_mdl(include_str!(
        "../../../fixtures/mdl/quad_model.mdl"
    ))
    .unwrap();
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, mesh) = graph.meshes().iter().next().unwrap();
    assert!(!mesh.positions().is_owned());
    let mut edited = mesh.clone();
    let mut positions = edited.positions().to_vec();
    positions[0] = Vec3::from_array([1., 2., 3.]);
    edited.geometry = edited
        .geometry
        .with_positions((positions).into_packed_array())
        .unwrap();
    imported
        .value
        .edit(|builder| builder.set_mesh(id, edited))
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.geosets().next().unwrap().vertices().get(0),
        Some([1., 2., 3.])
    );
    assert_eq!(
        output.geosets().next().unwrap().face_indices(),
        source.geosets().next().unwrap().face_indices()
    );
    assert_eq!(
        output.textures().next().unwrap().path.text(),
        "Textures\\Quad.blp"
    );
}

#[test]
fn invalid_unicode_names_need_an_explicit_loss_policy() {
    use wc3::model::conversion::LossPolicy;
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut node = mdlx::scene::Node::new("Name", 0).unwrap();
    let mut bytes = [0u8; 80];
    bytes[0] = 0xff;
    node.name = mdlx::mdx::FixedText::from_bytes(bytes);
    source.set_helpers([node]);
    assert!(io::mdlx::import(&source, &ImportOptions::default()).is_err());
    let output = io::mdlx::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!output.report.issues.is_empty());
    assert_eq!(output.value.nodes().iter().next().unwrap().1.name, "�");
}

#[test]
fn exterior_keys_keep_exact_milliseconds_across_large_clip_origins() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_sequences([
        mdlx::animation::Sequence::new("Late", [3_000_000_000, 4_000_000_000]).unwrap(),
    ]);
    let mut node = mdlx::scene::Node::new("Outside", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 1,
                value: [1., 2., 3.],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_helpers([node]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output
            .helpers()
            .next()
            .unwrap()
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .frame,
        1
    );
}

#[test]
fn shared_uv_transform_links_are_scoped_and_rebuild_animation() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_textures([mdlx::materials::Texture::new("UV.blp").unwrap()]);
    let mut layer = mdlx::materials::Layer::new();
    layer.texture_animation_id = 0;
    let mut material = mdlx::materials::Material::new();
    material.layers.push(layer);
    source.set_materials([material]);
    let mut transform = mdlx::animation::TextureAnimation::new();
    transform.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 0,
                value: [0.1, 0.2, 0.3],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_texture_animations([transform]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (_id, material) = graph.materials().iter().next().unwrap();
    let transform = &material.surface.steps[0]
        .shader
        .inputs()
        .binding
        .coordinates
        .transform;
    assert!(graph.is_animated(transform.translation));
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.materials().next().unwrap().layers[0].texture_animation_id,
        0
    );
    assert_eq!(
        output
            .texture_animations()
            .next()
            .unwrap()
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .value,
        [0.1, 0.2, 0.3]
    );
}

#[test]
fn unchanged_packed_geometry_is_borrowed_by_native_output() {
    let source = mdlx::Document::<mdlx::V800>::decode_mdl(include_str!(
        "../../../fixtures/mdl/quad_model.mdl"
    ))
    .unwrap();
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mesh = output.geosets().next().unwrap();
    assert!(!mesh.vertices().is_owned());
    assert!(!mesh.normals().is_owned());
    assert!(!mesh.uv_sets()[0].is_owned());
    assert_eq!(mesh.vertices(), source.geosets().next().unwrap().vertices());
}

#[test]
fn absolute_tracks_without_sequences_do_not_invent_playback_clips() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut node = mdlx::scene::Node::new("Timeline", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 10,
                value: [1., 2., 3.],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_helpers([node]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    assert_eq!(model.clips().len(), 0);
    assert_eq!(
        model
            .tracks()
            .filter(|v| v.clock() == TimeSource::Animation)
            .count(),
        1
    );
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.sequences().count(), 0);
    assert_eq!(
        output.nodes()[0]
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .frame,
        10
    );
}

#[test]
fn overlapping_sequences_share_one_authored_absolute_curve() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_sequences([
        mdlx::animation::Sequence::new("First", [0, 1000]).unwrap(),
        mdlx::animation::Sequence::new("Second", [500, 1500]).unwrap(),
    ]);
    let mut node = mdlx::scene::Node::new("Shared", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 750,
                value: [1., 2., 3.],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_helpers([node]);
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    assert_eq!(graph.clips().len(), 2);
    assert_eq!(
        graph
            .tracks()
            .filter(|v| v.clock() == TimeSource::Animation)
            .count(),
        1
    );
    assert!(graph
        .clips()
        .iter()
        .all(|(_, clip)| clip.animations.len() == 1));
    let selections: Vec<_> = graph
        .clips()
        .iter()
        .map(|(_, clip)| clip.animations[0])
        .collect();
    assert_eq!(selections[0], selections[1]);
    assert!(graph
        .global_tracks()
        .all(|channel| matches!(channel.clock(), TimeSource::Shared(_))));
    let DynamicTrack::Vector3(mut track) = graph
        .tracks()
        .find(|v| v.clock() == TimeSource::Animation)
        .unwrap()
        .clone()
    else {
        panic!("translation curve missing")
    };
    track.curve = wc3::model::Curve::linear(
        track.curve.times().clone(),
        (vec![[4., 5., 6.]])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    imported
        .value
        .edit(|builder| {
            let id = builder
                .model()
                .animations()
                .iter()
                .find(|(_, animation)| !animation.tracks.is_empty())
                .unwrap()
                .0;
            let mut animation = builder.model().animation(id).unwrap().clone();
            animation.tracks[0] = DynamicTrack::Vector3(track);
            builder.set_animation(id, animation)
        })
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let nodes = output.nodes();
    let track = nodes[0].translation.as_ref().unwrap();
    assert_eq!(track.linear_keys().unwrap().get(0).unwrap().frame, 750);
    assert_eq!(
        track.linear_keys().unwrap().get(0).unwrap().value,
        [4., 5., 6.]
    );
    let second = imported.value.clips().iter().nth(1).unwrap().0;
    assert!(
        imported
            .value
            .edit(|builder| {
                let mut clip = builder.model().clip(second).unwrap().clone();
                clip.animations.clear();
                builder.set_clip(second, clip)
            })
            .is_err(),
        "native MDLX cannot gate the shared timeline to only one sequence"
    );
}

#[test]
fn global_tracks_without_sequences_stay_on_the_shared_clock() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_global_sequences(&[1000]);
    let mut node = mdlx::scene::Node::new("Global", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 250,
                value: [1., 2., 3.],
            }],
            Some(0),
        )
        .unwrap(),
    );
    source.set_helpers([node]);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    assert_eq!(graph.clips().len(), 0);
    assert!(graph
        .tracks()
        .find(|v| v.clock() == TimeSource::Animation)
        .is_none());
    assert_eq!(
        graph
            .tracks()
            .filter(|v| matches!(v.clock(), TimeSource::Shared(_)))
            .count(),
        1
    );
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.sequences().count(), 0);
    assert_eq!(output.global_sequences(), [1000]);
    assert_eq!(
        output.nodes()[0]
            .translation
            .as_ref()
            .unwrap()
            .global_sequence_id(),
        Some(0)
    );
}

#[test]
fn tcb_curves_cannot_be_encoded_as_native_mdlx_tracks() {
    use wc3::model::{Curve, TcbParameters};
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut node = mdlx::scene::Node::new("TCB", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 0,
                value: [1., 2., 3.],
            }],
            None,
        )
        .unwrap(),
    );
    source.set_helpers([node]);
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let DynamicTrack::Vector3(mut track) = graph
        .tracks()
        .find(|v| v.clock() == TimeSource::Animation)
        .unwrap()
        .clone()
    else {
        panic!("translation missing")
    };
    track.curve = Curve::tcb(
        track.curve.times().clone(),
        track.curve.values().clone(),
        (vec![TcbParameters::default()]).into_zero_vec(),
    )
    .unwrap();
    if imported
        .value
        .edit(|builder| {
            let id = builder
                .model()
                .animations()
                .iter()
                .find(|(_, animation)| !animation.tracks.is_empty())
                .unwrap()
                .0;
            let mut animation = builder.model().animation(id).unwrap().clone();
            animation.tracks[0] = DynamicTrack::Vector3(track);
            builder.set_animation(id, animation)
        })
        .is_ok()
    {
        assert!(io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default()).is_err());
    }
}

#[test]
fn skinned_geometry_placement_edits_cannot_silently_disappear() {
    let source = mdlx::Document::<mdlx::V800>::decode_mdl(include_str!(
        "../../../fixtures/mdl/quad_model.mdl"
    ))
    .unwrap();
    let mut asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let anchor = asset.instances().iter().next().unwrap().1.node;
    let result = asset.edit(|builder| {
        let node = builder.model().node(anchor).unwrap().clone();
        builder
            .values_mut()
            .set(node.transform.translation, Vec3::from_array([1., 0., 0.]))?;
        builder.set_node(anchor, node)
    });
    assert!(result.is_err());
    assert_eq!(
        asset.values()[asset.node(anchor).unwrap().transform.translation],
        Vec3::from_array([0.; 3])
    );
}

#[test]
fn logical_sequence_flag_words_keep_unknown_bits_through_native_output() {
    let mut source = mdlx::Document::<mdlx::V800>::decode_mdl(include_str!(
        "../../../fixtures/mdl/quad_model.mdl"
    ))
    .unwrap();
    source.sequences_mut().next().unwrap().flags = SequenceFlags(0x8000_0000);
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        asset
            .clips()
            .iter()
            .next()
            .unwrap()
            .1
            .controls
            .unclassified
            .bits(),
        0x8000_0000
    );
    let output = io::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.sequences().next().unwrap().flags.bits(), 0x8000_0000);
}

#[test]
fn absent_model_info_stays_absent_while_named_drafts_declare_bounds() {
    let source = mdlx::Document::<mdlx::V800>::new();
    assert!(source.model_info().is_none());
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(asset.bounds().is_none());
    let output = io::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert!(output.model_info().is_none());
    let named = Model::<Mdlx>::builder("named").finish().unwrap();
    assert_eq!(named.bounds(), Some(Bounds::default()));
    let mut asset = named;
    assert!(asset
        .edit(|builder| {
            builder.metadata_mut().bounds = None;
            Ok(())
        })
        .is_err());
    assert!(asset.bounds().is_some());
}

#[test]
fn empty_material_declarations_require_explicit_semantic_loss() {
    use wc3::model::conversion::LossPolicy;
    let mut source = mdlx::Document::<mdlx::V800>::new();
    let mut material = mdlx::materials::Material::new();
    material.priority_plane = 7;
    source.set_materials([material]);
    assert!(io::mdlx::import(&source, &ImportOptions::default()).is_err());
    let imported = io::mdlx::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "material.empty_declarations"));
    assert!(imported
        .value
        .widen::<wc3::model::profiles::Universal>()
        .materials()
        .iter()
        .next()
        .unwrap()
        .1
        .surface
        .steps
        .is_empty());
    // The native document still owns its declaration after semantic conversion.
    assert_eq!(source.materials().next().unwrap().priority_plane, 7);
}

#[test]
fn distinct_selected_animations_preserve_coincident_event_occurrences() {
    use wc3::model::Animation;
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_sequences([mdlx::animation::Sequence::new("Events", [0, 1000]).unwrap()]);
    source.set_event_objects([mdlx::scene::EventObject::new(
        mdlx::scene::Node::new("Event", 0).unwrap(),
        u32::MAX,
        &[500],
    )]);
    let mut model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let clip = model.clips().iter().next().unwrap().0;
    let event = model.clip_events(clip).next().unwrap().clone();
    model
        .edit(|builder| {
            let animation = builder.add_animation(Animation::default());
            builder.add_animation_event(animation, event)?;
            let selection = builder
                .model()
                .clip(clip)
                .unwrap()
                .clone()
                .with_animation(animation);
            builder.set_clip(clip, selection)
        })
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(*output.event_objects().next().unwrap().frames(), [500, 500]);
}

#[test]
fn empty_shared_event_source_retains_its_clock_and_name() {
    let mut source = mdlx::Document::<mdlx::V800>::new();
    source.set_global_sequences(&[1000]);
    source.set_event_objects([mdlx::scene::EventObject::new(
        mdlx::scene::Node::new("Empty action", 0).unwrap(),
        0,
        &[],
    )]);
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(model.event_sources().len(), 1);
    assert!(model.features().is_empty());
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let event = output.event_objects().next().unwrap();
    assert_eq!(event.node.name.text(), "Empty action");
    assert_eq!(event.global_sequence_id, 0);
    assert!(event.frames().is_empty());
}

#[test]
fn independent_event_sources_can_share_an_anchor_without_becoming_helpers() {
    use wc3::model::{EventOccurrence, Time};
    let mut draft = Model::<Mdlx>::builder("");
    let node = draft.add_named_node("anchor").unwrap();
    for name in ["Left foot", "Right foot"] {
        let source = draft.add_event_source(profile::EventSource {
            node,
            clock: TimeSource::Animation,
            name: name.into(),
        });
        draft
            .add_global_event(EventOccurrence {
                source,
                time: Time::seconds(0.5).unwrap(),
            })
            .unwrap();
    }
    let model = draft.finish().unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let events: Vec<_> = output.event_objects().collect();
    assert_eq!(events.len(), 2);
    assert_ne!(events[0].node.object_id, events[1].node.object_id);
    assert_eq!(events[0].node.parent_id, events[1].node.parent_id);
    assert_eq!(events[0].node.name.text(), "Left foot");
    assert_eq!(events[1].node.name.text(), "Right foot");
    assert_eq!(*events[0].frames(), [500]);
    assert_eq!(*events[1].frames(), [500]);
}
