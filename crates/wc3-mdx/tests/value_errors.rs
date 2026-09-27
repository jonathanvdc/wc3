use wc3_mdx::geometry::Geoset;
use wc3_mdx::io::{Encodable, EncodeError, ValueError};
use wc3_mdx::scene::Node;

#[test]
fn construction_and_mutation_report_value_errors() {
    let bad_name: Result<Node, ValueError> = Node::new("bad\0name", 1);
    assert_eq!(
        bad_name.unwrap_err(),
        ValueError::InvalidString { max_bytes: 79 }
    );

    let mut geoset = Geoset::<wc3_mdx::V800>::new(&[], &[], &[]).unwrap();
    assert_eq!(
        geoset.set_vertex(0, [0.0; 3]),
        Err(ValueError::IndexOutOfBounds {
            tag: *b"GEOS",
            index: 0,
            len: 0,
        })
    );

    let node = Node::new("Valid", 1).unwrap();
    let encoded: Result<Vec<u8>, EncodeError> = node.encode();
    assert!(encoded.is_ok());
}
