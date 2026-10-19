use native::geometry::BindPoseMatrix;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::scene::{Camera, Node};
use wc3::model::formats::mdlx::{self as native};
use wc3::model::io;

fn matrix(translation: [f32; 3]) -> BindPoseMatrix {
    BindPoseMatrix([
        1.,
        0.,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        1.,
        translation[0],
        translation[1],
        translation[2],
    ])
}

#[test]
fn camera_bind_poses_follow_hierarchy_records_and_roundtrip_without_loss() {
    for hierarchy in [false, true] {
        let mut document = native::Document::<native::V1800>::new();
        let mut poses = Vec::new();
        if hierarchy {
            document.set_helpers([Node::new("helper", 0).unwrap()]);
            document.set_pivot_points(&[[0.; 3]]);
            poses.push(matrix([1., 2., 3.]));
        }
        let positions = [[750., 0., 100.], [20., 30., 40.]];
        document.set_cameras(positions.into_iter().enumerate().map(|(index, position)| {
            let mut camera = Camera::new(&format!("camera{index}")).unwrap();
            camera.position = position;
            camera.field_of_view = 0.76;
            camera.near_clip = 10.;
            camera.far_clip = 10000.;
            camera
        }));
        poses.extend(positions.map(matrix));
        document.set_bind_poses(poses.clone());
        let model = io::mdlx::import_owned(&document, &ImportOptions::default())
            .unwrap()
            .value;
        for (index, (_, node)) in model.nodes().iter().enumerate() {
            assert_eq!(node.properties.bind_pose, Some(poses[index].0));
        }
        let output = io::mdlx::plan::<native::V1800>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        assert_eq!(output.bind_poses().copied().collect::<Vec<_>>(), poses);
        assert_eq!(output.nodes().len(), usize::from(hierarchy));
        let restored = io::mdlx::import_owned(&output, &ImportOptions::default())
            .unwrap()
            .value;
        assert_eq!(
            restored
                .nodes()
                .iter()
                .map(|(_, node)| node.properties.bind_pose)
                .collect::<Vec<_>>(),
            model
                .nodes()
                .iter()
                .map(|(_, node)| node.properties.bind_pose)
                .collect::<Vec<_>>()
        );
        let repeated = io::mdlx::plan::<native::V1800>(&restored, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        assert_eq!(repeated.nodes().len(), usize::from(hierarchy));
        assert_eq!(repeated.bind_poses().copied().collect::<Vec<_>>(), poses);
        let mut missing_pose = model.clone();
        let last_camera = missing_pose.nodes().iter().last().unwrap().0;
        missing_pose
            .edit(|builder| {
                let mut node = builder.model().node(last_camera).unwrap().clone();
                node.properties.bind_pose = None;
                builder.set_node(last_camera, node)
            })
            .unwrap();
        let output = io::mdlx::plan::<native::V1800>(&missing_pose, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        assert_eq!(output.bind_poses().last(), Some(&matrix([0.; 3])));
        poses.push(matrix([9.; 3]));
        document.set_bind_poses(poses);
        assert!(io::mdlx::import_owned(&document, &ImportOptions::default())
            .unwrap_err()
            .to_string()
            .contains("pose has no associated hierarchy node or camera"));
    }
}
