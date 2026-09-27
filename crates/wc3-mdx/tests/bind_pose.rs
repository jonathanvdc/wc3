use wc3_mdx::chunks::{BindPoseChunk, ModelChunk, RawChunk};
use wc3_mdx::geometry::BindPoseMatrix;
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::Model;

#[test]
fn bind_pose_matrices_round_trip() {
    let matrix = [
        1.0f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ];
    let mut pose = BindPoseChunk::new(vec![BindPoseMatrix(matrix)]);
    assert_eq!(pose.records.first().map(|record| record.0), Some(matrix));
    let changed = [2.0; 12];
    pose.records[0] = BindPoseMatrix(changed);
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_bind_pose(&pose);
    let parsed = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    assert_eq!(parsed.bind_poses()[0].records[0].0, changed);
}

#[test]
fn malformed_bind_pose_chunk_is_rejected() {
    let mut model = Model::<wc3_mdx::V1800>::new();
    assert!(ModelChunk::<wc3_mdx::V1800>::from_raw(RawChunk::new(*b"BPOS", vec![0])).is_err());
    assert!(model.bind_poses().is_empty());

    let pose = BindPoseChunk::new(vec![BindPoseMatrix([0.0; 12])]);
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
                let mut model = Model::<wc3_mdx::V1800>::decode(&bytes).unwrap();
                if let Some(pose) = model.bind_poses().first() {
                    model.set_bind_pose(pose);
                    assert_eq!(model.encode().unwrap(), bytes);
                }
            }
        }
    }
}
