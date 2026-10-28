//! Verify the promised allocation behavior, including failure diagnostics and
//! direct per-record MDL -> MDX conversion into a preallocated output buffer.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::{Cursor as IoCursor, Write};
use std::ptr::eq;
use wc3::model::formats::mdlx::animation::Track;
use wc3::model::formats::mdlx::animation::{GlobalSequence, Interpolation, Sequence};
use wc3::model::formats::mdlx::materials::Texture;
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::mdl::{Lexer, Parser, Writer};
use wc3::model::formats::mdlx::scene::ModelInfo;
use wc3::model::formats::mdlx::Encoder;
use wc3::model::formats::mdlx::Vec3;
use wc3::model::formats::mdlx::ZeroVec;

struct CountingAllocator;
thread_local! { static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) }; }
fn record() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(value) = count.get() {
            count.set(Some(value + 1));
        }
    });
}
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record();
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record();
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record();
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

struct Measurement;
impl Drop for Measurement {
    fn drop(&mut self) {
        ALLOCATIONS.with(|count| count.set(None));
    }
}
fn measured<T>(work: impl FnOnce() -> T) -> (T, usize) {
    ALLOCATIONS.with(|count| count.set(Some(0)));
    let guard = Measurement;
    let result = work();
    let count = ALLOCATIONS.with(|count| count.get().unwrap());
    drop(guard);
    (result, count)
}

#[test]
fn lexing_reading_writing_and_diagnostics_do_not_allocate() {
    let mut storage = [0u8; 4096];
    let (result, count) = measured(|| {
        let source = r#"Bitmap { Image "Textures\雪.blp", WrapWidth, }"#;
        for token in Lexer::new(source) {
            token.unwrap();
        }
        let texture = Texture::decode_mdl(source).unwrap();
        let sequence = Sequence::decode_mdl(
            "Anim \"Stand\" { Interval { 0, 1000 }, BoundsRadius 1.2345678, }",
        )
        .unwrap();
        let interpolation = Interpolation::decode_mdl("Linear").unwrap();
        let track = Track::<Vec3>::decode_mdl("Track 0 { Linear, }").unwrap();
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.entry(&interpolation).unwrap();
        writer.write(&track).unwrap();
        writer.write(&texture).unwrap();
        writer.write(&sequence).unwrap();
        let info =
            ModelInfo::decode_mdl("Model \"Derived\" { BlendTime 150, BoundsRadius 1.2345678, }")
                .unwrap();
        writer.write(&info).unwrap();
        let mut sink = writer.finish().unwrap();
        let error = Texture::decode_mdl("Bitmap { Mystery 1, }").unwrap_err();
        write!(sink, "{}", error.diagnostic("Bitmap { Mystery 1, }")).unwrap();
        sink.position()
    });
    assert!(result > 0);
    assert_eq!(count, 0);
}

#[test]
fn counted_records_can_transcode_without_an_owned_collection() {
    let mut bytes = Vec::with_capacity(1024);
    let (_, count) = measured(|| {
        let mut parser = Parser::new("3 { Duration 1000, Duration 2500, Duration 5000, }");
        let mut encoder = Encoder::new(&mut bytes);
        for value in parser.counted::<GlobalSequence>().unwrap() {
            encoder.write(&value.unwrap()).unwrap();
        }
        parser.finish().unwrap();
    });
    assert_eq!(bytes.len(), 12);
    assert_eq!(count, 0);
}

