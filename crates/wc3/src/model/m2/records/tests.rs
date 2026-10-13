use super::*;
use crate::model::m2::codec::{Array, ArrayRef, Decoder, Encoder};
use crate::model::m2::{ModelLayout, Read, RecordLayout, Write};
use crate::model::m2::{V272PreSplit, V256, V257, V260, V263, V264, V265, V272, V273, V274};
use zerovec::ZeroVec;

fn roundtrip<'data, T: Read<'data> + Write + RecordLayout>(
    bytes: &'data [u8],
    version: u32,
    flags: u32,
) {
    let mut input = Decoder::new(bytes, version, ModelFlags::from_bits_retain(flags));
    let record = T::read(&mut input).unwrap();
    assert_eq!(input.position(), T::size(version, flags));
    let mut output = Encoder::new(version, ModelFlags::from_bits_retain(flags));
    record.write(&mut output).unwrap();
    assert_eq!(output.position(), T::size(version, flags));
    assert_eq!(output.finish(), bytes);
}
fn layouts<V: ModelLayout>() {
    for flags in [0, 0x200] {
        macro_rules!check{($($t:ty),*)=>{$(let bytes=vec![0;<$t>::size(V::VERSION,flags)];roundtrip::<$t>(&bytes,V::VERSION,flags);)*};}
        check!(
            Extent,
            Vertex,
            Material,
            Texture,
            Sequence<V>,
            Bone<V>,
            SkinSection<V>,
            Camera<V>,
            RibbonEmitter<V>,
            ParticleEmitter<V>,
            ColorAnimation<V>,
            TextureWeight<V>,
            TextureFlipbook<V>,
            TextureTransform<V>,
            Attachment<V>,
            Event<V>,
            Light<V>,
            Batch,
            ShadowBatch
        );
    }
}
#[test]
fn every_selected_native_record_layout_roundtrips() {
    layouts::<V256>();
    layouts::<V257>();
    layouts::<V260>();
    layouts::<V263>();
    layouts::<V264>();
    layouts::<V265>();
    layouts::<V272PreSplit>();
    layouts::<V272>();
    layouts::<V273>();
    layouts::<V274>();
}
#[test]
fn classic_bone_is_108_bytes_with_float_keys_and_no_crc() {
    let mut bytes = vec![0; 108];
    for (n, v) in [1.0_f32, 2.0, 3.0].iter().enumerate() {
        bytes[96 + n * 4..100 + n * 4].copy_from_slice(&v.to_le_bytes());
    }
    let bone = Bone::<V256>::read(&mut Decoder::new(&bytes, 256, ModelFlags::default())).unwrap();
    assert_eq!(bone.name_crc, ());
    assert_eq!(bone.pivot, [1.0, 2.0, 3.0]);
    assert_eq!(Bone::<V260>::size(260, 0), 112);
}
#[test]
fn legacy_particle_native_offsets_and_packing_remain_distinct() {
    let mut bytes = vec![0; 504];
    bytes[0x28..0x2c].copy_from_slice(&[1, 2, 3, 4]);
    bytes[0x15c..0x160].copy_from_slice(&1.5_f32.to_le_bytes());
    bytes[0x17c..0x180].copy_from_slice(&4.25_f32.to_le_bytes());
    let early =
        ParticleEmitter::<V260>::read(&mut Decoder::new(&bytes, 260, ModelFlags::default()))
            .unwrap();
    let late = ParticleEmitter::<V263>::read(&mut Decoder::new(&bytes, 263, ModelFlags::default()))
        .unwrap();
    assert_eq!(
        early.kind,
        WideParticleHeader {
            blending: ParticleBlend::Unknown(0x0201),
            emitter: ParticleGenerator::Unknown(0x0403)
        }
    );
    assert_eq!(
        late.kind,
        PackedParticleHeader {
            blending: PackedParticleBlend::Alpha,
            emitter: PackedParticleGenerator::Sphere,
            color_index: 0x0403
        }
    );
    assert_eq!(early.lifetime.scales[0], 1.5);
    assert_eq!(early.tail_length, 4.25);
    roundtrip::<ParticleEmitter<V260>>(&bytes, 260, 0);
    roundtrip::<ParticleEmitter<V263>>(&bytes, 263, 0);
}
#[test]
fn static_legacy_timeline_and_spline_values_relocate() {
    let track = Track::<f32, V260> {
        base: TrackBase {
            interpolation: Interpolation::Hermite,
            global_sequence: u16::MAX,
            timeline: LegacyTimeline {
                ranges: ZeroVec::alloc_from_slice(&[[0, 1], [0, 0]]),
                timestamps: ZeroVec::alloc_from_slice(&[100, 300]),
            },
        },
        values: LegacyValues::Splines(ZeroVec::alloc_from_slice(&[
            [1.0, 0.5, 1.5],
            [2.0, 1.5, 2.5],
        ])),
    };
    let mut output = Encoder::new(260, ModelFlags::default());
    track.write(&mut output).unwrap();
    let bytes = output.finish();
    let decoded =
        Track::<f32, V260>::read(&mut Decoder::new(&bytes, 260, ModelFlags::default())).unwrap();
    assert_eq!(decoded, track);
    decoded.validate().unwrap();
}
#[test]
fn modern_external_leaves_never_dereference_model_bytes() {
    let track = Track::<f32, V264> {
        base: TrackBase {
            interpolation: Interpolation::Linear,
            global_sequence: u16::MAX,
            timeline: ModernTimeline {
                timestamps: vec![Array::External(ArrayRef {
                    count: 2,
                    offset: 9000,
                })],
            },
        },
        values: TrackValues::Values(vec![Array::External(ArrayRef {
            count: 2,
            offset: 9016,
        })]),
    };
    let mut output = Encoder::new(264, ModelFlags::default());
    track.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut input = Decoder::new(&bytes, 264, ModelFlags::default());
    input.external_sequences = vec![true].into();
    assert_eq!(Track::<f32, V264>::read(&mut input).unwrap(), track);
    assert!(
        Track::<f32, V264>::read(&mut Decoder::new(&bytes, 264, ModelFlags::default())).is_err()
    );
}
#[test]
fn malformed_outer_array_is_rejected_before_allocation() {
    let mut bytes = vec![0; 20];
    bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        Track::<f32, V264>::read(&mut Decoder::new(&bytes, 264, ModelFlags::default())).is_err()
    );
}
#[test]
fn camera_splines_keep_triplets_even_with_constant_interpolation() {
    let mut camera = Camera::<V265>::new();
    camera.roll.base.timeline.timestamps = vec![Array::Inline(ZeroVec::alloc_from_slice(&[7]))];
    camera.roll.values.values = vec![Array::Inline(ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0]]))];
    let mut output = Encoder::new(265, ModelFlags::default());
    camera.write(&mut output).unwrap();
    let bytes = output.finish();
    assert_eq!(
        Camera::<V265>::read(&mut Decoder::new(&bytes, 265, ModelFlags::default())).unwrap(),
        camera
    );
    camera.validate().unwrap();
}
#[test]
fn typed_constructors_encode_without_runtime_version_variants() {
    fn check<V: ModelLayout>() {
        for flags in [0, 0x200] {
            let mut output = Encoder::new(V::VERSION, ModelFlags::from_bits_retain(flags));
            output.write_records(&[Bone::<V>::new()]).unwrap();
            output.write_records(&[Camera::<V>::new()]).unwrap();
            output
                .write_records(&[ParticleEmitter::<V>::new(ModelFlags::from_bits_retain(
                    flags,
                ))])
                .unwrap();
        }
    }
    check::<V256>();
    check::<V260>();
    check::<V263>();
    check::<V264>();
    check::<V265>();
    check::<V272PreSplit>();
    check::<V272>();
}
#[test]
fn vertex_material_leaf_arrays_borrow_unaligned_bytes() {
    let vertex = Vertex {
        position: [1.0, 2.0, 3.0],
        bone_weights: [255, 0, 0, 0],
        bone_indices: [7, 8, 9, 10],
        normal: [0.0, 0.0, 1.0],
        texture_coordinates: [[0.25, 0.5], [0.75, 1.0]],
    };
    let encoded = ZeroVec::alloc_from_slice(&[vertex]);
    let mut bytes = vec![0xff];
    bytes.extend_from_slice(encoded.as_bytes());
    let borrowed = ZeroVec::<Vertex>::parse_bytes(&bytes[1..]).unwrap();
    assert_eq!(borrowed.as_bytes().as_ptr(), bytes[1..].as_ptr());
    assert_eq!(borrowed.get(0), Some(vertex));
    let bytes = [0x34, 0x12, 0x78, 0x56];
    let material = ZeroVec::<Material>::parse_bytes(&bytes).unwrap();
    assert_eq!(material.as_bytes().as_ptr(), bytes.as_ptr());
    assert_eq!(
        material.get(0),
        Some(Material {
            flags: MaterialFlags::from_bits_retain(0x1234),
            blending_mode: BlendMode::Unknown(0x5678)
        })
    );
}

