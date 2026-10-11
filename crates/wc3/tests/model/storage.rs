use wc3::model::animation::{Track, TrackValue, ValueKeyframe};
use wc3::model::chunks::{BonesChunk, GeosetsChunk, ModelChunk, RawChunk, UnknownChunk};
use wc3::model::geometry::Geoset;
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Bone, Node};
use wc3::model::{mdx, ConversionOptions, Model, ModelArray, V1800, V800};

use zerovec::ule::AsULE;

#[test]
fn geometry_borrows_unaligned_source_and_only_an_edited_array_becomes_owned() {
    let geoset =
        Geoset::<V800>::new(&[[1.0, 2.0, 3.0]; 32], &[[0.0, 0.0, 1.0]; 32], &[0, 1, 2]).unwrap();
    let mut model = Model::<V800>::new();
    model.replace_records::<GeosetsChunk<V800>>(vec![geoset]);
    let mut source = vec![99];
    source.extend(model.encode_mdx().unwrap());
    let original = Model::<V800>::decode_mdx(&source[1..]).unwrap();
    let mut converted = original
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    let vertices = original.geosets().next().unwrap().vertices();
    let wire = vertices.as_bytes().unwrap();
    assert!(wire.as_ptr() >= source.as_ptr());
    assert!(wire.as_ptr() < source[source.len() - 1..].as_ptr());
    assert!(vertices.is_borrowed());
    assert_eq!(
        wire.as_ptr(),
        converted
            .geosets()
            .next()
            .unwrap()
            .vertices()
            .as_bytes()
            .unwrap()
            .as_ptr()
    );
    let normal_pointer = converted
        .geosets()
        .next()
        .unwrap()
        .normals()
        .as_bytes()
        .unwrap()
        .as_ptr();
    converted
        .records_mut::<GeosetsChunk<V1800>>()
        .next()
        .unwrap()
        .set_vertex(0, [9.0; 3])
        .unwrap();
    assert_eq!(vertices.get(0), Some([1.0, 2.0, 3.0]));
    assert_eq!(
        converted.geosets().next().unwrap().vertices().get(0),
        Some([9.0; 3])
    );
    assert!(!converted.geosets().next().unwrap().vertices().is_borrowed());
    assert_eq!(
        normal_pointer,
        converted
            .geosets()
            .next()
            .unwrap()
            .normals()
            .as_bytes()
            .unwrap()
            .as_ptr()
    );
    converted.encode_mdx().unwrap();
    assert!(vertices.is_borrowed());
}

#[test]
fn sorted_track_sampling_keeps_wire_storage_and_unsorted_input_sorts_stably() {
    let track = Track::linear(
        vec![
            ValueKeyframe {
                frame: -10,
                value: 0.0,
            },
            ValueKeyframe {
                frame: 10,
                value: 2.0,
            },
        ],
        None,
    )
    .unwrap();
    let bytes = track.encode_mdx().unwrap();
    let decoded = Track::<f32>::decode_mdx(&bytes).unwrap();
    let keys = decoded.linear_keys().unwrap();
    assert_eq!(keys.as_bytes().unwrap().as_ptr(), bytes[12..].as_ptr());
    assert_eq!(decoded.evaluate(0.0), Some(1.0));
    assert_eq!(decoded.key_frame_at_or_before(0.0), Some(-10));
    assert!(keys.is_borrowed());
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    let first = bytes[12..20].to_vec();
    let mut reversed = bytes.clone();
    reversed.copy_within(20..28, 12);
    reversed[20..28].copy_from_slice(&first);
    let sorted = Track::<f32>::decode_mdx(&reversed).unwrap();
    assert!(!sorted.linear_keys().unwrap().is_borrowed());
    assert_eq!(sorted.encode_mdx().unwrap(), bytes);
}

