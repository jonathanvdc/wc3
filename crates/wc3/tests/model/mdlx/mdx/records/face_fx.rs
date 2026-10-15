use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::scene::FaceFx;
use wc3::model::formats::mdlx::Document;

#[test]
fn face_animation_references_round_trip() {
    let entry = FaceFx::new("Talk", "FaceFX\\Footman.fafx").unwrap();
    let mut model = Document::<wc3::model::formats::mdlx::V1800>::new();
    model.set_face_fx([entry]);
    let parsed =
        Document::<wc3::model::formats::mdlx::V1800>::decode_mdx(&model.encode_mdx().unwrap())
            .unwrap()
            .into_owned();
    let entry = &parsed.face_fx().next().unwrap();
    assert_eq!(entry.name.text(), "Talk");
    assert_eq!(entry.path.text(), "FaceFX\\Footman.fafx");
}
