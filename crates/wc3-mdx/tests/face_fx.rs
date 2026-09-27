use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::scene::FaceFx;
use wc3_mdx::Model;

#[test]
fn face_animation_references_round_trip() {
    let entry = FaceFx::new("Talk", "FaceFX\\Footman.fafx").unwrap();
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_face_fx(&[entry]);
    let parsed = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    let entry = &parsed.face_fx()[0];
    assert_eq!(entry.name(), "Talk");
    assert_eq!(entry.path(), "FaceFX\\Footman.fafx");
}
