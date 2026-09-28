use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::scene::FaceFx;
use wc3::model::Model;

#[test]
fn face_animation_references_round_trip() {
    let entry = FaceFx::new("Talk", "FaceFX\\Footman.fafx").unwrap();
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_face_fx(&[entry]);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let entry = &parsed.face_fx()[0];
    assert_eq!(entry.name(), "Talk");
    assert_eq!(entry.path(), "FaceFX\\Footman.fafx");
}