#[test]
fn geosets_write_borrowed_mesh_sections_without_allocating() {
    use wc3::model::formats::mdlx::geometry::{Geoset, SkinWeights};
    use wc3::model::formats::mdlx::V1400;
    let mut geoset =
        Geoset::<V1400>::decode_mdl(include_str!("../../fixtures/mdl/quad_geoset.mdl")).unwrap();
    geoset.set_tangents(Some(ZeroVec::alloc_from_slice(&[[1.0, 0.0, 0.0, -1.0]; 4])));
    geoset
        .set_skin_weights(Some(ZeroVec::alloc_from_slice(
            &[SkinWeights {
                bone_indices: [0; 4],
                weights: [255, 0, 0, 0],
            }; 4],
        )))
        .unwrap();
    geoset.sequence_extents = [geoset.extent; 2].into_iter().collect();
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&geoset).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn model_assembly_writes_borrowed_collections_without_allocating() {
    use wc3::model::formats::mdlx::chunks::{GlidersChunk, GlobalSequencesChunk};
    use wc3::model::formats::mdlx::emitters::ParticleEmitter;
    use wc3::model::formats::mdlx::geometry::Geoset;
    use wc3::model::formats::mdlx::materials::Material;
    use wc3::model::formats::mdlx::scene::Glider;
    use wc3::model::formats::mdlx::{Document, V800};
    let mut model =
        Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"Borrowed\" {}")
            .unwrap();
    model.set_geosets([Geoset::<V800>::decode_mdl(include_str!(
        "../../fixtures/mdl/quad_geoset.mdl"
    ))
    .unwrap()]);
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(100)]).into());
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(200)]).into());
    model
        .chunks
        .push(GlidersChunk::new(vec![Glider { geoset_id: 0 }]).into());
    model.set_materials([Material::<V800>::decode_mdl(
        "Material { Layer { Alpha 1 { Linear, 0: 0.5, } } }",
    )
    .unwrap()]);
    model.set_particle_emitters([ParticleEmitter::decode_mdl(
        "ParticleEmitter \"p\" { ObjectId 0, Translation 1 { Linear, 0: { 1, 2, 3 }, } EmissionRate 1 { Linear, 0: 2, } }"
    ).unwrap()]);
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        assert_eq!(model.geosets().next().unwrap().vertices().len(), 4);
        assert_eq!(model.materials().next().unwrap().layers.len(), 1);
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&model).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn hive_dialect_writes_named_tracks_and_mesh_data_without_allocating() {
    use wc3::model::formats::mdlx::geometry::Geoset;
    use wc3::model::formats::mdlx::materials::Layer;
    use wc3::model::formats::mdlx::mdl::Dialect;
    use wc3::model::formats::mdlx::{V1400, V1800};
    let layer = Layer::<V1800>::decode_mdl("Layer { ShaderTypeId 1, NormalTextureID 1 { Hermite, GlobalSeqId 2, -1: 3, InTan 4, OutTan 5, } ORMTextureID 1 { Linear, 0: 1, } }").unwrap();
    let source = include_str!("../../fixtures/mdl/quad_geoset.mdl");
    let end = source.rfind('}').unwrap();
    let source = format!("{} SelectionFlags 128, LevelOfDetailName \"LOD\", SkinWeights 4 {{ {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, }} }}", &source[..end]);
    let geoset = Geoset::<V1400>::decode_mdl(&source).unwrap();
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        let mut writer =
            Writer::with_dialect(IoCursor::new(&mut storage[..]), Dialect::HiveWorkshop);
        writer.write(&layer).unwrap();
        writer.write(&geoset).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn borrowed_arrays_clone_inspect_and_encode_without_allocating() {
    use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};
    let track = Track::<f32>::decode_mdl("Track 2 { Linear, 0: 0, 100: 1, }").unwrap();
    let bytes = track.encode_mdx().unwrap();
    let track = Track::<f32>::decode_mdx(&bytes).unwrap();
    let mut output = Vec::with_capacity(128);
    let (_, count) = measured(|| {
        let clone = track.clone();
        let keys = clone.linear_keys().unwrap();
        assert_eq!(keys.iter().count(), 2);
        assert!(!keys.is_owned());
        Encoder::new(&mut output).write(&clone).unwrap();
    });
    assert_eq!(count, 0);
    assert!(!track.linear_keys().unwrap().is_owned());
    let mut array = track.linear_keys().unwrap().clone();
    let (_, first) = measured(|| {
        array.to_mut_slice();
    });
    let (_, second) = measured(|| {
        array.to_mut_slice();
    });
    assert_eq!(first, 1);
    assert_eq!(second, 0);
}

#[test]
fn named_collection_iterators_borrow_duplicate_chunks_without_allocating() {
    use wc3::model::formats::mdlx::chunks::{ModelChunk, RawChunk, SequencesChunk, UnknownChunk};
    use wc3::model::formats::mdlx::{
        CommonModelAccess, Document, DynamicDocument, Extended, NoExtensions, V800,
    };

    let first = Sequence::new("Stand", [0, 100]).unwrap();
    let second = Sequence::new("Walk", [101, 200]).unwrap();
    let mut model = Document::<Extended<V800, NoExtensions>>::new();
    model
        .chunks
        .push(ModelChunk::from(SequencesChunk::new(vec![first.clone()])));
    model
        .chunks
        .push(ModelChunk::from(SequencesChunk::new(Vec::new())));
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"FUTR", vec![1])).unwrap(),
    ));
    model
        .chunks
        .push(ModelChunk::from(SequencesChunk::new(vec![second.clone()])));
    let mut output = Vec::with_capacity(2);
    let (_, allocations) = measured(|| {
        let mut records = model.sequences();
        let ModelChunk::Sequences(chunk) = &model.chunks[1] else {
            unreachable!()
        };
        assert!(eq(records.next().unwrap(), &chunk.records[0]));
        assert_eq!(records.next().unwrap(), &second);
        assert!(records.next().is_none());
        assert!(records.next().is_none());
        assert!(model.sequences().any(|record| record == &first));
        output.extend(model.sequences().cloned());
    });
    assert_eq!(allocations, 0);
    assert_eq!(output, [first, second]);

    let runtime = DynamicDocument::<NoExtensions>::V800(model);
    let (names, allocations) = measured(|| {
        let mut records = runtime.sequences();
        let first = records.next().unwrap().name.text();
        let second = records.next().unwrap().name.text();
        assert!(records.next().is_none());
        (first, second)
    });
    assert_eq!(allocations, 0);
    assert_eq!(names.0, "Stand");
    assert_eq!(names.1, "Walk");
}

