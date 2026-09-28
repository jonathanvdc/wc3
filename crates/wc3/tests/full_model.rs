use wc3::model::animation::{GeosetAnimation, Sequence, TextureAnimation};
use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter};
use wc3::model::geometry::BindPoseMatrix;
use wc3::model::geometry::{CollisionShape, Geoset};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::materials::{Layer, Material, Texture};
use wc3::model::scene::{Attachment, Bone, Camera, EventObject, FaceFx, Light, ModelInfo, Node};
use wc3::model::{
    Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn full_model<V: ModelVersion>() -> Model<V> {
    let version = V::NUMBER;
    let mut model = Model::<V>::new();
    model.set_model_info(&ModelInfo::new("Complete Synthetic Model").unwrap());
    model.set_sequences(&[Sequence::new("Stand", [0, 1000]).unwrap()]);
    model.set_global_sequences(&[1000]);
    model.set_textures(&[Texture::new("Textures\\Sample.blp").unwrap()]);
    let mut material = Material::<V>::new();
    material.layers = (&[Layer::<V>::new()]).to_vec();
    model.set_materials(&[material]);
    model.set_texture_animations(&[TextureAnimation::new()]);
    model.set_geosets(&[
        Geoset::<V>::new(&[[0.0, 0.0, 0.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap(),
    ]);
    model.set_geoset_animations(&[GeosetAnimation::new(0)]);
    model.set_bones(&[Bone::new(Node::new("Root", 0).unwrap(), 0, 0)]);
    model.set_helpers(&[Node::new("Helper", 1).unwrap()]);
    model.set_attachments(&[Attachment::new(
        Node::new("Hand", 2).unwrap(),
        "Objects\\Attachment.mdx",
        0,
    )
    .unwrap()]);
    model.set_pivot_points(&[[0.0, 0.0, 0.0]; 3]);
    model.set_event_objects(&[EventObject::new(
        Node::new("Event", 3).unwrap(),
        u32::MAX,
        &[500],
    )]);
    model.set_collision_shapes(&[CollisionShape::new_box(
        Node::new("Bounds", 4).unwrap(),
        [[-1.0; 3], [1.0; 3]],
    )]);
    model.set_particle_emitters(&[ParticleEmitter::new(
        Node::new("Emitter", 5).unwrap(),
        "Textures\\Particle.blp",
    )
    .unwrap()]);
    model.set_particle_emitters2(&[ParticleEmitter2::new(Node::new("Emitter2", 6).unwrap())]);
    model.set_ribbon_emitters(&[RibbonEmitter::new(Node::new("Ribbon", 7).unwrap())]);
    model.set_cameras(&[Camera::<V>::new("Camera").unwrap()]);
    model.set_lights(&[Light::<V>::new(Node::new("Light", 8).unwrap(), 0)]);
    if version >= 900 {
        model
            .try_set_face_fx(&[FaceFx::new("Face", "Textures\\Face.blp").unwrap()])
            .unwrap();
        model
            .try_set_bind_poses(&[BindPoseMatrix([0.0; 12])])
            .unwrap();
        model
            .try_set_popcorn_emitters(&[PopcornEmitter::new(
                Node::new("Popcorn", 9).unwrap(),
                "Objects\\Effect.pkfx",
                "Stand",
            )
            .unwrap()])
            .unwrap();
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

fn check_version<V: ModelVersion>() {
    let version = V::NUMBER;
    let model = full_model::<V>();
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.encode_mdx().unwrap(), bytes, "version {version}");
    assert_eq!(parsed.version(), version);
    assert_eq!(parsed.materials()[0].layers.len(), 1);
}
