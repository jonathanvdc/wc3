//! Verify the promised allocation behavior, including failure diagnostics and
//! direct per-record MDL -> MDX conversion into a preallocated output buffer.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::{Cursor as IoCursor, Write};
use wc3::model::animation::{GlobalSequence, Interpolation, Sequence};
use wc3::model::materials::Texture;
use wc3::model::mdl::Read as _;
use wc3::model::mdl::{Lexer, Parser, Writer};
use wc3::model::scene::{ModelInfo, NodeTrack};
use wc3::model::Encoder;

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
        let track = NodeTrack::decode_mdl("Translation 0 { Linear, }").unwrap();
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
    geoset.set_tangents(Some(&[[1.0, 0.0, 0.0, -1.0]; 4]));
    geoset
        .set_skin_weights(Some(
            &[SkinWeights {
                bone_indices: [0; 4],
                weights: [255, 0, 0, 0],
            }; 4],
        ))
        .unwrap();
    geoset.sequence_extents = vec![geoset.extent; 2];
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
    model.set_geosets(&[
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
