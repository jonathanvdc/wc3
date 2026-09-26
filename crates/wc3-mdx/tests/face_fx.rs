use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::scene::FaceFx;
use wc3_mdx::Model;

#[test]
fn face_animation_references_round_trip() {
    let entry = FaceFx::new("Talk", "FaceFX\\Footman.fafx").unwrap();
    let mut model = Model::new(1800);
    model.set_face_fx(&[entry]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let entry = &parsed.face_fx()[0];
    assert_eq!(entry.name(), "Talk");
    assert_eq!(entry.path(), "FaceFX\\Footman.fafx");
}