#[test]
fn input_conversion_encodes_once_and_moves_existing_storage_without_allocating() {
    use wc3::model::formats::mdlx::{Borrowed, IntoZeroVec};

    let (owned, allocations): (ZeroVec<u32>, _) = measured(|| [1, 2, 3].into_zero_vec());
    assert_eq!(allocations, 1);
    let pointer = owned.as_bytes().as_ptr();
    let (moved, allocations) = measured(|| owned.into_zero_vec());
    assert_eq!(allocations, 0);
    assert_eq!(moved.as_bytes().as_ptr(), pointer);
    let borrowed = moved.as_borrowed();
    let (borrowed, allocations) = measured(|| borrowed.into_zero_vec());
    assert_eq!(allocations, 0);
    assert_eq!(borrowed.as_bytes().as_ptr(), pointer);
    assert!(!borrowed.is_owned());
}

#[test]
fn influence_inspection_never_allocates_per_vertex() {
    use wc3::model::profiles::Mdlx;
    use wc3::model::{
        Influence, InfluenceRows, IntoPackedArray, Model, NodeInfluence, ValueGroups,
        VertexInfluences,
    };

    let positive = Influence {
        joint: 7,
        weight: 1.,
    };
    let zero = Influence {
        joint: 19,
        weight: 0.,
    };
    let variable = ValueGroups::from_rows([vec![zero, positive], vec![]]);
    let three = [[zero, positive, zero]].into_packed_array();
    let four = [[zero, positive, zero, zero]].into_packed_array();
    let groups = ValueGroups::from_rows([vec![7u32, 19, 7], vec![]]);
    let mut builder = Model::<Mdlx>::builder("row allocation test");
    let definition = builder.new_node("bone");
    let node = builder.add_node(definition).unwrap();
    let nodes = ValueGroups::from_rows([vec![NodeInfluence { node, weight: 1. }], vec![]]);

    let (_, count) = measured(|| {
        for rows in [
            InfluenceRows::None,
            InfluenceRows::Variable(&variable),
            InfluenceRows::Three(&three),
            InfluenceRows::Four(&four),
            InfluenceRows::MatrixGroups(&groups),
            InfluenceRows::Nodes(&nodes),
        ] {
            for row in rows.iter() {
                let total: f32 = row.weights().sum();
                assert!(total == 0. || (total - 1.).abs() < f32::EPSILON);
                match row {
                    VertexInfluences::Joints(weights) => {
                        assert_eq!(weights.iter().count(), weights.len());
                        assert_eq!(weights.clone().into_iter().count(), weights.len());
                        assert!(weights.get(weights.len()).is_none());
                    }
                    VertexInfluences::Nodes(weights) => {
                        assert!(!weights.is_owned());
                        for influence in weights.iter() {
                            assert_eq!(influence.node, node);
                        }
                    }
                }
            }
            assert!(rows.get(rows.len()).is_none());
            assert!(rows.get(usize::MAX).is_none());
        }
    });
    assert_eq!(count, 0);
}

#[test]
fn triangle_projection_does_not_allocate_per_primitive() {
    use glam::Vec3;
    use wc3::model::{GeometryData, Mesh};

    let project = |count| {
        let mesh: Mesh<'_> = Mesh::new(
            GeometryData::triangles([Vec3::ZERO, Vec3::X, Vec3::Y], vec![[0, 1, 2]; count])
                .unwrap(),
        );
        let subset = mesh.whole_geometry().unwrap();
        let (batches, allocations) = measured(|| mesh.project(&subset, 0, [None; 3]).unwrap());
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].positions.len(), count * 3);
        allocations
    };
    let small = project(8);
    let large = project(4096);
    assert!(large <= 128, "projection allocated {large} times");
    assert!(
        large <= small + 64,
        "projection allocations grew from {small} to {large}"
    );
}
