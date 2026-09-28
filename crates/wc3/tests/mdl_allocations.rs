//! Verify the promised allocation behavior, including failure diagnostics and
//! direct per-record MDL -> MDX conversion into a preallocated output buffer.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::{Cursor as IoCursor, Write};
use wc3::model::animation::{GlobalSequence, Sequence};
use wc3::model::materials::Texture;
use wc3::model::mdl::Read as _;
use wc3::model::mdl::{Lexer, MdlWriter, Parser};
use wc3::model::scene::ModelInfo;
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
        let texture = Texture::parse_mdl(source).unwrap();
        let sequence =
            Sequence::parse_mdl("Anim \"Stand\" { Interval { 0, 1000 }, BoundsRadius 1.2345678, }")
                .unwrap();
        let mut writer = MdlWriter::new(IoCursor::new(&mut storage[..]));
        writer.write(&texture).unwrap();
        writer.write(&sequence).unwrap();
        let info =
            ModelInfo::parse_mdl("Model \"Derived\" { BlendTime 150, BoundsRadius 1.2345678, }")
                .unwrap();
        writer.write(&info).unwrap();
        let mut sink = writer.finish().unwrap();
        let error = Texture::parse_mdl("Bitmap { Mystery 1, }").unwrap_err();
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
