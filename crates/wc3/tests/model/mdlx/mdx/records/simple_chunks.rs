use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::Document;

#[test]
fn global_sequences_and_pivots_round_trip() {
    let mut model = Document::<wc3::model::formats::mdlx::V1100>::new();
    model.set_global_sequences(&[1000, 2500]);
    model.set_pivot_points(&[[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]);
    let parsed =
        Document::<wc3::model::formats::mdlx::V1100>::decode_mdx(&model.encode_mdx().unwrap())
            .unwrap()
            .into_owned();
    assert_eq!(parsed.global_sequences(), vec![1000, 2500]);
    assert_eq!(
        parsed.pivot_points(),
        vec![[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]
    );
}
