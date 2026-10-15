use wc3::model::common::IntoZeroVec;
use wc3::model::mdlx::mdl::Read as _;
use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions},
    mdlx::{
        self,
        animation::{SequenceFlags, Track, ValueKeyframe},
    },
    semantic::{
        profiles::{mdlx as profile, Mdlx},
        Bounds, Channel, Model,
    },
};

#[test]
fn additive_camera_keys_remain_exact_and_borrowed() {
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (_, feature) = graph.features().next().unwrap();
    let profile::Node::Camera(camera) = &feature.payload else {
        panic!("camera feature missing")
    };
    assert_eq!(camera.target, [30_000_000., 40., 50.]);
    let property = camera.eye_offset.property.as_ref().unwrap();
    let curve = graph
        .timeline_channels()
        .iter()
        .find_map(|channel| match channel {
            Channel::Vector3(track) if track.target.target() == property.target() => {
                Some(&track.curve)
            }
            _ => None,
        })
        .unwrap();
    assert!(curve.values().is_owned());
    assert_eq!(curve.values().get(0), Some([0.1, 0.2, 0.3]));
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let curve = graph
        .timeline_channels()
        .iter()
        .find_map(|channel| match channel {
            Channel::Unsigned(track) => Some(&track.curve),
            _ => None,
        })
        .unwrap();
    assert_eq!(curve.values().get(0), Some(16_777_217));
    assert!(curve.values().is_owned());
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
    let mut material = mdlx::materials::Material::new();
    let mut layer = mdlx::materials::Layer::new();
    layer.texture_id = mdlx::animation::Animatable::Static(u32::MAX);
    layer.filter_mode = mdlx::materials::LayerFilterMode::Unknown(8);
    material.layers.push(layer);
    source.set_materials([material]);
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let surface = &imported.value.materials().next().unwrap().1.surface;
    let material = surface;
    assert_eq!(material.layers[0].blend, profile::Blend::Unknown(8));
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
    source.set_global_sequences(&[u32::MAX]);
    source.set_sequences([
        mdlx::animation::Sequence::new("Long", [3_000_000_000, 4_000_000_000]).unwrap(),
    ]);
    let mut info = mdlx::scene::ModelInfo::new("Long").unwrap();
    info.blend_time = u32::MAX;
    source.set_model_info(&info);
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let source =
        mdlx::Model::<mdlx::V800>::decode_mdl(include_str!("../../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, mesh) = graph.meshes().next().unwrap();
    assert!(!mesh.positions().is_owned());
    let mut edited = mesh.clone();
    let mut positions = edited.positions().to_vec();
    positions[0] = [1., 2., 3.];
    edited.geometry = edited
        .geometry
        .with_positions((positions).into_zero_vec())
        .unwrap();
    imported
        .value
        .edit(|builder| builder.set_mesh(id, edited))
        .unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
    let mut node = mdlx::scene::Node::new("Name", 0).unwrap();
    let mut bytes = [0u8; 80];
    bytes[0] = 0xff;
    node.name = mdlx::mdx::FixedText::from_bytes(bytes);
    source.set_helpers([node]);
    assert!(conversion::mdlx::import(&source, &ImportOptions::default()).is_err());
    let output = conversion::mdlx::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!output.report.issues.is_empty());
    assert_eq!(output.value.nodes().next().unwrap().1.name, "�");
}

#[test]
fn exterior_keys_keep_exact_milliseconds_across_large_clip_origins() {
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, transform) = graph.texture_transforms().next().unwrap();
    assert_eq!(
        transform.translation.property.as_ref().unwrap(),
        &id.translation::<Mdlx>()
    );
    let material = &graph.materials().next().unwrap().1.surface;
    assert_eq!(material.layers[0].texture_transform, Some(id));
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let source =
        mdlx::Model::<mdlx::V800>::decode_mdl(include_str!("../../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    assert_eq!(model.clips().len(), 0);
    assert_eq!(model.timeline_channels().len(), 1);
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    assert_eq!(graph.clips().len(), 2);
    assert_eq!(graph.timeline_channels().len(), 1);
    assert!(graph.clips().all(|(_, clip)| clip.channels().is_empty()));
    let Channel::Vector3(mut track) = graph.timeline_channels()[0].clone() else {
        panic!("translation curve missing")
    };
    track.curve =
        wc3::model::semantic::Curve::linear(track.curve.times().clone(), vec![[4., 5., 6.]])
            .unwrap();
    imported
        .value
        .edit(|builder| builder.set_timeline_channel(0, Channel::Vector3(track)))
        .unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
}

#[test]
fn global_tracks_without_sequences_stay_on_the_shared_clock() {
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    assert_eq!(graph.clips().len(), 0);
    assert!(graph.timeline_channels().is_empty());
    assert_eq!(graph.shared_channels().len(), 1);
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    use wc3::model::semantic::{Curve, TcbParameters};
    let mut source = mdlx::Model::<mdlx::V800>::new();
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
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let Channel::Vector3(mut track) = graph.timeline_channels()[0].clone() else {
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
        .edit(|builder| builder.set_timeline_channel(0, Channel::Vector3(track)))
        .is_ok()
    {
        assert!(
            conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
                .is_err()
        );
    }
}

#[test]
fn skinned_geometry_placement_edits_cannot_silently_disappear() {
    let source =
        mdlx::Model::<mdlx::V800>::decode_mdl(include_str!("../../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let mut asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let anchor = asset.instances().next().unwrap().1.node;
    let result = asset.edit(|builder| {
        let mut node = builder.model().node(anchor).unwrap().clone();
        node.transform.translation = [1., 0., 0.];
        builder.set_node(anchor, node)
    });
    assert!(result.is_err());
    assert_eq!(asset.node(anchor).unwrap().transform.translation, [0.; 3]);
}

#[test]
fn logical_sequence_flag_words_keep_unknown_bits_through_native_output() {
    let mut source =
        mdlx::Model::<mdlx::V800>::decode_mdl(include_str!("../../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    source.sequences_mut().next().unwrap().flags = SequenceFlags(0x8000_0000);
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        asset.clips().next().unwrap().1.controls.unclassified.bits(),
        0x8000_0000
    );
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.sequences().next().unwrap().flags.bits(), 0x8000_0000);
}

#[test]
fn absent_model_info_stays_absent_while_named_drafts_declare_bounds() {
    let source = mdlx::Model::<mdlx::V800>::new();
    assert!(source.model_info().is_none());
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(asset.bounds().is_none());
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
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
            builder.set_bounds(None);
            Ok(())
        })
        .is_err());
    assert!(asset.bounds().is_some());
}
