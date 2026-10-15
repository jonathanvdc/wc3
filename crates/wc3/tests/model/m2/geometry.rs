//! Modern geometry wire fixtures follow the documented stream format codes,
//! independently of the production encoder. Undocumented metadata stays opaque.
use wc3::model::common::IntoZeroVec;
use wc3::model::formats::m2::companions::{
    GeometryChunk, GeometryFile, StreamValues, VertexStream,
};
use wc3::model::formats::m2::resources::FileKey;
use wc3::model::formats::m2::{Bundle, Chunked, File, ProfileRef, V274};

fn frame(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.to_vec();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
fn stream(tag: [u8; 4], format: [u8; 4], values: &[u8]) -> Vec<u8> {
    let mut payload = format.to_vec();
    payload.extend_from_slice(values);
    frame(tag, &payload)
}
fn positions() -> Vec<u8> {
    [1f32, 2.0, 3.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect()
}

#[test]
fn documented_formats_borrow_unaligned_elements_and_round_trip_exactly() {
    let mut bytes = frame(*b"M3VR", &[1, 2, 3]);
    bytes.extend(stream(*b"VPOS", *b"3F32", &positions()));
    bytes.extend(stream(*b"VINX", *b"1U16", &[0, 0, 1, 0]));
    let uv: Vec<_> = [0f32, 0.25]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    bytes.extend(stream(*b"VUV5", *b"2F32", &uv));
    bytes.extend(stream(*b"VCL0", *b"4U8N", &[20, 30, 40, 255]));
    let geometry = GeometryFile::read(&bytes).unwrap();
    let GeometryChunk::Stream { tag, stream } = &geometry.chunks[1] else {
        panic!("position stream")
    };
    assert_eq!(*tag, *b"VPOS");
    let StreamValues::Float3(values) = &stream.values else {
        panic!("float triples")
    };
    assert_eq!(values.get(0), Some([1.0, 2.0, 3.0]));
    assert_eq!(values.as_bytes().as_ptr(), bytes[23..].as_ptr());
    let GeometryChunk::Stream { stream, .. } = &geometry.chunks[2] else {
        panic!("index stream")
    };
    let StreamValues::U16(values) = &stream.values else {
        panic!("unsigned indices")
    };
    assert_eq!(values.iter().collect::<Vec<_>>(), [0, 1]);
    let GeometryChunk::Stream { stream, .. } = &geometry.chunks[3] else {
        panic!("UV stream")
    };
    let StreamValues::Float2(values) = &stream.values else {
        panic!("float pairs")
    };
    assert_eq!(values.get(0), Some([0.0, 0.25]));
    let GeometryChunk::Stream { stream, .. } = &geometry.chunks[4] else {
        panic!("color stream")
    };
    let StreamValues::Normalized4(values) = &stream.values else {
        panic!("normalized byte tuple")
    };
    assert_eq!(values.get(0), Some([20, 30, 40, 255]));
    assert_eq!(geometry.write().unwrap(), bytes);
}

#[test]
fn unknown_formats_structural_chunks_collision_bytes_and_duplicates_are_retained() {
    let mut bytes = stream(*b"VTAN", *b"PACK", &[5, 6, 7]);
    bytes.extend(frame(*b"VGEO", &[8, 9]));
    bytes.extend(frame(*b"CPOS", b"3F32unverified collision layout"));
    bytes.extend(frame(*b"TEST", &[1, 2, 3, 4, 5]));
    bytes.extend(stream(*b"VPOS", *b"3F32", &positions()));
    bytes.extend(stream(*b"VPOS", *b"3F32", &positions()));
    let geometry = GeometryFile::read(&bytes).unwrap();
    let GeometryChunk::Stream { stream, .. } = &geometry.chunks[0] else {
        panic!("tangent stream")
    };
    assert_eq!(stream.format, *b"PACK");
    assert!(matches!(stream.values, StreamValues::Raw(_)));
    assert!(matches!(
        geometry.chunks[2],
        GeometryChunk::CollisionPositions(_)
    ));
    assert_eq!(
        geometry
            .chunks
            .iter()
            .map(GeometryChunk::tag)
            .collect::<Vec<_>>(),
        [*b"VTAN", *b"VGEO", *b"CPOS", *b"TEST", *b"VPOS", *b"VPOS"]
    );
    assert_eq!(geometry.write().unwrap(), bytes);
    assert_eq!(geometry.into_owned().write().unwrap(), bytes);
}

#[test]
fn malformed_stream_widths_headers_and_format_edits_are_rejected() {
    for (format, values) in [
        (*b"1U16", vec![1]),
        (*b"2F32", vec![0; 7]),
        (*b"3F32", vec![0; 11]),
        (*b"4U8N", vec![0; 3]),
    ] {
        assert!(GeometryFile::read(&stream(*b"VPOS", format, &values)).is_err());
    }
    assert!(GeometryFile::read(&frame(*b"VPOS", &[1, 2, 3])).is_err());
    let mut bytes = stream(*b"VPOS", *b"3F32", &positions());
    bytes.pop();
    assert!(GeometryFile::read(&bytes).is_err());
    assert!(VertexStream::new(
        *b"1U16",
        StreamValues::Float3([[1.0, 2.0, 3.0]].into_zero_vec())
    )
    .is_err());
}

#[test]
fn resizing_stream_values_rebuilds_lengths_without_changing_neighboring_chunks() {
    let mut bytes = frame(*b"M3VR", &[4, 5, 6]);
    bytes.extend(stream(*b"VPOS", *b"3F32", &positions()));
    bytes.extend(frame(*b"VGEO", &[7, 8, 9]));
    let mut geometry = GeometryFile::read(&bytes).unwrap();
    let GeometryChunk::Stream { stream, .. } = &mut geometry.chunks[1] else {
        panic!("positions")
    };
    stream.values = StreamValues::Float3([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]].into_zero_vec());
    let written = geometry.write().unwrap();
    let decoded = GeometryFile::read(&written).unwrap();
    let GeometryChunk::Stream { stream, .. } = &decoded.chunks[1] else {
        panic!("positions")
    };
    let StreamValues::Float3(values) = &stream.values else {
        panic!("float triples")
    };
    assert_eq!(values.len(), 2);
    assert_eq!(values.get(1), Some([4.0, 5.0, 6.0]));
    assert_eq!(decoded.chunks[0], geometry.chunks[0]);
    assert_eq!(decoded.chunks[2], geometry.chunks[2]);
}

#[test]
fn bundle_resolver_loads_a_modern_geometry_document_for_a_skin_dependency() {
    let mut core = vec![0; 304];
    core[..4].copy_from_slice(b"MD20");
    core[4..8].copy_from_slice(&274u32.to_le_bytes());
    core[68..72].copy_from_slice(&1u32.to_le_bytes());
    let mut model = frame(*b"MD21", &core);
    model.extend(frame(*b"SFID", &7u32.to_le_bytes()));
    let mut geometry = frame(*b"M3VR", &[1]);
    geometry.extend(stream(*b"VPOS", *b"3F32", &positions()));
    let bundle = Bundle::<Chunked<V274>>::decode(&model, FileKey::Root, |dependency| {
        assert_eq!(dependency.key, FileKey::FileId(7));
        Ok(Some(geometry.as_slice()))
    })
    .unwrap();
    let Some(File::Geometry(file)) = bundle.file(&FileKey::FileId(7)) else {
        panic!("modern geometry companion")
    };
    assert!(matches!(file.chunks[1], GeometryChunk::Stream { .. }));
    assert!(matches!(
        bundle.profile(0, false).unwrap(),
        ProfileRef::Geometry(_)
    ));
    assert_eq!(bundle.geometry(0, false).unwrap().chunks.len(), 2);
    assert!(bundle.skin(0, false).is_err());
    let encoded = bundle.encode().unwrap();
    assert_eq!(encoded.files[&FileKey::FileId(7)], geometry);
}
