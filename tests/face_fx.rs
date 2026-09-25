use wc3_mdx::{FaceFx, Model};

#[test]
fn face_animation_references_round_trip() {
    let entry = FaceFx::new("Talk", "FaceFX\\Footman.fafx").unwrap();
    let mut model = Model::new(1800);
    model.set_face_fx(&[entry]).unwrap();
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let entry = &parsed.face_fx().unwrap()[0];
    assert_eq!(entry.name(), "Talk");
    assert_eq!(entry.path(), "FaceFX\\Footman.fafx");
}
