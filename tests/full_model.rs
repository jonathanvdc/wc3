use wc3_mdx::{Decodable, Encodable};
use wc3_mdx::{
    Attachment, BindPose, Bone, Camera, CollisionShape, EventObject, FaceFx, Geoset,
    GeosetAnimation, Layer, Light, Material, Model, ModelInfo, Node, ParticleEmitter,
    ParticleEmitter2, PopcornEmitter, RibbonEmitter, Sequence, Texture, TextureAnimation,
};

fn full_model(version: u32) -> Model {
    let mut model = Model::new(version);
    model.set_model_info(&ModelInfo::new("Complete Synthetic Model").unwrap());
    model.set_sequences(&[Sequence::new("Stand", [0, 1000]).unwrap()]);
    model.set_global_sequences(&[1000]);
    model.set_textures(&[Texture::new("Textures\\Sample.blp").unwrap()]);
    let mut material = Material::new(version);
    material.set_layers(&[Layer::new(version)]).unwrap();
    model.set_materials(&[material]).unwrap();
    model.set_texture_animations(&[TextureAnimation::new()]);
    model
        .set_geosets(&[
            Geoset::new(version, &[[0.0, 0.0, 0.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap(),
        ])
        .unwrap();
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
    model.set_cameras(&[Camera::new_for_version("Camera", version).unwrap()]);
    model.set_lights(&[Light::new_for_version(
        Node::new("Light", 8).unwrap(),
        0,
        version,
    )]);
    if version >= 900 {
        model.set_face_fx(&[FaceFx::new("Face", "Textures\\Face.blp").unwrap()]);
        model.set_bind_pose(&BindPose::new(&[[0.0; 12]]));
        model.set_popcorn_emitters(&[PopcornEmitter::new(
            Node::new("Popcorn", 9).unwrap(),
            "Objects\\Effect.pkfx",
            "Stand",
        )
        .unwrap()]);
    }
    model
}

#[test]
fn all_chunk_families_validate_and_round_trip_across_versions() {
    for version in [800, 900, 1000, 1100, 1200, 1800] {
        let model = full_model(version);
        model.validate().unwrap();
        let bytes = model.encode().unwrap();
        let parsed = Model::decode(&bytes, 800).unwrap();
        parsed.validate().unwrap();
        assert_eq!(parsed.encode().unwrap(), bytes, "version {version}");
        assert_eq!(parsed.version(), version);
        assert_eq!(parsed.materials()[0].layers().len(), 1);
    }
}
