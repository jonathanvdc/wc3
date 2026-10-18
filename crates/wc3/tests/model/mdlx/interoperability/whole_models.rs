use wc3::model::formats::mdlx::animation::{
    GeosetAnimation, Sequence, SequenceFlags, TextureAnimation, Track, ValueKeyframe,
};
use wc3::model::formats::mdlx::emitters::{
    ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter,
};
use wc3::model::formats::mdlx::geometry::BindPoseMatrix;
use wc3::model::formats::mdlx::geometry::{CollisionShape, Geoset};
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;

use wc3::model::formats::mdlx::materials::{Layer, Material, Texture};
use wc3::model::formats::mdlx::scene::{
    Attachment, Bone, Camera, EventObject, FaceFx, Light, ModelInfo, Node,
};
use wc3::model::formats::mdlx::{
    Document, FormatVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn full_model<'data, V: FormatVersion>() -> Document<'data, V> {
    let version = V::NUMBER;
    let mut model = Document::<V>::new();
    let mut info = ModelInfo::new("Complete Synthetic Document").unwrap();
    info.bounds_radius = 42.5;
    info.minimum_extent = [-1.0, -2.0, -3.0];
    info.maximum_extent = [1.0, 2.0, 3.0];
    info.blend_time = 150;
    model.set_model_info(&info);
    let mut sequence = Sequence::new("Stand", [0, 1000]).unwrap();
    sequence.move_speed = 270.0;
    sequence.flags = SequenceFlags(1);
    sequence.rarity = 0.5;
    sequence.sync_point = 500;
    sequence.extent.bounds_radius = 42.0;
    sequence.extent.minimum = [-2.0, -3.0, -4.0];
    sequence.extent.maximum = [2.0, 3.0, 4.0];
    model.set_sequences([sequence]);
    model.set_global_sequences(&[1000, 2500]);
    let mut texture = Texture::new("Textures\\Sample.blp").unwrap();
    texture.replaceable_id = 1;
    texture.flags.set_wrap_width(true);
    texture.flags.set_wrap_height(true);
    model.set_textures([texture]);
    let mut material = Material::<V>::new();
    material.layers = [Layer::<V>::new()].to_vec();
    model.set_materials([material]);
    let mut animation = TextureAnimation::new();
    animation.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 100,
                value: [1.0, 2.0, 3.0],
            }],
            None,
        )
        .unwrap(),
    );
    model.set_texture_animations([animation]);
    model.set_geosets([Geoset::<V>::new([[0.0, 0.0, 0.0]], [[0.0, 0.0, 1.0]], [0, 0, 0]).unwrap()]);
    model.set_geoset_animations([GeosetAnimation::decode_mdl(
        "GeosetAnim { GeosetId 0, DropShadow, static Alpha 0.5, static Color { 0.1, 0.2, 0.3 }, }",
    )
    .unwrap()]);
    model.set_bones([Bone::new(Node::new("Root", 0).unwrap(), 0, 0)]);
    model.set_helpers([Node::new("Helper", 1).unwrap()]);
    model.set_attachments([Attachment::decode_mdl(
        r#"Attachment "Hand" { ObjectId 2, AttachmentID 2, Path "Objects\Attachment.mdx", Visibility 1 { Linear, 50: 0.5, } }"#
    ).unwrap()]);
    model.set_pivot_points(&[[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]);
    model.set_event_objects([EventObject::new(
        Node::new("Event", 3).unwrap(),
        u32::MAX,
        &[500],
    )]);
    model.set_collision_shapes([CollisionShape::new_box(
        Node::new("Bounds", 4).unwrap(),
        [[-1.0; 3], [1.0; 3]],
    )]);
    model.set_particle_emitters([ParticleEmitter::decode_mdl(
        r#"ParticleEmitter "Emitter" { ObjectId 5, Path "smoke.mdl", static EmissionRate 10, static Gravity -9.8, static Longitude 0.2, static Latitude 0.4, static LifeSpan 2, static InitVelocity 5, Visibility 1 { Linear, 100: 0.5, } }"#
    ).unwrap()]);
    let mut sprites =
        ParticleEmitter2::decode_mdl(include_str!("../../../fixtures/mdl/smoke.mdl")).unwrap();
    sprites.node.object_id = 6;
    model.set_particle_emitters2([sprites]);
    model.set_ribbon_emitters([RibbonEmitter::decode_mdl(
        r#"RibbonEmitter "Ribbon" { ObjectId 7, static HeightAbove 3, Color 1 { Linear, 42: { 1, 0.5, 0.25 }, } MaterialID 9, TextureSlot 1 { Linear, 100: 7, } }"#
    ).unwrap()]);
    model
        .set_cameras([
            Camera::<V>::decode_mdl(include_str!("../../../fixtures/mdl/portrait.mdl")).unwrap(),
        ]);
    model.set_lights([Light::<V>::decode_mdl(
        r#"Light "Light" { ObjectId 8, Directional, static AttenuationStart 100, static AttenuationEnd 500, Color 1 { Linear, 250: { 1, 0.5, 0.25 }, } static Intensity 2, static AmbColor { 0.1, 0.2, 0.3 }, static AmbIntensity 0.5, }"#
    ).unwrap()]);
    if version >= 900 {
        model
            .try_set_face_fx([FaceFx::new("Face", "Textures\\Face.blp").unwrap()])
            .unwrap();
        model
            .try_set_bind_poses([BindPoseMatrix([2.0; 12])])
            .unwrap();
        let mut popcorn =
            PopcornEmitter::decode_mdl(include_str!("../../../fixtures/mdl/popcorn_fire.mdl"))
                .unwrap();
        popcorn.node.object_id = 9;
        model.try_set_popcorn_emitters([popcorn]).unwrap();
    }
    model
}

#[test]
fn all_chunk_families_validate_and_round_trip_across_versions() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1000>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1300>();
    check_version::<V1400>();
    check_version::<V1600>();
    check_version::<V1800>();
}

fn check_version<V: FormatVersion>() {
    let version = V::NUMBER;
    let model = full_model::<V>();
    let bytes = model.encode_mdx().unwrap();
    let parsed = Document::<V>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.encode_mdx().unwrap(), bytes, "version {version}");
    assert_eq!(parsed.version(), version);
    assert_eq!(parsed.model_info(), model.model_info());
    assert_eq!(parsed.global_sequences(), model.global_sequences());
    assert_eq!(parsed.pivot_points(), model.pivot_points());
    macro_rules! same_records {
        ($($accessor:ident),+ $(,)?) => {
            $(assert_eq!(
                parsed.$accessor().collect::<Vec<_>>(),
                model.$accessor().collect::<Vec<_>>(),
                "{} at version {version}", stringify!($accessor)
            );)+
        };
    }
    same_records!(
        sequences,
        textures,
        texture_animations,
        materials,
        geosets,
        geoset_animations,
        bones,
        helpers,
        attachments,
        event_objects,
        collision_shapes,
        particle_emitters,
        particle_emitters2,
        ribbon_emitters,
        cameras,
        lights
    );
    if version >= 900 {
        assert_eq!(
            parsed.try_bind_poses().unwrap().collect::<Vec<_>>(),
            model.try_bind_poses().unwrap().collect::<Vec<_>>()
        );
        assert_eq!(
            parsed.try_popcorn_emitters().unwrap().collect::<Vec<_>>(),
            model.try_popcorn_emitters().unwrap().collect::<Vec<_>>()
        );
        assert_eq!(
            parsed.try_face_fx().unwrap().collect::<Vec<_>>(),
            model.try_face_fx().unwrap().collect::<Vec<_>>()
        );
    }
}
