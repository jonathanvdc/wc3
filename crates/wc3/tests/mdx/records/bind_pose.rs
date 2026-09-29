use wc3::model::chunks::{BindPoseChunk, ModelChunk, RawChunk};
use wc3::model::geometry::BindPoseMatrix;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::Model;

#[test]
fn bind_pose_matrices_round_trip() {
    let matrix = [
        1.0f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ];
    let mut pose = BindPoseChunk::new(vec![BindPoseMatrix(matrix)]);
    assert_eq!(pose.records.first().map(|record| record.0), Some(matrix));
    let changed = [2.0; 12];
    pose.records[0] = BindPoseMatrix(changed);
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_bind_poses(&pose.records);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.bind_poses()[0].0, changed);
}

#[test]
fn malformed_bind_pose_chunk_is_rejected() {
    let mut model = Model::<wc3::model::V1800>::new();
    assert!(ModelChunk::<wc3::model::V1800>::from_raw(RawChunk::new(*b"BPOS", vec![0])).is_err());
    assert!(model.bind_poses().is_empty());

    let pose = BindPoseChunk::new(vec![BindPoseMatrix([0.0; 12])]);
    model.set_bind_poses(&pose.records);
    assert_eq!(model.bind_poses(), pose.records);
    assert_eq!(model.chunks.len(), 2);
}