#[test]
fn consuming_conversion_moves_records_and_geometry_allocations() {
    let mut model = Model::<V800>::new();
    model.replace_records::<BonesChunk>(vec![Bone::new(
        Node::new("bone", 0).unwrap(),
        u32::MAX,
        u32::MAX,
    )]);
    model.replace_records::<GeosetsChunk<V800>>(vec![Geoset::new(
        &[[1.0; 3]; 3],
        &[[0.0, 0.0, 1.0]; 3],
        &[0, 1, 2],
    )
    .unwrap()]);
    let bone = model.records::<BonesChunk>().next().unwrap() as *const Bone;
    let vertices = model
        .geosets()
        .next()
        .unwrap()
        .vertices()
        .as_native_slice()
        .unwrap()
        .as_ptr();
    let converted = model
        .into_convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(
        bone,
        converted.records::<BonesChunk>().next().unwrap() as *const Bone
    );
    assert_eq!(
        vertices,
        converted
            .geosets()
            .next()
            .unwrap()
            .vertices()
            .as_native_slice()
            .unwrap()
            .as_ptr()
    );
}

#[test]
fn opaque_payloads_borrow_the_source_and_can_resize_independently() {
    let mut model = Model::<V800>::new();
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"TEST", vec![1, 2, 3])).unwrap(),
    ));
    let source = model.encode_mdx().unwrap();
    let mut decoded = Model::<V800>::decode_mdx(&source).unwrap();
    let ModelChunk::Unknown(chunk) = decoded.chunks.last_mut().unwrap() else {
        panic!("missing chunk")
    };
    assert_eq!(
        chunk.raw().data.as_ptr(),
        source[source.len() - 3..].as_ptr()
    );
    chunk.data_mut().push(4);
    assert!(!chunk.raw().data.is_borrowed());
    assert_eq!(chunk.raw().data, [1, 2, 3, 4]);
    assert_eq!(&source[source.len() - 3..], &[1, 2, 3]);
    let bytes = decoded.encode_mdx().unwrap();
    assert_eq!(
        Model::<V800>::decode_mdx(&bytes)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        bytes
    );
}

#[test]
fn borrowed_views_are_cheap_and_owned_conversion_moves_existing_buffers() {
    let values = vec![1u32, 2, 3];
    let pointer = values.as_ptr();
    let array = ModelArray::new(values);
    assert_eq!(array.as_native_slice().unwrap().as_ptr(), pointer);
    let mut view = array.as_borrowed();
    assert_eq!(view.as_native_slice().unwrap().as_ptr(), pointer);
    assert_eq!(view.clone().as_native_slice().unwrap().as_ptr(), pointer);
    view.to_mut().push(4);
    assert_eq!(array, [1, 2, 3]);
    assert_eq!(view, [1, 2, 3, 4]);
    let deep = array.clone();
    assert_ne!(deep.as_native_slice().unwrap().as_ptr(), pointer);
    let owned = array.into_owned();
    assert_eq!(owned.as_native_slice().unwrap().as_ptr(), pointer);
}

#[test]
fn float_equality_uses_numeric_semantics_instead_of_wire_byte_equality() {
    let positive = [0u8; 4];
    let negative = (-0.0f32).to_le_bytes();
    let read = |bytes: &[u8]| {
        let mut bytes = bytes.to_vec();
        bytes.splice(
            0..0,
            [1, 0, 0, 0, 1, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0],
        );
        Track::<f32>::decode_mdx(&bytes).unwrap().into_owned()
    };
    assert_eq!(read(&positive), read(&negative));
    let nan = f32::NAN.to_le_bytes();
    assert_ne!(read(&nan), read(&nan));
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ValidScalar(char);
impl Default for ValidScalar {
    fn default() -> Self {
        Self('a')
    }
}
impl AsULE for ValidScalar {
    type ULE = <char as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.0.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self(char::from_unaligned(value))
    }
}
impl<'data> mdx::Read<'data> for ValidScalar {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        char::from_u32(cursor.read()?).map(Self).ok_or_else(|| {
            mdx::ReadError::new(
                cursor.absolute_position(),
                mdx::ReadErrorKind::InvalidValue {
                    field: "Unicode scalar track value",
                },
            )
        })
    }
}
impl mdx::Write for ValidScalar {
    fn write_mdx(&self, encoder: &mut mdx::Encoder<'_>) -> Result<(), mdx::WriteError> {
        encoder.write(&u32::from(self.0))
    }
}
impl TrackValue for ValidScalar {
    const COMPONENTS: usize = 1;
}
#[test]
fn custom_track_values_keep_semantic_validation() {
    let track = Track::step(
        vec![ValueKeyframe {
            frame: 0,
            value: ValidScalar::default(),
        }],
        None,
    )
    .unwrap();
    let mut bytes = track.encode_mdx().unwrap();
    bytes[16..20].copy_from_slice(&0x110000u32.to_le_bytes());
    assert!(Track::<ValidScalar>::decode_mdx(&bytes).is_err());
}

