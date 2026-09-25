use wc3_mdx::Record;
use wc3_mdx::{CollisionKind, CollisionShape, Model, Node};

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
    let mut model = Model::new(800);
    model.set_collision_shapes(&[box_shape, sphere, plane, cylinder]);
    let decoded = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let shapes = decoded.collision_shapes().unwrap();
    assert_eq!(shapes[0].node().name(), "Box");
    assert_eq!(shapes[0].kind(), CollisionKind::Box);
    assert_eq!(
        shapes[0].box_corners(),
        Some([[-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]])
    );
    assert_eq!(shapes[1].kind(), CollisionKind::Sphere);
    assert_eq!(shapes[1].sphere(), Some(([1.0, 2.0, 3.0], 4.0)));
    assert_eq!(shapes[2].kind(), CollisionKind::Plane);
    assert_eq!(shapes[2].points(), Some([[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]));
    assert_eq!(shapes[3].kind(), CollisionKind::Cylinder);
    assert_eq!(shapes[3].radius(), Some(0.5));
    assert_eq!(shapes[3].points(), Some([[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]]));
}
