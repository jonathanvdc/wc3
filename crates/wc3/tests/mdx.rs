use std::fmt::Debug;

use wc3::model::animation::Sequence;
use wc3::model::chunks::{
    AttachmentsChunk, BindPoseChunk, BonesChunk, CamerasChunk, CollisionShapesChunk,
    EventObjectsChunk, FaceFxChunk, GeosetAnimationsChunk, GeosetsChunk, GlobalSequencesChunk,
    HelpersChunk, KnownChunk, LightsChunk, MaterialsChunk, ModelInfoChunk, ParticleEmitters2Chunk,
    ParticleEmittersChunk, PivotPointsChunk, PopcornEmittersChunk, RibbonEmittersChunk,
    SequencesChunk, TextureAnimationsChunk, TexturesChunk, VersionChunk,
};
use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, RibbonEmitter};
use wc3::model::geometry::Geoset;
use wc3::model::mdx::{self, Cursor, Encoder, Read as _, Write as _};
use wc3::model::scene::{Bone, Camera, Node};
use wc3::model::{Model, V1800, V800};

fn round_trip<T>(value: &T)
where
    T: mdx::Read + mdx::Write + PartialEq + Debug,
{
    let bytes = value.encode_mdx().unwrap();
    let mut appended = vec![0xaa, 0xbb];
    Encoder::new(&mut appended).write(value).unwrap();
    assert_eq!(&appended[..2], &[0xaa, 0xbb]);
    assert_eq!(&appended[2..], bytes);
    assert_eq!(T::decode_mdx(&bytes).unwrap(), *value);
}

#[test]
fn one_api_handles_model_fixed_and_versioned_records() {
    let model = Model::<V800>::new();
    let bytes = model.encode_mdx().unwrap();
    assert_eq!(
        Model::<V800>::decode_mdx(&bytes)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        bytes
    );
    round_trip(&Sequence::new("Stand", [0, 100]).unwrap());
    round_trip(&Geoset::<V1800>::new(&[], &[], &[]).unwrap());
    assert!(Sequence::decode_mdx(&[0; 131]).is_err());
}

#[test]
fn cursor_read_advances_through_records_and_decode_rejects_trailing_bytes() {
    let first = Sequence::new("Stand", [0, 100]).unwrap();
    let second = Sequence::new("Walk", [101, 200]).unwrap();
    let mut bytes = first.encode_mdx().unwrap();
    bytes.extend_from_slice(&second.encode_mdx().unwrap());

    let mut cursor = Cursor::new(&bytes);
    let decoded: Sequence = cursor.read().unwrap();
    assert_eq!(decoded, first);
    let consumed = cursor.position();
    assert_eq!(consumed, first.encode_mdx().unwrap().len());
    assert_eq!(
        Sequence::decode_mdx(&bytes),
        Err(mdx::ReadError::TrailingRecordBytes {
            consumed,
            total: bytes.len(),
        })
    );

    let first = Geoset::<V800>::new(&[], &[], &[]).unwrap();
    let second = Geoset::<V800>::new(&[], &[], &[]).unwrap();
    let mut bytes = first.encode_mdx().unwrap();
    bytes.extend_from_slice(&second.encode_mdx().unwrap());
    let mut cursor = Cursor::new(&bytes);
    let decoded: Geoset<V800> = cursor.read().unwrap();
    assert_eq!(decoded, first);
    assert_eq!(cursor.position(), first.encode_mdx().unwrap().len());
    assert!(matches!(
        Geoset::<V800>::decode_mdx(&bytes),
        Err(mdx::ReadError::TrailingRecordBytes { .. })
    ));
}

#[test]
fn cursor_decoders_stop_at_the_next_record() {
    fn check<T>(first: T, second: T)
    where
        T: mdx::Read + mdx::Write + PartialEq + Debug,
    {
        let mut bytes = first.encode_mdx().unwrap();
        let first_len = bytes.len();
        bytes.extend_from_slice(&second.encode_mdx().unwrap());
        let mut cursor = Cursor::new(&bytes);
        assert_eq!(cursor.read::<T>().unwrap(), first);
        assert_eq!(cursor.position(), first_len);
        assert_eq!(cursor.read::<T>().unwrap(), second);
        cursor.finish().unwrap();
    }

    let first = Node::new("First", 1).unwrap();
    let second = Node::new("Second", 2).unwrap();
    check(first.clone(), second.clone());
    check(
        Bone::new(first.clone(), 1, 2),
        Bone::new(second.clone(), 3, 4),
    );
    check(
        Camera::<V800>::new("First").unwrap(),
        Camera::<V800>::new("Second").unwrap(),
    );
    check(
        ParticleEmitter::new(first.clone(), "first.mdx").unwrap(),
        ParticleEmitter::new(second.clone(), "second.mdx").unwrap(),
    );
    check(
        ParticleEmitter2::new(first.clone()),
        ParticleEmitter2::new(second.clone()),
    );
    check(RibbonEmitter::new(first), RibbonEmitter::new(second));
}

#[test]
fn known_top_level_chunks_have_distinct_chunk_records() {
    let tags = [
        VersionChunk::<V800>::TAG,
        ModelInfoChunk::TAG,
        SequencesChunk::TAG,
        GlobalSequencesChunk::TAG,
        TexturesChunk::TAG,
        MaterialsChunk::<V800>::TAG,
        GeosetsChunk::<V800>::TAG,
        GeosetAnimationsChunk::TAG,
        BonesChunk::TAG,
        HelpersChunk::TAG,
        AttachmentsChunk::TAG,
        EventObjectsChunk::TAG,
        CollisionShapesChunk::TAG,
        ParticleEmittersChunk::TAG,
        ParticleEmitters2Chunk::TAG,
        RibbonEmittersChunk::TAG,
        PopcornEmittersChunk::TAG,
        CamerasChunk::<V800>::TAG,
        LightsChunk::<V800>::TAG,
        TextureAnimationsChunk::TAG,
        FaceFxChunk::TAG,
        PivotPointsChunk::TAG,
        BindPoseChunk::TAG,
    ];
    let mut unique = tags.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), tags.len());
}