#[test]
fn skin_conversion_moves_native_storage_when_layouts_match() {
    use wc3::model::geometry::{SkinWeight, SkinWeights};
    use wc3::model::{V1300, V1400, V900};

    let mut geoset = Geoset::<V900>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    geoset
        .set_skin_weights(Some(&[SkinWeights::<u8> {
            bone_indices: [1, 2, 3, 4],
            weights: [128, 64, 32, 31],
        }]))
        .unwrap();
    let mut narrow = Model::<V900>::new();
    narrow.replace_records::<GeosetsChunk<V900>>(vec![geoset]);
    let pointer = narrow
        .geosets()
        .next()
        .unwrap()
        .skin_weights()
        .unwrap()
        .as_native_slice()
        .unwrap()
        .as_ptr();
    let narrow = narrow
        .into_convert::<V1300>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(
        narrow
            .geosets()
            .next()
            .unwrap()
            .skin_weights()
            .unwrap()
            .as_native_slice()
            .unwrap()
            .as_ptr(),
        pointer
    );
    let wide = narrow
        .into_convert::<V1400>(&ConversionOptions::strict())
        .unwrap()
        .model;
    let row = wide
        .geosets()
        .next()
        .unwrap()
        .skin_weights()
        .unwrap()
        .get(0)
        .unwrap();
    assert_eq!(row.bone_indices(), [1, 2, 3, 4]);
    assert_eq!(row.weights(), [128, 64, 32, 31]);
    let pointer = wide
        .geosets()
        .next()
        .unwrap()
        .skin_weights()
        .unwrap()
        .as_native_slice()
        .unwrap()
        .as_ptr();
    let wide = wide
        .into_convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(
        wide.geosets()
            .next()
            .unwrap()
            .skin_weights()
            .unwrap()
            .as_native_slice()
            .unwrap()
            .as_ptr(),
        pointer
    );
    let narrow = wide
        .into_convert::<V900>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(
        narrow
            .geosets()
            .next()
            .unwrap()
            .skin_weights()
            .unwrap()
            .get(0)
            .unwrap()
            .weights(),
        [128, 64, 32, 31]
    );
}

#[test]
fn skin_conversion_keeps_encoded_storage_borrowed_when_layouts_match() {
    use wc3::model::geometry::SkinWeights;
    use wc3::model::{V1300, V1400, V900};

    let mut narrow = Geoset::<V900>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    narrow
        .set_skin_weights(Some(&[SkinWeights::<u8> {
            bone_indices: [1; 4],
            weights: [255, 0, 0, 0],
        }]))
        .unwrap();
    let bytes = narrow.encode_mdx().unwrap();
    let decoded = Geoset::<V900>::decode_mdx(&bytes).unwrap();
    let pointer = decoded.skin_weights().unwrap().as_bytes().unwrap().as_ptr();
    let converted = decoded
        .convert::<V1300>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert!(converted.skin_weights().unwrap().is_borrowed());
    assert_eq!(
        converted
            .skin_weights()
            .unwrap()
            .as_bytes()
            .unwrap()
            .as_ptr(),
        pointer
    );

    let mut wide = Geoset::<V1400>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    wide.set_skin_weights(Some(&[SkinWeights::<u16> {
        bone_indices: [1000; 4],
        weights: [255, 0, 0, 0],
    }]))
    .unwrap();
    let bytes = wide.encode_mdx().unwrap();
    let decoded = Geoset::<V1400>::decode_mdx(&bytes).unwrap();
    let pointer = decoded.skin_weights().unwrap().as_bytes().unwrap().as_ptr();
    let converted = decoded
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert!(converted.skin_weights().unwrap().is_borrowed());
    assert_eq!(
        converted
            .skin_weights()
            .unwrap()
            .as_bytes()
            .unwrap()
            .as_ptr(),
        pointer
    );
}
