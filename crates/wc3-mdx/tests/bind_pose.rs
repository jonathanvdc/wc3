use wc3_mdx::chunks::{BindPose, ModelChunk, RawChunk};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::Model;

#[test]
fn bind_pose_matrices_round_trip() {
    let matrix = [
        1.0f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ];
    let mut pose = BindPose::new(&[matrix]);
    assert_eq!(pose.matrix(0), Some(matrix));
    let changed = [2.0; 12];
    assert!(pose.set_matrix(0, changed));
    let mut model = Model::new(1800);
    model.set_bind_pose(&pose);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.bind_poses()[0].matrix(0), Some(changed));
}

#[test]
fn bind_poses_skip_malformed_chunks_and_setter_replaces_them() {
    let mut model = Model::new(1800);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"BPOS", vec![0]), 1800));
    assert!(model.bind_poses().is_empty());

    let pose = BindPose::new(&[[0.0; 12]]);
    model.set_bind_pose(&pose);
    assert_eq!(model.bind_poses(), vec![pose]);
    assert_eq!(model.chunks().len(), 2);
}

#[test]
fn local_bind_poses_round_trip_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                let mut model = Model::decode(&bytes, 800).unwrap();
                if let Some(pose) = model.bind_poses().first() {
                    model.set_bind_pose(pose);
                    assert_eq!(model.encode().unwrap(), bytes);
                }
            }
        }
    }
}