#[test]
fn typed_track_views_keep_native_ranges_and_detach_buffers() {
    let track = Track::<f32, V260> {
        base: TrackBase {
            interpolation: Interpolation::Linear,
            global_sequence: u16::MAX,
            timeline: LegacyTimeline {
                ranges: ZeroVec::alloc_from_slice(&[[1, 2]]),
                timestamps: ZeroVec::alloc_from_slice(&[10, 30, 50]),
            },
        },
        values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[1.0, 3.0, 5.0])),
    };
    let borrowed: Track<'_, f32, V260> = track.as_borrowed();
    assert_eq!(
        borrowed.base.timeline.timestamps.as_bytes().as_ptr(),
        track.base.timeline.timestamps.as_bytes().as_ptr()
    );
    let window = borrowed.sequence(0).unwrap();
    assert_eq!(window.timestamps.iter().collect::<Vec<_>>(), [30, 50]);
    let SequenceValues::Values(values) = window.values else {
        panic!("ordinary keys")
    };
    assert_eq!(values.iter().collect::<Vec<_>>(), [3.0, 5.0]);
    let owned: Track<'static, f32, V260> = borrowed.into_owned();
    assert_ne!(
        owned.base.timeline.timestamps.as_bytes().as_ptr(),
        track.base.timeline.timestamps.as_bytes().as_ptr()
    );
    assert_eq!(owned, track);
}
