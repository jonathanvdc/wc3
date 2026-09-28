use wc3::model::animation::{AnimationTrack, CameraVisibility, ValueKeyframe};
use wc3::model::chunks::{GlidersChunk, ModelChunk, RawChunk};
use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::materials::{Layer, LayerShadingFlags, Material, MaterialRenderFlags, ShaderType};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Camera, CameraTrack, EventObject, Glider, Light, LightFalloff, Node};
use wc3::model::CommonModelAccess;
use wc3::model::{ConversionOptions, DynamicModel, Model, V1100, V1800, V800};

#[test]
fn signed_event_times_and_material_priority_preserve_wire_bits() {
    let frames = [i32::MIN, -3600, 0, i32::MAX];
    let event = EventObject::new(Node::new("Event", 0).unwrap(), u32::MAX, &frames);
    let bytes = event.encode_mdx().unwrap();
    let expected: Vec<_> = frames
        .iter()
        .flat_map(|frame| frame.to_le_bytes())
        .collect();
    assert_eq!(&bytes[bytes.len() - 16..], expected);
    assert_eq!(
        EventObject::decode_mdx(&bytes).unwrap().frames.as_slice(),
        frames
    );
    let mut material = Material::<V800>::new();
    material.priority_plane = -7;
    let bytes = material.encode_mdx().unwrap();
    assert_eq!(&bytes[4..8], &(-7i32).to_le_bytes());
    assert_eq!(
        Material::<V800>::decode_mdx(&bytes).unwrap().priority_plane,
        -7
    );
    let mut model = Model::<V800>::new();
    model.set_materials(&[material]);
    model.set_event_objects(&[event]);
    let converted = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(converted.materials()[0].priority_plane, -7);
    assert_eq!(converted.event_objects()[0].frames.as_slice(), frames);
}

#[test]
fn camera_dof_scalar_keywords_produce_correct_keyed_wire_tags() {
    for (source, tag, keyword, value) in [
        (
            "DOFDistance 180.0,",
            *b"IDUF",
            "FocusDistanceKeys",
            180.0f32,
        ),
        ("FocalLength 50.0,", *b"ELAF", "FocalLengthKeys", 50.0),
        ("FStop 2.8,", *b"PTSF", "FStopKeys", 2.8),
    ] {
        let track = CameraTrack::decode_mdl(source).unwrap();
        let mut expected = tag.to_vec();
        expected.extend_from_slice(&1u32.to_le_bytes()); // count
        expected.extend_from_slice(&0u32.to_le_bytes()); // stepped
        expected.extend_from_slice(&u32::MAX.to_le_bytes()); // no global sequence
        expected.extend_from_slice(&0i32.to_le_bytes()); // frame zero
        expected.extend_from_slice(&value.to_le_bytes());
        assert_eq!(track.encode_mdx().unwrap(), expected);
        assert_eq!(CameraTrack::decode_mdx(&expected).unwrap(), track);
        let text = track.encode_mdl().unwrap();
        assert!(text.starts_with(keyword));
        assert_eq!(CameraTrack::decode_mdl(&text).unwrap(), track);
    }
    assert!(CameraTrack::decode_mdl("FocusDistance 5.0,").is_err());
    assert!(CameraTrack::decode_mdl("FStop 2.8").is_err());
}

#[test]
fn camera_extended_tracks_survive_model_io_and_conversion() {
    let visibility = AnimationTrack::<CameraVisibility>::linear(
        vec![ValueKeyframe {
            frame: -100,
            value: 0.5,
        }],
        Some(0),
    )
    .unwrap();
    let tracks = vec![
        CameraTrack::Visibility(visibility),
        CameraTrack::focus_distance(180.0),
        CameraTrack::focal_length(50.0),
        CameraTrack::f_stop(2.8),
    ];
    let mut camera = Camera::<V1800>::new("Portrait").unwrap();
    camera.tracks = (&tracks).to_vec();
    let mut model = Model::<V1800>::new();
    model.set_cameras(&[camera]);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V1800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.cameras()[0].tracks.as_slice(), tracks);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    // These track tags are recognized by the new client without a layout gate.
    let converted = model
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(converted.cameras()[0].tracks.as_slice(), tracks);
    assert_eq!(
        Model::<V800>::decode_mdx(&converted.encode_mdx().unwrap())
            .unwrap()
            .cameras()[0]
            .tracks
            .as_slice(),
        tracks
    );
}

