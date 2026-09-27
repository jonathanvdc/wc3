use wc3_mdx::geometry::CollisionShape;
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::scene::Node;
use wc3_mdx::Model;

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
    let mut model = Model::<wc3_mdx::V800>::new();
    model.set_collision_shapes(&[box_shape, sphere, plane, cylinder]);
    let decoded = Model::<wc3_mdx::V800>::decode(&model.encode().unwrap()).unwrap();
    let shapes = decoded.collision_shapes();
    assert_eq!(shapes[0].node().name(), "Box");
    assert_eq!(
        shapes[0].box_corners(),
        Some([[-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]])
    );
    assert_eq!(shapes[1].sphere(), Some(([1.0, 2.0, 3.0], 4.0)));
    assert_eq!(shapes[2].points(), Some([[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]));
    assert_eq!(shapes[3].radius(), Some(0.5));
    assert_eq!(shapes[3].points(), Some([[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]]));
}
