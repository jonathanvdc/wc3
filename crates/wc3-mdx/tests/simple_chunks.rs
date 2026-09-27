use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::Model;

#[test]
fn global_sequences_and_pivots_round_trip() {
    let mut model = Model::<wc3_mdx::V1100>::new();
    model.set_global_sequences(&[1000, 2500]);
    model.set_pivot_points(&[[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]);
    let parsed = Model::<wc3_mdx::V1100>::decode(&model.encode().unwrap()).unwrap();
    assert_eq!(parsed.global_sequences(), vec![1000, 2500]);
    assert_eq!(
        parsed.pivot_points(),
        vec![[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]
    );
}