#[test]
fn gliders_keep_every_entry_in_all_versions_and_reject_partial_words() {
    let entries = [Glider { geoset_id: 3 }, Glider { geoset_id: 7 }];
    let expected = [
        b"DILG".as_slice(),
        &8u32.to_le_bytes(),
        &3u32.to_le_bytes(),
        &7u32.to_le_bytes(),
    ]
    .concat();
    assert_eq!(
        GlidersChunk::new(entries.to_vec()).encode_mdx().unwrap(),
        expected
    );
    let mut model = Model::<V800>::new();
    model.set_gliders(&entries);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.gliders(), entries);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    let newer = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(newer.gliders(), entries);
    let older = newer
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(older.gliders(), entries);
    let mut dynamic = DynamicModel::V1800(newer);
    assert_eq!(dynamic.gliders(), entries);
    dynamic.set_gliders(&[entries[1], entries[0], entries[1]]);
    assert_eq!(dynamic.gliders(), [entries[1], entries[0], entries[1]]);
    let chunk = ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0, 0])).unwrap();
    assert!(matches!(chunk, ModelChunk::Gliders(_)));
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0])).is_err());
    // Clearing removes every occurrence, including manually appended chunks.
    model
        .chunks
        .push(GlidersChunk::new(entries.to_vec()).into());
    model.set_gliders(&[]);
    assert!(model.chunk(*b"DILG").is_none());
    assert!(model.gliders().is_empty());
    // Existing gated access remains independent of ungated DILG.
    assert!(model.try_face_fx().is_err());
}

#[test]
fn named_shader_mapping_and_new_flags_preserve_raw_storage() {
    for (id, name) in [
        (0, "Shader_SD_Legacy"),
        (1, "Shader_HD_DefaultUnit"),
        (2, "Shader_SD_FixedFunction"),
        (24, "Shader_HD_Crystal"),
    ] {
        let shader = ShaderType::new(id);
        assert_eq!(shader.id(), id);
        assert_eq!(shader.name(), Some(name));
        assert_eq!(
            ShaderType::from_name(&name.to_ascii_lowercase()),
            Some(shader)
        );
        let mut layer = Layer::<V1100>::new();
        layer.set_shader_type(shader);
        assert_eq!(
            Layer::<V1100>::decode_mdx(&layer.encode_mdx().unwrap())
                .unwrap()
                .shader_type(),
            shader
        );
    }
    assert_eq!(ShaderType::default(), ShaderType::SD_LEGACY);
    for raw in [3u32, u32::MAX] {
        let bytes = raw.to_le_bytes();
        let shader = ShaderType::decode_mdx(&bytes).unwrap();
        assert_eq!(shader, ShaderType::new(raw));
        assert_eq!(shader.name(), None);
        assert_eq!(shader.encode_mdx().unwrap(), bytes);
        let mut layer = Layer::<V1100>::new();
        layer.set_shader_type(shader);
        let wire = layer.encode_mdx().unwrap();
        assert_eq!(
            Layer::<V1100>::decode_mdx(&wire).unwrap().shader_type(),
            shader
        );
    }
    assert!(ShaderType::new(3).name().is_none());
    assert!(ShaderType::from_name("typo").is_none());
    let mut flags = LayerShadingFlags(0);
    flags.set_wrap_width(true);
    flags.set_wrap_height(true);
    flags.set_back_faces_for_shadows(true);
    flags.set_ambient_occlusion(true);
    assert_eq!(flags.bits(), 0x60c);
    let mut layer = Layer::<V800>::new();
    layer.shading_flags = flags;
    assert_eq!(
        Layer::<V800>::decode_mdx(&layer.encode_mdx().unwrap())
            .unwrap()
            .shading_flags,
        flags
    );
    let mut material_flags = MaterialRenderFlags(0);
    material_flags.set_two_sided(true);
    material_flags.set_sort_primitives_near_z(true);
    assert_eq!(material_flags.bits(), 0xa);
}

#[test]
fn constructors_use_documented_defaults_without_changing_decoded_values() {
    let light = Light::<V800>::new(Node::new("Light", 0).unwrap(), 0);
    assert_eq!(light.color, [1.0; 3]);
    assert_eq!(light.ambient_color, [1.0; 3]);
    assert_eq!(
        light.falloff(),
        LightFalloff {
            quadratic: 0.0005,
            linear: 0.0,
            damping: 0.00001
        }
    );
    let mut emitter = PopcornEmitter::new(
        Node::new("Effect", 1).unwrap(),
        "effect.pkfx",
        "Always=on\r\nDeath=off",
    )
    .unwrap();
    assert_eq!(emitter.life_span, 1.0);
    assert_eq!(emitter.emission_rate, 1.0);
    assert_eq!(emitter.speed, 1.0);
    assert_eq!(emitter.alpha, 1.0);
    assert_eq!(emitter.color, [1.0; 3]);
    emitter.life_span = 0.0;
    emitter.color = [0.0; 3];
    let decoded = PopcornEmitter::decode_mdx(&emitter.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded.life_span, 0.0);
    assert_eq!(decoded.color, [0.0; 3]);
    assert_eq!(decoded.visibility_guide.text(), "Always=on\r\nDeath=off");
    assert_eq!(
        ParticleEmitter2::new(Node::new("p", 0).unwrap()).priority_plane,
        0u32
    );
}
