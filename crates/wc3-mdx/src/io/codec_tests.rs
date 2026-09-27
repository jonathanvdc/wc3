#[cfg(test)]
mod tests {
    use crate::Cursor;
    use crate::KnownChunk;
    use crate::{
        AttachmentsChunk, BindPoseChunk, Bone, BonesChunk, Camera, CamerasChunk,
        CollisionShapesChunk, DecodeError, EventObjectsChunk, FaceFxChunk, Geoset,
        GeosetAnimationsChunk, GeosetsChunk, GlobalSequencesChunk, HelpersChunk, LightsChunk,
        MaterialsChunk, Model, ModelInfoChunk, Node, ParticleEmitter, ParticleEmitter2,
        ParticleEmitters2Chunk, ParticleEmittersChunk, PivotPointsChunk, PopcornEmittersChunk,
        RibbonEmitter, RibbonEmittersChunk, Sequence, SequencesChunk, TextureAnimationsChunk,
        TexturesChunk, VersionChunk,
    };
    use crate::{Readable, Writable};

    fn round_trip<T: Readable + PartialEq + std::fmt::Debug>(value: &T)
    where
        T: Writable,
    {
        let bytes = value.encode().unwrap();
        let mut appended = vec![0xaa, 0xbb];
        crate::Encoder::new(&mut appended).write(value).unwrap();
        assert_eq!(&appended[..2], &[0xaa, 0xbb]);
        assert_eq!(&appended[2..], bytes);
        assert_eq!(T::decode(&bytes).unwrap(), *value);
    }

    #[test]
    fn one_api_handles_model_fixed_and_versioned_records() {
        let model = Model::<crate::V800>::new();
        let bytes = model.encode().unwrap();
        assert_eq!(
            Model::<crate::V800>::decode(&bytes)
                .unwrap()
                .encode()
                .unwrap(),
            bytes
        );
        round_trip(&Sequence::new("Stand", [0, 100]).unwrap());
        round_trip(&Geoset::<crate::V1800>::new(&[], &[], &[]).unwrap());
        assert!(Sequence::decode(&[0; 131]).is_err());
    }

    #[test]
    fn cursor_read_advances_through_records_and_decode_rejects_trailing_bytes() {
        let first = Sequence::new("Stand", [0, 100]).unwrap();
        let second = Sequence::new("Walk", [101, 200]).unwrap();
        let mut bytes = first.encode().unwrap();
        bytes.extend_from_slice(&second.encode().unwrap());

        let mut cursor = Cursor::new(&bytes);
        let decoded: Sequence = cursor.read().unwrap();
        assert_eq!(decoded, first);
        let consumed = cursor.position();
        assert_eq!(consumed, first.encode().unwrap().len());
        assert_eq!(
            Sequence::decode(&bytes),
            Err(DecodeError::TrailingRecordBytes {
                consumed,
                total: bytes.len(),
            })
        );

        let first = Geoset::<crate::V800>::new(&[], &[], &[]).unwrap();
        let second = Geoset::<crate::V800>::new(&[], &[], &[]).unwrap();
        let mut bytes = first.encode().unwrap();
        bytes.extend_from_slice(&second.encode().unwrap());
        let mut cursor = Cursor::new(&bytes);
        let decoded: Geoset<crate::V800> = cursor.read().unwrap();
        assert_eq!(decoded, first);
        assert_eq!(cursor.position(), first.encode().unwrap().len());
        assert!(matches!(
            Geoset::<crate::V800>::decode(&bytes),
            Err(DecodeError::TrailingRecordBytes { .. })
        ));
    }

    #[test]
    fn cursor_decoders_stop_at_the_next_record() {
        fn check<T: Readable + PartialEq + std::fmt::Debug>(first: T, second: T)
        where
            T: Writable,
        {
            let mut bytes = first.encode().unwrap();
            let first_len = bytes.len();
            bytes.extend_from_slice(&second.encode().unwrap());
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
            Camera::<crate::V800>::new("First").unwrap(),
            Camera::<crate::V800>::new("Second").unwrap(),
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
            VersionChunk::<crate::V800>::TAG,
            ModelInfoChunk::TAG,
            SequencesChunk::TAG,
            GlobalSequencesChunk::TAG,
            TexturesChunk::TAG,
            MaterialsChunk::<crate::V800>::TAG,
            GeosetsChunk::<crate::V800>::TAG,
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
            CamerasChunk::<crate::V800>::TAG,
            LightsChunk::<crate::V800>::TAG,
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
}
