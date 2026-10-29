//! Sequence-storage context is shared rather than copied into each chunk decoder.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use wc3::model::common::binary::ChunkHeader;
use wc3::model::formats::m2::companions::Skeleton;
use wc3::model::formats::m2::records::ModelFlags;
use wc3::model::formats::m2::V274;

struct Allocator;
thread_local! { static BYTES: Cell<Option<usize>> = const { Cell::new(None) }; }
fn record(bytes: usize) {
    let _ = BYTES.try_with(|total| {
        if let Some(value) = total.get() {
            total.set(Some(value + bytes));
        }
    });
}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn measured<T>(work: impl FnOnce() -> T) -> (T, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            BYTES.with(|total| total.set(None));
        }
    }
    BYTES.with(|total| total.set(Some(0)));
    let guard = Guard;
    let result = work();
    let bytes = BYTES.with(|total| total.get().unwrap());
    drop(guard);
    (result, bytes)
}

#[test]
fn inherited_sequence_context_cost_does_not_multiply_by_chunk_count() {
    let mut source = Vec::new();
    for _ in 0..128 {
        source.extend(
            ChunkHeader {
                tag: *b"TEST",
                size: 0,
            }
            .to_bytes(),
        );
    }
    let context = [true; 4096];
    let (without_context, baseline) = measured(|| {
        Skeleton::<V274>::read_with_sequences(&source, ModelFlags::default(), &[]).unwrap()
    });
    let (with_context, contextual) = measured(|| {
        Skeleton::<V274>::read_with_sequences(&source, ModelFlags::default(), &context).unwrap()
    });
    assert_eq!(with_context, without_context);
    let added = contextual.saturating_sub(baseline);
    // Allow allocator alignment overhead, but prohibit per-chunk sequence copies.
    assert!(
        added <= context.len() + 64,
        "{added} extra bytes for one shared context"
    );
    eprintln!("128 chunks, 4096 sequence flags: {added} extra allocated bytes");
}
