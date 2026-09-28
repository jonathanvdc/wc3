use wc3::model::geometry::{CollisionGeometry, CollisionShape};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn collision_primitives_round_trip() {
    let box_shape = CollisionShape::new_box(
        Node::new("Box", 1).unwrap(),
        [[-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]],
    );
    let sphere = CollisionShape::new_sphere(Node::new("Sphere", 2).unwrap(), [1.0, 2.0, 3.0], 4.0);
    let plane = CollisionShape::new_plane(
        Node::new("Plane", 3).unwrap(),
        [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
    );
    let cylinder = CollisionShape::new_cylinder(
        Node::new("Cylinder", 4).unwrap(),
        [[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
        0.5,
    );
    let mut model = Model::<wc3::model::V800>::new();
    model.set_collision_shapes(&[box_shape, sphere, plane, cylinder]);
    let decoded = Model::<wc3::model::V800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let shapes = decoded.collision_shapes();
    assert_eq!(shapes[0].node.name.text(), "Box");
    assert_eq!(
        shapes[0].box_corners(),
        Some([[-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]])
    );
    assert_eq!(shapes[1].sphere(), Some(([1.0, 2.0, 3.0], 4.0)));
    assert_eq!(shapes[2].points(), Some([[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]));
    assert_eq!(shapes[3].radius(), Some(0.5));
    assert_eq!(shapes[3].points(), Some([[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]]));
}

#[test]
fn edited_collision_geometry_round_trips() {
    let mut shape = CollisionShape::new_box(Node::new("Shape", 1).unwrap(), [[0.0; 3]; 2]);
    shape.geometry = CollisionGeometry::Cylinder([[0.0; 3], [0.0, 0.0, 2.0]], 0.5);
    shape.node.name.set_text("Cylinder").unwrap();
    let decoded = CollisionShape::decode_mdx(&shape.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded, shape);
}
