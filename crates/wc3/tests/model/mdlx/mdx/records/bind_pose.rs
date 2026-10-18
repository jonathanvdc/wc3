use wc3::model::formats::mdlx::chunks::{BindPoseChunk, ModelChunk, RawChunk};
use wc3::model::formats::mdlx::geometry::BindPoseMatrix;

use wc3::model::formats::mdlx::Document;

#[test]
fn malformed_bind_pose_chunk_is_rejected() {
    let mut model = Document::<wc3::model::formats::mdlx::V1800>::new();
    assert!(
        ModelChunk::<wc3::model::formats::mdlx::V1800>::from_raw(RawChunk::new(*b"BPOS", vec![0]))
            .is_err()
    );
    assert!(model.bind_poses().cloned().collect::<Vec<_>>().is_empty());

    let pose = BindPoseChunk::new(vec![BindPoseMatrix([0.0; 12])]);
    model.set_bind_poses(pose.records.iter().copied());
    assert_eq!(
        model.bind_poses().cloned().collect::<Vec<_>>(),
        pose.records
    );
    assert_eq!(model.chunks.len(), 2);
}
