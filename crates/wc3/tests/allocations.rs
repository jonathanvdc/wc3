//! Verify the promised allocation behavior, including failure diagnostics and
//! direct per-record MDL -> MDX conversion into a preallocated output buffer.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::{Cursor as IoCursor, Write};
use std::ptr::eq;
use wc3::model::animation::Track;
use wc3::model::animation::{GlobalSequence, Interpolation, Sequence};
use wc3::model::materials::Texture;
use wc3::model::mdl::Read as _;
use wc3::model::mdl::{Lexer, Parser, Writer};
use wc3::model::scene::ModelInfo;
use wc3::model::Encoder;
use wc3::model::Vec3;
use wc3::model::ZeroVec;

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
fn derived_records_write_existing_tracks_without_cloning() {
    use wc3::model::animation::TextureAnimation;
    use wc3::model::emitters::{ParticleEmitter, RibbonEmitter};
    let texture_animation = TextureAnimation::decode_mdl("TVertexAnim { Scaling 1 { Linear, 0: { 1, 2, 3 }, } Translation 1 { Linear, 0: { 4, 5, 6 }, } }").unwrap();
    let particle = ParticleEmitter::decode_mdl("ParticleEmitter \"a\" { ObjectId 0, EmitterUsesMdl, Translation 1 { Linear, 0: { 1, 2, 3 }, } EmissionRate 1 { Linear, 0: 2.0, } Visibility 1 { Linear, 0: 1.0, } }").unwrap();
    let ribbon = RibbonEmitter::decode_mdl("RibbonEmitter \"a\" { ObjectId 0, HeightAbove 1 { Linear, 0: 2.0, } TextureSlot 1 { Linear, 0: 3, } Visibility 1 { Linear, 0: 1.0, } }").unwrap();
    let mut storage = [0u8; 4096];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&texture_animation).unwrap();
        writer.write(&particle).unwrap();
        writer.write(&ribbon).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn version_selected_lights_and_layers_write_without_cloning_storage() {
    use wc3::model::materials::Layer;
    use wc3::model::scene::Light;
    use wc3::model::{V1600, V1800, V800};
    let light = Light::<V1600>::decode_mdl("Light \"a\" { ObjectId 0, Ambient, Translation 1 { Linear, 0: { 1, 2, 3 }, } Intensity 1 { Linear, 0: 1.0, } ShadowCastingStart 1 { Linear, 0: 2.0, } Damping 1 { Linear, 0: 0.00001, } }").unwrap();
    let sd = Layer::<V800>::decode_mdl(
        "Layer { Alpha 1 { Linear, 0: 1.0, } TextureID 1 { Linear, 0: 3, } }",
    )
    .unwrap();
    // Shader follows the texture bindings in input: setters resolve storage after
    // all fields have been parsed, independent of input order.
    let hd = Layer::<V1800>::decode_mdl("Layer { TextureID 1 { Linear, 0: 3, } static TextureID 7 <= 4, Shader \"Shader_HD_DefaultUnit\", EmissiveGain 1 { Linear, 0: 1.0, } FresnelOpacity 1 { Linear, 0: 0.0, } }").unwrap();
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&light).unwrap();
        writer.write(&sd).unwrap();
        writer.write(&hd).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn materials_write_layers_and_tracks_without_cloning() {
    use wc3::model::materials::Material;
    use wc3::model::V900;

    let material = Material::<V900>::decode_mdl(
        "Material { Shader \"Shader_HD_DefaultUnit\", Layer { Alpha 1 { Linear, 0: 0.5, } } }",
    )
    .unwrap();
    let mut storage = [0u8; 4096];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&material).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn geosets_write_borrowed_mesh_sections_without_allocating() {
    use wc3::model::geometry::{Geoset, SkinWeights};
    use wc3::model::V1400;
    let mut geoset =
        Geoset::<V1400>::decode_mdl(include_str!("fixtures/mdl/quad_geoset.mdl")).unwrap();
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
    use wc3::model::chunks::{GlidersChunk, GlobalSequencesChunk};
    use wc3::model::geometry::Geoset;
    use wc3::model::scene::Glider;
    use wc3::model::{Model, V800};
    let mut model =
        Model::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"Borrowed\" {}").unwrap();
    model.set_geosets([
        Geoset::<V800>::decode_mdl(include_str!("fixtures/mdl/quad_geoset.mdl")).unwrap(),
    ]);
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(100)]).into());
    model
        .chunks
        .push(GlobalSequencesChunk::new(vec![GlobalSequence(200)]).into());
    model
        .chunks
        .push(GlidersChunk::new(vec![Glider { geoset_id: 0 }]).into());
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&model).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn cameras_particle2_and_popcorn_write_borrowed_tracks_without_allocating() {
    use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
    use wc3::model::scene::Camera;
    use wc3::model::V1800;
    let camera = Camera::<V1800>::decode_mdl("Camera \"c\" { FieldOfView 1, FarClip 100, Translation 1 { Linear, -1: { 1, 2, 3 }, } DOFDistance 20, Target { Translation 1 { Linear, -2: { 3, 2, 1 }, } } Visibility 1 { Linear, 0: 1, } }").unwrap();
    let particle = ParticleEmitter2::decode_mdl("ParticleEmitter2 \"p\" { ObjectId 0, Speed 1 { Linear, -1: 2, } Visibility 1 { Linear, 0: 1, } }").unwrap();
    let popcorn = PopcornEmitter::decode_mdl("ParticleEmitterPopcorn \"p\" { ObjectId 0, Color 1 { Linear, -2: { 1, 2, 3 }, } LifeSpan 1 { Linear, 0: 1, } }").unwrap();
    let mut storage = [0u8; 8192];
    let (_, count) = measured(|| {
        let mut writer = Writer::new(IoCursor::new(&mut storage[..]));
        writer.write(&camera).unwrap();
        writer.write(&particle).unwrap();
        writer.write(&popcorn).unwrap();
        writer.finish().unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn hive_dialect_writes_named_tracks_and_mesh_data_without_allocating() {
    use wc3::model::geometry::Geoset;
    use wc3::model::materials::Layer;
    use wc3::model::mdl::Dialect;
    use wc3::model::{V1400, V1800};
    let layer = Layer::<V1800>::decode_mdl("Layer { ShaderTypeId 1, NormalTextureID 1 { Hermite, GlobalSeqId 2, -1: 3, InTan 4, OutTan 5, } ORMTextureID 1 { Linear, 0: 1, } }").unwrap();
    let source = include_str!("fixtures/mdl/quad_geoset.mdl");
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
fn borrowed_arrays_clone_sample_and_encode_without_allocating() {
    use wc3::model::mdx::{Read as _, Write as _};
    let track = Track::<f32>::decode_mdl("Track 2 { Linear, 0: 0, 100: 1, }").unwrap();
    let bytes = track.encode_mdx().unwrap();
    let track = Track::<f32>::decode_mdx(&bytes).unwrap();
    let mut output = Vec::with_capacity(128);
    let (_, count) = measured(|| {
        let clone = track.clone();
        assert_eq!(clone.evaluate(50.0), Some(0.5));
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
    use wc3::model::chunks::{ModelChunk, RawChunk, SequencesChunk, UnknownChunk};
    use wc3::model::{CommonModelAccess, DynamicModel, Extended, Model, NoExtensions, V800};

    let first = Sequence::new("Stand", [0, 100]).unwrap();
    let second = Sequence::new("Walk", [101, 200]).unwrap();
    let mut model = Model::<Extended<V800, NoExtensions>>::new();
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

    let runtime = DynamicModel::<NoExtensions>::V800(model);
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
fn runtime_collection_getters_do_not_allocate_for_any_version() {
    use wc3::model::emitters::PopcornEmitter;
    use wc3::model::geometry::BindPoseMatrix;
    use wc3::model::scene::{FaceFx, Node};
    use wc3::model::{CommonModelAccess, DynamicModel, Model, NoExtensions, TryModelAccess};

    macro_rules! check_version {
        ($variant:ident) => {{
            let mut model = DynamicModel::<NoExtensions>::$variant(Model::new());
            model.set_sequences([Sequence::new("Stand", [0, 100]).unwrap()]);
            model.set_textures([Texture::new("body.blp").unwrap()]);
            model.set_helpers([Node::new("Root", 0).unwrap()]);
            let supported = model
                .try_set_bind_poses([BindPoseMatrix([1.0; 12])])
                .is_ok();
            if supported {
                model
                    .try_set_face_fx([FaceFx::new("Talk", "talk.facefx").unwrap()])
                    .unwrap();
                model
                    .try_set_popcorn_emitters([PopcornEmitter::new(
                        Node::new("Fire", 1).unwrap(),
                        "fire.mdx",
                        "Stand",
                    )
                    .unwrap()])
                    .unwrap();
            }
            let (_, allocations) = measured(|| {
                assert_eq!(model.sequences().count(), 1);
                assert_eq!(model.textures().next().unwrap().path.text(), "body.blp");
                assert_eq!(model.helpers().next().unwrap().object_id, 0);
                if supported {
                    assert_eq!(model.try_bind_poses().unwrap().count(), 1);
                    assert_eq!(model.try_face_fx().unwrap().count(), 1);
                    assert_eq!(model.try_popcorn_emitters().unwrap().count(), 1);
                } else {
                    assert!(model.try_bind_poses().is_err());
                    assert!(model.try_face_fx().is_err());
                    assert!(model.try_popcorn_emitters().is_err());
                }
            });
            assert_eq!(allocations, 0, "{}", stringify!($variant));
        }};
    }
    check_version!(V800);
    check_version!(V900);
    check_version!(V1000);
    check_version!(V1100);
    check_version!(V1200);
    check_version!(V1300);
    check_version!(V1400);
    check_version!(V1600);
    check_version!(V1800);
}

#[test]
fn named_geometry_and_material_getters_stream_without_allocating() {
    use wc3::model::mdx::Read as _;
    use wc3::model::{Model, V800};

    let model = Model::<V800>::decode_mdx(include_bytes!("fixtures/mdl/quad_model.mdx")).unwrap();
    let ((vertices, materials), allocations) = measured(|| {
        let vertices = model
            .geosets()
            .map(|geoset| geoset.vertices().len())
            .sum::<usize>();
        let materials = model
            .materials()
            .filter(|material| !material.layers.is_empty())
            .count();
        (vertices, materials)
    });
    assert_eq!(allocations, 0);
    assert_eq!(vertices, 4);
    assert_eq!(materials, 1);
}

#[test]
fn mdl_tracks_construct_encoded_keys_without_an_intermediate_vector() {
    for source in [
        "Track 1 { Linear, 0: 2, }",
        "Track 1 { Hermite, 0: 2, InTan 1, OutTan 3, }",
    ] {
        let (track, allocations) = measured(|| Track::<f32>::decode_mdl(source).unwrap());
        assert_eq!(allocations, 1);
        assert_eq!(track.evaluate(0.0), Some(2.0));
    }
}

#[test]
fn mdl_skin_and_extent_adapters_move_their_encoded_buffers() {
    use wc3::model::geometry::Geoset;
    use wc3::model::V1400;

    let source = include_str!("fixtures/mdl/quad_geoset.mdl");
    let end = source.rfind('}').unwrap();
    let skin = format!("{} SkinWeights 4 {{ {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, {{ 0, 0, 0, 0, 255, 0, 0, 0 }}, }} }}", &source[..end]);
    let extent = format!("{} Anim {{ BoundsRadius 1, MinimumExtent {{ 0, 0, 0 }}, MaximumExtent {{ 1, 1, 1 }}, }} }}", &source[..end]);
    let (_, baseline) = measured(|| Geoset::<V1400>::decode_mdl(source).unwrap());
    let (skin, allocations) = measured(|| Geoset::<V1400>::decode_mdl(&skin).unwrap());
    assert_eq!(allocations, baseline + 2); // Encoded rows and section metadata.
    assert_eq!(skin.skin_weights().unwrap().len(), 4);
    let (extent, allocations) = measured(|| Geoset::<V1400>::decode_mdl(&extent).unwrap());
    assert_eq!(allocations, baseline + 1);
    assert_eq!(extent.sequence_extents.len(), 1);
}

#[test]
fn encoded_arrays_move_into_track_constructors_and_skin_setters() {
    use wc3::model::animation::ValueKeyframe;
    use wc3::model::geometry::{Geoset, SkinWeights};
    use wc3::model::{ZeroVec, V1300};

    let keys: ZeroVec<ValueKeyframe<f32>> = [ValueKeyframe {
        frame: 0,
        value: 1.0,
    }]
    .into_iter()
    .collect();
    let pointer = keys.as_bytes().as_ptr();
    let (track, allocations) = measured(|| Track::linear(keys, None).unwrap());
    assert_eq!(allocations, 0);
    assert_eq!(track.linear_keys().unwrap().as_bytes().as_ptr(), pointer);

    let mut geoset = Geoset::<V1300>::new([[0.0; 3]], [[0.0; 3]], []).unwrap();
    geoset
        .set_skin_weights(Some(
            [SkinWeights::<u8> {
                bone_indices: [1; 4],
                weights: [255, 0, 0, 0],
            }]
            .into_iter()
            .collect(),
        ))
        .unwrap();
    let rows: ZeroVec<SkinWeights<u8>> = [SkinWeights {
        bone_indices: [2; 4],
        weights: [255, 0, 0, 0],
    }]
    .into_iter()
    .collect();
    let pointer = rows.as_bytes().as_ptr();
    let (_, allocations) = measured(|| geoset.set_skin_weights(Some(rows)).unwrap());
    assert_eq!(allocations, 0);
    assert_eq!(geoset.skin_weights().unwrap().as_bytes().as_ptr(), pointer);
}

#[test]
fn input_conversion_encodes_once_and_moves_existing_storage_without_allocating() {
    use wc3::model::{Borrowed, IntoZeroVec};

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
